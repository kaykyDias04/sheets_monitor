//! Desktop PKCE: browser-system consent, loopback callback, backend-only credentials.
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{rngs::OsRng, RngCore};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, thread, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use url::Url;

pub const SCOPES: [&str; 3] = [
    "https://www.googleapis.com/auth/spreadsheets.readonly",
    "https://www.googleapis.com/auth/drive.metadata.readonly",
    "https://www.googleapis.com/auth/script.projects.readonly",
];
const MAX_TOKEN_BYTES: u64 = 64 * 1024;
const VAULT_SERVICE: &str = "SheetsMonitor.OAuth.v1";

#[derive(Clone, Deserialize, Serialize)]
pub struct OAuthConfig {
    pub client_id: String,
    #[serde(default, skip_serializing)]
    pub client_secret: Option<String>,
}

impl std::fmt::Debug for OAuthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuthConfig").field("client_id", &"[configured]")
            .field("client_secret", &"[redacted]").finish()
    }
}

impl OAuthConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !self.client_id.ends_with(".apps.googleusercontent.com")
            || self.client_id.len() > 256
            || !self.client_id.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_')) {
            return Err("Configure o client ID OAuth do tipo Aplicativo para computador".into());
        }
        if self.client_secret.as_ref().is_some_and(|s| s.len() > 2048 || s.contains(['\r','\n'])) {
            return Err("Client secret OAuth inválido".into());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Tokens {
    client_id: String,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: u64,
}

#[derive(Deserialize)]
struct TokenReply {
    access_token: String,
    token_type: String,
    #[serde(default)] refresh_token: Option<String>,
    expires_in: u64,
    #[serde(default)] scope: Option<String>,
}

fn now() -> Result<u64, String> {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs())
        .map_err(|_| "Relógio do sistema inválido".into())
}

fn vault_account(config: &OAuthConfig) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(config.client_id.as_bytes()))
}

fn vault_entry(config: &OAuthConfig) -> Result<keyring::Entry, String> {
    #[cfg(not(windows))]
    { let _ = config; return Err("Armazenamento OAuth requer Windows Credential Manager nesta V1".into()); }
    #[cfg(windows)]
    keyring::Entry::new(VAULT_SERVICE, &vault_account(config))
        .map_err(|_| "Não foi possível acessar o Windows Credential Manager".into())
}

fn load(config: &OAuthConfig) -> Result<Option<Tokens>, String> {
    match vault_entry(config)?.get_password() {
        Ok(raw) => {
            let tokens: Tokens = serde_json::from_str(&raw).map_err(|_| "Credencial local inválida; desconecte e reconecte".to_string())?;
            if tokens.client_id != config.client_id { return Err("Credencial pertence a outro client ID".into()); }
            Ok(Some(tokens))
        },
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("Falha ao ler credencial protegida do Windows".into()),
    }
}

fn save(config: &OAuthConfig, tokens: &Tokens) -> Result<(), String> {
    let data = serde_json::to_string(tokens).map_err(|_| "Falha ao preparar credencial".to_string())?;
    vault_entry(config)?.set_password(&data)
        .map_err(|_| "Falha ao salvar no Windows Credential Manager; sessão não foi concluída".into())
}

pub fn is_connected(config: &OAuthConfig) -> Result<bool, String> {
    config.validate()?;
    Ok(load(config)?.is_some())
}

/// Deletes only the local credential. Does not revoke grants or modify Google files.
pub fn disconnect(config: &OAuthConfig) -> Result<(), String> {
    config.validate()?;
    match vault_entry(config)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("Não foi possível remover a credencial do Windows".into()),
    }
}

fn http_client() -> Result<Client, String> {
    Client::builder().timeout(Duration::from_secs(30)).connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none()).build()
        .map_err(|_| "Falha ao iniciar transporte OAuth".into())
}

fn token_request(form: &[(&str, &str)]) -> Result<TokenReply, String> {
    let response = http_client()?.post("https://oauth2.googleapis.com/token").form(form).send()
        .map_err(|_| "Falha de rede no OAuth; tente conectar novamente".to_string())?;
    let status = response.status().as_u16();
    if status != 200 {
        return Err(format!("OAuth HTTP {status}: credencial recusada; verifique configuração e reconecte"));
    }
    let mut bytes = Vec::new();
    response.take(MAX_TOKEN_BYTES + 1).read_to_end(&mut bytes)
        .map_err(|_| "Resposta OAuth interrompida".to_string())?;
    if bytes.len() as u64 > MAX_TOKEN_BYTES { return Err("Resposta OAuth excedeu limite".into()); }
    serde_json::from_slice(&bytes).map_err(|_| "Resposta OAuth inválida".into())
}

fn into_tokens(config: &OAuthConfig, reply: TokenReply, old_refresh: Option<String>) -> Result<Tokens, String> {
    if !reply.token_type.eq_ignore_ascii_case("bearer") || reply.access_token.is_empty()
        || reply.access_token.len() > 16_384 || reply.expires_in == 0 || reply.expires_in > 31_536_000 {
        return Err("Resposta OAuth contém token inválido".into());
    }
    if let Some(ref scope) = reply.scope {
        let granted: Vec<_> = scope.split_whitespace().collect();
        if SCOPES.iter().any(|required| !granted.contains(required)) {
            return Err("Consentimento parcial: todos os três escopos de leitura são necessários".into());
        }
    }
    let refresh_token = reply.refresh_token.filter(|s| !s.is_empty()).or(old_refresh);
    Ok(Tokens { client_id: config.client_id.clone(), access_token: reply.access_token,
        refresh_token, expires_at: now()?.saturating_add(reply.expires_in) })
}

/// Returns a token only to Rust callers. Never expose this function as an IPC command.
pub fn access_token(config: &OAuthConfig) -> Result<String, String> {
    config.validate()?;
    let tokens = load(config)?.ok_or("Conecte uma conta Google antes de coletar")?;
    if tokens.expires_at > now()?.saturating_add(60) { return Ok(tokens.access_token); }
    let refresh = tokens.refresh_token.ok_or("Sessão expirada sem refresh token; reconecte")?;
    let mut form = vec![("client_id", config.client_id.as_str()), ("grant_type", "refresh_token"), ("refresh_token", refresh.as_str())];
    if let Some(ref secret) = config.client_secret { form.push(("client_secret", secret.as_str())); }
    let reply = token_request(&form)?;
    let tokens = into_tokens(config, reply, Some(refresh))?;
    save(config, &tokens)?;
    Ok(tokens.access_token)
}

fn random_urlsafe() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn challenge(verifier: &str) -> String { URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())) }

struct PendingAuthorization {
    listener: TcpListener,
    redirect_uri: String,
    state: String,
    verifier: String,
    authorization_url: Url,
    deadline: Instant,
}

impl PendingAuthorization {
    fn new(config: &OAuthConfig, timeout: Duration) -> Result<Self, String> {
        config.validate()?;
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|_| "Não foi possível abrir callback OAuth local".to_string())?;
        listener.set_nonblocking(true).map_err(|_| "Falha ao configurar callback OAuth".to_string())?;
        let port = listener.local_addr().map_err(|_| "Falha no endereço OAuth".to_string())?.port();
        let redirect_uri = format!("http://127.0.0.1:{port}/oauth/callback");
        let state = random_urlsafe();
        let verifier = random_urlsafe();
        let mut authorization_url = Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
            .map_err(|_| "Falha na URL OAuth".to_string())?;
        authorization_url.query_pairs_mut()
            .append_pair("client_id", &config.client_id).append_pair("redirect_uri", &redirect_uri)
            .append_pair("response_type", "code").append_pair("scope", &SCOPES.join(" "))
            .append_pair("state", &state).append_pair("code_challenge", &challenge(&verifier))
            .append_pair("code_challenge_method", "S256").append_pair("access_type", "offline")
            .append_pair("prompt", "consent");
        Ok(Self {listener,redirect_uri,state,verifier,authorization_url,deadline:Instant::now()+timeout})
    }

    // Ownership consumes the verifier/listener: one pending authorization cannot be replayed.
    fn wait(self) -> Result<(String, String, String), String> {
        while Instant::now() < self.deadline {
            match self.listener.accept() {
                Ok((mut stream, peer)) => {
                    if !peer.ip().is_loopback() { continue; }
                    let request = read_request(&mut stream, self.deadline.min(Instant::now()+Duration::from_secs(2)));
                    let result = request.and_then(|r| callback_code(&r, &self.redirect_uri, &self.state));
                    match result {
                        Ok(Callback::Code(code)) => {
                            respond(&mut stream, true);
                            return Ok((code,self.redirect_uri,self.verifier));
                        },
                        Ok(Callback::Denied) => {
                            respond(&mut stream, false);
                            return Err("Consentimento Google negado ou cancelado".into());
                        },
                        Err(_) => respond(&mut stream, false),
                    }
                },
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => thread::sleep(Duration::from_millis(50)),
                Err(_) => return Err("Callback OAuth local falhou".into()),
            }
        }
        Err("Tempo do consentimento esgotado; tente conectar novamente".into())
    }
}

fn read_request(stream: &mut TcpStream, deadline: Instant) -> Result<String, String> {
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 1024];
    while bytes.len() < 8192 {
        let remaining=deadline.checked_duration_since(Instant::now()).filter(|d| !d.is_zero()).ok_or("Callback expirado")?;
        stream.set_read_timeout(Some(remaining)).map_err(|_| "Callback inválido".to_string())?;
        let count = stream.read(&mut chunk).map_err(|_| "Callback incompleto".to_string())?;
        if count == 0 { break; }
        bytes.extend_from_slice(&chunk[..count]);
        if bytes.windows(4).any(|w| w == b"\r\n\r\n") {
            return String::from_utf8(bytes).map_err(|_| "Callback inválido".into());
        }
    }
    Err("Callback excedeu limite ou incompleto".into())
}

enum Callback { Code(String), Denied }

fn constant_time_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() { return false; }
    a.bytes().zip(b.bytes()).fold(0u8, |difference, (a,b)| difference | (a ^ b)) == 0
}

fn callback_code(request: &str, redirect_uri: &str, expected_state: &str) -> Result<Callback, String> {
    let mut lines = request.split("\r\n");
    let first = lines.next().ok_or("Callback vazio")?;
    let parts: Vec<_> = first.split_whitespace().collect();
    if parts.len() != 3 || parts[0] != "GET" || parts[2] != "HTTP/1.1" || !parts[1].starts_with('/') || parts[1].starts_with("//") {
        return Err("Método de callback inválido".into());
    }
    let expected = Url::parse(redirect_uri).map_err(|_| "Redirect inválido".to_string())?;
    let expected_host = format!("127.0.0.1:{}", expected.port().ok_or("Porta ausente")?);
    let hosts: Vec<_> = lines.filter_map(|line| line.split_once(':'))
        .filter(|(name,_)| name.eq_ignore_ascii_case("host")).map(|(_,value)| value.trim()).collect();
    if hosts.len() != 1 || hosts[0] != expected_host { return Err("Host de callback inválido".into()); }
    let callback = expected.join(parts[1]).map_err(|_| "URL callback inválida".to_string())?;
    if callback.path() != expected.path() || callback.origin() != expected.origin() || callback.fragment().is_some() {
        return Err("Caminho callback inválido".into());
    }
    let pairs: Vec<_> = callback.query_pairs().collect();
    let values = |key:&str| pairs.iter().filter(|(name,_)| name == key).map(|(_,value)| value.as_ref()).collect::<Vec<_>>();
    let states = values("state");
    if states.len()!=1 || !constant_time_equal(states[0], expected_state) { return Err("State OAuth inválido".into()); }
    let codes = values("code");
    let errors = values("error");
    if errors.len()==1 && codes.is_empty() { return Ok(Callback::Denied); }
    if codes.len()!=1 || !errors.is_empty() || codes[0].is_empty() || codes[0].len()>4096 { return Err("Código OAuth inválido".into()); }
    Ok(Callback::Code(codes[0].to_owned()))
}

fn respond(stream: &mut TcpStream, success: bool) {
    let body = if success { "Conexao recebida. Volte ao Sheets Monitor para concluir." } else { "Callback recusado. Volte ao Sheets Monitor." };
    let status = if success { "200 OK" } else { "400 Bad Request" };
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let _ = write!(stream,"HTTP/1.1 {status}\r\nContent-Type: text/plain; charset=utf-8\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",body.len());
}

/// Invoke only from an explicit user action. Requires desktop OAuth credentials.
pub fn connect(config: &OAuthConfig) -> Result<(), String> {
    let pending = PendingAuthorization::new(config, Duration::from_secs(180))?;
    open::that(pending.authorization_url.as_str()).map_err(|_| "Não foi possível abrir navegador do sistema".to_string())?;
    let (code,redirect,verifier) = pending.wait()?;
    let mut form = vec![("client_id",config.client_id.as_str()),("grant_type","authorization_code"),
        ("code",code.as_str()),("redirect_uri",redirect.as_str()),("code_verifier",verifier.as_str())];
    if let Some(ref secret)=config.client_secret { form.push(("client_secret",secret.as_str())); }
    let reply = token_request(&form)?;
    // A new consent can select a different Google account. Never reuse an old
    // account's refresh token when exchanging a fresh authorization code.
    let tokens=into_tokens(config,reply,None)?;
    save(config,&tokens)
}

/// Revokes the Google OAuth grant. Call only after an explicit request to revoke.
pub fn revoke(config: &OAuthConfig) -> Result<(), String> {
    config.validate()?;
    if let Some(tokens)=load(config)? {
        let token=tokens.refresh_token.unwrap_or(tokens.access_token);
        let response=http_client()?.post("https://oauth2.googleapis.com/revoke")
            .form(&[("token",token.as_str())]).send().map_err(|_| "Falha de rede na revogação; credencial local preservada".to_string())?;
        if response.status().as_u16()!=200 { return Err(format!("Revogação OAuth HTTP {}; credencial local preservada",response.status().as_u16())); }
    }
    disconnect(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config()->OAuthConfig { OAuthConfig {client_id:"fixture.apps.googleusercontent.com".into(),client_secret:Some("FIXTURE_SECRET".into())} }
    fn request(query:&str)->String {format!("GET /oauth/callback?{query} HTTP/1.1\r\nHost: 127.0.0.1:45001\r\n\r\n")}
    #[test]
    fn pkce_rfc7636_vector() {
        assert_eq!(challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),"E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
        let verifier=random_urlsafe();assert_eq!(verifier.len(),43);
        assert!(verifier.bytes().all(|b|b.is_ascii_alphanumeric() || b==b'-' || b==b'_'));
    }
    #[test]
    fn callback_requires_state_host_path_and_single_code() {
        let redirect="http://127.0.0.1:45001/oauth/callback";
        assert!(matches!(callback_code(&request("state=s&code=abc"),redirect,"s"),Ok(Callback::Code(ref c)) if c=="abc"));
        for query in ["state=wrong&code=abc","code=abc","state=s&state=s&code=abc","state=s&code=a&code=b","state=s&code=a&error=denied","state=s&code="] {
            assert!(callback_code(&request(query),redirect,"s").is_err());
        }
        assert!(callback_code(&request("state=s&code=abc").replace("Host: 127.0.0.1:45001","Host: evil.invalid"),redirect,"s").is_err());
        assert!(callback_code(&request("state=s&code=abc").replace("/oauth/callback","/wrong"),redirect,"s").is_err());
        assert!(matches!(callback_code(&request("state=s&error=access_denied"),redirect,"s"),Ok(Callback::Denied)));
    }
    #[test]
    fn consent_is_readonly_and_callback_binds_loopback_ephemeral_port() {
        let p=PendingAuthorization::new(&config(),Duration::from_secs(2)).unwrap();
        assert_eq!(p.listener.local_addr().unwrap().ip().to_string(),"127.0.0.1");
        assert_ne!(p.listener.local_addr().unwrap().port(),0);
        let query: std::collections::HashMap<_,_>=p.authorization_url.query_pairs().into_owned().collect();
        assert_eq!(query["code_challenge_method"],"S256");assert_eq!(query["response_type"],"code");
        assert_eq!(query["scope"],SCOPES.join(" "));
        assert!(SCOPES.iter().all(|s|s.ends_with(".readonly")));
        assert!(!p.authorization_url.as_str().contains("FIXTURE_SECRET"));
    }
    #[test]
    fn tokens_preserve_refresh_and_reject_partial_consent() {
        let reply=TokenReply {access_token:"fixture".into(),token_type:"Bearer".into(),refresh_token:None,expires_in:3600,scope:None};
        let tokens=into_tokens(&config(),reply,Some("refresh_fixture".into())).unwrap();
        assert_eq!(tokens.refresh_token.as_deref(),Some("refresh_fixture"));
        let partial=TokenReply {access_token:"fixture".into(),token_type:"Bearer".into(),refresh_token:None,expires_in:3600,scope:Some(SCOPES[0].into())};
        assert!(into_tokens(&config(),partial,None).is_err());
    }
    #[test]
    fn config_never_serializes_or_debugs_secret_and_vault_is_client_bound() {
        let a=config();let mut b=config();b.client_id="different.apps.googleusercontent.com".into();
        assert!(!serde_json::to_string(&a).unwrap().contains("FIXTURE_SECRET"));
        assert!(!format!("{a:?}").contains("FIXTURE_SECRET"));
        assert_ne!(vault_account(&a),vault_account(&b));
    }
    #[test]
    fn callback_timeout_is_bounded_without_browser() {
        let p=PendingAuthorization::new(&config(),Duration::from_millis(5)).unwrap();
        assert!(p.wait().unwrap_err().contains("esgotado"));
    }
    #[test]
    fn partial_callback_cannot_extend_absolute_deadline() {
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();
        let address=listener.local_addr().unwrap();
        let sender=thread::spawn(move || {
            let mut stream=TcpStream::connect(address).unwrap();
            for _ in 0..200 {
                if stream.write_all(b"G").is_err() {break;}
                thread::sleep(Duration::from_millis(10));
            }
        });
        let (mut stream,_)=listener.accept().unwrap();
        let start=Instant::now();
        assert!(read_request(&mut stream,start+Duration::from_millis(35)).is_err());
        assert!(start.elapsed()<Duration::from_millis(500));
        drop(stream);sender.join().unwrap();
    }
}
