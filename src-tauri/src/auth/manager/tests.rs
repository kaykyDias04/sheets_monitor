use super::*;
use std::{
    collections::VecDeque,
    io::Write,
    net::TcpStream,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
#[derive(Default)]
struct Vault {
    bytes: Mutex<Option<Vec<u8>>>,
    fail: AtomicBool,
}
impl CredentialVault for Vault {
    fn read(&self) -> AuthResult<Option<Zeroizing<Vec<u8>>>> {
        Ok(self.bytes.lock().unwrap().clone().map(Zeroizing::new))
    }
    fn write(&self, b: &[u8]) -> AuthResult<()> {
        if self.fail.load(Ordering::SeqCst) {
            return Err(AuthError::new(AuthCode::VaultUnavailable));
        }
        *self.bytes.lock().unwrap() = Some(b.to_vec());
        Ok(())
    }
    fn clear(&self) -> AuthResult<()> {
        *self.bytes.lock().unwrap() = None;
        Ok(())
    }
}
struct Mock {
    replies: Mutex<VecDeque<AuthResult<TokenResponse>>>,
    subject: Mutex<String>,
    denied: AtomicBool,
    revokes: AtomicUsize,
    calls: AtomicUsize,
    pause: Mutex<Option<(Arc<std::sync::Barrier>, Arc<std::sync::Barrier>)>>,
}
impl Mock {
    fn new() -> Self {
        Self {
            replies: Mutex::new(VecDeque::new()),
            subject: Mutex::new("account_1".into()),
            denied: AtomicBool::new(false),
            revokes: AtomicUsize::new(0),
            calls: AtomicUsize::new(0),
            pause: Mutex::new(None),
        }
    }
}
impl OAuthTransport for Mock {
    fn open_browser(&self, url: &url::Url) -> AuthResult<()> {
        let pairs: std::collections::HashMap<_, _> = url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        assert_eq!(pairs["code_challenge_method"], "S256");
        assert!(!pairs.contains_key("include_granted_scopes"));
        assert!(!url.as_str().contains("synthetic-secret"));
        let callback = url::Url::parse(&pairs["redirect_uri"]).unwrap();
        let address = format!("127.0.0.1:{}", callback.port().unwrap());
        let denied = self.denied.load(Ordering::SeqCst);
        let state = pairs["state"].clone();
        std::thread::spawn(move || {
            let mut stream = TcpStream::connect(&address).unwrap();
            let result = if denied {
                "error=access_denied"
            } else {
                "code=synthetic-code"
            };
            write!(
                stream,
                "GET /oauth/callback?state={state}&{result} HTTP/1.1\r\nHost: {address}\r\n\r\n"
            )
            .unwrap();
        });
        Ok(())
    }
    fn token(&self, request: &TokenRequest) -> AuthResult<TokenResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let pause = self.pause.lock().unwrap().take();
        if let Some((entered, release)) = pause {
            entered.wait();
            release.wait();
        }
        assert!(!format!("{request:?}").contains("synthetic"));
        self.replies
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| Ok(reply(Some("synthetic-refresh"))))
    }
    fn user_info(&self, _: &str) -> AuthResult<AccountIdentity> {
        Ok(AccountIdentity {
            subject: self.subject.lock().unwrap().clone(),
            email: Some("fixture@example.invalid".into()),
            name: Some("Synthetic account".into()),
        })
    }
    fn revoke(&self, _: &str) -> AuthResult<()> {
        self.revokes.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}
fn reply(refresh: Option<&str>) -> TokenResponse {
    TokenResponse {
        access_token: "synthetic-access".into(),
        token_type: "Bearer".into(),
        expires_in: 3600,
        refresh_token: refresh.map(str::to_owned),
        scope: Some(scopes(false).join(" ")),
    }
}
fn configured() -> (Arc<Vault>, Arc<Mock>, AuthManager) {
    let vault = Arc::new(Vault::default());
    let mock = Arc::new(Mock::new());
    let manager = AuthManager::new(vault.clone(), mock.clone());
    manager
        .configure(AuthConfigInput {
            client_id: "fixture.apps.googleusercontent.com".into(),
            client_secret: Some("synthetic-secret".into()),
            drive_metadata_opt_in: false,
        })
        .unwrap();
    (vault, mock, manager)
}
#[test]
fn code_exchange_recovers_refresh_session_without_serializing_secrets() {
    let (vault, mock, manager) = configured();
    let state = manager.connect().unwrap();
    assert_eq!(state.auth.status, "connected");
    let encoded = serde_json::to_string(&state).unwrap();
    for secret in ["synthetic-secret", "synthetic-access", "synthetic-refresh"] {
        assert!(!encoded.contains(secret));
    }
    assert!(
        !String::from_utf8_lossy(vault.bytes.lock().unwrap().as_ref().unwrap())
            .contains("synthetic-access")
    );
    let recovered = AuthManager::new(vault.clone(), mock.clone());
    mock.replies.lock().unwrap().push_back(Ok(reply(None)));
    let lease = recovered.authorized().unwrap();
    assert_eq!(lease.account.subject, "account_1");
    assert_eq!(mock.calls.load(Ordering::SeqCst), 2);
    assert!(
        String::from_utf8_lossy(vault.bytes.lock().unwrap().as_ref().unwrap())
            .contains("synthetic-refresh")
    );
    assert!(!format!("{lease:?}").contains("synthetic-access"));
}
#[test]
fn logout_discards_old_generation_without_revoking() {
    let (_, mock, manager) = configured();
    manager.connect().unwrap();
    let lease = manager.authorized().unwrap();
    let next = manager.disconnect().unwrap();
    assert_eq!(next.auth.status, "disconnected");
    assert!(manager
        .with_current(&lease, || panic!("stale write"))
        .is_err());
    assert_eq!(mock.revokes.load(Ordering::SeqCst), 0);
    assert!(manager.authorized().is_err());
}
#[test]
fn invalid_grant_clears_session_network_failure_preserves_it() {
    for code in [AuthCode::Network, AuthCode::InvalidGrant] {
        let (_, mock, manager) = configured();
        manager.connect().unwrap();
        let lease = manager.authorized().unwrap();
        mock.replies
            .lock()
            .unwrap()
            .push_back(Err(AuthError::new(code)));
        assert_eq!(manager.force_refresh(&lease).unwrap_err().code, code);
        assert_eq!(
            manager.state().unwrap().auth.status,
            if code == AuthCode::Network {
                "connected"
            } else {
                "reconnect_required"
            }
        );
    }
}
#[test]
fn identity_change_on_refresh_is_rejected() {
    let (_, mock, manager) = configured();
    manager.connect().unwrap();
    let lease = manager.authorized().unwrap();
    *mock.subject.lock().unwrap() = "other_account".into();
    assert_eq!(
        manager.force_refresh(&lease).unwrap_err().code,
        AuthCode::IdentityMismatch
    );
    assert_eq!(manager.state().unwrap().auth.status, "reconnect_required");
}
#[test]
fn consent_denied_and_vault_failure_never_report_connected() {
    let (vault, mock, manager) = configured();
    mock.denied.store(true, Ordering::SeqCst);
    assert_eq!(manager.connect().unwrap_err().code, AuthCode::ConsentDenied);
    assert_eq!(mock.calls.load(Ordering::SeqCst), 0);
    mock.denied.store(false, Ordering::SeqCst);
    vault.fail.store(true, Ordering::SeqCst);
    assert_eq!(
        manager.connect().unwrap_err().code,
        AuthCode::VaultUnavailable
    );
    assert_eq!(manager.state().unwrap().auth.status, "disconnected");
}
#[test]
fn revocation_requires_explicit_confirmation() {
    let (_, mock, manager) = configured();
    manager.connect().unwrap();
    assert_eq!(
        manager.revoke(false).unwrap_err().code,
        AuthCode::RevocationConfirmationRequired
    );
    assert_eq!(mock.revokes.load(Ordering::SeqCst), 0);
    assert_eq!(manager.revoke(true).unwrap().auth.status, "disconnected");
    assert_eq!(mock.revokes.load(Ordering::SeqCst), 1);
}
#[test]
fn invalid_tokens_and_configuration_fail_closed() {
    let (_, mock, manager) = configured();
    let mut invalid = reply(None);
    invalid.token_type = "Basic".into();
    mock.replies.lock().unwrap().push_back(Ok(invalid));
    assert_eq!(
        manager.connect().unwrap_err().code,
        AuthCode::InvalidResponse
    );
    assert!(manager
        .configure(AuthConfigInput {
            client_id: "https://evil.invalid".into(),
            client_secret: None,
            drive_metadata_opt_in: false
        })
        .is_err());
    assert_eq!(manager.state().unwrap().auth.status, "disconnected");
}

#[test]
fn logout_during_refresh_cannot_restore_or_persist_the_old_session() {
    let (vault, mock, manager) = configured();
    manager.connect().unwrap();
    let manager = Arc::new(manager);
    let lease = manager.authorized().unwrap();
    let entered = Arc::new(std::sync::Barrier::new(2));
    let release = Arc::new(std::sync::Barrier::new(2));
    *mock.pause.lock().unwrap() = Some((entered.clone(), release.clone()));
    let worker_manager = manager.clone();
    let worker = std::thread::spawn(move || worker_manager.force_refresh(&lease));
    entered.wait();
    manager.disconnect().unwrap();
    release.wait();
    assert_eq!(
        worker.join().unwrap().unwrap_err().code,
        AuthCode::OperationSuperseded
    );
    assert_eq!(manager.state().unwrap().auth.status, "disconnected");
    assert!(
        !String::from_utf8_lossy(vault.bytes.lock().unwrap().as_ref().unwrap())
            .contains("synthetic-refresh")
    );
}
#[test]
fn partial_consent_does_not_claim_ungranted_service_scopes() {
    let (_, mock, manager) = configured();
    let mut response = reply(None);
    response.scope = Some("openid email profile".into());
    mock.replies.lock().unwrap().push_back(Ok(response));
    manager.connect().unwrap();
    let session = manager.authorized().unwrap();
    assert!(!session.has_scope(super::super::SHEETS_SCOPE));
    assert!(!session.has_scope(super::super::DRIVE_SCOPE));
}
#[test]
fn corrupt_vault_fails_closed_and_explicit_local_logout_resets_it() {
    let vault = Arc::new(Vault::default());
    *vault.bytes.lock().unwrap() = Some(b"invalid".to_vec());
    let manager = AuthManager::new(vault.clone(), Arc::new(Mock::new()));
    assert_eq!(manager.state().unwrap_err().code, AuthCode::VaultCorrupt);
    assert_eq!(manager.disconnect().unwrap().auth.status, "unconfigured");
    assert!(vault.bytes.lock().unwrap().is_none());
}

#[test]
fn application_provisioning_is_optional_and_never_overwrites_existing_config() {
    let vault = Arc::new(Vault::default());
    let mock = Arc::new(Mock::new());
    let manager = AuthManager::new(vault, mock);
    assert!(!manager.configure_application(None).unwrap().configured);
    let first = manager
        .configure_application(Some(AuthConfigInput {
            client_id: "fixture.apps.googleusercontent.com".into(),
            client_secret: Some("synthetic-secret".into()),
            drive_metadata_opt_in: false,
        }))
        .unwrap();
    assert!(first.configured);
    let second = manager
        .configure_application(Some(AuthConfigInput {
            client_id: "other.apps.googleusercontent.com".into(),
            client_secret: None,
            drive_metadata_opt_in: true,
        }))
        .unwrap();
    assert_eq!(second.client_id, first.client_id);
    assert_eq!(second.generation, first.generation);
    assert!(!second.drive_metadata_opt_in);
}

#[test]
fn drive_opt_in_requires_official_consent_and_never_fabricates_grants() {
    let (_, _, manager) = configured();
    manager.connect().unwrap();
    let before = manager.state().unwrap();
    assert!(!before
        .auth
        .granted_scopes
        .iter()
        .any(|s| s == super::super::DRIVE_SCOPE));
    let enabled = manager.set_drive_opt_in(true).unwrap();
    assert!(enabled.drive_metadata_opt_in);
    assert_eq!(enabled.auth.status, "reconnect_required");
    assert_eq!(enabled.auth.granted_scopes, before.auth.granted_scopes);
    assert!(enabled.generation > before.generation);
    assert_eq!(enabled.auth.account, before.auth.account);
    let disabled = manager.set_drive_opt_in(false).unwrap();
    assert_eq!(disabled.auth.status, "connected");
    assert!(!disabled.drive_metadata_opt_in);
    assert_eq!(disabled.auth.granted_scopes, before.auth.granted_scopes);
}

#[test]
fn failed_drive_opt_in_preserves_previous_protected_configuration() {
    let (vault, _, manager) = configured();
    let before = manager.state().unwrap();
    vault.fail.store(true, Ordering::SeqCst);
    assert_eq!(
        manager.set_drive_opt_in(true).unwrap_err().code,
        AuthCode::VaultUnavailable
    );
    let after = manager.state().unwrap();
    assert_eq!(before.generation, after.generation);
    assert!(!after.drive_metadata_opt_in);
}

#[test]
fn opting_out_of_drive_does_not_clear_an_unrelated_reconnect_error() {
    let (_, _, manager) = configured();
    manager.set_drive_opt_in(true).unwrap();
    manager.memory().unwrap().reconnect_required = true;
    let next = manager.set_drive_opt_in(false).unwrap();
    assert_eq!(next.auth.status, "reconnect_required");
}

#[test]
fn invalid_application_provisioning_keeps_shell_state_unconfigured() {
    let vault = Arc::new(Vault::default());
    let manager = AuthManager::new(vault, Arc::new(Mock::new()));
    assert!(manager
        .configure_application(Some(AuthConfigInput {
            client_id: "invalid-client".into(),
            client_secret: None,
            drive_metadata_opt_in: false,
        }))
        .is_err());
    assert!(!manager.state().unwrap().configured);
}
