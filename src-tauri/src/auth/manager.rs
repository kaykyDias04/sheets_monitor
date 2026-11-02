use super::{
    protocol::{normalize_scope, scopes, PendingAuthorization},
    transport::{OAuthTransport, TokenGrant, TokenRequest, TokenResponse},
    vault::CredentialVault,
    AuthCode, AuthError, AuthResult,
};
use serde::{Deserialize, Serialize};
use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountIdentity {
    pub subject: String,
    pub email: Option<String>,
    pub name: Option<String>,
}
impl AccountIdentity {
    pub(super) fn validate(&self) -> AuthResult<()> {
        if self.subject.is_empty()
            || self.subject.len() > 255
            || self.subject.chars().any(char::is_control)
            || self
                .email
                .as_ref()
                .is_some_and(|s| s.len() > 512 || s.chars().any(char::is_control))
            || self
                .name
                .as_ref()
                .is_some_and(|s| s.len() > 1024 || s.chars().any(char::is_control))
        {
            return Err(AuthError::new(AuthCode::InvalidResponse));
        }
        Ok(())
    }
}
#[derive(Deserialize)]
pub struct AuthConfigInput {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub drive_metadata_opt_in: bool,
}
impl std::fmt::Debug for AuthConfigInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuthConfigInput([REDACTED])")
    }
}
impl Drop for AuthConfigInput {
    fn drop(&mut self) {
        if let Some(secret) = &mut self.client_secret {
            secret.zeroize();
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct StoredState {
    config: Option<StoredConfig>,
    session: Option<StoredSession>,
}
#[derive(Clone, Serialize, Deserialize)]
struct StoredConfig {
    client_id: String,
    client_secret: Option<String>,
    drive_metadata_opt_in: bool,
}
impl Drop for StoredConfig {
    fn drop(&mut self) {
        if let Some(secret) = &mut self.client_secret {
            secret.zeroize();
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
struct StoredSession {
    client_id: String,
    account: AccountIdentity,
    refresh_token: Option<String>,
    granted_scopes: Vec<String>,
}
impl Drop for StoredSession {
    fn drop(&mut self) {
        if let Some(token) = &mut self.refresh_token {
            token.zeroize();
        }
    }
}
struct Access {
    token: Zeroizing<String>,
    expires_at: u64,
}
struct Memory {
    stored: StoredState,
    access: Option<Access>,
    generation: u64,
    reconnect_required: bool,
    load_error: Option<AuthError>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AuthStatus {
    pub status: &'static str,
    pub account: Option<AccountIdentity>,
    pub granted_scopes: Vec<String>,
    pub expires_at: Option<u64>,
}
#[derive(Clone, Debug, Serialize)]
pub struct IntegrationState {
    pub configured: bool,
    pub client_id: Option<String>,
    pub auth: AuthStatus,
    pub drive_metadata_opt_in: bool,
    pub generation: u64,
}

/// A backend-only lease. Do not serialize it or expose its token through IPC.
pub struct AuthorizedSession {
    token: Zeroizing<String>,
    pub account: AccountIdentity,
    pub granted_scopes: Vec<String>,
    pub generation: u64,
    pub client_id: String,
}
impl AuthorizedSession {
    pub fn access_token(&self) -> &str {
        &self.token
    }
    pub fn has_scope(&self, scope: &str) -> bool {
        self.granted_scopes.iter().any(|s| s == scope)
    }
}
impl std::fmt::Debug for AuthorizedSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuthorizedSession([REDACTED])")
    }
}

pub struct AuthManager {
    vault: Arc<dyn CredentialVault>,
    transport: Arc<dyn OAuthTransport>,
    memory: Mutex<Memory>,
    auth_gate: Mutex<()>,
}
fn now() -> AuthResult<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|_| AuthError::new(AuthCode::InvalidResponse))
}
fn validate_config(config: &StoredConfig) -> AuthResult<()> {
    if !config.client_id.ends_with(".apps.googleusercontent.com")
        || config.client_id.len() > 256
        || !config
            .client_id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_'))
        || config
            .client_secret
            .as_ref()
            .is_some_and(|s| s.len() > 16384 || s.chars().any(char::is_control))
    {
        return Err(AuthError::new(AuthCode::InvalidConfiguration));
    }
    Ok(())
}
impl AuthManager {
    pub fn new(vault: Arc<dyn CredentialVault>, transport: Arc<dyn OAuthTransport>) -> Self {
        let loaded = (|| {
            let bytes = vault.read()?;
            let state = if let Some(bytes) = bytes {
                serde_json::from_slice::<StoredState>(&bytes)
                    .map_err(|_| AuthError::new(AuthCode::VaultCorrupt))?
            } else {
                StoredState::default()
            };
            if let Some(config) = &state.config {
                validate_config(config)?;
            }
            if let Some(session) = &state.session {
                session.account.validate()?;
                if state.config.as_ref().map(|c| c.client_id.as_str())
                    != Some(session.client_id.as_str())
                {
                    return Err(AuthError::new(AuthCode::VaultCorrupt));
                }
            }
            Ok::<_, AuthError>(state)
        })();
        let (stored, load_error) = match loaded {
            Ok(state) => (state, None),
            Err(error) => (StoredState::default(), Some(error)),
        };
        Self {
            vault,
            transport,
            memory: Mutex::new(Memory {
                stored,
                access: None,
                generation: 0,
                reconnect_required: false,
                load_error,
            }),
            auth_gate: Mutex::new(()),
        }
    }
    fn memory(&self) -> AuthResult<MutexGuard<'_, Memory>> {
        self.memory
            .lock()
            .map_err(|_| AuthError::new(AuthCode::VaultUnavailable))
    }
    fn save(&self, stored: &StoredState) -> AuthResult<()> {
        let bytes = Zeroizing::new(
            serde_json::to_vec(stored).map_err(|_| AuthError::new(AuthCode::VaultCorrupt))?,
        );
        self.vault.write(&bytes)
    }
    fn status(memory: &Memory) -> IntegrationState {
        let session = memory.stored.session.as_ref();
        let config = memory.stored.config.as_ref();
        let access_valid = memory
            .access
            .as_ref()
            .is_some_and(|a| a.expires_at > now().unwrap_or(u64::MAX));
        let recoverable =
            session.is_some_and(|s| s.refresh_token.as_ref().is_some_and(|t| !t.is_empty()));
        let status = if config.is_none() {
            "unconfigured"
        } else if memory.reconnect_required
            || (session.is_some() && !access_valid && !recoverable)
            || (config.is_some_and(|c| c.drive_metadata_opt_in)
                && session
                    .is_some_and(|s| !s.granted_scopes.iter().any(|v| v == super::DRIVE_SCOPE)))
        {
            "reconnect_required"
        } else if session.is_some() {
            "connected"
        } else {
            "disconnected"
        };
        IntegrationState {
            configured: config.is_some(),
            client_id: config.map(|c| c.client_id.clone()),
            auth: AuthStatus {
                status,
                account: session.map(|s| s.account.clone()),
                granted_scopes: session
                    .map(|s| s.granted_scopes.clone())
                    .unwrap_or_default(),
                expires_at: memory.access.as_ref().map(|a| a.expires_at),
            },
            drive_metadata_opt_in: config.is_some_and(|c| c.drive_metadata_opt_in),
            generation: memory.generation,
        }
    }
    pub fn state(&self) -> AuthResult<IntegrationState> {
        let memory = self.memory()?;
        if let Some(error) = &memory.load_error {
            return Err(error.clone());
        }
        Ok(Self::status(&memory))
    }
    pub fn configure(&self, mut input: AuthConfigInput) -> AuthResult<IntegrationState> {
        let _gate = self
            .auth_gate
            .try_lock()
            .map_err(|_| AuthError::new(AuthCode::AuthBusy))?;
        let mut memory = self.memory()?;
        let client_id = input.client_id.trim().to_owned();
        let previous_secret = memory
            .stored
            .config
            .as_ref()
            .filter(|c| c.client_id == client_id)
            .and_then(|c| c.client_secret.clone());
        let secret = input
            .client_secret
            .take()
            .or(previous_secret)
            .filter(|s| !s.is_empty());
        let config = StoredConfig {
            client_id,
            client_secret: secret,
            drive_metadata_opt_in: input.drive_metadata_opt_in,
        };
        validate_config(&config)?;
        let stored = StoredState {
            config: Some(config),
            session: None,
        };
        self.save(&stored)?;
        memory.generation = memory.generation.wrapping_add(1);
        memory.stored = stored;
        memory.access = None;
        memory.reconnect_required = false;
        memory.load_error = None;
        Ok(Self::status(&memory))
    }
    /// Startup-only application provisioning. Never replaces a user's existing
    /// protected configuration or grants Drive access without an explicit choice.
    pub fn configure_application(
        &self,
        input: Option<AuthConfigInput>,
    ) -> AuthResult<IntegrationState> {
        let current = self.state()?;
        if current.configured {
            return Ok(current);
        }
        match input {
            Some(input) => self.configure(input),
            None => Ok(current),
        }
    }
    /// Changing the requested Drive scope does not manufacture a Google grant.
    /// Enabling it without an existing grant requires full desktop reconsent.
    pub fn set_drive_opt_in(&self, enabled: bool) -> AuthResult<IntegrationState> {
        let _gate = self
            .auth_gate
            .try_lock()
            .map_err(|_| AuthError::new(AuthCode::AuthBusy))?;
        let mut memory = self.memory()?;
        if let Some(error) = &memory.load_error {
            return Err(error.clone());
        }
        let mut stored = memory.stored.clone();
        let config = stored
            .config
            .as_mut()
            .ok_or_else(|| AuthError::new(AuthCode::Unconfigured))?;
        if config.drive_metadata_opt_in == enabled {
            return Ok(Self::status(&memory));
        }
        config.drive_metadata_opt_in = enabled;
        self.save(&stored)?;
        memory.stored = stored;
        memory.generation = memory.generation.wrapping_add(1);
        Ok(Self::status(&memory))
    }
    pub fn connect(&self) -> AuthResult<IntegrationState> {
        self.connect_with_timeout(Duration::from_secs(180))
    }
    fn connect_with_timeout(&self, timeout: Duration) -> AuthResult<IntegrationState> {
        let _gate = self
            .auth_gate
            .try_lock()
            .map_err(|_| AuthError::new(AuthCode::AuthBusy))?;
        let (config, generation) = {
            let memory = self.memory()?;
            (
                memory
                    .stored
                    .config
                    .clone()
                    .ok_or_else(|| AuthError::new(AuthCode::Unconfigured))?,
                memory.generation,
            )
        };
        let pending =
            PendingAuthorization::new(&config.client_id, config.drive_metadata_opt_in, timeout)?;
        self.transport.open_browser(&pending.url)?;
        let (code, redirect_uri, verifier) =
            pending.wait(|| self.memory().map_or(true, |m| m.generation != generation))?;
        let request = TokenRequest {
            client_id: config.client_id.clone(),
            client_secret: config.client_secret.clone().map(Zeroizing::new),
            grant: TokenGrant::AuthorizationCode {
                code,
                verifier,
                redirect_uri,
            },
        };
        let reply = self.transport.token(&request)?;
        let (access, refresh, granted) =
            validate_tokens(reply, None, &scopes(config.drive_metadata_opt_in))?;
        if !granted.iter().any(|s| s == "openid") {
            return Err(AuthError::new(AuthCode::InsufficientScope));
        }
        let account = self.transport.user_info(&access.token)?;
        account.validate()?;
        let stored = StoredState {
            config: Some(config.clone()),
            session: Some(StoredSession {
                client_id: config.client_id.clone(),
                account,
                refresh_token: refresh,
                granted_scopes: granted,
            }),
        };
        let mut memory = self.memory()?;
        if memory.generation != generation {
            return Err(AuthError::new(AuthCode::OperationSuperseded));
        }
        self.save(&stored)?;
        memory.stored = stored;
        memory.access = Some(access);
        memory.generation = memory.generation.wrapping_add(1);
        memory.reconnect_required = false;
        memory.load_error = None;
        Ok(Self::status(&memory))
    }
    /// Logout is allowed while consent/refresh is in flight. Its generation
    /// invalidates results before persistence and it never revokes Google grants.
    pub fn disconnect(&self) -> AuthResult<IntegrationState> {
        let mut memory = self.memory()?;
        memory.generation = memory.generation.wrapping_add(1);
        memory.access = None;
        memory.stored.session = None;
        memory.reconnect_required = false;
        let result = if memory.load_error.is_some() {
            self.vault.clear()
        } else {
            self.save(&memory.stored)
        };
        if let Err(error) = result {
            memory.load_error = Some(error.clone());
            return Err(error);
        }
        memory.load_error = None;
        Ok(Self::status(&memory))
    }
    pub fn revoke(&self, confirmed: bool) -> AuthResult<IntegrationState> {
        if !confirmed {
            return Err(AuthError::new(AuthCode::RevocationConfirmationRequired));
        }
        let _gate = self
            .auth_gate
            .try_lock()
            .map_err(|_| AuthError::new(AuthCode::AuthBusy))?;
        let (token, generation) = {
            let memory = self.memory()?;
            let token = memory
                .stored
                .session
                .as_ref()
                .and_then(|s| s.refresh_token.clone())
                .map(Zeroizing::new)
                .or_else(|| memory.access.as_ref().map(|a| a.token.clone()))
                .ok_or_else(|| AuthError::new(AuthCode::ReconnectRequired))?;
            (token, memory.generation)
        };
        self.transport.revoke(&token)?;
        if self.memory()?.generation != generation {
            return Err(AuthError::new(AuthCode::OperationSuperseded));
        }
        self.disconnect()
    }
    pub fn authorized(&self) -> AuthResult<AuthorizedSession> {
        let _gate = self
            .auth_gate
            .try_lock()
            .map_err(|_| AuthError::new(AuthCode::AuthBusy))?;
        let (config, session, generation) = {
            let memory = self.memory()?;
            if let Some(error) = &memory.load_error {
                return Err(error.clone());
            }
            let config = memory
                .stored
                .config
                .clone()
                .ok_or_else(|| AuthError::new(AuthCode::Unconfigured))?;
            let session = memory
                .stored
                .session
                .clone()
                .ok_or_else(|| AuthError::new(AuthCode::ReconnectRequired))?;
            if let Some(access) = memory
                .access
                .as_ref()
                .filter(|a| a.expires_at > now().unwrap_or(u64::MAX).saturating_add(60))
            {
                return Ok(lease(&memory, &config, &session, access));
            }
            (config, session, memory.generation)
        };
        let refresh = session
            .refresh_token
            .as_ref()
            .filter(|t| !t.is_empty())
            .ok_or_else(|| AuthError::new(AuthCode::ReconnectRequired))?;
        let request = TokenRequest {
            client_id: config.client_id.clone(),
            client_secret: config.client_secret.clone().map(Zeroizing::new),
            grant: TokenGrant::Refresh {
                refresh_token: Zeroizing::new(refresh.clone()),
            },
        };
        let result = (|| {
            let reply = self.transport.token(&request)?;
            let (access, refresh, granted) =
                validate_tokens(reply, Some(refresh.clone()), &session.granted_scopes)?;
            if !granted.iter().any(|s| s == "openid") {
                return Err(AuthError::new(AuthCode::InsufficientScope));
            }
            let account = self.transport.user_info(&access.token)?;
            if account.subject != session.account.subject {
                return Err(AuthError::new(AuthCode::IdentityMismatch));
            }
            Ok((access, refresh, granted, account))
        })();
        let mut memory = self.memory()?;
        if memory.generation != generation {
            return Err(AuthError::new(AuthCode::OperationSuperseded));
        }
        let (access, refresh, granted, account) = match result {
            Ok(result) => result,
            Err(error) => {
                if matches!(
                    error.code,
                    AuthCode::InvalidGrant | AuthCode::IdentityMismatch
                ) {
                    memory.access = None;
                    memory.stored.session = None;
                    memory.reconnect_required = true;
                    memory.generation = memory.generation.wrapping_add(1);
                    if let Err(vault_error) = self.save(&memory.stored) {
                        memory.load_error = Some(vault_error.clone());
                        return Err(vault_error);
                    }
                }
                return Err(error);
            }
        };
        let mut stored = memory.stored.clone();
        stored.session = Some(StoredSession {
            client_id: config.client_id.clone(),
            account,
            refresh_token: refresh,
            granted_scopes: granted,
        });
        self.save(&stored)?;
        memory.stored = stored;
        memory.access = Some(access);
        Ok(lease(
            &memory,
            &config,
            memory
                .stored
                .session
                .as_ref()
                .ok_or_else(|| AuthError::new(AuthCode::ReconnectRequired))?,
            memory
                .access
                .as_ref()
                .ok_or_else(|| AuthError::new(AuthCode::ReconnectRequired))?,
        ))
    }
    /// An API 401 invalidates only this lease; refresh remains bounded by the caller.
    pub fn force_refresh(&self, session: &AuthorizedSession) -> AuthResult<AuthorizedSession> {
        {
            let mut memory = self.memory()?;
            if memory.generation != session.generation || memory.stored.session.is_none() {
                return Err(AuthError::new(AuthCode::OperationSuperseded));
            }
            memory.access = None;
        }
        self.authorized()
    }
    /// Protects only a short local commit/response assembly, never network I/O.
    pub fn with_current<T>(
        &self,
        session: &AuthorizedSession,
        operation: impl FnOnce() -> T,
    ) -> AuthResult<T> {
        self.with_generation(session.generation, operation)
    }
    pub fn with_generation<T>(
        &self,
        generation: u64,
        operation: impl FnOnce() -> T,
    ) -> AuthResult<T> {
        let memory = self.memory()?;
        if memory.generation != generation || memory.stored.session.is_none() {
            return Err(AuthError::new(AuthCode::OperationSuperseded));
        }
        Ok(operation())
    }
}
fn lease(
    memory: &Memory,
    config: &StoredConfig,
    session: &StoredSession,
    access: &Access,
) -> AuthorizedSession {
    AuthorizedSession {
        token: access.token.clone(),
        account: session.account.clone(),
        granted_scopes: session.granted_scopes.clone(),
        generation: memory.generation,
        client_id: config.client_id.clone(),
    }
}
fn validate_tokens(
    mut reply: TokenResponse,
    old_refresh: Option<String>,
    previous_scopes: &[String],
) -> AuthResult<(Access, Option<String>, Vec<String>)> {
    if !reply.token_type.eq_ignore_ascii_case("bearer")
        || reply.access_token.is_empty()
        || reply.access_token.len() > 16384
        || reply.access_token.chars().any(char::is_control)
        || reply.expires_in == 0
        || reply.expires_in > 31_536_000
        || reply
            .refresh_token
            .as_ref()
            .is_some_and(|t| t.len() > 16384 || t.chars().any(char::is_control))
    {
        return Err(AuthError::new(AuthCode::InvalidResponse));
    }
    let permitted = scopes(true);
    let granted = if let Some(scope) = &reply.scope {
        scope
            .split_whitespace()
            .map(normalize_scope)
            .filter(|s| permitted.iter().any(|p| p == s))
            .map(str::to_owned)
            .collect()
    } else {
        previous_scopes.to_vec()
    };
    let access = Access {
        token: Zeroizing::new(std::mem::take(&mut reply.access_token)),
        expires_at: now()?.saturating_add(reply.expires_in),
    };
    Ok((
        access,
        reply
            .refresh_token
            .take()
            .filter(|s| !s.is_empty())
            .or(old_refresh),
        granted,
    ))
}

#[cfg(test)]
mod tests;
