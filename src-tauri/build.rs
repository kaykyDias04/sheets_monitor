use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn valid_client_id(value: &str) -> bool {
    value.ends_with(".apps.googleusercontent.com")
        && value.len() <= 256
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_'))
}

fn read_oauth_json(path: &Path) -> (Option<String>, Option<String>) {
    let raw = fs::read_to_string(path).unwrap_or_else(|err| {
        panic!(
            "Falha ao ler configuração OAuth em {}: {err}",
            path.display()
        )
    });
    let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap_or_else(|err| {
        panic!("JSON OAuth inválido em {}: {err}", path.display());
    });
    if parsed.get("web").is_some() && parsed.get("installed").is_none() {
        panic!(
            "O arquivo OAuth em {} é do tipo 'web' (aplicação confidencial). Use exclusivamente um cliente OAuth do tipo Desktop App ('installed').",
            path.display()
        );
    }
    let root = parsed.get("installed").unwrap_or(&parsed);
    let client_id = root
        .get("client_id")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned);
    let client_secret = root
        .get("client_secret")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned);
    (client_id, client_secret)
}

fn configure_embedded_oauth(manifest_dir: &Path) {
    println!("cargo:rerun-if-env-changed=SHEETS_MONITOR_OAUTH_CLIENT_ID");
    println!("cargo:rerun-if-env-changed=SHEETS_MONITOR_OAUTH_DESKTOP_SECRET");
    println!("cargo:rerun-if-env-changed=SHEETS_MONITOR_OAUTH_CLIENT_JSON");
    println!("cargo:rerun-if-env-changed=SHEETS_MONITOR_REQUIRE_OAUTH");
    println!("cargo:rerun-if-env-changed=SHEETS_MONITOR_ALLOW_PUBLIC_ONLY");

    let public_config = manifest_dir.join("oauth-client.json");
    println!("cargo:rerun-if-changed={}", public_config.display());

    let repo_root = manifest_dir.parent().unwrap_or(manifest_dir);
    let local_candidates = [
        repo_root.join(".secrets").join("oauth-desktop.json"),
        repo_root.join(".secrets").join("client_secret.json"),
    ];
    for candidate in &local_candidates {
        println!("cargo:rerun-if-changed={}", candidate.display());
    }

    let mut client_id = env::var("SHEETS_MONITOR_OAUTH_CLIENT_ID")
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty());
    let mut client_secret = env::var("SHEETS_MONITOR_OAUTH_DESKTOP_SECRET")
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty());

    if let Ok(explicit_json) = env::var("SHEETS_MONITOR_OAUTH_CLIENT_JSON") {
        let trimmed = explicit_json.trim();
        if !trimmed.is_empty() {
            let json_path = PathBuf::from(trimmed);
            println!("cargo:rerun-if-changed={}", json_path.display());
            let (json_id, json_secret) = read_oauth_json(&json_path);
            if client_id.is_none() {
                client_id = json_id;
            }
            if client_secret.is_none() {
                client_secret = json_secret;
            }
        }
    }

    if client_id.is_none() || client_secret.is_none() {
        for candidate in &local_candidates {
            if candidate.is_file() {
                let (json_id, json_secret) = read_oauth_json(candidate);
                if client_id.is_none() {
                    client_id = json_id;
                }
                if client_secret.is_none() {
                    client_secret = json_secret;
                }
                break;
            }
        }
    }

    if (client_id.is_none() || client_secret.is_none()) && public_config.is_file() {
        let (public_id, public_secret) = read_oauth_json(&public_config);
        if client_id.is_none() {
            client_id = public_id;
        }
        if client_secret.is_none() {
            client_secret = public_secret;
        }
    }

    if let Some(ref id) = client_id {
        assert!(
            valid_client_id(id),
            "Client ID OAuth Desktop inválido: esperado sufixo '.apps.googleusercontent.com'."
        );
        println!("cargo:rustc-env=SHEETS_MONITOR_EMBEDDED_CLIENT_ID={id}");
    }
    if let Some(ref secret) = client_secret {
        assert!(
            secret.len() <= 16384 && !secret.chars().any(char::is_control),
            "Credencial complementar do cliente OAuth Desktop inválida."
        );
        println!("cargo:rustc-env=SHEETS_MONITOR_EMBEDDED_DESKTOP_SECRET={secret}");
    }

    let require_oauth = env::var("SHEETS_MONITOR_REQUIRE_OAUTH")
        .ok()
        .is_some_and(|v| matches!(v.trim(), "1" | "true" | "TRUE" | "yes"));
    let allow_public_only = env::var("SHEETS_MONITOR_ALLOW_PUBLIC_ONLY")
        .ok()
        .is_some_and(|v| matches!(v.trim(), "1" | "true" | "TRUE" | "yes"));

    if require_oauth {
        assert!(
            client_id.is_some(),
            "Build de distribuição bloqueado: nenhum Client ID OAuth Desktop foi provisionado (defina src-tauri/oauth-client.json, .secrets/oauth-desktop.json ou SHEETS_MONITOR_OAUTH_CLIENT_JSON)."
        );
        assert!(
            client_secret.is_some() || allow_public_only,
            "Build de distribuição bloqueado: o cliente OAuth Desktop ('installed') do Google requer o JSON completo do cliente Desktop em .secrets/oauth-desktop.json ou SHEETS_MONITOR_OAUTH_CLIENT_JSON (ou defina SHEETS_MONITOR_ALLOW_PUBLIC_ONLY=1 se o cliente não emitir secret)."
        );
    }
}

fn main() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"));

    // Tauri embeds the application's manifest in its product binary, while this
    // optional example needs its own Common Controls 6 activation context.
    if env::var_os("CARGO_FEATURE_NATIVE_PROBE").is_some()
        && env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        let manifest = manifest_dir.join("native-probe.manifest");
        println!("cargo:rerun-if-changed=native-probe.manifest");
        println!("cargo:rustc-link-arg-examples=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-examples=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }

    configure_embedded_oauth(&manifest_dir);

    if env::var_os("CARGO_FEATURE_DESKTOP").is_some() {
        println!("cargo:rerun-if-changed=../dist");
        tauri_build::build()
    }
}
