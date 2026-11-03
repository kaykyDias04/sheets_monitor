//! Local IPC. Blocking OAuth and GET readers run outside the UI thread.
use crate::auth::{
    AuthConfigInput, AuthManager, HttpsOAuthTransport, IntegrationState, WindowsVault,
};
use crate::database::{AppSettings, Database};
use crate::desktop::{DesktopCallbacks, DesktopShell, DesktopStatus, NativeAction};
use crate::integration::{
    CommandError, IntegrationRuntime, ScriptAccessResult, ScriptInput, SheetAccessResult,
    SheetInput,
};
use crate::integrity::runtime::{
    ApproveInput, BaselineSummary, CaptureInput, CapturePreview, EngineEnvelope, ScopeProposal,
    ScopeQuery, ValidationResult,
};
use crate::integrity::{BaselineVersion, IntegrityEvent, IntegrityReport, ScriptSnapshot};
use crate::monitoring::{
    MonitoringControl, MonitoringSchedule, MonitoringService, NotificationRecord, ServiceStatus,
};
use crate::notifications::native::NativeNotifier;
use crate::notifications::NotificationService;
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::Arc;
use std::{fs, path::PathBuf, time::Duration};
use tauri::{Emitter, Manager, State};

struct FoundationState {
    database_path: PathBuf,
    integration: Option<IntegrationRuntime>,
    monitoring: Option<Arc<MonitoringService>>,
    notifications: Option<Arc<NotificationService>>,
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
        return Err("Use a preferência explícita de inicialização com Windows.".into());
    }
    state.database()?.save_settings(&settings)
}

fn runtime(state: &FoundationState) -> Result<IntegrationRuntime, CommandError> {
    state.integration.clone().ok_or_else(CommandError::local)
}
#[derive(Serialize)]
struct MonitoringStatus {
    control: MonitoringControl,
    schedules: Vec<MonitoringSchedule>,
    eligible_count: usize,
    interval_seconds: u32,
    service: ServiceStatus,
}
fn permitted_candidates(
    db: &Database,
    subject: &str,
    session: &IntegrationState,
) -> Result<Vec<crate::monitoring::MonitoringCandidate>, CommandError> {
    if session.auth.status != "connected"
        || !session
            .auth
            .granted_scopes
            .iter()
            .any(|scope| scope == crate::auth::SHEETS_SCOPE)
    {
        return Ok(vec![]);
    }
    let scripts = session
        .auth
        .granted_scopes
        .iter()
        .any(|scope| scope == crate::auth::SCRIPT_SCOPE);
    Ok(db
        .monitoring_candidates(subject)?
        .into_iter()
        .filter(|candidate| candidate.scope.scripts.is_empty() || scripts)
        .collect())
}
fn monitoring_status(
    rt: &IntegrationRuntime,
    service: &MonitoringService,
) -> Result<EngineEnvelope<MonitoringStatus>, CommandError> {
    let session = rt.auth.state()?;
    rt.management_local(Some(session.generation), |db, subject| {
        let candidates = permitted_candidates(db, subject, &session)?;
        let schedules = db
            .monitoring_schedules(subject)?
            .into_iter()
            .filter(|row| {
                candidates
                    .iter()
                    .any(|candidate| candidate.spreadsheet_id == row.spreadsheet_id)
            })
            .collect();
        Ok(MonitoringStatus {
            control: db.monitoring_control(subject)?,
            schedules,
            eligible_count: candidates.len(),
            interval_seconds: db.management_preferences(subject)?.interval_seconds,
            service: service.status(),
        })
    })
}
#[tauri::command]
async fn monitoring_get_status(
    state: State<'_, FoundationState>,
) -> Result<EngineEnvelope<MonitoringStatus>, CommandError> {
    let rt = runtime(&state)?;
    let service = state.monitoring.clone().ok_or_else(CommandError::local)?;
    blocking(move || monitoring_status(&rt, &service)).await
}
#[tauri::command]
async fn monitoring_set_enabled(
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    enabled: bool,
    expected_generation: u64,
) -> Result<EngineEnvelope<MonitoringStatus>, CommandError> {
    let rt = runtime(&state)?;
    let service = state.monitoring.clone().ok_or_else(CommandError::local)?;
    blocking(move || {
        rt.management_local(Some(expected_generation), |db, subject| {
            db.set_monitoring_enabled(subject, enabled, chrono::Utc::now().timestamp_millis())?;
            Ok(())
        })?;
        service.cancel_active();
        service.wake();
        emit_changed(&app);
        monitoring_status(&rt, &service)
    })
    .await
}
#[tauri::command]
async fn monitoring_check_now(
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    expected_generation: u64,
) -> Result<EngineEnvelope<usize>, CommandError> {
    let rt = runtime(&state)?;
    let service = state.monitoring.clone().ok_or_else(CommandError::local)?;
    blocking(move || {
        let session = rt.auth.state()?;
        let queued = rt.management_local(Some(expected_generation), |db, subject| {
            let candidates = permitted_candidates(db, subject, &session)?;
            for candidate in &candidates {
                service.request_check_for(
                    subject,
                    session.generation,
                    candidate.spreadsheet_id.clone(),
                )?;
            }
            Ok(candidates.len())
        })?;
        emit_changed(&app);
        Ok(queued)
    })
    .await
}
#[tauri::command]
async fn monitoring_check_file(
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    input: crate::management::Mutation,
) -> Result<EngineEnvelope<usize>, CommandError> {
    let rt = runtime(&state)?;
    let service = state.monitoring.clone().ok_or_else(CommandError::local)?;
    blocking(move || {
        let session = rt.auth.state()?;
        let queued = rt.management_local(Some(input.expected_generation), |db, subject| {
            let candidates = permitted_candidates(db, subject, &session)?;
            let candidate = candidates
                .iter()
                .find(|candidate| candidate.spreadsheet_id == input.spreadsheet_id)
                .ok_or_else(|| CommandError {
                    code: "configuration_pending".into(),
                    message: "Revise as permissões, o escopo atual e a referência desta planilha."
                        .into(),
                    retryable: false,
                })?;
            if candidate.registry_revision != input.expected_revision {
                return Err(CommandError {
                    code: "configuration_conflict".into(),
                    message: "A configuração mudou. Atualize a planilha antes de verificar.".into(),
                    retryable: false,
                });
            }
            service.request_check_for(
                subject,
                session.generation,
                candidate.spreadsheet_id.clone(),
            )?;
            Ok(1)
        })?;
        emit_changed(&app);
        Ok(queued)
    })
    .await
}
#[tauri::command]
async fn monitoring_notification_history(
    state: State<'_, FoundationState>,
    offset: u32,
    limit: u32,
) -> Result<EngineEnvelope<crate::management::Page<NotificationRecord>>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || {
        rt.management_local(None, |db, subject| {
            db.notification_history(subject, offset, limit)
        })
    })
    .await
}
#[tauri::command]
async fn monitoring_notification(
    state: State<'_, FoundationState>,
    id: String,
) -> Result<EngineEnvelope<Option<NotificationRecord>>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.management_local(None, |db, subject| db.notification_by_id(subject, &id)))
        .await
}
fn refresh_tray(app: tauri::AppHandle) {
    tauri::async_runtime::spawn_blocking(move || {
        let Some(shell) = app.try_state::<DesktopShell>() else {
            return;
        };
        let Some(state) = app.try_state::<FoundationState>() else {
            return;
        };
        let Ok(rt) = runtime(&state) else {
            shell.set_monitoring_state("fault");
            return;
        };
        let Some(service) = &state.monitoring else {
            return;
        };
        let label = match monitoring_status(&rt, service) {
            Ok(status)
                if !status.data.service.running
                    || matches!(
                        status.data.service.warning.as_deref(),
                        Some("local_storage" | "worker_unavailable")
                    ) =>
            {
                "fault"
            }
            Ok(status) if !status.data.control.enabled => "paused",
            Ok(status)
                if status.data.service.warning.is_some()
                    || status
                        .data
                        .schedules
                        .iter()
                        .any(|row| row.operational_code.is_some()) =>
            {
                "indeterminate"
            }
            Ok(status) if status.data.eligible_count > 0 => "running",
            Err(error) if error.code == "local_storage" => "fault",
            _ => "indeterminate",
        };
        shell.set_monitoring_state(label);
    });
}
fn emit_changed(app: &tauri::AppHandle) {
    let _ = app.emit("monitoring:changed", ());
    refresh_tray(app.clone());
    if let Some(state) = app.try_state::<FoundationState>() {
        if let Some(service) = &state.notifications {
            service.wake();
        }
    }
}
fn wake_services(app: &tauri::AppHandle) {
    if let Some(state) = app.try_state::<FoundationState>() {
        if let Some(service) = &state.monitoring {
            service.wake();
        }
        if let Some(service) = &state.notifications {
            service.wake();
        }
    }
    emit_changed(app);
}
#[tauri::command]
fn desktop_get_status(shell: State<'_, DesktopShell>) -> DesktopStatus {
    shell.status()
}
#[tauri::command]
async fn desktop_set_startup(
    shell: State<'_, DesktopShell>,
    enabled: bool,
) -> Result<DesktopStatus, CommandError> {
    let shell = shell.inner().clone();
    blocking(move || {
        shell.set_startup(enabled).map_err(|code| CommandError {
            code,
            message:
                "Não foi possível alterar a inicialização com Windows. Consulte o estado observado."
                    .into(),
            retryable: false,
        })
    })
    .await
}
#[tauri::command]
fn desktop_exit(shell: State<'_, DesktopShell>) {
    shell.request_exit();
}
fn handle_native_action(app: tauri::AppHandle, action: NativeAction) {
    if matches!(action, NativeAction::Open | NativeAction::Settings) {
        if let Some(shell) = app.try_state::<DesktopShell>() {
            shell.show_main();
        }
        let _ = app.emit(
            "desktop:navigate",
            if matches!(action, NativeAction::Settings) {
                "settings"
            } else {
                "overview"
            },
        );
        return;
    }
    if matches!(action, NativeAction::Exit) {
        if let Some(shell) = app.try_state::<DesktopShell>() {
            shell.request_exit();
        }
        return;
    }
    let Some(state) = app.try_state::<FoundationState>() else {
        return;
    };
    let Ok(rt) = runtime(&state) else {
        return;
    };
    let Some(service) = state.monitoring.clone() else {
        return;
    };
    tauri::async_runtime::spawn_blocking(move || {
        let Ok(session) = rt.auth.state() else {
            return;
        };
        if session.auth.status != "connected" {
            emit_changed(&app);
            return;
        }
        let result = rt.management_local(Some(session.generation), |db, subject| {
            match action {
                NativeAction::CheckNow => {
                    for candidate in permitted_candidates(db, subject, &session)? {
                        service.request_check_for(
                            subject,
                            session.generation,
                            candidate.spreadsheet_id,
                        )?;
                    }
                }
                NativeAction::Pause | NativeAction::Resume => {
                    db.set_monitoring_enabled(
                        subject,
                        matches!(action, NativeAction::Resume),
                        chrono::Utc::now().timestamp_millis(),
                    )?;
                }
                _ => {}
            }
            Ok(())
        });
        if result.is_ok() {
            if matches!(action, NativeAction::Pause | NativeAction::Resume) {
                service.cancel_active();
            }
            service.wake();
        }
        emit_changed(&app);
    });
}
fn open_current_notification(app: tauri::AppHandle, rt: IntegrationRuntime, id: String) {
    tauri::async_runtime::spawn_blocking(move || {
        let Ok(session) = rt.auth.state() else {
            return;
        };
        if session.auth.status != "connected" {
            return;
        }
        let Some(account) = session.auth.account else {
            return;
        };
        let _ = rt.auth.with_generation(session.generation, || {
            let db = Database::open(&rt.database_path).map_err(|_| CommandError::local())?;
            if db.notification_by_id(&account.subject, &id)?.is_some() {
                if let Some(shell) = app.try_state::<DesktopShell>() {
                    shell.show_main();
                }
                let _ = app.emit("notification:open", id);
            }
            Ok::<(), CommandError>(())
        });
    });
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
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    input: AuthConfigInput,
) -> Result<IntegrationState, CommandError> {
    let runtime = runtime(&state)?;
    blocking(move || {
        let next = runtime.auth.configure(input)?;
        runtime.persist_state(&next)?;
        wake_services(&app);
        Ok(next)
    })
    .await
}
#[tauri::command]
async fn integration_connect(
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
) -> Result<IntegrationState, CommandError> {
    let runtime = runtime(&state)?;
    blocking(move || {
        let next = runtime.auth.connect()?;
        runtime.persist_state(&next)?;
        wake_services(&app);
        Ok(next)
    })
    .await
}
#[tauri::command]
async fn integration_set_drive_opt_in(
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    enabled: bool,
) -> Result<IntegrationState, CommandError> {
    let runtime = runtime(&state)?;
    blocking(move || {
        let next = runtime.auth.set_drive_opt_in(enabled)?;
        runtime.persist_state(&next)?;
        wake_services(&app);
        Ok(next)
    })
    .await
}
#[tauri::command]
async fn integration_disconnect(
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
) -> Result<IntegrationState, CommandError> {
    let runtime = runtime(&state)?;
    if let Some(service) = &state.monitoring {
        service.cancel_active();
    }
    blocking(move || {
        let next = runtime.auth.disconnect()?;
        wake_services(&app);
        Ok(next)
    })
    .await
}
#[tauri::command]
async fn integration_revoke(
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    confirmed: bool,
) -> Result<IntegrationState, CommandError> {
    let runtime = runtime(&state)?;
    if let Some(service) = &state.monitoring {
        service.cancel_active();
    }
    blocking(move || {
        let next = runtime.auth.revoke(confirmed)?;
        wake_services(&app);
        Ok(next)
    })
    .await
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
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    input: crate::management::RegisterInput,
) -> Result<EngineEnvelope<crate::management::RegisteredSpreadsheet>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || {
        let value = rt.register_spreadsheet(input);
        wake_services(&app);
        value
    })
    .await
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
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    input: crate::management::RemoveInput,
) -> Result<EngineEnvelope<()>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || {
        let value = rt.remove_spreadsheet(input);
        wake_services(&app);
        value
    })
    .await
}

#[tauri::command]
async fn pause_spreadsheet(
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    input: crate::management::Mutation,
) -> Result<EngineEnvelope<crate::management::RegisteredSpreadsheet>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || {
        let value = rt.pause_spreadsheet(input);
        wake_services(&app);
        value
    })
    .await
}

#[tauri::command]
async fn resume_spreadsheet(
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    input: crate::management::Mutation,
) -> Result<EngineEnvelope<crate::management::RegisteredSpreadsheet>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || {
        let value = rt.resume_spreadsheet(input);
        wake_services(&app);
        value
    })
    .await
}

#[tauri::command]
async fn update_monitored_tabs(
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    input: crate::management::TabsInput,
) -> Result<EngineEnvelope<crate::management::RegisteredSpreadsheet>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || {
        let value = rt.update_monitored_tabs(input);
        wake_services(&app);
        value
    })
    .await
}

#[tauri::command]
async fn update_integrity_rules(
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    input: crate::management::RulesInput,
) -> Result<EngineEnvelope<crate::management::RegisteredSpreadsheet>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || {
        let value = rt.update_integrity_rules(input);
        wake_services(&app);
        value
    })
    .await
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
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    input: crate::management::ApprovalInput,
) -> Result<EngineEnvelope<BaselineVersion>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || {
        let value = rt.approve_baseline(input);
        wake_services(&app);
        value
    })
    .await
}

#[tauri::command]
async fn run_spreadsheet_check(
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    input: crate::management::Mutation,
) -> Result<EngineEnvelope<ValidationResult>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || {
        let value = rt.run_spreadsheet_check(input);
        emit_changed(&app);
        value
    })
    .await
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
    app: tauri::AppHandle,
    state: State<'_, FoundationState>,
    input: crate::management::PreferencesInput,
) -> Result<EngineEnvelope<crate::management::Preferences>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || {
        let value = rt.save_management_preferences(input)?;
        wake_services(&app);
        Ok(value)
    })
    .await
}

#[tauri::command]
async fn open_spreadsheet(
    state: State<'_, FoundationState>,
    input: crate::management::SpreadsheetQuery,
) -> Result<EngineEnvelope<()>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.open_spreadsheet(input)).await
}

#[tauri::command]
async fn propose_integrity_rules(
    state: State<'_, FoundationState>,
    input: crate::management::Mutation,
) -> Result<EngineEnvelope<crate::integrity::MonitorScope>, CommandError> {
    let rt = runtime(&state)?;
    blocking(move || rt.propose_integrity_rules(input)).await
}

pub fn run() {
    crate::desktop::builder()
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
            let rt = IntegrationRuntime {
                database_path: directory.join("foundation-v1.db"),
                auth,
            };
            let mut database = Database::open(&rt.database_path).map_err(std::io::Error::other)?;
            database.seed_demo().map_err(std::io::Error::other)?;
            // Vault/configuration errors stay visible through the shell without fake login.
            if let Ok(session) = rt.auth.state() {
                let _ = rt.persist_state(&session);
            }
            let changed_app = app.handle().clone();
            let changed: Arc<dyn Fn() + Send + Sync> = Arc::new(move || emit_changed(&changed_app));
            let open_app = app.handle().clone();
            let open_rt = rt.clone();
            let sink = Arc::new(NativeNotifier::new(
                app.handle().clone(),
                Arc::new(move |id| {
                    open_current_notification(open_app.clone(), open_rt.clone(), id)
                }),
            ));
            let notifications = NotificationService::start(rt.clone(), sink, changed.clone());
            let monitoring = MonitoringService::start(rt.clone(), changed);
            app.manage(FoundationState {
                database_path: rt.database_path.clone(),
                integration: Some(rt.clone()),
                monitoring: Some(monitoring.clone()),
                notifications: Some(notifications.clone()),
            });
            let action_app = app.handle().clone();
            let shell = crate::desktop::install(
                app.handle(),
                DesktopCallbacks {
                    action: Arc::new(move |action| {
                        handle_native_action(action_app.clone(), action)
                    }),
                    shutdown: Arc::new(move || {
                        monitoring.cancel_active();
                        let _ = notifications.shutdown(Duration::from_secs(2));
                        let _ = monitoring.shutdown(Duration::from_secs(2));
                    }),
                },
            );
            // This flag never enables startup. Hide only with a managed actual registration.
            let startup = shell.status();
            let from_startup = std::env::args().any(|arg| arg == "--from-autostart");
            let minimize = from_startup
                && startup.tray_available
                && startup.startup.registered
                && startup.startup.managed
                && rt
                    .management_local(None, |db, subject| {
                        Ok(db.management_preferences(subject)?.run_minimized)
                    })
                    .is_ok_and(|v| v.data);
            app.manage(shell.clone());
            emit_changed(app.handle());
            if minimize {
                shell.hide_for_startup();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_get_status,
            desktop_set_startup,
            desktop_exit,
            foundation_get_dashboard,
            foundation_save_settings,
            monitoring_get_status,
            monitoring_set_enabled,
            monitoring_check_now,
            monitoring_check_file,
            monitoring_notification_history,
            monitoring_notification,
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
            propose_integrity_rules,
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
        .build(tauri::generate_context!())
        .expect("Falha ao iniciar a fundação local do Sheets Monitor")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                if let Some(shell) = app.try_state::<DesktopShell>() {
                    shell.handle_exit_requested(&api);
                }
            }
        });
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
            monitoring: None,
            notifications: None,
        };
        state.database().unwrap().seed_demo().unwrap();
        let dashboard = state.dashboard().unwrap();
        assert_eq!(dashboard["is_demo"], true);
        assert_eq!(dashboard["database_version"], 5);
        let sheets = dashboard["spreadsheets"].as_array().unwrap();
        assert!(!sheets.is_empty());
        assert!(sheets.iter().all(|sheet| sheet["is_demo"] == true));
        assert_eq!(dashboard["settings"]["interval_seconds"], 60);
        assert_eq!(dashboard["settings"]["start_with_windows"], false);
        let forbidden = ["access_token", "refresh_token", "client_secret"];
        let encoded = dashboard.to_string();
        assert!(forbidden.iter().all(|name| !encoded.contains(name)));
    }
    #[test]
    fn eligible_ui_and_queue_candidates_require_current_read_only_grants() {
        use crate::{
            auth::{AccountIdentity, AuthStatus, SCRIPT_SCOPE, SHEETS_SCOPE},
            google::{SheetMetadata, SpreadsheetMetadata},
            integrity::fixtures,
            management::Mutation,
        };
        let mut db = Database::memory().unwrap();
        db.upsert_account("fixture_account", None, None).unwrap();
        let metadata = SpreadsheetMetadata {
            spreadsheet_id: "fixture_sheet".into(),
            spreadsheet_url: None,
            title: "Synthetic scope permissions".into(),
            locale: None,
            time_zone: None,
            sheets: vec![SheetMetadata {
                sheet_id: 7,
                title: "Data".into(),
                index: 0,
                sheet_type: Some("GRID".into()),
                row_count: Some(100),
                column_count: Some(4),
                hidden: false,
            }],
        };
        let row = db
            .register_operational_spreadsheet("fixture_account", &metadata)
            .unwrap();
        let mutation = |revision| Mutation {
            spreadsheet_id: "fixture_sheet".into(),
            expected_generation: 7,
            expected_revision: revision,
        };
        let row = db
            .set_registered_tabs("fixture_account", &mutation(row.revision), &[7])
            .unwrap();
        let mut snapshot = fixtures::snapshot();
        fixtures::attach_script(&mut snapshot);
        for (service, resource_id, associated) in [
            (
                crate::database::QueryService::SheetsMetadata,
                "fixture_sheet",
                None,
            ),
            (
                crate::database::QueryService::AppsScript,
                "fixture_script",
                Some("fixture_sheet".to_owned()),
            ),
        ] {
            db.record_query(
                "fixture_account",
                &crate::database::IntegrationQueryInput {
                    service,
                    resource_id: resource_id.into(),
                    resource_name: Some("Synthetic permission fixture".into()),
                    associated_spreadsheet_id: associated,
                    source_hash: None,
                    error: None,
                },
            )
            .unwrap();
        }
        let row = db
            .set_registered_rules("fixture_account", &mutation(row.revision), &snapshot.scope)
            .unwrap();
        db.save_structural_snapshot(&snapshot).unwrap();
        db.approve_registered_structural_baseline(
            "fixture_account",
            &snapshot.id,
            0,
            true,
            "synthetic:test",
            Some(row.revision),
        )
        .unwrap();
        assert_eq!(
            db.monitoring_candidates("fixture_account").unwrap().len(),
            1
        );
        let mut session = IntegrationState {
            configured: true,
            client_id: None,
            drive_metadata_opt_in: false,
            generation: 7,
            auth: AuthStatus {
                status: "connected",
                account: Some(AccountIdentity {
                    subject: "fixture_account".into(),
                    email: None,
                    name: None,
                }),
                granted_scopes: vec![SHEETS_SCOPE.into()],
                expires_at: None,
            },
        };
        assert!(permitted_candidates(&db, "fixture_account", &session)
            .unwrap()
            .is_empty());
        session.auth.granted_scopes.push(SCRIPT_SCOPE.into());
        assert_eq!(
            permitted_candidates(&db, "fixture_account", &session)
                .unwrap()
                .len(),
            1
        );
        session
            .auth
            .granted_scopes
            .retain(|scope| scope != SHEETS_SCOPE);
        assert!(permitted_candidates(&db, "fixture_account", &session)
            .unwrap()
            .is_empty());
        session.auth.granted_scopes.push(SHEETS_SCOPE.into());
        session.auth.status = "reconnect_required";
        assert!(permitted_candidates(&db, "fixture_account", &session)
            .unwrap()
            .is_empty());
    }
}
