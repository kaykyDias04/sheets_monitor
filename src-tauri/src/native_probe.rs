//! Feature-gated native QA. All identities, baselines and observations are synthetic.
//! Never opens production app data or Windows credentials and never calls Google.
use crate::{
    auth::{
        AccountIdentity, AuthError, AuthManager, CredentialVault, OAuthTransport, TokenRequest,
        TokenResponse, SHEETS_SCOPE,
    },
    database::Database,
    desktop::{self, DesktopCallbacks, DesktopShell, NativeAction},
    integration::{CommandError, IntegrationRuntime},
    integrity::{
        evaluate, fixtures,
        runtime::{EngineEnvelope, ValidationResult},
    },
    management::{Mutation, Preferences},
    monitoring::{
        work, CheckExecutor, MonitoringClock, MonitoringService, NotificationOutcome,
        NotificationRecord,
    },
    notifications::{native::NativeNotifier, service::NotificationSink},
};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicI64, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tauri::Manager;
use zeroize::Zeroizing;

const SUBJECT: &str = "fixture_account";
struct Vault(Vec<u8>);
impl CredentialVault for Vault {
    fn read(&self) -> Result<Option<Zeroizing<Vec<u8>>>, AuthError> {
        Ok(Some(Zeroizing::new(self.0.clone())))
    }
    fn write(&self, _: &[u8]) -> Result<(), AuthError> {
        Ok(())
    }
    fn clear(&self) -> Result<(), AuthError> {
        Ok(())
    }
}
struct NoNetwork(Arc<AtomicUsize>);
impl OAuthTransport for NoNetwork {
    fn open_browser(&self, _: &url::Url) -> Result<(), AuthError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        panic!("synthetic probe forbids OAuth")
    }
    fn token(&self, _: &TokenRequest) -> Result<TokenResponse, AuthError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        panic!("synthetic probe forbids tokens")
    }
    fn user_info(&self, _: &str) -> Result<AccountIdentity, AuthError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        panic!("synthetic probe forbids Google")
    }
    fn revoke(&self, _: &str) -> Result<(), AuthError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        panic!("synthetic probe forbids revocation")
    }
}
struct Clock(AtomicI64);
impl MonitoringClock for Clock {
    fn now_ms(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
    fn jitter_ms(&self) -> u64 {
        0
    }
}
struct Executor {
    clock: Arc<Clock>,
    completed: AtomicUsize,
}
impl CheckExecutor for Executor {
    fn execute(
        &self,
        runtime: &IntegrationRuntime,
        input: Mutation,
    ) -> Result<EngineEnvelope<ValidationResult>, CommandError> {
        work::checkpoint()?;
        let mut db = Database::open(&runtime.database_path).map_err(|_| CommandError::local())?;
        let mut snapshot = fixtures::snapshot();
        snapshot.id = uuid::Uuid::new_v4().to_string();
        let now = self.clock.now_ms();
        snapshot.capture_started_at = chrono::DateTime::from_timestamp_millis(now)
            .ok_or_else(CommandError::local)?
            .to_rfc3339();
        snapshot.captured_at = snapshot.capture_started_at.clone();
        db.save_structural_snapshot(&snapshot)?;
        let baseline = db
            .active_structural_baseline(SUBJECT, &snapshot.spreadsheet_id, &snapshot.scope_hash)?
            .ok_or_else(CommandError::local)?;
        let observation =
            db.content_observation(SUBJECT, &snapshot.spreadsheet_id, &snapshot.scope_hash)?;
        let mut report = evaluate(
            &baseline,
            observation.snapshot.as_ref(),
            &snapshot,
            &uuid::Uuid::new_v4().to_string(),
        )?;
        report.observation_revision = observation.revision;
        report.previous_snapshot_id = db
            .previous_structural_snapshot(SUBJECT, &snapshot.spreadsheet_id, &snapshot.scope_hash)?
            .map(|s| s.id);
        report.detected_at = snapshot.captured_at.clone();
        let lease = work::current_lease().ok_or_else(CommandError::local)?;
        work::checkpoint()?;
        let events = db.persist_monitored_integrity_report(
            &snapshot,
            &report,
            Some(input.expected_revision),
            &lease,
            now,
        )?;
        self.completed.fetch_add(1, Ordering::SeqCst);
        Ok(EngineEnvelope {
            generation: input.expected_generation,
            account_subject: SUBJECT.into(),
            data: ValidationResult { report, events },
        })
    }
}
fn fixture(path: &Path, network: Arc<AtomicUsize>) -> Result<IntegrationRuntime, String> {
    let raw = json!({"config":{"client_id":"fixture.apps.googleusercontent.com","client_secret":null,"drive_metadata_opt_in":false},"session":{"client_id":"fixture.apps.googleusercontent.com","account":{"subject":SUBJECT,"email":null,"name":null},"refresh_token":"synthetic-refresh","granted_scopes":["openid",SHEETS_SCOPE]}});
    let runtime = IntegrationRuntime {
        auth: Arc::new(AuthManager::new(
            Arc::new(Vault(
                serde_json::to_vec(&raw).map_err(|_| "fixture_encode")?,
            )),
            Arc::new(NoNetwork(network)),
        )),
        database_path: path.into(),
    };
    let mut db = Database::open(path)?;
    db.upsert_account(SUBJECT, None, Some("TESTE SINTÉTICO"))?;
    let metadata = crate::google::SpreadsheetMetadata {
        spreadsheet_id: "fixture_sheet".into(),
        spreadsheet_url: None,
        title: "TESTE SINTÉTICO — Sprint 05".into(),
        locale: None,
        time_zone: None,
        sheets: vec![crate::google::SheetMetadata {
            sheet_id: 7,
            title: "Data".into(),
            index: 0,
            sheet_type: Some("GRID".into()),
            row_count: Some(100),
            column_count: Some(8),
            hidden: false,
        }],
    };
    let row = db
        .register_operational_spreadsheet(SUBJECT, &metadata)
        .map_err(|_| "fixture_registry")?;
    let mutation = |revision| Mutation {
        spreadsheet_id: "fixture_sheet".into(),
        expected_generation: 0,
        expected_revision: revision,
    };
    let row = db
        .set_registered_tabs(SUBJECT, &mutation(row.revision), &[7])
        .map_err(|_| "fixture_tabs")?;
    let row = db
        .set_registered_rules(SUBJECT, &mutation(row.revision), &fixtures::scope())
        .map_err(|_| "fixture_rules")?;
    let snapshot = fixtures::snapshot();
    db.save_structural_snapshot(&snapshot)
        .map_err(|_| "fixture_snapshot")?;
    db.approve_registered_structural_baseline(
        SUBJECT,
        &snapshot.id,
        0,
        true,
        "synthetic:test",
        Some(row.revision),
    )
    .map_err(|_| "fixture_baseline")?;
    db.set_monitoring_enabled(SUBJECT, true, chrono::Utc::now().timestamp_millis())
        .map_err(|_| "fixture_control")?;
    Ok(runtime)
}
fn until(mut condition: impl FnMut() -> bool) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(8);
    while !condition() {
        if Instant::now() >= deadline {
            return Err("probe_condition_timeout".into());
        }
        thread::sleep(Duration::from_millis(20));
    }
    Ok(())
}
fn save(path: &Path, report: &Arc<Mutex<Value>>) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(&*report.lock().map_err(|_| "probe_report_lock")?)
        .map_err(|_| "probe_report_encode")?;
    std::fs::write(path, bytes).map_err(|_| "probe_report_write".into())
}

/// CLI: `native_probe --report ABSOLUTE_PATH [--test-toast]`.
/// The optional toast flag requires prior human authorization; default sends none.
pub fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let mut report_path = None;
    let mut toast = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--report" => report_path = args.next().map(PathBuf::from),
            "--test-toast" => toast = true,
            _ => return Err("probe_unknown_argument".into()),
        }
    }
    let report_path = report_path
        .filter(|p| p.is_absolute())
        .ok_or("probe_absolute_report_required")?;
    if let Some(parent) = report_path.parent() {
        std::fs::create_dir_all(parent).map_err(|_| "probe_report_directory")?;
    }
    let directory = std::env::temp_dir().join(format!(
        "sheets-monitor-sprint05-probe-{}",
        uuid::Uuid::new_v4()
    ));
    std::fs::create_dir(&directory).map_err(|_| "probe_directory")?;
    let database_path = directory.join("synthetic.db");
    let network = Arc::new(AtomicUsize::new(0));
    let runtime = fixture(&database_path, network.clone())?;
    let clock = Arc::new(Clock(AtomicI64::new(chrono::Utc::now().timestamp_millis())));
    let executor = Arc::new(Executor {
        clock: clock.clone(),
        completed: AtomicUsize::new(0),
    });
    let report = Arc::new(Mutex::new(
        json!({"synthetic":true,"process_id":std::process::id(),"application_identifier":"com.sheetsmonitor.desktop.sprint05probe","all_baseline_approvals_synthetic":true,"production_database_opened":false,"production_vault_opened":false,"google_requests":0,"toast_requested":toast,"toast_display_confirmed":false,"startup_setting_modified":false,"physical_menu_clicks_verified":false,"callbacks_exercised_programmatically":true}),
    ));
    let mut context = tauri::generate_context!();
    context.config_mut().identifier = "com.sheetsmonitor.desktop.sprint05probe".into();
    context.config_mut().product_name = Some("Sheets Monitor — TESTE SINTÉTICO".into());
    context.config_mut().app.windows.clear();
    let setup_report = report.clone();
    let setup_path = report_path.clone();
    let app=desktop::builder().setup(move|app|{
        let window=tauri::WebviewWindowBuilder::new(app,"main",tauri::WebviewUrl::External("about:blank".parse().expect("constant URL"))).title("Sheets Monitor — TESTE SINTÉTICO Sprint 05").inner_size(740.0,360.0).build()?;
        window.eval("document.body.style.cssText='font:22px Segoe UI;padding:32px;background:#102033;color:white';document.body.textContent='TESTE SINTÉTICO — Sprint 05. Banco temporário e relógio simulado. Nenhuma chamada Google.'")?;
        let service=MonitoringService::start_with_executor(runtime.clone(),Arc::new(||{}),clock.clone(),executor.clone());
        let shared=MonitoringService::start_with_executor(runtime.clone(),Arc::new(||{}),clock.clone(),executor.clone());
        let action_runtime=runtime.clone();let action_service=service.clone();let action_clock=clock.clone();
        let action:Arc<dyn Fn(NativeAction)+Send+Sync>=Arc::new(move|action|{
            if let Ok(mut db)=Database::open(&action_runtime.database_path){match action{
                NativeAction::Pause=>{let _=db.set_monitoring_enabled(SUBJECT,false,action_clock.now_ms());},
                NativeAction::Resume=>{let _=db.set_monitoring_enabled(SUBJECT,true,action_clock.now_ms());},
                NativeAction::CheckNow=>{let _=action_service.request_check("fixture_sheet".into());},_=>{}
            }}action_service.wake();
        });
        let shutdown_service=service.clone();let shutdown_report=setup_report.clone();let shutdown_path=setup_path.clone();
        let shell=desktop::install(app.handle(),DesktopCallbacks{action:action.clone(),shutdown:Arc::new(move||{
            let clean=shutdown_service.shutdown(Duration::from_secs(3));if let Ok(mut report)=shutdown_report.lock(){report["bounded_shutdown"]=json!(clean);}let _=save(&shutdown_path,&shutdown_report);
        })});
        app.manage(shell.clone());
        {let mut report=setup_report.lock().expect("probe report");report["window_created"]=json!(window.is_visible().unwrap_or(false));report["tray_available"]=json!(shell.status().tray_available);report["shared_coordinator"]=json!(Arc::ptr_eq(&service,&shared));}
        let probe_report=setup_report.clone();let probe_path=setup_path.clone();let app_handle=app.handle().clone();
        thread::spawn(move||{
            let result=exercise(&runtime,&clock,&executor,&service,&shell,&window,&action,toast,&app_handle,&probe_report);
            if let Ok(mut report)=probe_report.lock(){report["google_requests"]=json!(network.load(Ordering::SeqCst));report["probe_passed"]=json!(result.is_ok());if let Err(code)=result{report["error_code"]=json!(code);}}
            let _=save(&probe_path,&probe_report);shell.request_exit();
        });
        Ok(())
    }).build(context).map_err(|_|"probe_native_build")?;
    app.run(|handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            if let Some(shell) = handle.try_state::<DesktopShell>() {
                shell.handle_exit_requested(&api);
            }
        }
    });
    // No recursive deletion: remove only known synthetic files from this UUID directory.
    for name in ["synthetic.db", "synthetic.db-wal", "synthetic.db-shm"] {
        let _ = std::fs::remove_file(directory.join(name));
    }
    let _ = std::fs::remove_dir(directory);
    if report.lock().map_err(|_| "probe_report_lock")?["probe_passed"] == json!(true) {
        Ok(())
    } else {
        Err("probe_failed_see_report".into())
    }
}

#[allow(clippy::too_many_arguments)]
fn exercise(
    runtime: &IntegrationRuntime,
    clock: &Clock,
    executor: &Executor,
    service: &MonitoringService,
    shell: &DesktopShell,
    window: &tauri::WebviewWindow,
    action: &Arc<dyn Fn(NativeAction) + Send + Sync>,
    toast: bool,
    app: &tauri::AppHandle,
    report: &Arc<Mutex<Value>>,
) -> Result<(), String> {
    let readiness = crate::notifications::native::readiness(app);
    {
        let mut result = report.lock().map_err(|_| "probe_report_lock")?;
        result["notification_backend"] = json!("windows_winrt");
        result["notification_readiness"] =
            serde_json::to_value(readiness).map_err(|_| "probe_readiness_encode")?;
    }
    until(|| executor.completed.load(Ordering::SeqCst) >= 1 && service.status().active == 0)?;
    let db = Database::open(&runtime.database_path)?;
    let schedule = db
        .monitoring_schedules(SUBJECT)
        .map_err(|_| "probe_schedule")?
        .remove(0);
    let delta60 = schedule.next_run_ms - clock.now_ms();
    drop(db);
    let before_hidden = executor.completed.load(Ordering::SeqCst);
    window.close().map_err(|_| "probe_close")?;
    until(|| !window.is_visible().unwrap_or(true))?;
    clock.0.fetch_add(60_000, Ordering::SeqCst);
    service.wake();
    until(|| {
        executor.completed.load(Ordering::SeqCst) > before_hidden && service.status().active == 0
    })?;
    let checks_while_hidden = executor.completed.load(Ordering::SeqCst) - before_hidden;
    action(NativeAction::Pause);
    until(|| {
        Database::open(&runtime.database_path)
            .ok()
            .and_then(|db| db.monitoring_control(SUBJECT).ok())
            .is_some_and(|control| !control.enabled)
            && service.status().active == 0
    })?;
    let paused_count = executor.completed.load(Ordering::SeqCst);
    clock.0.fetch_add(300_000, Ordering::SeqCst);
    service.wake();
    thread::sleep(Duration::from_millis(650));
    let paused = executor.completed.load(Ordering::SeqCst) == paused_count;
    let mut db = Database::open(&runtime.database_path)?;
    db.save_management_preferences(
        SUBJECT,
        &Preferences {
            interval_seconds: 300,
            ..Default::default()
        },
    )
    .map_err(|_| "probe_interval")?;
    drop(db);
    action(NativeAction::Resume);
    until(|| {
        executor.completed.load(Ordering::SeqCst) > paused_count && service.status().active == 0
    })?;
    let db = Database::open(&runtime.database_path)?;
    let schedule = db
        .monitoring_schedules(SUBJECT)
        .map_err(|_| "probe_schedule")?
        .remove(0);
    let delta300 = schedule.next_run_ms - clock.now_ms();
    drop(db);
    shell.show_main();
    until(|| window.is_visible().unwrap_or(false))?;
    let native_failures = Arc::new(Mutex::new(Vec::<String>::new()));
    let mut native_submission_error = None::<String>;
    let toast_submission = if toast {
        let mut db = Database::open(&runtime.database_path)?;
        db.save_management_preferences(
            SUBJECT,
            &Preferences {
                interval_seconds: 300,
                notifications_enabled: true,
                ..Default::default()
            },
        )
        .map_err(|_| "probe_toast_optin")?;
        let record = NotificationRecord {
            id: "synthetic-probe-test-toast".into(),
            event_id: "synthetic-test".into(),
            source: "operational".into(),
            spreadsheet_id: "fixture_sheet".into(),
            spreadsheet_name: "TESTE SINTÉTICO — Sprint 05".into(),
            sheet_id: None,
            sheet_title: None,
            severity: "info".into(),
            diagnosis:
                "TESTE autorizado de notificação nativa. Nenhuma alteração Google foi realizada."
                    .into(),
            detected_at: chrono::Utc::now().to_rfc3339(),
            state: "pending".into(),
            attempt_count: 1,
            last_attempt_ms: None,
            submitted_ms: None,
            group_id: "synthetic-probe-test-toast".into(),
        };
        let notifier = NativeNotifier::new(app.clone(), Arc::new(|_| {}));
        let callback_failures = native_failures.clone();
        let failed = Arc::new(move |code: String| {
            // Failure callbacks only capture diagnostics; never enter auth/DB.
            callback_failures
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .push(code);
        });
        let submission = match notifier.submit_with_failure(&record, failed) {
            NotificationOutcome::DispatchSubmitted => {
                // Native failure callbacks can outlive Show. Keep this notifier
                // and the event loop alive during explicitly authorized QA;
                // elapsed time still does not prove OS display or user reading.
                thread::sleep(Duration::from_secs(15));
                "dispatch_submitted"
            }
            NotificationOutcome::Retryable { code, .. } => {
                native_submission_error = Some(code);
                "not_submitted"
            }
            NotificationOutcome::PermissionBlocked => {
                native_submission_error = Some("permission_blocked".into());
                "not_submitted"
            }
            NotificationOutcome::Unsupported => {
                native_submission_error = Some("unsupported".into());
                "not_submitted"
            }
        };
        // Explicit lifetime: keep native delegates alive throughout observation.
        drop(notifier);
        submission
    } else {
        "not_requested"
    };
    let mut result = report.lock().map_err(|_| "probe_report_lock")?;
    result["hidden_after_close"] = json!(true);
    result["checks_while_hidden"] = json!(checks_while_hidden);
    result["completed_checks"] = json!(executor.completed.load(Ordering::SeqCst));
    result["interval_60_next_delta_ms"] = json!(delta60);
    result["interval_300_next_delta_ms"] = json!(delta300);
    result["paused_no_execution"] = json!(paused);
    result["resumed"] = json!(true);
    result["window_reopened"] = json!(true);
    result["toast_submission"] = json!(toast_submission);
    result["native_failure_codes"] = json!(*native_failures
        .lock()
        .unwrap_or_else(|error| error.into_inner()));
    result["native_submission_error"] = json!(native_submission_error);
    if delta60 != 60_000 || delta300 != 300_000 || !paused || !shell.status().tray_available {
        return Err("probe_invariant_failed".into());
    }
    Ok(())
}
