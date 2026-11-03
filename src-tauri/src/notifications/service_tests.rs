//! Synthetic dispatcher tests; no native toast or Google request is sent.
use super::*;
use crate::{
    auth::{
        AccountIdentity, AuthError, AuthManager, CredentialVault, OAuthTransport, TokenRequest,
        TokenResponse, SHEETS_SCOPE,
    },
    google::{SheetMetadata, SpreadsheetMetadata},
    integrity::fixtures,
    management::{Mutation, Preferences},
    monitoring::CheckOutcome,
};
use std::sync::{atomic::AtomicUsize, mpsc};
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
        panic!("No consent in notification fixture")
    }
    fn token(&self, _: &TokenRequest) -> Result<TokenResponse, AuthError> {
        panic!("No token request in notification fixture")
    }
    fn user_info(&self, _: &str) -> Result<AccountIdentity, AuthError> {
        panic!("No remote identity")
    }
    fn revoke(&self, _: &str) -> Result<(), AuthError> {
        panic!("No revocation")
    }
}
fn runtime(path: std::path::PathBuf, subject: &str) -> IntegrationRuntime {
    let raw = serde_json::json!({"config":{"client_id":"fixture.apps.googleusercontent.com","client_secret":null,"drive_metadata_opt_in":false},"session":{"client_id":"fixture.apps.googleusercontent.com","account":{"subject":subject,"email":null,"name":null},"refresh_token":"synthetic-refresh","granted_scopes":["openid",SHEETS_SCOPE]}});
    IntegrationRuntime {
        auth: Arc::new(AuthManager::new(
            Arc::new(Vault(serde_json::to_vec(&raw).unwrap())),
            Arc::new(OAuth),
        )),
        database_path: path,
    }
}
fn fixture(notifications: bool) -> (tempfile::TempDir, IntegrationRuntime, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notifications.db");
    let rt = runtime(path.clone(), "fixture_account");
    let mut db = Database::open(&path).unwrap();
    db.upsert_account("fixture_account", None, None).unwrap();
    db.upsert_account("other_account", None, None).unwrap();
    let meta = SpreadsheetMetadata {
        spreadsheet_id: "fixture_sheet".into(),
        spreadsheet_url: None,
        title: "Synthetic notification fixture".into(),
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
    let m = |revision| Mutation {
        spreadsheet_id: "fixture_sheet".into(),
        expected_generation: 0,
        expected_revision: revision,
    };
    let row = db
        .set_registered_tabs("fixture_account", &m(row.revision), &[7])
        .unwrap();
    let row = db
        .set_registered_rules("fixture_account", &m(row.revision), &fixtures::scope())
        .unwrap();
    let snapshot = fixtures::snapshot();
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
    db.save_management_preferences(
        "fixture_account",
        &Preferences {
            notifications_enabled: notifications,
            ..Default::default()
        },
    )
    .unwrap();
    let now = chrono::Utc::now().timestamp_millis();
    let candidate = db
        .monitoring_candidates("fixture_account")
        .unwrap()
        .remove(0);
    db.ensure_monitoring_schedule("fixture_account", &candidate, now)
        .unwrap();
    let lease = db
        .claim_monitoring_check(
            "fixture_account",
            "fixture_sheet",
            candidate.registry_revision,
            0,
            "fixture-worker",
            now,
            300_000,
            true,
        )
        .unwrap()
        .unwrap();
    db.finish_monitoring_check(
        "fixture_account",
        &lease,
        CheckOutcome::OperationalFailure {
            code: "network_failed".into(),
        },
        now + 60_000,
        now,
    )
    .unwrap();
    let id = db
        .notification_history("fixture_account", 0, 10)
        .unwrap()
        .items
        .remove(0)
        .id;
    (dir, rt, id)
}
struct Sink {
    calls: AtomicUsize,
    retry: bool,
}
impl NotificationSink for Sink {
    fn submit(&self, _: &NotificationRecord) -> NotificationOutcome {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.retry {
            NotificationOutcome::Retryable {
                code: "preparation_failed".into(),
                next_attempt_ms: chrono::Utc::now().timestamp_millis() + 60_000,
            }
        } else {
            NotificationOutcome::DispatchSubmitted
        }
    }
}
fn service(runtime: IntegrationRuntime, sink: Arc<dyn NotificationSink>) -> NotificationService {
    NotificationService {
        runtime,
        sink,
        notify: Arc::new(|| {}),
        owner: uuid::Uuid::new_v4().to_string(),
        stopped: AtomicBool::new(false),
        active: Mutex::new(false),
        wake: Condvar::new(),
    }
}
#[test]
fn actual_pump_submits_once_and_restart_does_not_replay() {
    let (_dir, rt, id) = fixture(true);
    let sink = Arc::new(Sink {
        calls: AtomicUsize::new(0),
        retry: false,
    });
    service(rt.clone(), sink.clone()).pump().unwrap();
    service(rt.clone(), sink.clone()).pump().unwrap();
    assert_eq!(sink.calls.load(Ordering::SeqCst), 1);
    let row = Database::open(&rt.database_path)
        .unwrap()
        .notification_by_id("fixture_account", &id)
        .unwrap()
        .unwrap();
    assert_eq!(row.state, "dispatch_submitted");
    assert!(row.submitted_ms.is_some());
    assert_eq!(row.attempt_count, 1);
}
#[test]
fn actual_pump_preference_off_never_submits() {
    let (_dir, rt, id) = fixture(false);
    let sink = Arc::new(Sink {
        calls: AtomicUsize::new(0),
        retry: false,
    });
    service(rt.clone(), sink.clone()).pump().unwrap();
    assert_eq!(sink.calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        Database::open(&rt.database_path)
            .unwrap()
            .notification_by_id("fixture_account", &id)
            .unwrap()
            .unwrap()
            .state,
        "suppressed_by_preference"
    );
}
#[test]
fn actual_pump_does_not_dispatch_another_accounts_outbox() {
    let (_dir, rt, _) = fixture(true);
    let sink = Arc::new(Sink {
        calls: AtomicUsize::new(0),
        retry: false,
    });
    service(
        runtime(rt.database_path.clone(), "other_account"),
        sink.clone(),
    )
    .pump()
    .unwrap();
    assert_eq!(sink.calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        Database::open(&rt.database_path)
            .unwrap()
            .notification_history("fixture_account", 0, 10)
            .unwrap()
            .items[0]
            .state,
        "pending"
    );
}
#[test]
fn generation_change_to_disconnected_keeps_pending_without_submission() {
    let (_dir, rt, _) = fixture(true);
    let before = rt.auth.state().unwrap().generation;
    rt.auth.disconnect().unwrap();
    assert!(rt.auth.state().unwrap().generation > before);
    let sink = Arc::new(Sink {
        calls: AtomicUsize::new(0),
        retry: false,
    });
    service(rt.clone(), sink.clone()).pump().unwrap();
    assert_eq!(sink.calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        Database::open(&rt.database_path)
            .unwrap()
            .notification_history("fixture_account", 0, 10)
            .unwrap()
            .items[0]
            .state,
        "pending"
    );
}
#[test]
fn preparation_failure_is_a_durable_retry_without_busy_loop() {
    let (_dir, rt, id) = fixture(true);
    let sink = Arc::new(Sink {
        calls: AtomicUsize::new(0),
        retry: true,
    });
    let service = service(rt.clone(), sink.clone());
    service.pump().unwrap();
    service.pump().unwrap();
    assert_eq!(sink.calls.load(Ordering::SeqCst), 1);
    let row = Database::open(&rt.database_path)
        .unwrap()
        .notification_by_id("fixture_account", &id)
        .unwrap()
        .unwrap();
    assert_eq!(row.state, "retry");
    assert!(row.submitted_ms.is_none());
    assert_eq!(row.attempt_count, 1);
}
struct BlockingSink {
    entered: mpsc::Sender<()>,
    release: Mutex<mpsc::Receiver<()>>,
    calls: AtomicUsize,
}
impl NotificationSink for BlockingSink {
    fn submit(&self, _: &NotificationRecord) -> NotificationOutcome {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.entered.send(()).unwrap();
        self.release
            .lock()
            .unwrap()
            .recv_timeout(Duration::from_secs(3))
            .unwrap();
        NotificationOutcome::DispatchSubmitted
    }
}
#[test]
fn actual_worker_shutdown_is_bounded_while_submission_drains() {
    let (_dir, rt, id) = fixture(true);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let sink = Arc::new(BlockingSink {
        entered: entered_tx,
        release: Mutex::new(release_rx),
        calls: AtomicUsize::new(0),
    });
    let worker = NotificationService::start(rt.clone(), sink.clone(), Arc::new(|| {}));
    entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    let start = Instant::now();
    assert!(!worker.shutdown(Duration::from_millis(10)));
    assert!(start.elapsed() < Duration::from_secs(1));
    release_tx.send(()).unwrap();
    assert!(worker.shutdown(Duration::from_secs(3)));
    worker.pump().unwrap();
    assert_eq!(sink.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        Database::open(&rt.database_path)
            .unwrap()
            .notification_by_id("fixture_account", &id)
            .unwrap()
            .unwrap()
            .state,
        "dispatch_submitted"
    );
}
#[test]
fn authentication_generation_is_locked_through_native_submission() {
    let (_dir, rt, _) = fixture(true);
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let sink = Arc::new(BlockingSink {
        entered: entered_tx,
        release: Mutex::new(release_rx),
        calls: AtomicUsize::new(0),
    });
    let pump = Arc::new(service(rt.clone(), sink.clone()));
    let running = pump.clone();
    let handle = thread::spawn(move || running.pump().unwrap());
    entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    let (done_tx, done_rx) = mpsc::channel();
    let auth = rt.auth.clone();
    let disconnect = thread::spawn(move || {
        auth.disconnect().unwrap();
        done_tx.send(()).unwrap();
    });
    assert!(done_rx.recv_timeout(Duration::from_millis(20)).is_err());
    release_tx.send(()).unwrap();
    handle.join().unwrap();
    done_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    disconnect.join().unwrap();
    pump.pump().unwrap();
    assert_eq!(sink.calls.load(Ordering::SeqCst), 1);
    assert_eq!(rt.auth.state().unwrap().auth.status, "disconnected");
}
