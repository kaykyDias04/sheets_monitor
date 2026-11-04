//! Secure self-updater backed by GitHub Releases and Tauri's per-user NSIS installer.
//! Checks discreetly on startup, validates HTTPS GitHub origins and SHA-256 digests,
//! respects SemVer 2.0, avoids interrupting active checks, and refuses unsafe self-overwrite
//! on isolated/portable executables.

use crate::integration::CommandError;
use crate::monitoring::work::WorkCoordinator;
use reqwest::{blocking::Client, redirect::Policy, StatusCode};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    cmp::Ordering,
    fs::{self, File},
    io::{BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use url::Url;

const GITHUB_OWNER: &str = "kaykyDias04";
const GITHUB_REPO: &str = "sheets_monitor";
const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/kaykyDias04/sheets_monitor/releases/latest";
const MIN_INSTALLER_BYTES: u64 = 32 * 1024;
const MAX_INSTALLER_BYTES: u64 = 300 * 1024 * 1024;
const MAX_METADATA_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub prerelease: Vec<String>,
}

impl SemVer {
    pub fn parse(raw: &str) -> Option<Self> {
        let trimmed = raw.trim().strip_prefix(['v', 'V']).unwrap_or(raw.trim());
        if trimmed.is_empty() || trimmed.len() > 128 {
            return None;
        }
        let without_build = trimmed.split('+').next()?;
        let (core, pre_raw) = match without_build.split_once('-') {
            Some((c, p)) => {
                if p.is_empty() {
                    return None;
                }
                (c, Some(p))
            }
            None => (without_build, None),
        };
        let mut parts = core.split('.');
        let major = parse_numeric_part(parts.next()?)?;
        let minor = parse_numeric_part(parts.next()?)?;
        let patch = parse_numeric_part(parts.next()?)?;
        if parts.next().is_some() {
            return None;
        }
        let prerelease = match pre_raw {
            Some(pre) => {
                let mut items = Vec::new();
                for ident in pre.split('.') {
                    if ident.is_empty()
                        || !ident
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
                    {
                        return None;
                    }
                    items.push(ident.to_owned());
                }
                items
            }
            None => Vec::new(),
        };
        Some(Self {
            major,
            minor,
            patch,
            prerelease,
        })
    }

    pub fn is_stable(&self) -> bool {
        self.prerelease.is_empty()
    }

    pub fn normalized(&self) -> String {
        if self.prerelease.is_empty() {
            format!("{}.{}.{}", self.major, self.minor, self.patch)
        } else {
            format!(
                "{}.{}.{}-{}",
                self.major,
                self.minor,
                self.patch,
                self.prerelease.join(".")
            )
        }
    }
}

fn parse_numeric_part(part: &str) -> Option<u64> {
    if part.is_empty() || (part.len() > 1 && part.starts_with('0')) {
        return None;
    }
    part.parse::<u64>().ok()
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch)) {
            Ordering::Equal => match (self.prerelease.is_empty(), other.prerelease.is_empty()) {
                (true, true) => Ordering::Equal,
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                (false, false) => {
                    for (a, b) in self.prerelease.iter().zip(other.prerelease.iter()) {
                        let ord = match (a.parse::<u64>(), b.parse::<u64>()) {
                            (Ok(na), Ok(nb)) => na.cmp(&nb),
                            (Ok(_), Err(_)) => Ordering::Less,
                            (Err(_), Ok(_)) => Ordering::Greater,
                            (Err(_), Err(_)) => a.cmp(b),
                        };
                        if ord != Ordering::Equal {
                            return ord;
                        }
                    }
                    self.prerelease.len().cmp(&other.prerelease.len())
                }
            },
            non_eq => non_eq,
        }
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallMode {
    PerUserNsis,
    PerMachineNsis,
    PortableOrDevelopment,
}

impl InstallMode {
    pub fn can_auto_update(self) -> bool {
        matches!(self, Self::PerUserNsis)
    }
}

pub fn classify_install_layout(
    exe_dir: &Path,
    has_uninstaller: bool,
    hkcu_install_dir: Option<&Path>,
    hklm_install_dir: Option<&Path>,
) -> InstallMode {
    let norm_exe = normalize_win_path(exe_dir);
    if let Some(hklm) = hklm_install_dir {
        if normalize_win_path(hklm) == norm_exe {
            return InstallMode::PerMachineNsis;
        }
    }
    if let Some(hkcu) = hkcu_install_dir {
        if normalize_win_path(hkcu) == norm_exe && has_uninstaller {
            return InstallMode::PerUserNsis;
        }
    }
    if has_uninstaller && !is_protected_system_dir(&norm_exe) && !is_cargo_target_dir(&norm_exe) {
        return InstallMode::PerUserNsis;
    }
    if has_uninstaller && is_protected_system_dir(&norm_exe) {
        return InstallMode::PerMachineNsis;
    }
    InstallMode::PortableOrDevelopment
}

fn normalize_win_path(path: &Path) -> String {
    path.to_string_lossy()
        .trim()
        .trim_matches('"')
        .trim_end_matches(['\\', '/'])
        .replace('/', "\\")
        .to_ascii_lowercase()
}

fn is_protected_system_dir(norm: &str) -> bool {
    let program_files = std::env::var("ProgramFiles")
        .unwrap_or_else(|_| "C:\\Program Files".into())
        .to_ascii_lowercase();
    let program_files_x86 = std::env::var("ProgramFiles(x86)")
        .unwrap_or_else(|_| "C:\\Program Files (x86)".into())
        .to_ascii_lowercase();
    norm.starts_with(&program_files) || norm.starts_with(&program_files_x86)
}

fn is_cargo_target_dir(norm: &str) -> bool {
    norm.contains("\\target\\debug")
        || norm.contains("\\target\\release")
        || norm.contains("\\x86_64-pc-windows-msvc\\")
}

pub fn detect_current_install_mode() -> InstallMode {
    let Ok(exe_path) = std::env::current_exe() else {
        return InstallMode::PortableOrDevelopment;
    };
    let Some(exe_dir) = exe_path.parent() else {
        return InstallMode::PortableOrDevelopment;
    };
    let has_uninstaller = exe_dir.join("uninstall.exe").is_file();
    let (hkcu_dir, hklm_dir) = read_nsis_registry_locations();
    classify_install_layout(
        exe_dir,
        has_uninstaller,
        hkcu_dir.as_deref(),
        hklm_dir.as_deref(),
    )
}

#[cfg(windows)]
fn read_nsis_registry_locations() -> (Option<PathBuf>, Option<PathBuf>) {
    use winreg::{enums::*, RegKey};
    let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Sheets Monitor";
    let read_from = |hive: winreg::HKEY| -> Option<PathBuf> {
        let root = RegKey::predef(hive);
        let key = root.open_subkey_with_flags(subkey, KEY_READ).ok()?;
        let loc: String = key.get_value("InstallLocation").ok()?;
        let cleaned = loc.trim().trim_matches('"').trim();
        (!cleaned.is_empty()).then(|| PathBuf::from(cleaned))
    };
    (read_from(HKEY_CURRENT_USER), read_from(HKEY_LOCAL_MACHINE))
}

#[cfg(not(windows))]
fn read_nsis_registry_locations() -> (Option<PathBuf>, Option<PathBuf>) {
    (None, None)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct UpdaterDiskState {
    pub last_checked_at: Option<String>,
    pub dismissed_version: Option<String>,
    pub last_error_code: Option<String>,
    pub last_error_at: Option<String>,
    pub staged_version: Option<String>,
    pub staged_path: Option<String>,
    pub staged_sha256: Option<String>,
    pub staged_release_url: Option<String>,
    pub staged_notes: Option<String>,
    pub staged_published_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdaterStatus {
    /// `"idle" | "checking" | "up_to_date" | "downloading" | "ready_to_install" | "manual_required" | "installing" | "error"`
    pub state: String,
    pub current_version: String,
    pub available_version: Option<String>,
    pub release_url: Option<String>,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
    pub install_mode: InstallMode,
    pub can_auto_update: bool,
    pub dismissed: bool,
    pub last_checked_at: Option<String>,
    pub last_error_message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize, Clone)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    #[serde(default)]
    digest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReleaseManifest {
    #[serde(default)]
    sha256: Option<String>,
    #[serde(default)]
    public_asset_name: Option<String>,
}

pub trait ReleaseTransport: Send + Sync {
    /// Returns `Ok(None)` when the repository has no stable release yet (HTTP 404).
    fn fetch_latest_release(&self) -> Result<Option<String>, String>;
    fn fetch_metadata_text(&self, url: &str) -> Result<String, String>;
    fn download_installer(
        &self,
        url: &str,
        dest: &Path,
        expected_size: u64,
    ) -> Result<String, String>;
}

pub struct HttpsReleaseTransport {
    api_client: Client,
    asset_client: Client,
}

impl HttpsReleaseTransport {
    pub fn new(current_version: &str) -> Result<Self, String> {
        let ua = format!("SheetsMonitor/{current_version}");
        let api_client = Client::builder()
            .user_agent(&ua)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(20))
            .redirect(Policy::custom(|attempt| {
                let url = attempt.url();
                if attempt.previous().len() >= 4 || !is_trusted_github_api_url(url) {
                    attempt.stop()
                } else {
                    attempt.follow()
                }
            }))
            .build()
            .map_err(|_| "client_init_failed".to_owned())?;
        let asset_client = Client::builder()
            .user_agent(&ua)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(180))
            .redirect(Policy::custom(|attempt| {
                let url = attempt.url();
                if attempt.previous().len() >= 6 || !is_trusted_github_download_host(url) {
                    attempt.stop()
                } else {
                    attempt.follow()
                }
            }))
            .build()
            .map_err(|_| "client_init_failed".to_owned())?;
        Ok(Self {
            api_client,
            asset_client,
        })
    }
}

fn is_trusted_github_api_url(url: &Url) -> bool {
    url.scheme() == "https" && url.host_str() == Some("api.github.com")
}

fn is_trusted_github_download_host(url: &Url) -> bool {
    if url.scheme() != "https" {
        return false;
    }
    let Some(host) = url.host_str() else {
        return false;
    };
    host == "github.com"
        || host == "objects.githubusercontent.com"
        || host == "release-assets.githubusercontent.com"
        || host.ends_with(".githubusercontent.com")
}

pub fn verify_github_release_html_url(raw: &str) -> Result<(), &'static str> {
    let parsed = Url::parse(raw.trim()).map_err(|_| "untrusted_origin")?;
    if parsed.scheme() != "https" || parsed.host_str() != Some("github.com") {
        return Err("untrusted_origin");
    }
    let expected_prefix = format!("/{GITHUB_OWNER}/{GITHUB_REPO}/releases/tag/");
    if !parsed.path().starts_with(&expected_prefix) {
        return Err("untrusted_origin");
    }
    Ok(())
}

pub fn verify_github_asset_url(raw: &str) -> Result<(), &'static str> {
    let parsed = Url::parse(raw.trim()).map_err(|_| "untrusted_origin")?;
    if parsed.scheme() != "https" || parsed.host_str() != Some("github.com") {
        return Err("untrusted_origin");
    }
    let expected_prefix = format!("/{GITHUB_OWNER}/{GITHUB_REPO}/releases/download/");
    if !parsed.path().starts_with(&expected_prefix) {
        return Err("untrusted_origin");
    }
    Ok(())
}

impl ReleaseTransport for HttpsReleaseTransport {
    fn fetch_latest_release(&self) -> Result<Option<String>, String> {
        let resp = self
            .api_client
            .get(LATEST_RELEASE_API)
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .map_err(|_| "network_unavailable".to_owned())?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !resp.status().is_success() {
            return Err(format!("github_http_{}", resp.status().as_u16()));
        }
        let mut body = String::new();
        resp.take(MAX_METADATA_BYTES as u64)
            .read_to_string(&mut body)
            .map_err(|_| "network_read_failed".to_owned())?;
        Ok(Some(body))
    }

    fn fetch_metadata_text(&self, url: &str) -> Result<String, String> {
        verify_github_asset_url(url).map_err(str::to_owned)?;
        let resp = self
            .asset_client
            .get(url)
            .send()
            .map_err(|_| "network_unavailable".to_owned())?;
        if !resp.status().is_success() {
            return Err(format!("asset_http_{}", resp.status().as_u16()));
        }
        let mut body = String::new();
        resp.take(MAX_METADATA_BYTES as u64)
            .read_to_string(&mut body)
            .map_err(|_| "network_read_failed".to_owned())?;
        Ok(body)
    }

    fn download_installer(
        &self,
        url: &str,
        dest: &Path,
        expected_size: u64,
    ) -> Result<String, String> {
        verify_github_asset_url(url).map_err(str::to_owned)?;
        if !(MIN_INSTALLER_BYTES..=MAX_INSTALLER_BYTES).contains(&expected_size) {
            return Err("invalid_package_size".into());
        }
        let resp = self
            .asset_client
            .get(url)
            .send()
            .map_err(|_| "network_unavailable".to_owned())?;
        if !resp.status().is_success() {
            return Err(format!("asset_http_{}", resp.status().as_u16()));
        }
        stream_and_hash_reader(resp, dest, expected_size)
    }
}

pub fn stream_and_hash_reader(
    mut reader: impl Read,
    dest: &Path,
    expected_size: u64,
) -> Result<String, String> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|_| "disk_write_failed".to_owned())?;
    }
    let file = File::create(dest).map_err(|_| "disk_write_failed".to_owned())?;
    let mut writer = BufWriter::new(file);
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    let mut header = [0u8; 2];
    let mut header_seen = 0usize;
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|_| "network_read_failed".to_owned())?;
        if n == 0 {
            break;
        }
        if header_seen < 2 {
            let take = (2 - header_seen).min(n);
            header[header_seen..header_seen + take].copy_from_slice(&buf[..take]);
            header_seen += take;
        }
        total = total.saturating_add(n as u64);
        if total > expected_size || total > MAX_INSTALLER_BYTES {
            let _ = fs::remove_file(dest);
            return Err("invalid_package_size".into());
        }
        hasher.update(&buf[..n]);
        writer
            .write_all(&buf[..n])
            .map_err(|_| "disk_write_failed".to_owned())?;
    }
    writer.flush().map_err(|_| "disk_write_failed".to_owned())?;
    drop(writer);
    if total != expected_size {
        let _ = fs::remove_file(dest);
        return Err("invalid_package_size".into());
    }
    if header_seen < 2 || header != *b"MZ" {
        let _ = fs::remove_file(dest);
        return Err("invalid_pe_signature".into());
    }
    Ok(hex_lower(&hasher.finalize()))
}

fn verify_file_sha256_and_pe(path: &Path, expected_hex: &str) -> Result<(), &'static str> {
    let file = File::open(path).map_err(|_| "staged_installer_missing")?;
    let meta = file.metadata().map_err(|_| "staged_installer_missing")?;
    if meta.len() < 2 || meta.len() > MAX_INSTALLER_BYTES {
        return Err("invalid_package_size");
    }
    let mut reader = BufReader::new(file);
    let mut header = [0u8; 2];
    reader
        .read_exact(&mut header)
        .map_err(|_| "invalid_pe_signature")?;
    if header != *b"MZ" {
        return Err("invalid_pe_signature");
    }
    let mut hasher = Sha256::new();
    hasher.update(header);
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader.read(&mut buf).map_err(|_| "disk_read_failed")?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let actual = hex_lower(&hasher.finalize());
    if !actual.eq_ignore_ascii_case(expected_hex.trim()) {
        return Err("integrity_mismatch");
    }
    Ok(())
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

fn valid_sha256_hex(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let hex = trimmed.strip_prefix("sha256:").unwrap_or(trimmed).trim();
    if hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(hex.to_ascii_lowercase())
    } else {
        None
    }
}

fn parse_sha256sums_for_asset(text: &str, asset_name: &str) -> Option<String> {
    let normalized_asset = asset_name.replace(' ', ".");
    let space_asset = asset_name.replace('.', " ");
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let Some(hash_raw) = parts.next() else {
            continue;
        };
        let rest = parts.collect::<Vec<_>>().join(" ");
        let target = rest.trim_start_matches('*').trim();
        if target.eq_ignore_ascii_case(asset_name)
            || target.eq_ignore_ascii_case(&normalized_asset)
            || target.eq_ignore_ascii_case(&space_asset)
        {
            if let Some(valid) = valid_sha256_hex(hash_raw) {
                return Some(valid);
            }
        }
    }
    None
}

fn select_windows_x64_nsis_asset(assets: &[GithubAsset]) -> Option<&GithubAsset> {
    assets
        .iter()
        .find(|a| {
            let lower = a.name.to_ascii_lowercase();
            lower.ends_with("x64-setup.exe") && !lower.contains("arm64")
        })
        .or_else(|| {
            assets.iter().find(|a| {
                let lower = a.name.to_ascii_lowercase();
                lower.ends_with("-setup.exe")
                    && !lower.contains("arm64")
                    && !lower.contains("x86-setup")
            })
        })
}

fn safe_asset_filename(name: &str) -> Option<String> {
    let cleaned: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(*c, '.' | '-' | '_'))
        .collect();
    if cleaned.len() < 10
        || !cleaned.to_ascii_lowercase().ends_with(".exe")
        || cleaned.contains("..")
    {
        None
    } else {
        Some(cleaned)
    }
}

fn human_error_message(code: &str) -> String {
    match code {
        "network_unavailable" | "network_read_failed" => {
            "Sem conexão com o GitHub no momento. A verificação será repetida na próxima inicialização."
                .into()
        }
        "integrity_mismatch" | "invalid_pe_signature" | "invalid_package_size" => {
            "O pacote baixado falhou na verificação de integridade SHA-256 e foi descartado com segurança."
                .into()
        }
        "missing_checksum" => {
            "A versão publicada não possui manifesto SHA-256 verificável; atualização bloqueada por segurança."
                .into()
        }
        "untrusted_origin" => {
            "Origem da atualização rejeitada por segurança."
                .into()
        }
        _ => "Não foi possível concluir a verificação de atualizações no momento.".into(),
    }
}

struct InnerState {
    runtime_state: String,
    available_version: Option<String>,
    release_url: Option<String>,
    release_notes: Option<String>,
    published_at: Option<String>,
    disk: UpdaterDiskState,
}

pub struct UpdaterService {
    app_data_dir: PathBuf,
    current_version: String,
    install_mode: InstallMode,
    transport: Arc<dyn ReleaseTransport>,
    notify: Arc<dyn Fn(&UpdaterStatus) + Send + Sync>,
    state: Mutex<InnerState>,
    check_lock: Mutex<()>,
}

impl UpdaterService {
    pub fn new(
        app_data_dir: PathBuf,
        current_version: String,
        install_mode: InstallMode,
        transport: Arc<dyn ReleaseTransport>,
        notify: Arc<dyn Fn(&UpdaterStatus) + Send + Sync>,
    ) -> Arc<Self> {
        let disk = load_disk_state(&app_data_dir);
        let mut initial_state = "idle".to_owned();
        let mut available_version = None;
        let mut release_url = None;
        let mut release_notes = None;
        let mut published_at = None;
        if let (Some(current), Some(staged_ver), Some(staged_path), Some(staged_sha)) = (
            SemVer::parse(&current_version),
            disk.staged_version.as_deref().and_then(SemVer::parse),
            disk.staged_path.as_deref(),
            disk.staged_sha256.as_deref(),
        ) {
            if staged_ver.is_stable()
                && staged_ver > current
                && verify_file_sha256_and_pe(Path::new(staged_path), staged_sha).is_ok()
            {
                let ver_str = staged_ver.normalized();
                if disk.dismissed_version.as_deref() != Some(ver_str.as_str()) {
                    initial_state = if install_mode.can_auto_update() {
                        "ready_to_install".into()
                    } else {
                        "manual_required".into()
                    };
                }
                available_version = Some(ver_str);
                release_url = disk.staged_release_url.clone();
                release_notes = disk.staged_notes.clone();
                published_at = disk.staged_published_at.clone();
            }
        }
        Arc::new(Self {
            app_data_dir,
            current_version,
            install_mode,
            transport,
            notify,
            state: Mutex::new(InnerState {
                runtime_state: initial_state,
                available_version,
                release_url,
                release_notes,
                published_at,
                disk,
            }),
            check_lock: Mutex::new(()),
        })
    }

    pub fn spawn_startup_check(self: &Arc<Self>, delay: Duration) {
        let svc = self.clone();
        let _ = std::thread::Builder::new()
            .name("sheets-monitor-updater".into())
            .spawn(move || {
                if !delay.is_zero() {
                    std::thread::sleep(delay);
                }
                let _ = svc.check_for_updates(false);
            });
    }

    pub fn status(&self) -> UpdaterStatus {
        let guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
        self.snapshot_locked(&guard)
    }

    fn snapshot_locked(&self, guard: &InnerState) -> UpdaterStatus {
        let dismissed = guard
            .available_version
            .as_deref()
            .is_some_and(|v| guard.disk.dismissed_version.as_deref() == Some(v));
        UpdaterStatus {
            state: guard.runtime_state.clone(),
            current_version: self.current_version.clone(),
            available_version: guard.available_version.clone(),
            release_url: guard.release_url.clone(),
            release_notes: guard.release_notes.clone(),
            published_at: guard.published_at.clone(),
            install_mode: self.install_mode,
            can_auto_update: self.install_mode.can_auto_update(),
            dismissed,
            last_checked_at: guard.disk.last_checked_at.clone(),
            last_error_message: guard
                .disk
                .last_error_code
                .as_deref()
                .map(human_error_message),
        }
    }

    fn emit(&self, status: &UpdaterStatus) {
        (self.notify)(status);
    }

    pub fn dismiss_version(&self, version: &str) -> Result<UpdaterStatus, CommandError> {
        let parsed = SemVer::parse(version).ok_or_else(|| CommandError {
            code: "invalid_version".into(),
            message: "Versão informada para adiamento é inválida.".into(),
            retryable: false,
        })?;
        let status = {
            let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
            guard.disk.dismissed_version = Some(parsed.normalized());
            if guard.runtime_state == "ready_to_install" || guard.runtime_state == "manual_required"
            {
                guard.runtime_state = "idle".into();
            }
            save_disk_state(&self.app_data_dir, &guard.disk);
            self.snapshot_locked(&guard)
        };
        self.emit(&status);
        Ok(status)
    }

    pub fn check_for_updates(&self, manual: bool) -> UpdaterStatus {
        let Ok(_serial) = self.check_lock.try_lock() else {
            return self.status();
        };
        {
            let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
            if guard.runtime_state == "installing" {
                return self.snapshot_locked(&guard);
            }
            guard.runtime_state = "checking".into();
            if manual {
                let status = self.snapshot_locked(&guard);
                drop(guard);
                self.emit(&status);
            }
        }

        let now = chrono::Utc::now().to_rfc3339();
        let outcome = self.perform_check_and_stage(manual);
        let status = {
            let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
            guard.disk.last_checked_at = Some(now.clone());
            match outcome {
                Ok(CheckOutcome::UpToDate) => {
                    guard.runtime_state = "up_to_date".into();
                    guard.available_version = None;
                    guard.release_url = None;
                    guard.release_notes = None;
                    guard.published_at = None;
                    guard.disk.last_error_code = None;
                    guard.disk.last_error_at = None;
                }
                Ok(CheckOutcome::Dismissed {
                    version,
                    html_url,
                    notes,
                    published_at,
                }) => {
                    guard.runtime_state = "idle".into();
                    guard.available_version = Some(version);
                    guard.release_url = Some(html_url);
                    guard.release_notes = notes;
                    guard.published_at = published_at;
                    guard.disk.last_error_code = None;
                }
                Ok(CheckOutcome::ManualRequired {
                    version,
                    html_url,
                    notes,
                    published_at,
                }) => {
                    guard.runtime_state = "manual_required".into();
                    guard.available_version = Some(version);
                    guard.release_url = Some(html_url);
                    guard.release_notes = notes;
                    guard.published_at = published_at;
                    guard.disk.last_error_code = None;
                    if manual {
                        guard.disk.dismissed_version = None;
                    }
                }
                Ok(CheckOutcome::Staged {
                    version,
                    path,
                    sha256,
                    html_url,
                    notes,
                    published_at,
                }) => {
                    guard.runtime_state = "ready_to_install".into();
                    guard.available_version = Some(version.clone());
                    guard.release_url = Some(html_url.clone());
                    guard.release_notes = notes.clone();
                    guard.published_at = published_at.clone();
                    guard.disk.staged_version = Some(version);
                    guard.disk.staged_path = Some(path.to_string_lossy().into_owned());
                    guard.disk.staged_sha256 = Some(sha256);
                    guard.disk.staged_release_url = Some(html_url);
                    guard.disk.staged_notes = notes;
                    guard.disk.staged_published_at = published_at;
                    guard.disk.last_error_code = None;
                    guard.disk.last_error_at = None;
                    if manual {
                        guard.disk.dismissed_version = None;
                    }
                }
                Err(code) => {
                    guard.runtime_state = if manual {
                        "error".into()
                    } else {
                        "idle".into()
                    };
                    guard.disk.last_error_code = Some(code);
                    guard.disk.last_error_at = Some(now);
                }
            }
            save_disk_state(&self.app_data_dir, &guard.disk);
            self.snapshot_locked(&guard)
        };
        if manual
            || matches!(
                status.state.as_str(),
                "ready_to_install" | "manual_required"
            )
        {
            self.emit(&status);
        }
        status
    }

    fn perform_check_and_stage(&self, manual: bool) -> Result<CheckOutcome, String> {
        let current = SemVer::parse(&self.current_version)
            .ok_or_else(|| "invalid_current_version".to_owned())?;
        let Some(raw_json) = self.transport.fetch_latest_release()? else {
            return Ok(CheckOutcome::UpToDate);
        };
        let release: GithubRelease =
            serde_json::from_str(&raw_json).map_err(|_| "invalid_release_payload".to_owned())?;
        if release.draft || release.prerelease {
            return Ok(CheckOutcome::UpToDate);
        }
        let remote =
            SemVer::parse(&release.tag_name).ok_or_else(|| "invalid_remote_version".to_owned())?;
        if !remote.is_stable() || remote <= current {
            return Ok(CheckOutcome::UpToDate);
        }
        verify_github_release_html_url(&release.html_url).map_err(str::to_owned)?;
        let version_str = remote.normalized();
        let notes = release
            .body
            .as_deref()
            .map(str::trim)
            .filter(|b| !b.is_empty())
            .map(|b| b.chars().take(1200).collect::<String>());
        let dismissed = {
            let guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
            guard.disk.dismissed_version.as_deref() == Some(version_str.as_str())
        };
        if dismissed && !manual {
            return Ok(CheckOutcome::Dismissed {
                version: version_str,
                html_url: release.html_url,
                notes,
                published_at: release.published_at,
            });
        }
        if !self.install_mode.can_auto_update() {
            return Ok(CheckOutcome::ManualRequired {
                version: version_str,
                html_url: release.html_url,
                notes,
                published_at: release.published_at,
            });
        }

        let installer_asset = select_windows_x64_nsis_asset(&release.assets)
            .ok_or_else(|| "missing_windows_asset".to_owned())?;
        verify_github_asset_url(&installer_asset.browser_download_url).map_err(str::to_owned)?;
        let safe_name = safe_asset_filename(&installer_asset.name)
            .ok_or_else(|| "invalid_asset_name".to_owned())?;
        let expected_sha256 = self.resolve_verified_checksum(&release.assets, installer_asset)?;

        let target_dir = self.app_data_dir.join("updates").join(&version_str);
        let final_path = target_dir.join(&safe_name);
        if verify_file_sha256_and_pe(&final_path, &expected_sha256).is_ok() {
            return Ok(CheckOutcome::Staged {
                version: version_str,
                path: final_path,
                sha256: expected_sha256,
                html_url: release.html_url,
                notes,
                published_at: release.published_at,
            });
        }

        {
            let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
            guard.runtime_state = "downloading".into();
            guard.available_version = Some(version_str.clone());
            if manual {
                let status = self.snapshot_locked(&guard);
                drop(guard);
                self.emit(&status);
            }
        }

        let part_path = target_dir.join(format!("{safe_name}.part"));
        let actual_sha256 = self.transport.download_installer(
            &installer_asset.browser_download_url,
            &part_path,
            installer_asset.size,
        )?;
        if !actual_sha256.eq_ignore_ascii_case(&expected_sha256) {
            let _ = fs::remove_file(&part_path);
            return Err("integrity_mismatch".into());
        }
        if final_path.exists() {
            let _ = fs::remove_file(&final_path);
        }
        fs::rename(&part_path, &final_path).map_err(|_| {
            let _ = fs::remove_file(&part_path);
            "disk_write_failed".to_owned()
        })?;
        prune_old_staged_updates(&self.app_data_dir.join("updates"), &version_str);
        Ok(CheckOutcome::Staged {
            version: version_str,
            path: final_path,
            sha256: expected_sha256,
            html_url: release.html_url,
            notes,
            published_at: release.published_at,
        })
    }

    fn resolve_verified_checksum(
        &self,
        assets: &[GithubAsset],
        installer: &GithubAsset,
    ) -> Result<String, String> {
        let mut resolved: Option<String> = None;
        if let Some(digest_raw) = installer.digest.as_deref() {
            if let Some(hex) = valid_sha256_hex(digest_raw) {
                resolved = Some(hex);
            }
        }
        if let Some(sums_asset) = assets
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case("SHA256SUMS.txt"))
        {
            let text = self
                .transport
                .fetch_metadata_text(&sums_asset.browser_download_url)?;
            if let Some(sums_hex) = parse_sha256sums_for_asset(&text, &installer.name) {
                if let Some(existing) = &resolved {
                    if !existing.eq_ignore_ascii_case(&sums_hex) {
                        return Err("integrity_mismatch".into());
                    }
                } else {
                    resolved = Some(sums_hex);
                }
            }
        }
        if resolved.is_none() {
            if let Some(manifest_asset) = assets
                .iter()
                .find(|a| a.name.eq_ignore_ascii_case("artifact-manifest.json"))
            {
                let text = self
                    .transport
                    .fetch_metadata_text(&manifest_asset.browser_download_url)?;
                if let Ok(manifest) = serde_json::from_str::<ReleaseManifest>(&text) {
                    if manifest.public_asset_name.as_deref().is_none_or(|name| {
                        name.eq_ignore_ascii_case(&installer.name)
                            || name.replace('.', " ").eq_ignore_ascii_case(&installer.name)
                    }) {
                        if let Some(hex) = manifest.sha256.as_deref().and_then(valid_sha256_hex) {
                            resolved = Some(hex);
                        }
                    }
                }
            }
        }
        resolved.ok_or_else(|| "missing_checksum".into())
    }

    /// Waits gracefully for any active Google checks to finish (without cancelling them),
    /// re-verifies SHA-256 on disk, and returns the verified NSIS installer path.
    pub fn prepare_install(&self, database_path: &Path) -> Result<PathBuf, CommandError> {
        if !self.install_mode.can_auto_update() {
            return Err(CommandError {
                code: "manual_update_required".into(),
                message: "Esta execução não utiliza uma instalação NSIS por usuário; baixe o instalador oficial para atualizar.".into(),
                retryable: false,
            });
        }
        let (staged_path, staged_sha256, staged_version) = {
            let guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
            let (Some(p), Some(sha), Some(ver)) = (
                guard.disk.staged_path.clone(),
                guard.disk.staged_sha256.clone(),
                guard.disk.staged_version.clone(),
            ) else {
                return Err(CommandError {
                    code: "update_not_staged".into(),
                    message: "Nenhum pacote de atualização verificado está pronto para instalação."
                        .into(),
                    retryable: false,
                });
            };
            (PathBuf::from(p), sha, ver)
        };
        let current = SemVer::parse(&self.current_version).ok_or_else(CommandError::local)?;
        let remote = SemVer::parse(&staged_version).ok_or_else(CommandError::local)?;
        if !remote.is_stable() || remote <= current {
            return Err(CommandError {
                code: "already_up_to_date".into(),
                message: "A versão instalada já é igual ou superior ao pacote selecionado.".into(),
                retryable: false,
            });
        }

        let coordinator = WorkCoordinator::for_database(database_path);
        let deadline = Instant::now() + Duration::from_secs(25);
        while coordinator.active_count() > 0 {
            if Instant::now() >= deadline {
                return Err(CommandError {
                    code: "checks_in_progress".into(),
                    message: "Há verificações de planilhas em andamento. Aguarde alguns instantes e tente aplicar a atualização novamente.".into(),
                    retryable: true,
                });
            }
            std::thread::sleep(Duration::from_millis(200));
        }

        verify_file_sha256_and_pe(&staged_path, &staged_sha256).map_err(|_| CommandError {
            code: "integrity_mismatch".into(),
            message: "A verificação final de integridade do instalador falhou; tente verificar atualizações novamente.".into(),
            retryable: true,
        })?;

        let status = {
            let mut guard = self.state.lock().unwrap_or_else(|e| e.into_inner());
            guard.runtime_state = "installing".into();
            self.snapshot_locked(&guard)
        };
        self.emit(&status);
        Ok(staged_path)
    }
}

enum CheckOutcome {
    UpToDate,
    Dismissed {
        version: String,
        html_url: String,
        notes: Option<String>,
        published_at: Option<String>,
    },
    ManualRequired {
        version: String,
        html_url: String,
        notes: Option<String>,
        published_at: Option<String>,
    },
    Staged {
        version: String,
        path: PathBuf,
        sha256: String,
        html_url: String,
        notes: Option<String>,
        published_at: Option<String>,
    },
}

fn state_file_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("updater-state.json")
}

fn load_disk_state(app_data_dir: &Path) -> UpdaterDiskState {
    fs::read_to_string(state_file_path(app_data_dir))
        .ok()
        .and_then(|raw| serde_json::from_str::<UpdaterDiskState>(&raw).ok())
        .unwrap_or_default()
}

fn save_disk_state(app_data_dir: &Path, state: &UpdaterDiskState) {
    if fs::create_dir_all(app_data_dir).is_err() {
        return;
    }
    if let Ok(encoded) = serde_json::to_vec_pretty(state) {
        let target = state_file_path(app_data_dir);
        let temp = app_data_dir.join("updater-state.json.tmp");
        if fs::write(&temp, encoded).is_ok() {
            let _ = fs::rename(&temp, &target);
        }
    }
}

fn prune_old_staged_updates(updates_dir: &Path, keep_version: &str) {
    let Ok(entries) = fs::read_dir(updates_dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy() != keep_version {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockTransport {
        release_json: Result<Option<String>, String>,
        texts: HashMap<String, String>,
        binary: Vec<u8>,
    }

    impl ReleaseTransport for MockTransport {
        fn fetch_latest_release(&self) -> Result<Option<String>, String> {
            self.release_json.clone()
        }
        fn fetch_metadata_text(&self, url: &str) -> Result<String, String> {
            verify_github_asset_url(url).map_err(str::to_owned)?;
            self.texts
                .get(url)
                .cloned()
                .ok_or_else(|| "not_found".into())
        }
        fn download_installer(
            &self,
            url: &str,
            dest: &Path,
            expected_size: u64,
        ) -> Result<String, String> {
            verify_github_asset_url(url).map_err(str::to_owned)?;
            stream_and_hash_reader(self.binary.as_slice(), dest, expected_size)
        }
    }

    fn fake_pe_installer() -> (Vec<u8>, String) {
        let mut bytes = vec![b'M', b'Z'];
        bytes.extend(vec![0x90u8; 40_000]);
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let sha = hex_lower(&hasher.finalize());
        (bytes, sha)
    }

    #[test]
    fn semver_ordering_follows_semver_2_rules() {
        let v104 = SemVer::parse("v1.0.4").unwrap();
        let v105 = SemVer::parse("1.0.5").unwrap();
        let v1010 = SemVer::parse("v1.0.10").unwrap();
        let v105_rc1 = SemVer::parse("1.0.5-rc.1").unwrap();
        let v105_rc2 = SemVer::parse("1.0.5-rc.2").unwrap();
        assert!(v104 < v105);
        assert!(v105 < v1010);
        assert!(v105_rc1 < v105_rc2);
        assert!(v105_rc2 < v105);
        assert!(!v105_rc1.is_stable());
        assert!(v105.is_stable());
        assert!(SemVer::parse("01.0.0").is_none());
    }

    #[test]
    fn stages_verified_release_respects_dismissal_and_prepares_install() {
        let dir = tempfile::tempdir().unwrap();
        let (pe_bytes, sha) = fake_pe_installer();
        let asset_url = "https://github.com/kaykyDias04/sheets_monitor/releases/download/v1.0.6/Sheets.Monitor_1.0.6_x64-setup.exe";
        let sums_url =
            "https://github.com/kaykyDias04/sheets_monitor/releases/download/v1.0.6/SHA256SUMS.txt";
        let release_json = serde_json::json!({
            "tag_name": "v1.0.6",
            "html_url": "https://github.com/kaykyDias04/sheets_monitor/releases/tag/v1.0.6",
            "body": "Melhorias de estabilidade.",
            "draft": false,
            "prerelease": false,
            "published_at": "2026-11-04T15:00:00Z",
            "assets": [
                {
                    "name": "Sheets.Monitor_1.0.6_x64-setup.exe",
                    "browser_download_url": asset_url,
                    "size": pe_bytes.len(),
                    "digest": format!("sha256:{sha}")
                },
                {
                    "name": "SHA256SUMS.txt",
                    "browser_download_url": sums_url,
                    "size": 96
                }
            ]
        })
        .to_string();
        let mut texts = HashMap::new();
        texts.insert(
            sums_url.to_owned(),
            format!(
                "{}  Sheets.Monitor_1.0.6_x64-setup.exe\n",
                sha.to_uppercase()
            ),
        );
        let svc = UpdaterService::new(
            dir.path().to_path_buf(),
            "1.0.5".into(),
            InstallMode::PerUserNsis,
            Arc::new(MockTransport {
                release_json: Ok(Some(release_json)),
                texts,
                binary: pe_bytes,
            }),
            Arc::new(|_| {}),
        );

        let status = svc.check_for_updates(false);
        assert_eq!(status.state, "ready_to_install");
        assert_eq!(status.available_version.as_deref(), Some("1.0.6"));
        assert!(status.can_auto_update);

        let installer_path = svc.prepare_install(&dir.path().join("db.sqlite")).unwrap();
        assert!(installer_path.is_file());

        // Dismissing prevents repeated startup prompts for the same version
        let svc2 = UpdaterService::new(
            dir.path().to_path_buf(),
            "1.0.5".into(),
            InstallMode::PerUserNsis,
            svc.transport.clone(),
            Arc::new(|_| {}),
        );
        svc2.dismiss_version("1.0.6").unwrap();
        let startup_again = svc2.check_for_updates(false);
        assert_eq!(startup_again.state, "idle");
        assert!(startup_again.dismissed);
    }

    #[test]
    fn handles_network_failure_missing_release_tampered_hash_and_portable_exe_safely() {
        let dir = tempfile::tempdir().unwrap();
        let offline = UpdaterService::new(
            dir.path().to_path_buf(),
            "1.0.5".into(),
            InstallMode::PerUserNsis,
            Arc::new(MockTransport {
                release_json: Err("network_unavailable".into()),
                texts: HashMap::new(),
                binary: Vec::new(),
            }),
            Arc::new(|_| {}),
        );
        let st = offline.check_for_updates(false);
        assert_eq!(st.state, "idle");
        assert!(st.last_error_message.is_some());

        let no_release = UpdaterService::new(
            dir.path().to_path_buf(),
            "1.0.5".into(),
            InstallMode::PerUserNsis,
            Arc::new(MockTransport {
                release_json: Ok(None),
                texts: HashMap::new(),
                binary: Vec::new(),
            }),
            Arc::new(|_| {}),
        );
        assert_eq!(no_release.check_for_updates(true).state, "up_to_date");

        // Tampered hash is rejected and never staged
        let (pe_bytes, _) = fake_pe_installer();
        let bad_sha = "0".repeat(64);
        let tampered_json = serde_json::json!({
            "tag_name": "v1.0.6",
            "html_url": "https://github.com/kaykyDias04/sheets_monitor/releases/tag/v1.0.6",
            "draft": false,
            "prerelease": false,
            "assets": [{
                "name": "Sheets.Monitor_1.0.6_x64-setup.exe",
                "browser_download_url": "https://github.com/kaykyDias04/sheets_monitor/releases/download/v1.0.6/Sheets.Monitor_1.0.6_x64-setup.exe",
                "size": pe_bytes.len(),
                "digest": format!("sha256:{bad_sha}")
            }]
        })
        .to_string();
        let tampered = UpdaterService::new(
            dir.path().to_path_buf(),
            "1.0.5".into(),
            InstallMode::PerUserNsis,
            Arc::new(MockTransport {
                release_json: Ok(Some(tampered_json.clone())),
                texts: HashMap::new(),
                binary: pe_bytes,
            }),
            Arc::new(|_| {}),
        );
        assert_eq!(tampered.check_for_updates(true).state, "error");
        assert!(tampered
            .prepare_install(&dir.path().join("db.sqlite"))
            .is_err());

        // Isolated portable executable does not assume self-update capability
        let portable = UpdaterService::new(
            dir.path().to_path_buf(),
            "1.0.5".into(),
            InstallMode::PortableOrDevelopment,
            Arc::new(MockTransport {
                release_json: Ok(Some(tampered_json)),
                texts: HashMap::new(),
                binary: Vec::new(),
            }),
            Arc::new(|_| {}),
        );
        let pst = portable.check_for_updates(false);
        assert_eq!(pst.state, "manual_required");
        assert!(!pst.can_auto_update);
        assert!(portable
            .prepare_install(&dir.path().join("db.sqlite"))
            .is_err());
    }

    #[test]
    fn install_layout_distinguishes_per_user_per_machine_and_isolated_binaries() {
        let user_dir = Path::new("C:\\Users\\demo\\AppData\\Local\\Programs\\Sheets Monitor");
        assert_eq!(
            classify_install_layout(user_dir, true, Some(user_dir), None),
            InstallMode::PerUserNsis
        );
        assert_eq!(
            classify_install_layout(user_dir, false, None, None),
            InstallMode::PortableOrDevelopment
        );
        let machine_dir = Path::new("C:\\Program Files\\Sheets Monitor");
        assert_eq!(
            classify_install_layout(machine_dir, true, None, Some(machine_dir)),
            InstallMode::PerMachineNsis
        );
    }
}
