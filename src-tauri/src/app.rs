//! Local IPC. Blocking OAuth and GET readers run outside the UI thread.
use crate::auth::{
    AuthConfigInput, AuthManager, HttpsOAuthTransport, IntegrationState, WindowsVault,
};
use crate::database::{AppSettings, Database};
use crate::integration::{
    CommandError, IntegrationRuntime, ScriptAccessResult, ScriptInput, SheetAccessResult,
    SheetInput,
};
use serde_json::{json, Value};
use std::sync::Arc;
use std::{fs, path::PathBuf};
use tauri::{Manager, State};

struct FoundationState {
    database_path: PathBuf,
    integration: Option<IntegrationRuntime>,
}

impl FoundationState {
    fn database(&self) -> Result<Database, String> {
        Database::open(&self.database_path)
    }

    fn dashboard(&self) -> Result<Value, String> {
        let database = self.database()?;
        Ok(json!({
            "spreadsheets": database.list_demo_spreadsheets()?,
            "activities": database.demo_history()?,
            "settings": database.settings()?,
            "database_version": database.schema_version()?,
            "is_demo": true
        }))
    }
}

#[tauri::command]
fn foundation_get_dashboard(state: State<'_, FoundationState>) -> Result<Value, String> {
    state.dashboard()
}

#[tauri::command]
fn foundation_save_settings(
    state: State<'_, FoundationState>,
    settings: AppSettings,
) -> Result<(), String> {
    if settings.start_with_windows {
        return Err("Inicialização com Windows será implementada em uma sprint futura".into());
    }
    state.database()?.save_settings(&settings)
}

fn runtime(state: &FoundationState) -> Result<IntegrationRuntime, CommandError> {
    state.integration.clone().ok_or_else(CommandError::local)
}
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, CommandError> + Send + 'static,
) -> Result<T, CommandError> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|_| CommandError::local())?
}
#[tauri::command]
async fn integration_get_state(
    state: State<'_, FoundationState>,
) -> Result<IntegrationState, CommandError> {
    let runtime = runtime(&state)?;
    blocking(move || {
        let next = runtime.auth.state()?;
        runtime.persist_state(&next)?;
        Ok(next)
    })
    .await
}
#[tauri::command]
async fn integration_configure(
    state: State<'_, FoundationState>,
    input: AuthConfigInput,
) -> Result<IntegrationState, CommandError> {
    let runtime = runtime(&state)?;
    blocking(move || {
        let next = runtime.auth.configure(input)?;
        runtime.persist_state(&next)?;
        Ok(next)
    })
    .await
}
#[tauri::command]
async fn integration_connect(
    state: State<'_, FoundationState>,
) -> Result<IntegrationState, CommandError> {
    let runtime = runtime(&state)?;
    blocking(move || {
        let next = runtime.auth.connect()?;
        runtime.persist_state(&next)?;
        Ok(next)
    })
    .await
}
#[tauri::command]
async fn integration_disconnect(
    state: State<'_, FoundationState>,
) -> Result<IntegrationState, CommandError> {
    let runtime = runtime(&state)?;
    blocking(move || Ok(runtime.auth.disconnect()?)).await
}
#[tauri::command]
async fn integration_revoke(
    state: State<'_, FoundationState>,
    confirmed: bool,
) -> Result<IntegrationState, CommandError> {
    let runtime = runtime(&state)?;
    blocking(move || Ok(runtime.auth.revoke(confirmed)?)).await
}
#[tauri::command]
async fn integration_test_sheet(
    state: State<'_, FoundationState>,
    input: SheetInput,
) -> Result<SheetAccessResult, CommandError> {
    let runtime = runtime(&state)?;
    blocking(move || runtime.sheet(input)).await
}
#[tauri::command]
async fn integration_test_script(
    state: State<'_, FoundationState>,
    input: ScriptInput,
) -> Result<ScriptAccessResult, CommandError> {
    let runtime = runtime(&state)?;
    blocking(move || runtime.script(input)).await
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            fs::create_dir_all(&directory)?;
            let state = FoundationState {
                database_path: directory.join("foundation-v1.db"),
                integration: Some(IntegrationRuntime {
                    database_path: directory.join("foundation-v1.db"),
                    auth: Arc::new(AuthManager::new(
                        Arc::new(WindowsVault::production()),
                        Arc::new(HttpsOAuthTransport::new()?),
                    )),
                }),
            };
            let mut database = state.database().map_err(std::io::Error::other)?;
            database.seed_demo().map_err(std::io::Error::other)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            foundation_get_dashboard,
            foundation_save_settings,
            integration_get_state,
            integration_configure,
            integration_connect,
            integration_disconnect,
            integration_revoke,
            integration_test_sheet,
            integration_test_script
        ])
        .run(tauri::generate_context!())
        .expect("Falha ao iniciar a fundação local do Sheets Monitor");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_serializes_persisted_examples_with_explicit_demo_marker() {
        let directory = tempfile::tempdir().unwrap();
        let state = FoundationState {
            database_path: directory.path().join("foundation-v1.db"),
            integration: None,
        };
        state.database().unwrap().seed_demo().unwrap();
        let dashboard = state.dashboard().unwrap();
        assert_eq!(dashboard["is_demo"], true);
        assert_eq!(dashboard["database_version"], 2);
        let sheets = dashboard["spreadsheets"].as_array().unwrap();
        assert!(!sheets.is_empty());
        assert!(sheets.iter().all(|sheet| sheet["is_demo"] == true));
        assert_eq!(dashboard["settings"]["interval_seconds"], 60);
        assert_eq!(dashboard["settings"]["start_with_windows"], false);
        let forbidden = ["access_token", "refresh_token", "client_secret"];
        let encoded = dashboard.to_string();
        assert!(forbidden.iter().all(|name| !encoded.contains(name)));
    }
}
