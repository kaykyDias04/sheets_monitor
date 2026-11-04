use super::{manager::AccountIdentity, AuthCode, AuthError, AuthResult};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{io::Read, time::Duration};
use url::Url;
use zeroize::Zeroizing;
pub struct TokenRequest {
    pub client_id: String,
    pub client_secret: Option<Zeroizing<String>>,
    pub grant: TokenGrant,
}
pub enum TokenGrant {
    AuthorizationCode {
        code: Zeroizing<String>,
        verifier: Zeroizing<String>,
        redirect_uri: String,
    },
    Refresh {
        refresh_token: Zeroizing<String>,
    },
}
impl std::fmt::Debug for TokenRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TokenRequest([REDACTED])")
    }
}
#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
}
impl std::fmt::Debug for TokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TokenResponse([REDACTED])")
    }
}
impl Drop for TokenResponse {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.access_token.zeroize();
        if let Some(token) = &mut self.refresh_token {
            token.zeroize();
        }
    }
}
pub trait OAuthTransport: Send + Sync {
    fn open_browser(&self, authorization_url: &Url) -> AuthResult<()>;
    fn token(&self, request: &TokenRequest) -> AuthResult<TokenResponse>;
    fn user_info(&self, access_token: &str) -> AuthResult<AccountIdentity>;
    fn revoke(&self, token: &str) -> AuthResult<()>;
}
pub struct HttpsOAuthTransport {
    client: Client,
}
impl HttpsOAuthTransport {
    pub fn new() -> AuthResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| AuthError::new(AuthCode::Network))?;
        Ok(Self { client })
    }
}
fn network(error: reqwest::Error) -> AuthError {
    AuthError::new(if error.is_timeout() {
        AuthCode::Timeout
    } else {
        AuthCode::Network
    })
}
fn bounded(mut response: reqwest::blocking::Response) -> AuthResult<Zeroizing<Vec<u8>>> {
    if response
        .content_length()
        .is_some_and(|length| length > 65536)
    {
        return Err(AuthError::new(AuthCode::InvalidResponse));
    }
    let mut bytes = Zeroizing::new(Vec::new());
    response
        .by_ref()
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(|_| AuthError::new(AuthCode::Network))?;
    if bytes.len() > 65536 {
        return Err(AuthError::new(AuthCode::InvalidResponse));
    }
    Ok(bytes)
}
fn observe_retry_after(status: u16, headers: &reqwest::header::HeaderMap) {
    if matches!(status, 429 | 500..=599) {
        if let Some(delay) = headers
            .get("retry-after")
            .and_then(|h| h.to_str().ok())
            .and_then(crate::google::parse_retry_after)
        {
            crate::monitoring::work::defer_retry(delay);
        }
    }
}
impl OAuthTransport for HttpsOAuthTransport {
    fn open_browser(&self, url: &Url) -> AuthResult<()> {
        if url.scheme() != "https"
            || url.host_str() != Some("accounts.google.com")
            || url.path() != "/o/oauth2/v2/auth"
            || url.port().is_some()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(AuthError::new(AuthCode::InvalidConfiguration));
        }
        open::that(url.as_str()).map_err(|_| AuthError::new(AuthCode::BrowserUnavailable))
    }
    fn token(&self, request: &TokenRequest) -> AuthResult<TokenResponse> {
        let mut form = vec![("client_id", request.client_id.as_str())];
        if let Some(secret) = &request.client_secret {
            form.push(("client_secret", secret.as_str()));
        }
        match &request.grant {
            TokenGrant::AuthorizationCode {
                code,
                verifier,
                redirect_uri,
            } => form.extend([
                ("grant_type", "authorization_code"),
                ("code", code.as_str()),
                ("code_verifier", verifier.as_str()),
                ("redirect_uri", redirect_uri.as_str()),
            ]),
            TokenGrant::Refresh { refresh_token } => form.extend([
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token.as_str()),
            ]),
        }
        // Code/refresh requests are deliberately not replayed automatically.
        let response = self
            .client
            .post("https://oauth2.googleapis.com/token")
            .form(&form)
            .send()
            .map_err(network)?;
        let status = response.status().as_u16();
        observe_retry_after(status, response.headers());
        let bytes = bounded(response)?;
        if status != 200 {
            let error_code = serde_json::from_slice::<serde_json::Value>(&bytes)
                .ok()
                .and_then(|v| v.get("error").and_then(|s| s.as_str()).map(str::to_owned));
            return Err(AuthError::new(match error_code.as_deref() {
                Some("invalid_grant") => AuthCode::InvalidGrant,
                Some("invalid_client") | Some("unauthorized_client") => {
                    AuthCode::InvalidConfiguration
                }
                _ if matches!(status, 429 | 500..=599) => AuthCode::Network,
                _ => AuthCode::AuthorizationFailed,
            }));
        }
        serde_json::from_slice(&bytes).map_err(|_| AuthError::new(AuthCode::InvalidResponse))
    }
    fn user_info(&self, access_token: &str) -> AuthResult<AccountIdentity> {
        // Identity comes from Google's TLS-authenticated UserInfo resource; ID
        // tokens are never decoded or trusted without signature verification.
        let response = self
            .client
            .get("https://openidconnect.googleapis.com/v1/userinfo")
            .bearer_auth(access_token)
            .send()
            .map_err(network)?;
        let status = response.status().as_u16();
        observe_retry_after(status, response.headers());
        if status != 200 {
            return Err(AuthError::new(if status == 401 {
                AuthCode::InvalidGrant
            } else if status == 403 {
                AuthCode::InsufficientScope
            } else {
                AuthCode::Network
            }));
        }
        #[derive(Deserialize)]
        struct Info {
            sub: String,
            email: Option<String>,
            name: Option<String>,
            #[serde(default)]
            email_verified: bool,
        }
        let info: Info = serde_json::from_slice(&bounded(response)?)
            .map_err(|_| AuthError::new(AuthCode::InvalidResponse))?;
        let identity = AccountIdentity {
            subject: info.sub,
            email: if info.email_verified {
                info.email
            } else {
                None
            },
            name: info.name,
        };
        identity.validate()?;
        Ok(identity)
    }
    fn revoke(&self, token: &str) -> AuthResult<()> {
        let response = self
            .client
            .post("https://oauth2.googleapis.com/revoke")
            .form(&[("token", token)])
            .send()
            .map_err(network)?;
        match response.status().as_u16() {
            200 => Ok(()),
            400 => Err(AuthError::new(AuthCode::AuthorizationFailed)),
            _ => Err(AuthError::new(AuthCode::Network)),
        }
    }
}

#[cfg(test)]
mod retry_hint_tests {
    use super::*;
    #[test]
    fn oauth_retry_after_survives_without_replaying_post_or_leaking_context() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "retry-after",
            reqwest::header::HeaderValue::from_static("7200"),
        );
        let (_, hint) = crate::monitoring::work::in_operation(
            crate::monitoring::work::Cancellation::default(),
            None,
            || observe_retry_after(429, &headers),
        );
        assert_eq!(hint, Some(7_200_000));
        assert_eq!(
            crate::monitoring::work::in_operation(
                crate::monitoring::work::Cancellation::default(),
                None,
                || observe_retry_after(401, &headers)
            )
            .1,
            None
        );
    }
}
