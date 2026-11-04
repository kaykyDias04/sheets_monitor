use super::*;
use crate::{
    auth::{
        AccountIdentity, AuthError, AuthManager, CredentialVault, OAuthTransport, TokenRequest,
        TokenResponse,
    },
    google::{SheetMetadata, SpreadsheetMetadata},
    integrity::runtime::{EngineEnvelope, ValidationResult},
    integrity::{evaluate, fixtures, BaselineVersion},
};
use std::sync::atomic::{AtomicI64, AtomicU8, AtomicUsize};
use zeroize::Zeroizing;
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
struct OAuth;
impl OAuthTransport for OAuth {
    fn open_browser(&self, _: &url::Url) -> Result<(), AuthError> {
        panic!("No OAuth consent in scheduler")
    }
    fn token(&self, _: &TokenRequest) -> Result<TokenResponse, AuthError> {
        panic!("Fixture executor does not need tokens")
    }
    fn user_info(&self, _: &str) -> Result<AccountIdentity, AuthError> {
        panic!("No remote identity")
    }
    fn revoke(&self, _: &str) -> Result<(), AuthError> {
        panic!("No revocation")
    }
}
struct Clock(AtomicI64);
impl MonitoringClock for Clock {
    fn now_ms(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
    fn jitter_ms(&self) -> u64 {
        123
    }
}
struct Executor {
    mode: AtomicU8,
    started: AtomicUsize,
    active: AtomicUsize,
    max: AtomicUsize,
}
impl CheckExecutor for Executor {
    fn execute(
        &self,
        _: &IntegrationRuntime,
        input: Mutation,
    ) -> Result<EngineEnvelope<ValidationResult>, CommandError> {
        self.started.fetch_add(1, Ordering::SeqCst);
        let count = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.max.fetch_max(count, Ordering::SeqCst);
        while self.mode.load(Ordering::SeqCst) == 0 && !work::cancelled() {
            thread::yield_now();
        }
        self.active.fetch_sub(1, Ordering::SeqCst);
        work::checkpoint()?;
        if self.mode.load(Ordering::SeqCst) == 3 {
            work::defer_retry(Duration::from_secs(3600));
            return Err(CommandError {
                code: "rate_limited".into(),
                message: "synthetic 429".into(),
                retryable: true,
            });
        }
        if self.mode.load(Ordering::SeqCst) == 1 {
            return Err(CommandError {
                code: "network".into(),
                message: "synthetic".into(),
                retryable: true,
            });
        }
        let mut snapshot = fixtures::snapshot();
        snapshot.spreadsheet_id = input.spreadsheet_id;
        let baseline = BaselineVersion {
            id: "synthetic".into(),
            version: 1,
            active: true,
            approved_at: snapshot.captured_at.clone(),
            approval_origin: "synthetic:test".into(),
            snapshot: snapshot.clone(),
        };
        Ok(EngineEnvelope {
            generation: input.expected_generation,
            account_subject: "fixture_account".into(),
            data: ValidationResult {
                report: evaluate(&baseline, Some(&snapshot), &snapshot, "synthetic").unwrap(),
                events: vec![],
            },
        })
    }
}
fn fixture(
    count: usize,
    mode: u8,
) -> (
    tempfile::TempDir,
    Arc<MonitoringService>,
    Arc<Clock>,
    Arc<Executor>,
) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("monitoring.db");
    let raw = serde_json::json!({"config":{"client_id":"fixture.apps.googleusercontent.com","client_secret":null,"drive_metadata_opt_in":false},"session":{"client_id":"fixture.apps.googleusercontent.com","account":{"subject":"fixture_account","email":null,"name":null},"refresh_token":"synthetic-refresh","granted_scopes":["openid",SHEETS_SCOPE]}});
    let runtime = IntegrationRuntime {
        auth: Arc::new(AuthManager::new(
            Arc::new(Vault(serde_json::to_vec(&raw).unwrap())),
            Arc::new(OAuth),
        )),
        database_path: path.clone(),
    };
    let mut db = Database::open(&path).unwrap();
    db.upsert_account("fixture_account", None, None).unwrap();
    for index in 0..count {
        let id = format!("file_{index}");
        let meta = SpreadsheetMetadata {
            spreadsheet_id: id.clone(),
            spreadsheet_url: None,
            title: "Synthetic scheduler fixture".into(),
            locale: None,
            time_zone: None,
            sheets: vec![SheetMetadata {
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
            .register_operational_spreadsheet("fixture_account", &meta)
            .unwrap();
        let mutation = |revision| Mutation {
            spreadsheet_id: id.clone(),
            expected_generation: 0,
            expected_revision: revision,
        };
        let row = db
            .set_registered_tabs("fixture_account", &mutation(row.revision), &[7])
            .unwrap();
        let row = db
            .set_registered_rules(
                "fixture_account",
                &mutation(row.revision),
                &fixtures::scope(),
            )
            .unwrap();
        let mut snapshot = fixtures::snapshot();
        snapshot.spreadsheet_id = id;
        snapshot.id = uuid::Uuid::new_v4().to_string();
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
    }
    db.set_monitoring_enabled("fixture_account", true, 1_000_000)
        .unwrap();
    let clock = Arc::new(Clock(AtomicI64::new(1_000_000)));
    let executor = Arc::new(Executor {
        mode: AtomicU8::new(mode),
        started: AtomicUsize::new(0),
        active: AtomicUsize::new(0),
        max: AtomicUsize::new(0),
    });
    let service = Arc::new(MonitoringService {
        executor: executor.clone(),
        coordinator: WorkCoordinator::for_database(&path),
        runtime,
        clock: clock.clone(),
        notify: Arc::new(|| {}),
        owner: uuid::Uuid::new_v4().to_string(),
        stopped: AtomicBool::new(false),
        state: Mutex::new(State {
            subject: None,
            last_tick: 0,
            interval: 60_000,
            active: HashMap::new(),
            manual: BTreeMap::new(),
            warning: None,
            control_generation: None,
            known: HashMap::new(),
            maintenance_ms: 0,
            pending_count: 0,
        }),
        wake: Condvar::new(),
    });
    (dir, service, clock, executor)
}
fn until(mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !predicate() {
        assert!(Instant::now() < deadline, "bounded fixture completion");
        thread::yield_now();
    }
}
/// Production scheduler and SQLite, synthetic clock/transport/account only.
/// This is deliberately not evidence of Google API throughput or a soak test.
#[test]
fn thirty_workbooks_queue_quota_recovery_and_persistence() {
    let started_at = Instant::now();
    let (_dir, service, clock, executor) = fixture(30, 0);
    for index in 0..30 {
        service.request_check(format!("file_{index}")).unwrap();
    }
    service.tick().unwrap();
    until(|| executor.started.load(Ordering::SeqCst) == 3);
    assert_eq!(service.status().active, 3);
    assert_eq!(executor.max.load(Ordering::SeqCst), 3);
    executor.mode.store(2, Ordering::SeqCst);

    let drain = |expected| {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            service.tick().unwrap();
            if executor.started.load(Ordering::SeqCst) == expected && service.status().active == 0 {
                break;
            }
            assert!(Instant::now() < deadline, "30-file queue must drain");
            thread::sleep(Duration::from_millis(5));
        }
    };
    drain(30);
    let mut db = Database::open(&service.runtime.database_path).unwrap();
    let rows = db.monitoring_schedules("fixture_account").unwrap();
    assert_eq!(rows.len(), 30);
    assert!(rows.iter().all(|r| r.failure_count == 0
        && r.operational_code.is_none()
        && r.next_run_ms == clock.now_ms() + 60_000));

    let mut prefs = db.management_preferences("fixture_account").unwrap();
    prefs.interval_seconds = 300;
    db.save_management_preferences("fixture_account", &prefs)
        .unwrap();
    executor.mode.store(3, Ordering::SeqCst);
    for index in 0..30 {
        service.request_check(format!("file_{index}")).unwrap();
    }
    drain(60);
    let rows = db.monitoring_schedules("fixture_account").unwrap();
    assert!(rows.iter().all(|r| r.failure_count == 1
        && r.operational_code.as_deref() == Some("rate_limited")
        && r.next_run_ms == clock.now_ms() + 3_600_000));
    let embargo = rows[0].next_run_ms;
    drop(db);
    // Reopen persistence, then simulate coordinator restart and manual pressure.
    let db = Database::open(&service.runtime.database_path).unwrap();
    assert_eq!(
        db.monitoring_schedules("fixture_account").unwrap().len(),
        30
    );
    service.state.lock().unwrap().last_tick = 0;
    clock.0.fetch_add(1_000, Ordering::SeqCst);
    for index in 0..30 {
        service.request_check(format!("file_{index}")).unwrap();
    }
    service.tick().unwrap();
    assert_eq!(executor.started.load(Ordering::SeqCst), 60);
    executor.mode.store(2, Ordering::SeqCst);
    // Keep normal cadence during the simulated embargo. A single large jump
    // intentionally triggers resume staggering, covered by the separate gap test.
    while clock.now_ms() < embargo {
        clock
            .0
            .store((clock.now_ms() + 60_000).min(embargo), Ordering::SeqCst);
        service.tick().unwrap();
        if clock.now_ms() < embargo {
            assert_eq!(executor.started.load(Ordering::SeqCst), 60);
        }
    }
    drain(90);
    let rows = db.monitoring_schedules("fixture_account").unwrap();
    assert!(rows.iter().all(|r| r.failure_count == 0
        && r.operational_code.is_none()
        && r.next_run_ms == clock.now_ms() + 300_000));
    assert_eq!(executor.max.load(Ordering::SeqCst), 3);
    assert!(service.shutdown(Duration::from_secs(5)));
    eprintln!("synthetic_load files=30 checks=90 max_parallel=3 intervals=60,300 quota_failures=30 recovery=30 elapsed_ms={}", started_at.elapsed().as_millis());
}
#[test]
fn backend_workers_have_no_webview_and_share_global_three_file_dedup() {
    let (_dir, service, _, executor) = fixture(4, 0);
    service.tick().unwrap();
    until(|| executor.started.load(Ordering::SeqCst) == 1);
    for id in ["file_0", "file_1", "file_2", "file_3"] {
        service.request_check(id.into()).unwrap();
    }
    service.tick().unwrap();
    until(|| executor.started.load(Ordering::SeqCst) == 3);
    service.tick().unwrap();
    assert_eq!(executor.max.load(Ordering::SeqCst), 3);
    assert_eq!(service.status().active, 3);
    let gate = WorkCoordinator::for_database(&service.runtime.database_path);
    assert!(gate.acquire("file_0", Cancellation::default()).is_err());
    assert!(gate
        .acquire("manual_extra", Cancellation::default())
        .is_err());
    assert!(service.shutdown(Duration::from_secs(5)));
    assert_eq!(service.status().active, 0);
}
#[test]
fn pause_cancels_without_outage_and_resume_and_restart_stagger_without_catchup() {
    let (_dir, service, clock, executor) = fixture(3, 0);
    service.tick().unwrap();
    until(|| executor.started.load(Ordering::SeqCst) == 1);
    let mut db = Database::open(&service.runtime.database_path).unwrap();
    db.set_monitoring_enabled("fixture_account", false, clock.now_ms())
        .unwrap();
    service.tick().unwrap();
    until(|| service.status().active == 0);
    assert!(db
        .monitoring_schedules("fixture_account")
        .unwrap()
        .iter()
        .all(|r| r.operational_code.is_none()));
    executor.mode.store(2, Ordering::SeqCst);
    clock.0.fetch_add(60_000, Ordering::SeqCst);
    db.set_monitoring_enabled("fixture_account", true, clock.now_ms())
        .unwrap();
    service.tick().unwrap();
    until(|| service.status().active == 0);
    clock.0.fetch_add(3_600_000, Ordering::SeqCst);
    service.tick().unwrap();
    until(|| service.status().active == 0);
    let rows = db.monitoring_schedules("fixture_account").unwrap();
    assert!(rows.iter().all(|r| r.next_run_ms >= clock.now_ms()));
    assert!(
        rows.iter()
            .filter(|r| r.next_run_ms > clock.now_ms())
            .count()
            >= 2
    );
    assert!(service.shutdown(Duration::from_secs(5)));
}
#[test]
fn network_is_indeterminate_backoff_and_recovery_is_distinct() {
    let (_dir, service, clock, executor) = fixture(1, 1);
    service.tick().unwrap();
    until(|| service.status().active == 0);
    let db = Database::open(&service.runtime.database_path).unwrap();
    let row = db
        .monitoring_schedules("fixture_account")
        .unwrap()
        .remove(0);
    assert_eq!(row.operational_code.as_deref(), Some("network"));
    assert_eq!(row.failure_count, 1);
    assert!(row.next_run_ms >= clock.now_ms() + 60_000);
    executor.mode.store(2, Ordering::SeqCst);
    clock.0.store(row.next_run_ms, Ordering::SeqCst);
    service.tick().unwrap();
    until(|| service.status().active == 0);
    let row = db
        .monitoring_schedules("fixture_account")
        .unwrap()
        .remove(0);
    assert!(row.operational_code.is_none());
    assert_eq!(row.failure_count, 0);
    assert!(service.shutdown(Duration::from_secs(5)));
}
#[test]
fn interval_change_to_300_is_persisted_from_completion_and_restart_replans_once() {
    let (_dir, service, clock, executor) = fixture(2, 2);
    service.tick().unwrap();
    until(|| service.status().active == 0);
    let mut db = Database::open(&service.runtime.database_path).unwrap();
    let mut prefs = db.management_preferences("fixture_account").unwrap();
    prefs.interval_seconds = 300;
    db.save_management_preferences("fixture_account", &prefs)
        .unwrap();
    clock.0.fetch_add(1_000, Ordering::SeqCst);
    service.tick().unwrap();
    until(|| service.status().active == 0);
    let first = db
        .monitoring_schedules("fixture_account")
        .unwrap()
        .into_iter()
        .find(|r| r.spreadsheet_id == "file_0")
        .unwrap();
    assert_eq!(first.next_run_ms, clock.now_ms() + 300_000);
    let previous = executor.started.load(Ordering::SeqCst);
    service.tick().unwrap();
    assert_eq!(executor.started.load(Ordering::SeqCst), previous);
    service.state.lock().unwrap().last_tick = 0;
    clock.0.fetch_add(600_000, Ordering::SeqCst);
    service.tick().unwrap();
    until(|| service.status().active == 0);
    let rows = db.monitoring_schedules("fixture_account").unwrap();
    assert!(rows.iter().all(|r| r.next_run_ms >= clock.now_ms()));
    assert!(rows
        .iter()
        .any(|r| r.next_run_ms == clock.now_ms() + 150_000));
    assert!(service.shutdown(Duration::from_secs(5)));
}
#[test]
fn retry_after_embargo_survives_restart_pause_resume_interval_and_manual_queue() {
    let (_dir, service, clock, executor) = fixture(1, 3);
    service.tick().unwrap();
    until(|| service.status().active == 0);
    let mut db = Database::open(&service.runtime.database_path).unwrap();
    let deadline = db
        .monitoring_schedules("fixture_account")
        .unwrap()
        .remove(0)
        .next_run_ms;
    assert_eq!(deadline, clock.now_ms() + 3_600_000);
    assert_eq!(executor.started.load(Ordering::SeqCst), 1);
    service.state.lock().unwrap().last_tick = 0;
    clock.0.fetch_add(1000, Ordering::SeqCst);
    service.tick().unwrap();
    db.set_monitoring_enabled("fixture_account", false, clock.now_ms())
        .unwrap();
    service.tick().unwrap();
    db.set_monitoring_enabled("fixture_account", true, clock.now_ms())
        .unwrap();
    service.tick().unwrap();
    let mut prefs = db.management_preferences("fixture_account").unwrap();
    prefs.interval_seconds = 300;
    db.save_management_preferences("fixture_account", &prefs)
        .unwrap();
    service.tick().unwrap();
    service.request_check("file_0".into()).unwrap();
    service.tick().unwrap();
    assert_eq!(executor.started.load(Ordering::SeqCst), 1);
    assert_eq!(
        db.monitoring_schedules("fixture_account")
            .unwrap()
            .remove(0)
            .next_run_ms,
        deadline
    );
    executor.mode.store(2, Ordering::SeqCst);
    clock.0.store(deadline, Ordering::SeqCst);
    service.tick().unwrap();
    until(|| service.status().active == 0);
    assert_eq!(executor.started.load(Ordering::SeqCst), 2);
    assert!(service.shutdown(Duration::from_secs(5)));
}
#[test]
fn logout_before_failure_commit_discards_old_account_operational_transition() {
    let (_dir, service, _, executor) = fixture(1, 0);
    service.tick().unwrap();
    until(|| executor.started.load(Ordering::SeqCst) == 1);
    service.runtime.auth.disconnect().unwrap();
    executor.mode.store(1, Ordering::SeqCst);
    until(|| service.status().active == 0);
    let db = Database::open(&service.runtime.database_path).unwrap();
    assert!(db.monitoring_schedules("fixture_account").unwrap()[0]
        .operational_code
        .is_none());
    assert!(service.shutdown(Duration::from_secs(5)));
}

#[test]
fn current_identity_manual_queue_survives_first_tick_while_globally_paused() {
    let (_dir, service, clock, executor) = fixture(2, 2);
    let mut db = Database::open(&service.runtime.database_path).unwrap();
    db.set_monitoring_enabled("fixture_account", false, clock.now_ms())
        .unwrap();
    service
        .request_check_for("fixture_account", 0, "file_1".into())
        .unwrap();
    service.tick().unwrap();
    until(|| service.status().active == 0);
    assert_eq!(executor.started.load(Ordering::SeqCst), 1);
    let rows = db.monitoring_schedules("fixture_account").unwrap();
    assert!(rows
        .iter()
        .find(|row| row.spreadsheet_id == "file_1")
        .unwrap()
        .last_completed_ms
        .is_some());
    assert!(rows
        .iter()
        .find(|row| row.spreadsheet_id == "file_0")
        .unwrap()
        .last_completed_ms
        .is_none());
    assert!(service.shutdown(Duration::from_secs(5)));
}
#[test]
fn new_account_queue_survives_old_tick_identity_and_old_generation_is_discarded() {
    let (_dir, service, clock, executor) = fixture(2, 2);
    let mut db = Database::open(&service.runtime.database_path).unwrap();
    db.set_monitoring_enabled("fixture_account", false, clock.now_ms())
        .unwrap();
    service.state.lock().unwrap().subject = Some(("previous_subject".into(), 999));
    service
        .request_check_for("fixture_account", 0, "file_1".into())
        .unwrap();
    service
        .request_check_for("previous_subject", 999, "file_0".into())
        .unwrap();
    service.tick().unwrap();
    until(|| service.status().active == 0);
    assert_eq!(executor.started.load(Ordering::SeqCst), 1);
    assert!(db
        .monitoring_schedules("fixture_account")
        .unwrap()
        .iter()
        .find(|row| row.spreadsheet_id == "file_1")
        .unwrap()
        .last_completed_ms
        .is_some());
    service
        .request_check_for("fixture_account", 999, "file_0".into())
        .unwrap();
    service.tick().unwrap();
    assert_eq!(executor.started.load(Ordering::SeqCst), 1);
    assert!(service.state.lock().unwrap().manual.is_empty());
    assert!(service.shutdown(Duration::from_secs(5)));
}
