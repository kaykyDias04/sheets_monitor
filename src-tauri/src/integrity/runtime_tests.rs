use super::*;
use crate::auth::{
    AccountIdentity, AuthError, AuthManager, CredentialVault, OAuthTransport, TokenRequest,
    TokenResponse,
};
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;
struct Vault(Mutex<Vec<u8>>);
impl CredentialVault for Vault {
    fn read(&self) -> Result<Option<Zeroizing<Vec<u8>>>, AuthError> {
        Ok(Some(Zeroizing::new(self.0.lock().unwrap().clone())))
    }
    fn write(&self, b: &[u8]) -> Result<(), AuthError> {
        *self.0.lock().unwrap() = b.to_vec();
        Ok(())
    }
    fn clear(&self) -> Result<(), AuthError> {
        self.0.lock().unwrap().clear();
        Ok(())
    }
}
struct Transport;
impl OAuthTransport for Transport {
    fn open_browser(&self, _: &url::Url) -> Result<(), AuthError> {
        panic!("No login automation in tests")
    }
    fn token(&self, _: &TokenRequest) -> Result<TokenResponse, AuthError> {
        Ok(TokenResponse {
            access_token: "synthetic-access".into(),
            token_type: "Bearer".into(),
            expires_in: 3600,
            refresh_token: None,
            scope: None,
        })
    }
    fn user_info(&self, _: &str) -> Result<AccountIdentity, AuthError> {
        Ok(AccountIdentity {
            subject: "fixture_account".into(),
            email: None,
            name: None,
        })
    }
    fn revoke(&self, _: &str) -> Result<(), AuthError> {
        panic!("No real revocation")
    }
}
fn fixture() -> (tempfile::TempDir, IntegrationRuntime, StructuralSnapshot) {
    let dir = tempfile::tempdir().unwrap();
    let data = serde_json::json!({"config":{"client_id":"fixture.apps.googleusercontent.com","client_secret":null,"drive_metadata_opt_in":false},"session":{"client_id":"fixture.apps.googleusercontent.com","account":{"subject":"fixture_account","email":null,"name":null},"refresh_token":"synthetic-refresh","granted_scopes":["openid",SHEETS_SCOPE]}});
    let auth = AuthManager::new(
        Arc::new(Vault(Mutex::new(serde_json::to_vec(&data).unwrap()))),
        Arc::new(Transport),
    );
    let rt = IntegrationRuntime {
        auth: Arc::new(auth),
        database_path: dir.path().join("runtime.db"),
    };
    let mut db = rt.engine_database().unwrap();
    db.upsert_account("fixture_account", None, None).unwrap();
    let snap = crate::integrity::fixtures::snapshot();
    db.save_structural_snapshot(&snap).unwrap();
    (dir, rt, snap)
}
fn input(snapshot: &StructuralSnapshot) -> ApproveInput {
    ApproveInput {
        expected_generation: 0,
        snapshot_id: snapshot.id.clone(),
        snapshot_fingerprint: fingerprint(snapshot),
        expected_version: 0,
        confirmed: true,
    }
}
#[test]
fn approval_never_happens_implicitly_and_requires_matching_fingerprint() {
    let (_dir, rt, snapshot) = fixture();
    let mut request = input(&snapshot);
    request.confirmed = false;
    assert_eq!(
        rt.approve_integrity_baseline(request).err().unwrap().code,
        "integrity_approval_required"
    );
    let mut request = input(&snapshot);
    request.snapshot_fingerprint = "f".repeat(64);
    assert_eq!(
        rt.approve_integrity_baseline(request).err().unwrap().code,
        "integrity_baseline_conflict"
    );
    assert!(rt
        .engine_database()
        .unwrap()
        .active_structural_baseline(
            &snapshot.account_subject,
            &snapshot.spreadsheet_id,
            &snapshot.scope_hash
        )
        .unwrap()
        .is_none());
    let result = rt.approve_integrity_baseline(input(&snapshot)).unwrap();
    assert_eq!(result.data.version, 1);
    assert_eq!(result.data.approval_origin, "human:native");
    assert_eq!(
        rt.approve_integrity_baseline(input(&snapshot))
            .err()
            .unwrap()
            .code,
        "integrity_baseline_conflict"
    );
}
#[test]
fn incomplete_snapshot_is_not_approved_even_with_confirmation() {
    let (_dir, rt, mut snapshot) = fixture();
    snapshot.id = "incomplete-runtime".into();
    snapshot.coverage.complete = false;
    rt.engine_database()
        .unwrap()
        .save_structural_snapshot(&snapshot)
        .unwrap();
    assert_eq!(
        rt.approve_integrity_baseline(input(&snapshot))
            .err()
            .unwrap()
            .code,
        "integrity_invalid_snapshot"
    );
}
#[test]
fn foreign_account_snapshot_is_not_available_to_current_session() {
    let (_dir, rt, mut snapshot) = fixture();
    snapshot.id = "foreign-runtime".into();
    snapshot.account_subject = "other_subject".into();
    let mut db = rt.engine_database().unwrap();
    db.upsert_account("other_subject", None, None).unwrap();
    db.save_structural_snapshot(&snapshot).unwrap();
    assert_eq!(
        rt.approve_integrity_baseline(input(&snapshot))
            .err()
            .unwrap()
            .code,
        "integrity_not_found"
    );
}
#[test]
fn validation_uses_persisted_baseline_and_cas_context_without_network_reads() {
    let (_dir, rt, snapshot) = fixture();
    assert_eq!(
        rt.validate_integrity_snapshot(snapshot.id.clone(), true)
            .err()
            .unwrap()
            .code,
        "integrity_missing_baseline"
    );
    rt.approve_integrity_baseline(input(&snapshot)).unwrap();
    let first = rt
        .validate_integrity_snapshot(snapshot.id.clone(), true)
        .unwrap();
    assert_eq!(first.data.report.status, IntegrityStatus::Online);
    let again = rt.validate_integrity_snapshot(snapshot.id, true).unwrap();
    assert!(again.data.events.is_empty());
    assert_eq!(again.account_subject, "fixture_account");
    assert!(!serde_json::to_string(&again)
        .unwrap()
        .contains("synthetic-access"));
}
#[test]
fn stale_human_approval_generation_is_rejected_before_any_baseline_write() {
    let (_dir, rt, snapshot) = fixture();
    let mut request = input(&snapshot);
    request.expected_generation = 1;
    assert_eq!(
        rt.approve_integrity_baseline(request).err().unwrap().code,
        "integrity_baseline_conflict"
    );
    assert!(rt
        .engine_database()
        .unwrap()
        .active_structural_baseline(
            &snapshot.account_subject,
            &snapshot.spreadsheet_id,
            &snapshot.scope_hash
        )
        .unwrap()
        .is_none());
}
#[test]
fn capture_permits_limit_concurrency_and_release_on_drop() {
    use crate::monitoring::work::{Cancellation, WorkCoordinator};
    let dir = tempfile::tempdir().unwrap();
    let gate = WorkCoordinator::for_database(&dir.path().join("permits.db"));
    let a = gate.acquire("a", Cancellation::default()).unwrap();
    let b = gate.acquire("b", Cancellation::default()).unwrap();
    let c = gate.acquire("c", Cancellation::default()).unwrap();
    assert!(gate.acquire("d", Cancellation::default()).is_err());
    drop(b);
    let d = gate.acquire("d", Cancellation::default()).unwrap();
    drop((a, c, d));
    assert_eq!(gate.active_count(), 0);
}
#[test]
fn detection_timestamp_is_the_analysis_time_not_an_old_capture_time() {
    let (_dir, rt, mut snapshot) = fixture();
    snapshot.id = "old-capture".into();
    snapshot.capture_started_at = "2000-01-01T00:00:00Z".into();
    snapshot.captured_at = "2000-01-01T00:00:01Z".into();
    rt.engine_database()
        .unwrap()
        .save_structural_snapshot(&snapshot)
        .unwrap();
    rt.approve_integrity_baseline(input(&snapshot)).unwrap();
    let start = chrono::Utc::now();
    let report = rt
        .validate_integrity_snapshot(snapshot.id, true)
        .unwrap()
        .data
        .report;
    assert!(chrono::DateTime::parse_from_rfc3339(&report.detected_at).unwrap() >= start);
}
#[test]
fn cancelled_check_never_persists_a_report_or_changes_approval() {
    let (_dir, rt, snapshot) = fixture();
    rt.approve_integrity_baseline(input(&snapshot)).unwrap();
    let token = crate::monitoring::work::Cancellation::default();
    token.cancel();
    let (result, _) = crate::monitoring::work::in_operation(token, None, || {
        rt.validate_integrity_snapshot(snapshot.id.clone(), true)
    });
    assert_eq!(result.err().unwrap().code, "cancelled");
    let db = rt.engine_database().unwrap();
    assert!(db
        .latest_integrity_report(
            &snapshot.account_subject,
            &snapshot.spreadsheet_id,
            &snapshot.scope_hash
        )
        .unwrap()
        .is_none());
    assert_eq!(
        db.active_structural_baseline(
            &snapshot.account_subject,
            &snapshot.spreadsheet_id,
            &snapshot.scope_hash
        )
        .unwrap()
        .unwrap()
        .version,
        1
    );
}
