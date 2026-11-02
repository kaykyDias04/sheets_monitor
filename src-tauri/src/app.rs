//! Local IPC. Blocking OAuth and GET readers run outside the UI thread.
use crate::auth::{
    AuthConfigInput, AuthManager, HttpsOAuthTransport, IntegrationState, WindowsVault,
};
use crate::database::{AppSettings, Database};
use crate::integration::{
    CommandError, IntegrationRuntime, ScriptAccessResult, ScriptInput, SheetAccessResult,
    SheetInput,
};
use crate::integrity::runtime::{
    ApproveInput, BaselineSummary, CaptureInput, CapturePreview, EngineEnvelope, ScopeProposal,
    ScopeQuery, ValidationResult,
};
use crate::integrity::{BaselineVersion, IntegrityEvent, IntegrityReport, ScriptSnapshot};
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
async fn integration_set_drive_opt_in(
    state: State<'_, FoundationState>,
    enabled: bool,
) -> Result<IntegrationState, CommandError> {
    let runtime = runtime(&state)?;
    blocking(move || {
        let next = runtime.auth.set_drive_opt_in(enabled)?;
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

#[tauri::command]
async fn integrity_propose_scope(
    state: State<'_, FoundationState>,
    url: String,
) -> Result<EngineEnvelope<ScopeProposal>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.propose_integrity_scope(url)).await
}

#[tauri::command]
async fn discover_spreadsheets(
    state: State<'_, FoundationState>,
    input: crate::google::DiscoveryQuery,
) -> Result<EngineEnvelope<crate::google::DiscoveryPage>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.discover_spreadsheets(input)).await
}
#[tauri::command]
async fn integrity_capture_snapshot(
    state: State<'_, FoundationState>,
    input: CaptureInput,
) -> Result<EngineEnvelope<CapturePreview>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.capture_integrity_snapshot(input)).await
}
#[tauri::command]
async fn integrity_create_baseline(
    state: State<'_, FoundationState>,
    input: ApproveInput,
) -> Result<EngineEnvelope<BaselineVersion>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.approve_integrity_baseline(input)).await
}
#[tauri::command]
async fn integrity_list_baselines(
    state: State<'_, FoundationState>,
    input: ScopeQuery,
    offset: Option<u32>,
) -> Result<EngineEnvelope<Vec<BaselineSummary>>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.integrity_baselines(input, offset.unwrap_or(0))).await
}
#[tauri::command]
async fn integrity_get_active_baseline(
    state: State<'_, FoundationState>,
    input: ScopeQuery,
) -> Result<EngineEnvelope<Option<BaselineVersion>>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.integrity_active_baseline(input)).await
}
#[tauri::command]
async fn integrity_compare_snapshot(
    state: State<'_, FoundationState>,
    snapshot_id: String,
) -> Result<EngineEnvelope<ValidationResult>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.validate_integrity_snapshot(snapshot_id, false)).await
}
#[tauri::command]
async fn integrity_validate_integrity(
    state: State<'_, FoundationState>,
    snapshot_id: String,
) -> Result<EngineEnvelope<ValidationResult>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.validate_integrity_snapshot(snapshot_id, true)).await
}
#[tauri::command]
async fn integrity_analyze_script_dependencies(
    state: State<'_, FoundationState>,
    snapshot_id: String,
) -> Result<EngineEnvelope<Vec<ScriptSnapshot>>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.integrity_script_analysis(snapshot_id)).await
}
#[tauri::command]
async fn integrity_list_integrity_events(
    state: State<'_, FoundationState>,
    input: ScopeQuery,
) -> Result<EngineEnvelope<Vec<IntegrityEvent>>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.integrity_event_list(input)).await
}
#[tauri::command]
async fn integrity_get_integrity_report(
    state: State<'_, FoundationState>,
    input: ScopeQuery,
) -> Result<EngineEnvelope<Option<IntegrityReport>>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.integrity_report(input)).await
}
#[tauri::command]
async fn integrity_prune_history(
    state: State<'_, FoundationState>,
    input: ScopeQuery,
) -> Result<EngineEnvelope<usize>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.prune_integrity(input)).await
}

#[tauri::command]
async fn list_registered_spreadsheets(
    state: State<'_, FoundationState>,
) -> Result<EngineEnvelope<Vec<crate::management::RegisteredSpreadsheet>>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.list_registered_spreadsheets()).await
}

#[tauri::command]
async fn get_dashboard_summary(
    state: State<'_, FoundationState>,
) -> Result<EngineEnvelope<crate::management::DashboardSummary>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.get_dashboard_summary()).await
}

#[tauri::command]
async fn register_spreadsheet(
    state: State<'_, FoundationState>,
    input: crate::management::RegisterInput,
) -> Result<EngineEnvelope<crate::management::RegisteredSpreadsheet>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.register_spreadsheet(input)).await
}

#[tauri::command]
async fn list_spreadsheet_tabs(
    state: State<'_, FoundationState>,
    input: crate::management::SpreadsheetQuery,
) -> Result<EngineEnvelope<crate::management::RegisteredSpreadsheet>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.list_spreadsheet_tabs(input)).await
}

#[tauri::command]
async fn remove_spreadsheet(
    state: State<'_, FoundationState>,
    input: crate::management::RemoveInput,
) -> Result<EngineEnvelope<()>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.remove_spreadsheet(input)).await
}

#[tauri::command]
async fn pause_spreadsheet(
    state: State<'_, FoundationState>,
    input: crate::management::Mutation,
) -> Result<EngineEnvelope<crate::management::RegisteredSpreadsheet>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.pause_spreadsheet(input)).await
}

#[tauri::command]
async fn resume_spreadsheet(
    state: State<'_, FoundationState>,
    input: crate::management::Mutation,
) -> Result<EngineEnvelope<crate::management::RegisteredSpreadsheet>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.resume_spreadsheet(input)).await
}

#[tauri::command]
async fn update_monitored_tabs(
    state: State<'_, FoundationState>,
    input: crate::management::TabsInput,
) -> Result<EngineEnvelope<crate::management::RegisteredSpreadsheet>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.update_monitored_tabs(input)).await
}

#[tauri::command]
async fn update_integrity_rules(
    state: State<'_, FoundationState>,
    input: crate::management::RulesInput,
) -> Result<EngineEnvelope<crate::management::RegisteredSpreadsheet>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.update_integrity_rules(input)).await
}

#[tauri::command]
async fn preview_baseline(
    state: State<'_, FoundationState>,
    input: crate::management::Mutation,
) -> Result<EngineEnvelope<crate::management::OperationalPreview>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.preview_baseline(input)).await
}

#[tauri::command]
async fn approve_baseline(
    state: State<'_, FoundationState>,
    input: crate::management::ApprovalInput,
) -> Result<EngineEnvelope<BaselineVersion>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.approve_baseline(input)).await
}

#[tauri::command]
async fn run_spreadsheet_check(
    state: State<'_, FoundationState>,
    input: crate::management::Mutation,
) -> Result<EngineEnvelope<ValidationResult>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.run_spreadsheet_check(input)).await
}

#[tauri::command]
async fn get_spreadsheet_diagnostics(
    state: State<'_, FoundationState>,
    input: crate::management::SpreadsheetQuery,
) -> Result<EngineEnvelope<crate::management::Diagnostics>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.get_spreadsheet_diagnostics(input)).await
}

#[tauri::command]
async fn list_change_events(
    state: State<'_, FoundationState>,
    input: crate::management::EventFilter,
) -> Result<EngineEnvelope<crate::management::Page<crate::management::Activity>>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.list_change_events(input)).await
}

#[tauri::command]
async fn list_associated_scripts(
    state: State<'_, FoundationState>,
    input: crate::management::SpreadsheetQuery,
) -> Result<EngineEnvelope<Vec<crate::management::ScriptSummary>>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.list_associated_scripts(input)).await
}

#[tauri::command]
async fn list_baseline_versions(
    state: State<'_, FoundationState>,
    input: crate::management::BaselinesQuery,
) -> Result<EngineEnvelope<crate::management::Page<BaselineSummary>>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.list_baseline_versions(input)).await
}

#[tauri::command]
async fn compare_baseline_versions(
    state: State<'_, FoundationState>,
    input: crate::management::CompareBaselinesInput,
) -> Result<EngineEnvelope<crate::management::BaselineComparison>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.compare_baseline_versions(input)).await
}

#[tauri::command]
async fn get_management_preferences(
    state: State<'_, FoundationState>,
) -> Result<EngineEnvelope<crate::management::Preferences>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.get_management_preferences()).await
}

#[tauri::command]
async fn save_management_preferences(
    state: State<'_, FoundationState>,
    input: crate::management::PreferencesInput,
) -> Result<EngineEnvelope<crate::management::Preferences>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.save_management_preferences(input)).await
}

#[tauri::command]
async fn open_spreadsheet(
    state: State<'_, FoundationState>,
    input: crate::management::SpreadsheetQuery,
) -> Result<EngineEnvelope<()>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.open_spreadsheet(input)).await
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            fs::create_dir_all(&directory)?;
            let auth = Arc::new(AuthManager::new(
                Arc::new(WindowsVault::production()),
                Arc::new(HttpsOAuthTransport::new()?),
            ));
            // The maintainer provisions a Desktop client at build time. A local
            // development configuration is preserved and no consent is inferred.
            let provisioned =
                option_env!("SHEETS_MONITOR_OAUTH_CLIENT_ID").map(|id| AuthConfigInput {
                    client_id: id.into(),
                    client_secret: option_env!("SHEETS_MONITOR_OAUTH_DESKTOP_SECRET")
                        .map(str::to_owned),
                    drive_metadata_opt_in: false,
                });
            // Vault errors remain actionable through IPC without aborting the shell.
            // Invalid build provisioning leaves the application unconfigured.
            let _ = auth.configure_application(provisioned);
            let state = FoundationState {
                database_path: directory.join("foundation-v1.db"),
                integration: Some(IntegrationRuntime {
                    database_path: directory.join("foundation-v1.db"),
                    auth,
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
            integration_set_drive_opt_in,
            integration_disconnect,
            integration_revoke,
            integration_test_sheet,
            integration_test_script,
            discover_spreadsheets,
            open_spreadsheet,
            list_registered_spreadsheets,
            get_dashboard_summary,
            register_spreadsheet,
            list_spreadsheet_tabs,
            remove_spreadsheet,
            pause_spreadsheet,
            resume_spreadsheet,
            update_monitored_tabs,
            update_integrity_rules,
            preview_baseline,
            approve_baseline,
            run_spreadsheet_check,
            get_spreadsheet_diagnostics,
            list_change_events,
            list_associated_scripts,
            list_baseline_versions,
            compare_baseline_versions,
            get_management_preferences,
            save_management_preferences,
            integrity_propose_scope,
            integrity_capture_snapshot,
            integrity_create_baseline,
            integrity_list_baselines,
            integrity_get_active_baseline,
            integrity_compare_snapshot,
            integrity_validate_integrity,
            integrity_analyze_script_dependencies,
            integrity_list_integrity_events,
            integrity_get_integrity_report,
            integrity_prune_history
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
        assert_eq!(dashboard["database_version"], 4);
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
