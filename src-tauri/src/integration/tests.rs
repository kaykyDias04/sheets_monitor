use super::*;
use crate::auth::{
    AccountIdentity, AuthCode, AuthError, CredentialVault, OAuthTransport, TokenRequest,
    TokenResponse,
};
use std::{
    cell::Cell,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};
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
struct Transport {
    calls: AtomicUsize,
}
impl OAuthTransport for Transport {
    fn open_browser(&self, _: &url::Url) -> Result<(), AuthError> {
        panic!("manual read must never open browser")
    }
    fn token(&self, _: &TokenRequest) -> Result<TokenResponse, AuthError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
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
        panic!("read must never revoke")
    }
}
fn runtime(scopes: Vec<&str>) -> (tempfile::TempDir, IntegrationRuntime, Arc<Transport>) {
    let dir = tempfile::tempdir().unwrap();
    let vault=Vault(Mutex::new(serde_json::to_vec(&serde_json::json!({"config":{"client_id":"fixture.apps.googleusercontent.com","client_secret":null,"drive_metadata_opt_in":false},"session":{"client_id":"fixture.apps.googleusercontent.com","account":{"subject":"fixture_account","email":null,"name":null},"refresh_token":"synthetic-refresh","granted_scopes":scopes}})).unwrap()));
    let transport = Arc::new(Transport {
        calls: AtomicUsize::new(0),
    });
    let runtime = IntegrationRuntime {
        auth: Arc::new(AuthManager::new(Arc::new(vault), transport.clone())),
        database_path: dir.path().join("fixture.db"),
    };
    (dir, runtime, transport)
}
#[test]
fn api_401_refreshes_once_and_never_loops() {
    let (_dir, runtime, transport) = runtime(vec!["openid", SHEETS_SCOPE]);
    let mut lease = runtime.auth.authorized().unwrap();
    let reads = Cell::new(0);
    let result = runtime
        .read::<()>(&mut lease, SHEETS_SCOPE, |_| {
            reads.set(reads.get() + 1);
            Err(ApiError::Unauthorized)
        })
        .unwrap();
    assert_eq!(result, Err(ApiError::Unauthorized));
    assert_eq!(reads.get(), 2);
    assert_eq!(transport.calls.load(Ordering::SeqCst), 2);
}
#[test]
fn absent_scopes_do_not_call_google_or_create_resources() {
    let (_dir, runtime, transport) = runtime(vec!["openid"]);
    let result = runtime
        .sheet(SheetInput {
            url: "sheet_fixture".into(),
            ranges: vec!["A1".into()],
        })
        .unwrap();
    assert_eq!(result.sheets.status, "not_authorized");
    assert_eq!(result.values.status, "not_authorized");
    assert_eq!(result.drive.status, "not_requested");
    let encoded = serde_json::to_string(&result).unwrap();
    assert!(!encoded.contains("synthetic-access"));
    assert!(!encoded.contains("synthetic-refresh"));
    assert!(Database::open(&runtime.database_path)
        .unwrap()
        .integration_queries("fixture_account")
        .unwrap()
        .is_empty());
    assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
}
#[test]
fn disconnected_runtime_never_reads_or_persists_as_success() {
    let (_dir, runtime, transport) = runtime(vec!["openid", SHEETS_SCOPE]);
    runtime.auth.disconnect().unwrap();
    let error = runtime
        .sheet(SheetInput {
            url: "sheet_fixture".into(),
            ranges: vec!["A1".into()],
        })
        .err()
        .unwrap();
    assert_eq!(error.code, "reconnect_required");
    assert_eq!(transport.calls.load(Ordering::SeqCst), 0);
    assert_eq!(
        CommandError::from(AuthError::new(AuthCode::Network)).code,
        "network"
    );
}

#[test]
fn discovery_without_explicit_drive_opt_in_does_not_refresh_or_create_database() {
    let (_dir, runtime, transport) = runtime(vec!["openid", DRIVE_SCOPE]);
    let result = runtime.discover_spreadsheets(crate::google::DiscoveryQuery::default());
    assert_eq!(result.err().unwrap().code, "insufficient_scope");
    assert_eq!(transport.calls.load(Ordering::SeqCst), 0);
    assert!(!runtime.database_path.exists());
}

#[test]
fn discovery_requires_effective_scope_even_after_requesting_drive() {
    let (_dir, runtime, transport) = runtime(vec!["openid", SHEETS_SCOPE]);
    runtime.auth.set_drive_opt_in(true).unwrap();
    let result = runtime.discover_spreadsheets(crate::google::DiscoveryQuery::default());
    assert_eq!(result.err().unwrap().code, "forbidden");
    assert_eq!(transport.calls.load(Ordering::SeqCst), 1);
    assert!(!runtime.database_path.exists());
}
