//! Sprint 06: isolated synthetic service → engine → SQLite → outbox → mock sink.
//! No Google client, Windows vault, native toast, WebView or startup setting is used.
use super::*;
use crate::{
    auth::{
        AccountIdentity, AuthError, AuthManager, CredentialVault, OAuthTransport, TokenRequest,
        TokenResponse,
    },
    google::{SheetMetadata, SpreadsheetMetadata},
    integrity::{
        evaluate, fingerprint, fixtures,
        runtime::{EngineEnvelope, ValidationResult},
        IntegrityStatus, StructuralSnapshot,
    },
    management::Preferences,
    notifications::service::{NotificationService, NotificationSink},
};
use serde_json::json;
use std::{
    collections::BTreeSet,
    path::Path,
    sync::atomic::{AtomicI64, AtomicU8, AtomicUsize},
};
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
struct NoNetwork;
impl OAuthTransport for NoNetwork {
    fn open_browser(&self, _: &url::Url) -> Result<(), AuthError> {
        panic!("no remote consent in synthetic campaign")
    }
    fn token(&self, _: &TokenRequest) -> Result<TokenResponse, AuthError> {
        panic!("no token request in synthetic campaign")
    }
    fn user_info(&self, _: &str) -> Result<AccountIdentity, AuthError> {
        panic!("no Google identity in synthetic campaign")
    }
    fn revoke(&self, _: &str) -> Result<(), AuthError> {
        panic!("no remote revocation in synthetic campaign")
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
#[derive(Default)]
struct Sink(Mutex<Vec<(String, i64)>>);
impl NotificationSink for Sink {
    fn submit(&self, record: &NotificationRecord) -> NotificationOutcome {
        self.0
            .lock()
            .unwrap()
            .push((record.id.clone(), chrono::Utc::now().timestamp_millis()));
        NotificationOutcome::DispatchSubmitted
    }
}
struct Executor {
    clock: Arc<Clock>,
    mode: AtomicU8,
    started: AtomicUsize,
    completed: AtomicUsize,
    active: Mutex<BTreeSet<String>>,
    maximum: AtomicUsize,
    gate: (Mutex<bool>, Condvar),
}
struct Active<'a>(&'a Executor, String);
impl Drop for Active<'_> {
    fn drop(&mut self) {
        assert!(self.0.active.lock().unwrap().remove(&self.1));
    }
}
impl CheckExecutor for Executor {
    fn execute(
        &self,
        runtime: &IntegrationRuntime,
        input: Mutation,
    ) -> Result<EngineEnvelope<ValidationResult>, CommandError> {
        {
            let mut active = self.active.lock().unwrap();
            assert!(
                active.insert(input.spreadsheet_id.clone()),
                "duplicate concurrent file"
            );
            self.maximum.fetch_max(active.len(), Ordering::SeqCst);
        }
        let _active = Active(self, input.spreadsheet_id.clone());
        self.started.fetch_add(1, Ordering::SeqCst);
        let (lock, wake) = &self.gate;
        let mut open = lock.lock().unwrap();
        while !*open {
            open = wake.wait(open).unwrap();
        }
        drop(open);
        work::checkpoint()?;
        let mode = self.mode.load(Ordering::SeqCst);
        if mode == 10 || mode == 11 {
            if mode == 11 {
                work::defer_retry(Duration::from_secs(3600));
            }
            return Err(CommandError {
                code: if mode == 10 {
                    "network"
                } else {
                    "rate_limited"
                }
                .into(),
                message: "synthetic failure".into(),
                retryable: true,
            });
        }
        let mut db = Database::open(&runtime.database_path).map_err(|_| CommandError::local())?;
        let row = db.registered_spreadsheet(SUBJECT, &input.spreadsheet_id)?;
        let baseline = db
            .active_structural_baseline(
                SUBJECT,
                &input.spreadsheet_id,
                row.scope_hash.as_ref().unwrap(),
            )?
            .unwrap();
        let mut snapshot = baseline.snapshot.clone();
        mutate(&mut snapshot, mode);
        snapshot.id = uuid::Uuid::new_v4().to_string();
        // Data timestamps follow wall UTC so the real dispatcher can claim them;
        // only scheduling/lease timestamps use the injected logical clock.
        snapshot.capture_started_at = chrono::Utc::now().to_rfc3339();
        snapshot.captured_at = snapshot.capture_started_at.clone();
        let observation =
            db.content_observation(SUBJECT, &snapshot.spreadsheet_id, &snapshot.scope_hash)?;
        db.save_structural_snapshot(&snapshot)?;
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
        let lease = work::current_lease().unwrap();
        work::checkpoint()?;
        let events = db.persist_monitored_integrity_report(
            &snapshot,
            &report,
            Some(input.expected_revision),
            &lease,
            self.clock.now_ms(),
        )?;
        self.completed.fetch_add(1, Ordering::SeqCst);
        Ok(EngineEnvelope {
            generation: input.expected_generation,
            account_subject: SUBJECT.into(),
            data: ValidationResult { report, events },
        })
    }
}
fn mutate(snapshot: &mut StructuralSnapshot, mode: u8) {
    match mode {
        1 => {
            let row = &mut snapshot.sheets[0].regions[0].rows[0];
            row.cell_hashes.insert("amount".into(), fingerprint(&9999));
            row.fingerprint = fingerprint(&row.cell_hashes);
        }
        2 => {
            snapshot.sheets[0].regions[0].headers[0].column = 1;
            snapshot.sheets[0].regions[0].headers[1].column = 0;
        }
        3 => {
            snapshot.sheets[0].regions[0].headers.remove(1);
        }
        4 => snapshot.sheets[0].regions[0].formulas[0].formula = None,
        5 => snapshot.sheets[0].regions[0].formulas[0].error = Some("REF".into()),
        6 => snapshot.sheets.clear(),
        7 => {
            snapshot.sheets[0].regions[0].headers[3].header = "Amount".into();
        }
        8 => {
            snapshot.sheets[0].regions[0]
                .rows
                .push(fixtures::row(3, "added_record", 30));
        }
        9 => {
            snapshot.sheets[0].regions[0].rows.pop();
            snapshot.sheets[0].regions[0].formulas.pop();
        }
        _ => {}
    }
}
struct Campaign {
    _directory: tempfile::TempDir,
    runtime: IntegrationRuntime,
    clock: Arc<Clock>,
    executor: Arc<Executor>,
    service: Arc<MonitoringService>,
    sink: Arc<Sink>,
    notifications: Arc<NotificationService>,
    count: usize,
}
fn service(
    runtime: IntegrationRuntime,
    clock: Arc<Clock>,
    executor: Arc<Executor>,
) -> Arc<MonitoringService> {
    Arc::new(MonitoringService {
        coordinator: WorkCoordinator::for_database(&runtime.database_path),
        runtime,
        clock,
        executor,
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
    })
}
fn wait(mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while !predicate() {
        assert!(
            Instant::now() < deadline,
            "bounded integrated campaign completion"
        );
        thread::sleep(Duration::from_millis(2));
    }
}
fn metadata(id: String) -> SpreadsheetMetadata {
    SpreadsheetMetadata {
        spreadsheet_id: id,
        spreadsheet_url: None,
        title: "SYNTHETIC Sprint06".into(),
        locale: None,
        time_zone: None,
        sheets: vec![SheetMetadata {
            sheet_id: 7,
            title: "Data".into(),
            index: 0,
            sheet_type: Some("GRID".into()),
            row_count: Some(1200),
            column_count: Some(4),
            hidden: false,
        }],
    }
}
fn fixture(count: usize, large: bool) -> Campaign {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("synthetic.db");
    let raw = json!({"config":{"client_id":"fixture.apps.googleusercontent.com","client_secret":null,"drive_metadata_opt_in":false},"session":{"client_id":"fixture.apps.googleusercontent.com","account":{"subject":SUBJECT,"email":null,"name":null},"refresh_token":"synthetic-refresh","granted_scopes":["openid",SHEETS_SCOPE]}});
    let runtime = IntegrationRuntime {
        auth: Arc::new(AuthManager::new(
            Arc::new(Vault(serde_json::to_vec(&raw).unwrap())),
            Arc::new(NoNetwork),
        )),
        database_path: path,
    };
    let clock = Arc::new(Clock(AtomicI64::new(
        chrono::Utc::now().timestamp_millis() - 86_400_000,
    )));
    let mut db = Database::open(&runtime.database_path).unwrap();
    db.upsert_account(SUBJECT, None, None).unwrap();
    assert_eq!(
        db.management_preferences(SUBJECT).unwrap(),
        Preferences::default()
    );
    db.save_management_preferences(
        SUBJECT,
        &Preferences {
            notifications_enabled: true,
            ..Default::default()
        },
    )
    .unwrap();
    for index in 0..count {
        let id = format!("synthetic_file_{index:02}");
        let mut snapshot = fixtures::snapshot();
        snapshot.spreadsheet_id = id.clone();
        snapshot.id = uuid::Uuid::new_v4().to_string();
        snapshot.capture_started_at = chrono::Utc::now().to_rfc3339();
        snapshot.captured_at = snapshot.capture_started_at.clone();
        if large {
            snapshot.scope.sheets[0].regions[0].range.end_row = 1200;
            snapshot.scope.sheets[0].regions[0].formulas.clear();
            snapshot.sheets[0].row_count = 1200;
            let region = &mut snapshot.sheets[0].regions[0];
            region.range.end_row = 1200;
            region.formulas.clear();
            region.rows = (0..1000)
                .map(|r| fixtures::row(r + 1, &format!("record_{r}"), i64::from(r)))
                .collect();
            fixtures::rehash(&mut snapshot);
        }
        let row = db
            .register_operational_spreadsheet(SUBJECT, &metadata(id.clone()))
            .unwrap();
        let mutation = |revision| Mutation {
            spreadsheet_id: id.clone(),
            expected_generation: 0,
            expected_revision: revision,
        };
        let row = db
            .set_registered_tabs(SUBJECT, &mutation(row.revision), &[7])
            .unwrap();
        let row = db
            .set_registered_rules(SUBJECT, &mutation(row.revision), &snapshot.scope)
            .unwrap();
        db.save_structural_snapshot(&snapshot).unwrap();
        assert!(db
            .active_structural_baseline(SUBJECT, &id, &snapshot.scope_hash)
            .unwrap()
            .is_none());
        db.approve_registered_structural_baseline(
            SUBJECT,
            &snapshot.id,
            0,
            true,
            "synthetic:test",
            Some(row.revision),
        )
        .unwrap();
    }
    // Start paused: all three initial workers can be admitted deliberately.
    db.set_monitoring_enabled(SUBJECT, false, clock.now_ms())
        .unwrap();
    let executor = Arc::new(Executor {
        clock: clock.clone(),
        mode: AtomicU8::new(0),
        started: AtomicUsize::new(0),
        completed: AtomicUsize::new(0),
        active: Mutex::new(BTreeSet::new()),
        maximum: AtomicUsize::new(0),
        gate: (Mutex::new(true), Condvar::new()),
    });
    let service = service(runtime.clone(), clock.clone(), executor.clone());
    service.tick().unwrap();
    let sink = Arc::new(Sink::default());
    let notifications = NotificationService::start(runtime.clone(), sink.clone(), Arc::new(|| {}));
    Campaign {
        _directory: directory,
        runtime,
        clock,
        executor,
        service,
        sink,
        notifications,
        count,
    }
}
impl Campaign {
    fn db(&self) -> Database {
        Database::open(&self.runtime.database_path).unwrap()
    }
    fn run_cycle(&self, mode: u8) -> u128 {
        let start = Instant::now();
        self.executor.mode.store(mode, Ordering::SeqCst);
        *self.executor.gate.0.lock().unwrap() = false;
        let target = self.executor.started.load(Ordering::SeqCst) + self.count;
        for index in 0..self.count {
            self.service
                .request_check_for(SUBJECT, 0, format!("synthetic_file_{index:02}"))
                .unwrap();
        }
        self.service.tick().unwrap();
        wait(|| self.executor.active.lock().unwrap().len() == self.count.min(3));
        assert_eq!(self.service.status().active, self.count.min(3));
        *self.executor.gate.0.lock().unwrap() = true;
        self.executor.gate.1.notify_all();
        wait(|| {
            self.service.tick().unwrap();
            self.executor.started.load(Ordering::SeqCst) == target
                && self.service.status().active == 0
        });
        assert!(self.executor.maximum.load(Ordering::SeqCst) <= 3);
        start.elapsed().as_millis()
    }
    fn events(&self) -> usize {
        let db = self.db();
        (0..self.count)
            .map(|index| {
                let id = format!("synthetic_file_{index:02}");
                let row = db.registered_spreadsheet(SUBJECT, &id).unwrap();
                db.integrity_events(SUBJECT, &id, row.scope_hash.as_ref().unwrap(), 1000)
                    .unwrap()
                    .len()
            })
            .sum()
    }
    fn assert_status(&self, expected: IntegrityStatus) {
        let db = self.db();
        for index in 0..self.count {
            let id = format!("synthetic_file_{index:02}");
            let row = db.registered_spreadsheet(SUBJECT, &id).unwrap();
            assert_eq!(
                db.latest_integrity_report(SUBJECT, &id, row.scope_hash.as_ref().unwrap())
                    .unwrap()
                    .unwrap()
                    .status,
                expected
            );
        }
    }
    fn drain_notifications(&self) {
        self.notifications.wake();
        wait(|| {
            let page = self.db().notification_history(SUBJECT, 0, 100).unwrap();
            page.items
                .iter()
                .all(|record| record.state == "dispatch_submitted")
        });
        let records = self.sink.0.lock().unwrap();
        assert_eq!(
            records
                .iter()
                .map(|(id, _)| id)
                .collect::<BTreeSet<_>>()
                .len(),
            records.len()
        );
    }
}
impl Drop for Campaign {
    fn drop(&mut self) {
        *self.executor.gate.0.lock().unwrap() = true;
        self.executor.gate.1.notify_all();
        assert!(self.service.shutdown(Duration::from_secs(5)));
        assert!(self.notifications.shutdown(Duration::from_secs(5)));
    }
}
fn disk_bytes(path: &Path) -> u64 {
    [
        path.to_path_buf(),
        PathBuf::from(format!("{}-wal", path.display())),
        PathBuf::from(format!("{}-shm", path.display())),
    ]
    .iter()
    .filter_map(|p| std::fs::metadata(p).ok().map(|m| m.len()))
    .sum()
}

#[test]
fn sprint06_progressive_5_15_30_integrated_load() {
    for count in [5, 15, 30] {
        let setup = Instant::now();
        let mut campaign = fixture(count, true);
        let setup_ms = setup.elapsed().as_millis();
        let before = disk_bytes(&campaign.runtime.database_path);
        let initial_ms = campaign.run_cycle(0);
        campaign.assert_status(IntegrityStatus::Online);
        assert_eq!(campaign.events(), 0);
        let change_at = chrono::Utc::now().timestamp_millis();
        let changed_ms = campaign.run_cycle(1);
        campaign.assert_status(IntegrityStatus::Online);
        assert_eq!(campaign.events(), count);
        campaign.drain_notifications();
        assert_eq!(campaign.sink.0.lock().unwrap().len(), count);
        let max_notification_ms = campaign
            .sink
            .0
            .lock()
            .unwrap()
            .iter()
            .map(|(_, submitted)| submitted - change_at)
            .max()
            .unwrap();
        let repeated_ms = campaign.run_cycle(1);
        assert_eq!(campaign.events(), count);
        campaign.drain_notifications();
        assert_eq!(campaign.sink.0.lock().unwrap().len(), count);
        assert_eq!(campaign.executor.maximum.load(Ordering::SeqCst), 3);
        let before_restart = campaign
            .db()
            .structural_baselines(SUBJECT, "synthetic_file_00", &fixtures_scope_hash(true))
            .unwrap();
        assert_eq!(before_restart.len(), 1);
        assert!(campaign.service.shutdown(Duration::from_secs(5)));
        campaign.service = service(
            campaign.runtime.clone(),
            campaign.clock.clone(),
            campaign.executor.clone(),
        );
        campaign.service.tick().unwrap();
        let restart_ms = campaign.run_cycle(1);
        assert_eq!(campaign.events(), count);
        assert_eq!(
            campaign
                .db()
                .structural_baselines(SUBJECT, "synthetic_file_00", &fixtures_scope_hash(true))
                .unwrap()[0]
                .id,
            before_restart[0].id
        );
        let after = disk_bytes(&campaign.runtime.database_path);
        println!(
            "SPRINT06_METRIC {}",
            json!({"synthetic":true,"files":count,"records_per_file":1000,"setup_wall_ms":setup_ms,"initial_cycle_wall_ms":initial_ms,"changed_cycle_wall_ms":changed_ms,"unchanged_cycle_wall_ms":repeated_ms,"restart_cycle_wall_ms":restart_ms,"maximum_active":campaign.executor.maximum.load(Ordering::SeqCst),"events":campaign.events(),"mock_submissions":campaign.sink.0.lock().unwrap().len(),"mutation_to_last_mock_submission_wall_ms":max_notification_ms,"sqlite_before_bytes_including_sidecars":before,"sqlite_after_bytes_including_sidecars":after,"logical_interval_seconds":60,"history_days":90,"google_requests":0,"native_toasts":0,"autostart_modified":false})
        );
    }
}
fn fixtures_scope_hash(large: bool) -> String {
    let mut scope = fixtures::scope();
    if large {
        scope.sheets[0].regions[0].range.end_row = 1200;
        scope.sheets[0].regions[0].formulas.clear();
    }
    scope.hash()
}

#[test]
fn sprint06_critical_mutations_recover_and_repeat_without_spam() {
    for (mode, expected) in [
        (2, IntegrityStatus::Offline),
        (3, IntegrityStatus::Offline),
        (4, IntegrityStatus::Offline),
        (5, IntegrityStatus::Offline),
        (6, IntegrityStatus::Offline),
        (7, IntegrityStatus::Indeterminate),
        (8, IntegrityStatus::Online),
        (9, IntegrityStatus::Online),
    ] {
        let campaign = fixture(1, false);
        campaign.run_cycle(0);
        campaign.run_cycle(mode);
        campaign.assert_status(expected);
        let events = campaign.events();
        assert!(events > 0);
        // Inconclusive coverage backs off manual requests; advance to its deadline.
        campaign.clock.0.fetch_add(60_000, Ordering::SeqCst);
        campaign.run_cycle(mode);
        assert_eq!(campaign.events(), events);
        campaign.clock.0.fetch_add(60_000, Ordering::SeqCst);
        campaign.run_cycle(0);
        campaign.assert_status(IntegrityStatus::Online);
        let recovered = campaign.events();
        campaign.run_cycle(0);
        assert_eq!(campaign.events(), recovered);
        campaign.drain_notifications();
    }
}

#[test]
fn sprint06_network_quota_restart_and_capacity_warning() {
    let mut campaign = fixture(1, false);
    campaign.run_cycle(0);
    campaign.run_cycle(10);
    let row = campaign
        .db()
        .monitoring_schedules(SUBJECT)
        .unwrap()
        .remove(0);
    assert_eq!(row.operational_code.as_deref(), Some("network"));
    let started = campaign.executor.started.load(Ordering::SeqCst);
    campaign
        .service
        .request_check_for(SUBJECT, 0, "synthetic_file_00".into())
        .unwrap();
    campaign.service.tick().unwrap();
    assert_eq!(campaign.executor.started.load(Ordering::SeqCst), started);
    campaign.clock.0.store(row.next_run_ms, Ordering::SeqCst);
    campaign.run_cycle(11);
    let row = campaign
        .db()
        .monitoring_schedules(SUBJECT)
        .unwrap()
        .remove(0);
    assert_eq!(row.operational_code.as_deref(), Some("rate_limited"));
    assert_eq!(row.next_run_ms - campaign.clock.now_ms(), 3_600_000);
    assert!(campaign.service.shutdown(Duration::from_secs(5)));
    campaign.service = service(
        campaign.runtime.clone(),
        campaign.clock.clone(),
        campaign.executor.clone(),
    );
    campaign.service.tick().unwrap();
    assert_eq!(
        campaign.db().monitoring_schedules(SUBJECT).unwrap()[0].next_run_ms,
        row.next_run_ms
    );
    campaign.clock.0.store(row.next_run_ms, Ordering::SeqCst);
    campaign.run_cycle(0);
    assert_eq!(
        campaign.db().monitoring_schedules(SUBJECT).unwrap()[0].failure_count,
        0
    );
    assert_eq!(
        campaign
            .db()
            .monitoring_operational_history(SUBJECT, 0, 100)
            .unwrap()
            .total,
        3
    );
    // Hold a real worker, advance logical duration past its selected interval,
    // then observe the warning from completion rather than asserting source text.
    *campaign.executor.gate.0.lock().unwrap() = false;
    campaign
        .service
        .request_check_for(SUBJECT, 0, "synthetic_file_00".into())
        .unwrap();
    campaign.service.tick().unwrap();
    wait(|| campaign.executor.active.lock().unwrap().len() == 1);
    campaign.clock.0.fetch_add(60_001, Ordering::SeqCst);
    *campaign.executor.gate.0.lock().unwrap() = true;
    campaign.executor.gate.1.notify_all();
    wait(|| campaign.service.status().active == 0);
    assert_eq!(
        campaign.service.status().warning.as_deref(),
        Some("interval_capacity_exceeded")
    );
    campaign
        .db()
        .save_management_preferences(
            SUBJECT,
            &Preferences {
                interval_seconds: 300,
                notifications_enabled: true,
                ..Default::default()
            },
        )
        .unwrap();
    campaign.run_cycle(0);
    assert_eq!(
        campaign.db().monitoring_schedules(SUBJECT).unwrap()[0].next_run_ms
            - campaign.clock.now_ms(),
        300_000
    );
    campaign.drain_notifications();
}
