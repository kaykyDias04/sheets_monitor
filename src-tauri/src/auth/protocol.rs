use super::{
    AuthCode, AuthError, AuthResult, AUDIT_SCOPE, CLOUD_SCOPE, DRIVE_SCOPE, SCRIPT_SCOPE,
    SHEETS_SCOPE,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::{Duration, Instant},
};
use url::Url;
use zeroize::Zeroizing;

pub(super) fn scopes(drive: bool, cloud: bool, audit: bool) -> Vec<String> {
    let mut scopes = vec![
        "openid".into(),
        "email".into(),
        "profile".into(),
        SHEETS_SCOPE.into(),
        SCRIPT_SCOPE.into(),
    ];
    if drive {
        scopes.push(DRIVE_SCOPE.into());
    }
    if cloud {
        scopes.push(CLOUD_SCOPE.into());
    }
    if audit {
        scopes.push(AUDIT_SCOPE.into());
    }
    scopes
}
pub(super) fn normalize_scope(scope: &str) -> &str {
    match scope {
        "https://www.googleapis.com/auth/userinfo.email" => "email",
        "https://www.googleapis.com/auth/userinfo.profile" => "profile",
        other => other,
    }
}
fn random_urlsafe() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}
fn challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

pub(super) struct PendingAuthorization {
    listener: TcpListener,
    pub url: Url,
    pub redirect_uri: String,
    verifier: Zeroizing<String>,
    state: Zeroizing<String>,
    deadline: Instant,
}
impl PendingAuthorization {
    pub fn new(
        client_id: &str,
        requested_scopes: &[String],
        timeout: Duration,
    ) -> AuthResult<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|_| AuthError::new(AuthCode::CallbackInvalid))?;
        listener
            .set_nonblocking(true)
            .map_err(|_| AuthError::new(AuthCode::CallbackInvalid))?;
        let port = listener
            .local_addr()
            .map_err(|_| AuthError::new(AuthCode::CallbackInvalid))?
            .port();
        let redirect_uri = format!("http://127.0.0.1:{port}/oauth/callback");
        let verifier = Zeroizing::new(random_urlsafe());
        let state = Zeroizing::new(random_urlsafe());
        let mut url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
            .map_err(|_| AuthError::new(AuthCode::InvalidConfiguration))?;
        url.query_pairs_mut()
            .append_pair("client_id", client_id)
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", &requested_scopes.join(" "))
            .append_pair("state", &state)
            .append_pair("code_challenge", &challenge(&verifier))
            .append_pair("code_challenge_method", "S256")
            .append_pair("access_type", "offline")
            .append_pair("prompt", "consent select_account");
        // Native Google clients do not support incremental authorization. No
        // include_granted_scopes=true: every consent requests the full selected set.
        Ok(Self {
            listener,
            url,
            redirect_uri,
            verifier,
            state,
            deadline: Instant::now() + timeout,
        })
    }
    pub fn wait(
        self,
        cancelled: impl Fn() -> bool,
    ) -> AuthResult<(Zeroizing<String>, String, Zeroizing<String>)> {
        while Instant::now() < self.deadline {
            if cancelled() {
                return Err(AuthError::new(AuthCode::Cancelled));
            }
            match self.listener.accept() {
                Ok((mut stream, peer)) => {
                    if !peer.ip().is_loopback() {
                        continue;
                    }
                    let deadline = self.deadline.min(Instant::now() + Duration::from_secs(2));
                    let parsed = read_request(&mut stream, deadline)
                        .and_then(|request| callback(&request, &self.redirect_uri, &self.state));
                    match parsed {
                        Ok(code) => {
                            respond(&mut stream, true);
                            return Ok((Zeroizing::new(code), self.redirect_uri, self.verifier));
                        }
                        Err(error) if error.code == AuthCode::ConsentDenied => {
                            respond(&mut stream, false);
                            return Err(error);
                        }
                        Err(_) => respond(&mut stream, false),
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(25))
                }
                Err(_) => return Err(AuthError::new(AuthCode::CallbackInvalid)),
            }
        }
        Err(AuthError::new(AuthCode::Timeout))
    }
}

fn read_request(stream: &mut TcpStream, deadline: Instant) -> AuthResult<Zeroizing<String>> {
    let mut data = Zeroizing::new(Vec::new());
    let mut buffer = [0u8; 1024];
    while data.len() < 8192 {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or_else(|| AuthError::new(AuthCode::Timeout))?;
        stream
            .set_read_timeout(Some(remaining))
            .map_err(|_| AuthError::new(AuthCode::CallbackInvalid))?;
        let count = stream
            .read(&mut buffer)
            .map_err(|_| AuthError::new(AuthCode::CallbackInvalid))?;
        if count == 0 {
            break;
        }
        data.extend_from_slice(&buffer[..count]);
        if data.windows(4).any(|window| window == b"\r\n\r\n") {
            return String::from_utf8(data.to_vec())
                .map(Zeroizing::new)
                .map_err(|_| AuthError::new(AuthCode::CallbackInvalid));
        }
    }
    Err(AuthError::new(AuthCode::CallbackInvalid))
}
fn constant_time_equal(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |difference, (a, b)| difference | (a ^ b))
            == 0
}
fn callback(request: &str, redirect_uri: &str, state: &str) -> AuthResult<String> {
    let bad = || AuthError::new(AuthCode::CallbackInvalid);
    let mut lines = request.split("\r\n");
    let parts: Vec<_> = lines.next().ok_or_else(bad)?.split_whitespace().collect();
    if parts.len() != 3
        || parts[0] != "GET"
        || parts[2] != "HTTP/1.1"
        || !parts[1].starts_with('/')
        || parts[1].starts_with("//")
    {
        return Err(bad());
    }
    let redirect = Url::parse(redirect_uri).map_err(|_| bad())?;
    let expected_host = format!("127.0.0.1:{}", redirect.port().ok_or_else(bad)?);
    let hosts: Vec<_> = lines
        .filter_map(|line| line.split_once(':'))
        .filter(|(name, _)| name.eq_ignore_ascii_case("host"))
        .map(|(_, value)| value.trim())
        .collect();
    if hosts.len() != 1 || hosts[0] != expected_host {
        return Err(bad());
    }
    let parsed = redirect.join(parts[1]).map_err(|_| bad())?;
    if parsed.origin() != redirect.origin()
        || parsed.path() != redirect.path()
        || parsed.fragment().is_some()
    {
        return Err(bad());
    }
    let pairs: Vec<_> = parsed.query_pairs().collect();
    let values = |key: &str| {
        pairs
            .iter()
            .filter(|(name, _)| name == key)
            .map(|(_, value)| value.as_ref())
            .collect::<Vec<_>>()
    };
    let states = values("state");
    let codes = values("code");
    let errors = values("error");
    if states.len() != 1 || !constant_time_equal(states[0], state) {
        return Err(bad());
    }
    if errors.len() == 1 && codes.is_empty() {
        return Err(AuthError::new(AuthCode::ConsentDenied));
    }
    if codes.len() != 1 || !errors.is_empty() || codes[0].is_empty() || codes[0].len() > 4096 {
        return Err(bad());
    }
    Ok(codes[0].to_owned())
}
fn respond(stream: &mut TcpStream, success: bool) {
    let (status, title, detail) = if success {
        (
            "200 OK",
            "Autenticação concluída",
            "Autorização recebida com sucesso. Você já pode fechar esta aba e voltar ao Sheets Monitor.",
        )
    } else {
        (
            "400 Bad Request",
            "Autorização não concluída",
            "Não foi possível concluir o login no navegador. Volte ao Sheets Monitor para conferir os detalhes.",
        )
    };
    let body = format!(
        "<!doctype html><html lang=\"pt-BR\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Sheets Monitor — {title}</title><style>body{{font-family:'Segoe UI',system-ui,sans-serif;background:#f4efe4;color:#24231f;display:grid;place-items:center;min-height:100vh;margin:0;padding:24px}}main{{max-width:440px;background:#fbf8f2;border:1px solid #dfd7c7;border-radius:12px;padding:28px;box-shadow:0 4px 16px rgba(36,35,31,.06)}}h1{{font-size:19px;margin:0 0 10px;color:#1f563b}}p{{font-size:14px;line-height:1.6;margin:0;color:#625c51}}</style></head><body><main><h1>{title}</h1><p>{detail}</p></main></body></html>"
    );
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
    let _ = write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nContent-Security-Policy: default-src 'none'; style-src 'unsafe-inline'\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(query: &str) -> String {
        format!("GET /oauth/callback?{query} HTTP/1.1\r\nHost: 127.0.0.1:40001\r\n\r\n")
    }
    #[test]
    fn pkce_matches_official_rfc7636_vector_and_requested_scopes_are_read_only() {
        assert_eq!(
            challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
        assert_eq!(random_urlsafe().len(), 43);
        let pending = PendingAuthorization::new(
            "fixture.apps.googleusercontent.com",
            &scopes(false, false, false),
            Duration::from_secs(2),
        )
        .unwrap();
        let query: std::collections::HashMap<_, _> =
            pending.url.query_pairs().into_owned().collect();
        assert_eq!(query["code_challenge_method"], "S256");
        assert!(!query.contains_key("include_granted_scopes"));
        assert!(!query["scope"].contains("drive"));
        assert!(scopes(true, false, false).contains(&DRIVE_SCOPE.to_string()));
        assert!(!scopes(true, false, false)
            .iter()
            .any(|s| s.ends_with("drive.file") || s.ends_with("drive.readonly")));
        assert!(pending.listener.local_addr().unwrap().ip().is_loopback());
    }
    #[test]
    fn callback_rejects_wrong_state_host_path_duplicate_and_injected_parameters() {
        let redirect = "http://127.0.0.1:40001/oauth/callback";
        assert_eq!(
            callback(&request("state=ok&code=fixture"), redirect, "ok").unwrap(),
            "fixture"
        );
        for query in [
            "state=no&code=fixture",
            "code=fixture",
            "state=ok&state=ok&code=x",
            "state=ok&code=x&code=y",
            "state=ok&code=x&error=denied",
        ] {
            assert_eq!(
                callback(&request(query), redirect, "ok").unwrap_err().code,
                AuthCode::CallbackInvalid
            );
        }
        for altered in [
            request("state=ok&code=x").replace("Host: 127.0.0.1:40001", "Host: evil.invalid"),
            request("state=ok&code=x").replace("\r\n\r\n", "\r\nHost: 127.0.0.1:40001\r\n\r\n"),
            request("state=ok&code=x").replace("/oauth/callback", "/wrong"),
            request("state=ok&code=x").replace("GET /", "GET http://evil.invalid/"),
        ] {
            assert!(callback(&altered, redirect, "ok").is_err());
        }
        assert_eq!(
            callback(&request("state=ok&error=access_denied"), redirect, "ok")
                .unwrap_err()
                .code,
            AuthCode::ConsentDenied
        );
    }
    #[test]
    fn loopback_is_single_use_times_out_and_can_be_cancelled_without_browser() {
        let pending = PendingAuthorization::new(
            "fixture.apps.googleusercontent.com",
            &scopes(false, false, false),
            Duration::from_millis(5),
        )
        .unwrap();
        assert_eq!(pending.wait(|| false).unwrap_err().code, AuthCode::Timeout);
        let pending = PendingAuthorization::new(
            "fixture.apps.googleusercontent.com",
            &scopes(false, false, false),
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(pending.wait(|| true).unwrap_err().code, AuthCode::Cancelled);
    }
    #[test]
    fn fragmented_local_request_cannot_extend_absolute_deadline() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let sender = thread::spawn(move || {
            let mut stream = TcpStream::connect(address).unwrap();
            for _ in 0..200 {
                if stream.write_all(b"G").is_err() {
                    break;
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
        let (mut stream, _) = listener.accept().unwrap();
        let start = Instant::now();
        assert!(read_request(&mut stream, start + Duration::from_millis(40)).is_err());
        assert!(start.elapsed() < Duration::from_millis(500));
        drop(stream);
        sender.join().unwrap();
    }
}
