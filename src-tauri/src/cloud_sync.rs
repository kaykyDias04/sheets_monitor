//! Immutable configuration commits in Drive appDataFolder; no Google document writes.
use crate::{
    database::Database,
    google::SheetMetadata,
    integration::{CommandError, IntegrationRuntime},
    integrity::runtime::EngineEnvelope,
    integrity::{BaselineVersion, MonitorScope},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    time::Duration,
};

pub const MAX_BYTES: usize = 8 * 1024 * 1024;
const MAX_COMMITS: usize = 2048;
const PREFIX: &str = "sheets-monitor-config-v1-";
pub type SyncResult<T> = Result<T, CommandError>;
pub(crate) fn error(code: &str) -> CommandError {
    let message=match code {
        "sync_busy"=>"Outra sincronização está em andamento.",
        "cloud_opt_in_required"=>"Ative a sincronização neste dispositivo antes de continuar.",
        "cloud_account"|"cloud_unauthorized"=>"Reconecte a conta Google para sincronizar.",
        "cloud_forbidden"=>"A conta não concedeu acesso à pasta privada de configuração.",
        "cloud_rate_limited"=>"A cota do Drive foi atingida. A sincronização aguardará antes de tentar novamente.",
        "cloud_network"|"cloud_server"=>"O serviço de sincronização está indisponível. A configuração local permanece preservada.",
        "cloud_local_changed"=>"A configuração local mudou durante a sincronização. Atualize e tente novamente.",
        "cloud_invalid_resolution"|"cloud_stale_resolution"=>"A decisão de conflito está desatualizada ou é inválida. Atualize as cópias disponíveis.",
        "cloud_response_limit"|"cloud_listing_limit"=>"A sincronização excedeu o limite seguro de arquivos ou tamanho. Nenhuma lista parcial foi aplicada.",
        "cloud_immutable_collision"=>"Uma referência recebida conflita com uma versão imutável existente. Nenhuma referência foi sobrescrita.",
        "cloud_missing_parent"|"cloud_invalid_graph"=>"O histórico remoto está incompleto ou inconsistente. A configuração local foi preservada.",
        "cloud_storage"|"cloud_backup_invalid"=>"Não foi possível salvar ou verificar a recuperação local. A alteração foi cancelada.",
        "cloud_invalid_data"|"cloud_invalid_baseline"|"cloud_invalid_commit"|"cloud_hash_mismatch"=>"A configuração recebida não passou pela validação de conta, referência ou integridade.",
        _=>"O Drive recusou a sincronização. A configuração local foi preservada.",
    };
    CommandError {
        code: code.into(),
        message: message.into(),
        retryable: matches!(
            code,
            "sync_busy"
                | "cloud_network"
                | "cloud_rate_limited"
                | "cloud_server"
                | "cloud_local_changed"
        ),
    }
}
pub(crate) fn encode<T: Serialize>(v: &T) -> SyncResult<String> {
    serde_json::to_string(v).map_err(|_| error("cloud_invalid_data"))
}
pub(crate) fn decode<T: serde::de::DeserializeOwned>(s: &str) -> SyncResult<T> {
    serde_json::from_str(s).map_err(|_| error("cloud_invalid_data"))
}
/// JSON objects use serde_json's sorted map representation, including nested objects.
pub fn canonical_hash<T: Serialize>(v: &T) -> SyncResult<String> {
    let value = serde_json::to_value(v).map_err(|_| error("cloud_invalid_data"))?;
    Ok(format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&value).map_err(|_| error("cloud_invalid_data"))?)
    ))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncPreferences {
    pub interval_seconds: u32,
    pub notifications_enabled: bool,
    pub history_days: u16,
    pub theme: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncRegistry {
    pub spreadsheet_id: String,
    pub name: String,
    pub paused: bool,
    pub removed: bool,
    pub scope: Option<MonitorScope>,
    pub selected_sheet_ids: Vec<u32>,
    pub tabs: Vec<SheetMetadata>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub registries: Vec<SyncRegistry>,
    pub preferences: SyncPreferences,
    pub baselines: Vec<BaselineVersion>,
}
impl Configuration {
    pub fn validate(&self, subject: &str) -> SyncResult<()> {
        if self.registries.iter().filter(|r| !r.removed).count() > 30
            || self.registries.len() > 10000
            || !(60..=600).contains(&self.preferences.interval_seconds)
            || !(1..=3650).contains(&self.preferences.history_days)
            || !matches!(self.preferences.theme.as_str(), "system" | "light" | "dark")
        {
            return Err(error("cloud_invalid_data"));
        }
        let mut ids = BTreeSet::new();
        for r in &self.registries {
            crate::google::validate_google_id(&r.spreadsheet_id)
                .map_err(|_| error("cloud_invalid_data"))?;
            if !ids.insert(&r.spreadsheet_id)
                || r.name.trim().is_empty()
                || r.name.len() > 4096
                || r.tabs.len() > crate::integrity::MAX_MONITORED_TABS
            {
                return Err(error("cloud_invalid_data"));
            }
            let tabs: BTreeSet<_> = r.tabs.iter().map(|t| t.sheet_id).collect();
            let selected: BTreeSet<_> = r.selected_sheet_ids.iter().copied().collect();
            if tabs.len() != r.tabs.len()
                || selected.len() != r.selected_sheet_ids.len()
                || r.selected_sheet_ids.len() > crate::integrity::MAX_MONITORED_TABS
            {
                return Err(error("cloud_invalid_data"));
            }
            if let Some(scope) = &r.scope {
                scope.validate().map_err(|_| error("cloud_invalid_data"))?;
            }
        }
        let mut baseline_ids = BTreeSet::new();
        let mut active = BTreeSet::new();
        for b in &self.baselines {
            crate::integrity::validate_baseline_snapshot(&b.snapshot)
                .map_err(|_| error("cloud_invalid_baseline"))?;
            let s = &b.snapshot;
            if s.account_subject != subject
                || !ids.contains(&s.spreadsheet_id)
                || b.version == 0
                || b.id.is_empty()
                || b.id.len() > 128
                || b.approval_origin != "human:native"
                || chrono::DateTime::parse_from_rfc3339(&b.approved_at).is_err()
                || !s.coverage.complete
                || !s.coverage.metadata_complete
                || s.coverage.issues.iter().any(|i| i.critical)
                || s.connectivity != crate::integrity::Connectivity::Reachable
                || !baseline_ids.insert(&b.id)
                || b.active && !active.insert((&s.spreadsheet_id, &s.scope_hash))
            {
                return Err(error("cloud_invalid_baseline"));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Commit {
    pub schema: u32,
    pub account_subject: String,
    pub parents: Vec<String>,
    pub device_id: String,
    pub created_at: String,
    pub payload: Configuration,
}
impl Commit {
    pub fn id(&self) -> SyncResult<String> {
        canonical_hash(self)
    }
    fn validate(&self, subject: &str) -> SyncResult<()> {
        if self.schema != 1
            || self.account_subject != subject
            || self.device_id.is_empty()
            || self.device_id.len() > 128
            || self.parents.len() > MAX_COMMITS
            || self.parents.windows(2).any(|p| p[0] >= p[1])
            || self.parents.iter().any(|p| !valid_hash(p))
            || chrono::DateTime::parse_from_rfc3339(&self.created_at).is_err()
            || encode(self)?.len() > MAX_BYTES
        {
            return Err(error("cloud_invalid_commit"));
        }
        self.payload.validate(subject)
    }
}
fn valid_hash(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncConflict {
    pub commit_id: String,
    pub created_at: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncStatus {
    pub state: String,
    pub message: String,
    pub conflicts: Vec<SyncConflict>,
    pub last_synced_at: Option<String>,
    #[serde(default)]
    pub conflict_token: Option<String>,
}
fn status(state: &str, conflicts: Vec<SyncConflict>, last_synced_at: Option<String>) -> SyncStatus {
    SyncStatus {
        state: state.into(),
        message: match state {
            "conflict" => {
                "Configurações divergentes preservadas. Escolha explicitamente qual manter."
            }
            "local_changed" => {
                "Configuração alterada durante a sincronização; nova tentativa necessária."
            }
            _ => "Configuração sincronizada.",
        }
        .into(),
        conflicts,
        last_synced_at,
        conflict_token: None,
    }
}
#[derive(Clone)]
pub struct RemoteFile {
    pub file_id: String,
    pub commit_id: String,
}
pub trait SyncTransport {
    fn list(&mut self) -> SyncResult<Vec<RemoteFile>>;
    fn download(&mut self, file: &RemoteFile) -> SyncResult<Vec<u8>>;
    fn upload(&mut self, id: &str, bytes: &[u8]) -> SyncResult<()>;
}
pub struct DriveTransport {
    client: reqwest::blocking::Client,
    token: zeroize::Zeroizing<String>,
}
impl DriveTransport {
    pub fn new(token: &str) -> SyncResult<Self> {
        Ok(Self {
            client: reqwest::blocking::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(30))
                .build()
                .map_err(|_| error("cloud_network"))?,
            token: zeroize::Zeroizing::new(token.into()),
        })
    }
    fn response(
        &self,
        response: Result<reqwest::blocking::Response, reqwest::Error>,
    ) -> SyncResult<Vec<u8>> {
        let response = response.map_err(|_| error("cloud_network"))?;
        let status = response.status();
        let retry_after = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|h| h.to_str().ok())
            .and_then(crate::google::parse_retry_after);
        if !status.is_success() {
            if let Some(delay) = retry_after {
                crate::monitoring::work::defer_retry(delay);
            }
        }
        if response
            .content_length()
            .is_some_and(|n| n > MAX_BYTES as u64)
        {
            return Err(error("cloud_response_limit"));
        }
        let mut bytes = Vec::new();
        response
            .take(MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| error("cloud_network"))?;
        if bytes.len() > MAX_BYTES {
            return Err(error("cloud_response_limit"));
        }
        if !status.is_success() {
            let quota = status.as_u16() == 429
                || status.as_u16() == 403 && crate::google::rate_limited(&bytes);
            if quota {
                crate::monitoring::work::defer_retry(
                    retry_after.unwrap_or(Duration::from_secs(60)),
                );
            }
            return Err(error(if quota {
                "cloud_rate_limited"
            } else {
                match status.as_u16() {
                    401 => "cloud_unauthorized",
                    403 => "cloud_forbidden",
                    500..=599 => "cloud_server",
                    _ => "cloud_http",
                }
            }));
        }
        Ok(bytes)
    }
}
impl SyncTransport for DriveTransport {
    fn list(&mut self) -> SyncResult<Vec<RemoteFile>> {
        #[derive(Deserialize)]
        struct File {
            id: String,
            name: String,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Page {
            #[serde(default)]
            files: Vec<File>,
            next_page_token: Option<String>,
            #[serde(default)]
            incomplete_search: bool,
        }
        let mut files = Vec::new();
        let mut token = String::new();
        let mut tokens = BTreeSet::new();
        for _ in 0..32 {
            let bytes = self.response(
                self.client
                    .get("https://www.googleapis.com/drive/v3/files")
                    .bearer_auth(self.token.as_str())
                    .query(&[
                        ("spaces", "appDataFolder"),
                        (
                            "q",
                            "trashed = false and name contains 'sheets-monitor-config-v1-'",
                        ),
                        ("pageSize", "100"),
                        ("fields", "files(id,name),nextPageToken,incompleteSearch"),
                        ("pageToken", token.as_str()),
                    ])
                    .send(),
            )?;
            let page: Page =
                serde_json::from_slice(&bytes).map_err(|_| error("cloud_invalid_data"))?;
            if page.incomplete_search {
                return Err(error("cloud_listing_limit"));
            }
            for f in page.files {
                if let Some(id) = f
                    .name
                    .strip_prefix(PREFIX)
                    .and_then(|s| s.strip_suffix(".json"))
                {
                    if valid_hash(id) {
                        crate::google::validate_google_id(&f.id)
                            .map_err(|_| error("cloud_invalid_data"))?;
                        files.push(RemoteFile {
                            file_id: f.id,
                            commit_id: id.into(),
                        });
                    }
                }
            }
            if files.len() > MAX_COMMITS {
                return Err(error("cloud_listing_limit"));
            }
            match page.next_page_token {
                None => return Ok(files),
                Some(next) if tokens.insert(next.clone()) => token = next,
                _ => return Err(error("cloud_listing_limit")),
            }
        }
        Err(error("cloud_listing_limit"))
    }
    fn download(&mut self, file: &RemoteFile) -> SyncResult<Vec<u8>> {
        crate::google::validate_google_id(&file.file_id)
            .map_err(|_| error("cloud_invalid_data"))?;
        self.response(
            self.client
                .get(format!(
                    "https://www.googleapis.com/drive/v3/files/{}",
                    file.file_id
                ))
                .bearer_auth(self.token.as_str())
                .query(&[("alt", "media")])
                .send(),
        )
    }
    fn upload(&mut self, id: &str, bytes: &[u8]) -> SyncResult<()> {
        if !valid_hash(id) || bytes.len() > MAX_BYTES {
            return Err(error("cloud_invalid_commit"));
        }
        let boundary = format!("sm-{}", uuid::Uuid::new_v4());
        let metadata = serde_json::json!({"name":format!("{PREFIX}{id}.json"),"parents":["appDataFolder"],"mimeType":"application/json"});
        let mut body=format!("--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{metadata}\r\n--{boundary}\r\nContent-Type: application/json\r\n\r\n").into_bytes();
        body.extend_from_slice(bytes);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        self.response(
            self.client
                .post("https://www.googleapis.com/upload/drive/v3/files")
                .bearer_auth(self.token.as_str())
                .query(&[("uploadType", "multipart"), ("fields", "id")])
                .header(
                    reqwest::header::CONTENT_TYPE,
                    format!("multipart/related; boundary={boundary}"),
                )
                .body(body)
                .send(),
        )?;
        Ok(())
    }
}
fn descends(commits: &BTreeMap<String, Commit>, head: &str, ancestor: &str) -> bool {
    let mut pending = vec![head];
    let mut seen = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if id == ancestor {
            return true;
        }
        if seen.insert(id) {
            if let Some(c) = commits.get(id) {
                pending.extend(c.parents.iter().map(String::as_str));
            }
        }
    }
    false
}
/// All network work occurs outside SQLite transactions. The caller supplies the session guard for the final CAS.
pub fn synchronize<T: SyncTransport>(
    db: &mut Database,
    subject: &str,
    transport: &mut T,
    resolution: Option<&str>,
    expected_conflict_token: Option<&str>,
    finish: impl FnOnce(&mut Database, &str, &str, Option<&Configuration>) -> SyncResult<SyncStatus>,
) -> SyncResult<SyncStatus> {
    if resolution
        .is_some_and(|r| r != "keep_local" && !r.strip_prefix("remote:").is_some_and(valid_hash))
    {
        return Err(error("cloud_invalid_resolution"));
    }
    let capture = db.cloud_capture(subject)?;
    let files = transport.list()?;
    if files.len() > MAX_COMMITS {
        return Err(error("cloud_listing_limit"));
    }
    let mut commits = BTreeMap::new();
    let mut total = 0usize;
    for file in files {
        if commits.contains_key(&file.commit_id) {
            continue;
        }
        let commit = match db.cloud_cached(subject, &file.commit_id)? {
            Some(c) => c,
            None => {
                let bytes = transport.download(&file)?;
                total = total.saturating_add(bytes.len());
                if bytes.len() > MAX_BYTES || total > 64 * 1024 * 1024 {
                    return Err(error("cloud_response_limit"));
                }
                serde_json::from_slice::<Commit>(&bytes)
                    .map_err(|_| error("cloud_invalid_commit"))?
            }
        };
        commit.validate(subject)?;
        if commit.id()? != file.commit_id {
            return Err(error("cloud_hash_mismatch"));
        }
        db.cloud_cache(subject, &file.commit_id, &commit)?;
        commits.insert(file.commit_id, commit);
    }
    for (id, c) in &commits {
        if c.parents
            .iter()
            .any(|p| p == id || !commits.contains_key(p))
        {
            return Err(error("cloud_missing_parent"));
        }
    }
    // Reject cycles before considering heads or ancestry.
    let mut remaining: BTreeSet<_> = commits.keys().cloned().collect();
    while !remaining.is_empty() {
        let ready: Vec<_> = remaining
            .iter()
            .filter(|id| commits[*id].parents.iter().all(|p| !remaining.contains(p)))
            .cloned()
            .collect();
        if ready.is_empty() {
            return Err(error("cloud_invalid_graph"));
        }
        for id in ready {
            remaining.remove(&id);
        }
    }
    let parent_ids: BTreeSet<_> = commits.values().flat_map(|c| c.parents.iter()).collect();
    let heads: Vec<_> = commits
        .keys()
        .filter(|id| !parent_ids.contains(id))
        .cloned()
        .collect();
    let local_hash = canonical_hash(&capture.payload)?;
    let conflict_token = canonical_hash(&(
        subject,
        &local_hash,
        &capture.base_commit,
        &capture.base_payload_hash,
        &heads,
    ))?;
    if resolution.is_some() && expected_conflict_token != Some(conflict_token.as_str()) {
        return Err(error("cloud_stale_resolution"));
    }
    let local_changed = capture.base_payload_hash.as_deref() != Some(&local_hash);
    let remote_unchanged = heads.len() == 1 && capture.base_commit.as_ref() == heads.first();
    let same = heads.len() == 1 && canonical_hash(&commits[&heads[0]].payload)? == local_hash;
    let empty = capture.payload.registries.is_empty()
        && capture.payload.baselines.is_empty()
        && capture.base_commit.is_none()
        && db.cloud_preferences_default(subject)?;
    let can_pull = heads.len() == 1
        && (same
            || empty
            || !local_changed
                && capture
                    .base_commit
                    .as_ref()
                    .is_some_and(|base| descends(&commits, &heads[0], base)));
    let conflict = !heads.is_empty() && !remote_unchanged && !can_pull;
    if resolution.is_none() && conflict {
        let mut result = status(
            "conflict",
            heads
                .iter()
                .map(|id| SyncConflict {
                    commit_id: id.clone(),
                    created_at: commits[id].created_at.clone(),
                })
                .collect(),
            capture.last_synced_at,
        );
        result.conflict_token = Some(conflict_token);
        return Ok(result);
    }
    if resolution.is_none() && can_pull {
        let id = &heads[0];
        return finish(
            db,
            &local_hash,
            id,
            if same {
                None
            } else {
                Some(&commits[id].payload)
            },
        );
    }
    if resolution.is_none() && remote_unchanged && !local_changed {
        return finish(db, &local_hash, &heads[0], None);
    }
    let selected = match resolution.and_then(|r| r.strip_prefix("remote:")) {
        Some(id) => {
            if !heads.iter().any(|h| h == id) {
                return Err(error("cloud_stale_resolution"));
            }
            Some(&commits[id].payload)
        }
        None => None,
    };
    let mut parents = heads;
    if let Some(base) = capture.base_commit {
        if commits.contains_key(&base) && !parents.contains(&base) {
            parents.push(base);
        }
    }
    parents.sort();
    parents.dedup();
    let commit = Commit {
        schema: 1,
        account_subject: subject.into(),
        parents,
        device_id: capture.device_id,
        created_at: chrono::Utc::now().to_rfc3339(),
        payload: selected.unwrap_or(&capture.payload).clone(),
    };
    commit.validate(subject)?;
    let id = commit.id()?;
    // A retry of an ambiguous upload is harmless: immutable, content-addressed files are deduplicated during listing.
    transport.upload(&id, encode(&commit)?.as_bytes())?;
    db.cloud_cache(subject, &id, &commit)?;
    finish(db, &local_hash, &id, selected)
}
static SYNC_GATES: std::sync::Mutex<BTreeSet<std::path::PathBuf>> =
    std::sync::Mutex::new(BTreeSet::new());
struct SyncGate(std::path::PathBuf);
impl SyncGate {
    fn acquire(path: &std::path::Path) -> SyncResult<Self> {
        let path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if !SYNC_GATES
            .lock()
            .map_err(|_| error("sync_busy"))?
            .insert(path.clone())
        {
            return Err(error("sync_busy"));
        }
        Ok(Self(path))
    }
}
impl Drop for SyncGate {
    fn drop(&mut self) {
        if let Ok(mut gates) = SYNC_GATES.lock() {
            gates.remove(&self.0);
        }
    }
}
impl IntegrationRuntime {
    pub fn sync_configuration(
        &self,
        expected_generation: u64,
        resolution: Option<String>,
        expected_conflict_token: Option<String>,
    ) -> SyncResult<EngineEnvelope<SyncStatus>> {
        let _gate = SyncGate::acquire(&self.database_path)?;
        if !self.auth.features()?.0 {
            return Err(error("cloud_opt_in_required"));
        }
        let session = self.auth.authorized()?;
        if session.generation != expected_generation {
            return Err(
                crate::auth::AuthError::new(crate::auth::AuthCode::OperationSuperseded).into(),
            );
        }
        if !session.has_scope(crate::auth::CLOUD_SCOPE) {
            return Err(
                crate::auth::AuthError::new(crate::auth::AuthCode::InsufficientScope).into(),
            );
        }
        let mut db = Database::open(&self.database_path).map_err(|_| CommandError::local())?;
        self.auth.with_current(&session, || {
            db.upsert_account(
                &session.account.subject,
                session.account.email.as_deref(),
                session.account.name.as_deref(),
            )
            .map_err(|_| CommandError::local())?;
            Ok::<_, CommandError>(())
        })??;
        let mut transport = DriveTransport::new(session.access_token())?;
        let result = synchronize(
            &mut db,
            &session.account.subject,
            &mut transport,
            resolution.as_deref(),
            expected_conflict_token.as_deref(),
            |db, hash, id, payload| {
                self.auth.with_current(&session, || {
                    db.cloud_finish(&session.account.subject, hash, id, payload)
                })?
            },
        );
        let data = match result {
            Ok(data) => data,
            Err(failure) => {
                self.auth.with_current(&session, || {
                    let mut saved = db.cloud_status(&session.account.subject)?;
                    saved.state = if failure.retryable {
                        "offline"
                    } else {
                        "error"
                    }
                    .into();
                    saved.message = failure.message.clone();
                    db.cloud_save_status(&session.account.subject, &saved)
                })??;
                return Err(failure);
            }
        };
        self.auth.with_current(&session, || {
            db.cloud_save_status(&session.account.subject, &data)?;
            Ok(EngineEnvelope {
                generation: session.generation,
                account_subject: session.account.subject.clone(),
                data,
            })
        })?
    }
    pub fn cloud_sync_status(&self) -> SyncResult<EngineEnvelope<SyncStatus>> {
        let enabled = self.auth.features()?.0;
        let state = self.auth.state()?;
        let subject = state
            .auth
            .account
            .as_ref()
            .ok_or_else(|| error("cloud_account"))?
            .subject
            .clone();
        let db = Database::open(&self.database_path).map_err(|_| CommandError::local())?;
        let mut data = db.cloud_status(&subject)?;
        if !enabled {
            data.state = "disabled".into();
            data.message = "Sincronização desativada neste dispositivo.".into();
        }
        self.auth.with_generation(state.generation, || {
            Ok(EngineEnvelope {
                generation: state.generation,
                account_subject: subject,
                data,
            })
        })?
    }
}
