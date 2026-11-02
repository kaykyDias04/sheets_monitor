//! Read-only Google transport. No caller-provided endpoint can receive a bearer token.
use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::{collections::{BTreeMap, BTreeSet}, io::Read, thread, time::{Duration,Instant}};
use url::Url;

pub const MAX_RESPONSE_BYTES: u64 = 64 * 1024 * 1024;

pub struct HttpReply {
    pub status: u16,
    pub body: Vec<u8>,
    pub retry_after: Option<u64>,
}

pub trait ReadTransport {
    fn get(&self, url: &Url, bearer: &str, timeout:Duration) -> Result<HttpReply, String>;
    fn wait(&self, duration: Duration) { thread::sleep(duration); }
}

pub struct HttpsTransport { client: Client }

impl HttpsTransport {
    pub fn new() -> Result<Self, String> {
        let client = Client::builder().timeout(Duration::from_secs(45))
            .connect_timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build().map_err(|_| "Não foi possível iniciar transporte HTTPS".to_string())?;
        Ok(Self { client })
    }
}

impl ReadTransport for HttpsTransport {
    fn get(&self, url: &Url, bearer: &str, timeout:Duration) -> Result<HttpReply, String> {
        validate_api_url(url)?;
        let response = self.client.get(url.clone()).timeout(timeout.min(Duration::from_secs(45))).bearer_auth(bearer).send()
            .map_err(|_| "Falha de rede ao consultar Google; snapshot anterior preservado".to_string())?;
        let status = response.status().as_u16();
        let retry_after = response.headers().get("retry-after")
            .and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok());
        if response.content_length().is_some_and(|length| length > MAX_RESPONSE_BYTES) {
            return Err("Resposta Google excedeu 64 MiB; coleta incompleta descartada".into());
        }
        let mut body = Vec::new();
        response.take(MAX_RESPONSE_BYTES + 1).read_to_end(&mut body)
            .map_err(|_| "Resposta Google interrompida; coleta descartada".to_string())?;
        if body.len() as u64 > MAX_RESPONSE_BYTES {
            return Err("Resposta Google excedeu 64 MiB; coleta incompleta descartada".into());
        }
        Ok(HttpReply { status, body, retry_after })
    }
}

fn validate_api_url(url: &Url) -> Result<(), String> {
    let allowed = match url.host_str() {
        Some("sheets.googleapis.com") => url.path().starts_with("/v4/spreadsheets/"),
        Some("www.googleapis.com") => url.path().starts_with("/drive/v3/files/") || url.path() == "/drive/v3/about",
        Some("script.googleapis.com") => url.path().starts_with("/v1/projects/"),
        _ => false,
    };
    if !allowed || url.scheme() != "https" || url.port().is_some_and(|p| p != 443)
        || !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return Err("Endpoint Google não permitido".into());
    }
    Ok(())
}

pub fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 256 || !id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-') {
        return Err("ID Google inválido: informe somente o ID, sem URL".into());
    }
    Ok(())
}

pub struct GoogleApi<T: ReadTransport = HttpsTransport> {
    transport: T,
    bearer: String,
}

impl GoogleApi<HttpsTransport> {
    pub fn new(access_token: String) -> Result<Self, String> {
        Ok(Self { transport: HttpsTransport::new()?, bearer: access_token })
    }
}

impl<T: ReadTransport> GoogleApi<T> {
    /// Stable identity for partitioning local baselines, not an email or display name.
    pub fn account_identity(&self) -> Result<String, String> {
        let mut url = Url::parse("https://www.googleapis.com/drive/v3/about")
            .map_err(|_| "URL de identidade inválida".to_string())?;
        url.query_pairs_mut().append_pair("fields", "user(permissionId)");
        let identity = self.get_json(url,Instant::now()+Duration::from_secs(60))?.get("user").and_then(|u| u.get("permissionId"))
            .and_then(Value::as_str).map(str::to_owned).ok_or("Google não informou identidade estável da conta")?;
        validate_id(&identity)?;
        Ok(identity)
    }

    fn get_json(&self, url: Url, deadline:Instant) -> Result<Value, String> {
        validate_api_url(&url)?;
        for attempt in 0..4 {
            let remaining=deadline.checked_duration_since(Instant::now()).filter(|d|!d.is_zero())
                .ok_or("Tempo total de coleta esgotado; captura anterior preservada")?;
            let reply = self.transport.get(&url, &self.bearer,remaining)?;
            if Instant::now()>=deadline {return Err("Tempo total de coleta esgotado; captura anterior preservada".into());}
            let retry = matches!(reply.status, 429 | 500 | 502 | 503 | 504)
                || (reply.status == 403 && rate_limited(&reply.body));
            if retry && attempt < 3 {
                let delay = reply.retry_after.map(|seconds| Duration::from_secs(seconds.min(5)))
                    .unwrap_or(Duration::from_millis(250 * (1 << attempt)));
                if deadline.checked_duration_since(Instant::now()).is_none_or(|remaining|remaining<=delay) {
                    return Err("Tempo total de coleta esgotado; captura anterior preservada".into());
                }
                self.transport.wait(delay);
                continue;
            }
            if reply.status != 200 {
                let meaning = match reply.status {
                    401 => "sessão expirada; reconecte a conta",
                    403 => "acesso negado, API desativada ou quota insuficiente",
                    404 => "recurso inexistente ou sem acesso",
                    429 => "quota excedida após tentativas limitadas",
                    _ => "consulta falhou; coleta incompleta descartada",
                };
                return Err(format!("Google HTTP {}: {}", reply.status, meaning));
            }
            let value: Value = serde_json::from_slice(&reply.body)
                .map_err(|_| "Google retornou JSON inválido; coleta descartada".to_string())?;
            if !value.is_object() { return Err("Google retornou recurso inválido".into()); }
            return Ok(value);
        }
        Err("Tentativas Google esgotadas".into())
    }

    /// Full collection is atomic from the caller's perspective: an unavailable script,
    /// permission page, or grid produces an error, never a misleading partial snapshot.
    pub fn fetch_snapshot(&self, sheet_id: &str, script_ids: &[String]) -> Result<Value, String> {
        let deadline=Instant::now()+Duration::from_secs(600);
        validate_id(sheet_id)?;
        if script_ids.len() > 30 { return Err("Máximo de 30 scripts autorizados por monitor".into()); }
        let mut seen = BTreeSet::new();
        for id in script_ids {
            validate_id(id)?;
            if !seen.insert(id) { return Err("ID de script duplicado".into()); }
        }
        let mut sheets_url = Url::parse(&format!("https://sheets.googleapis.com/v4/spreadsheets/{sheet_id}"))
            .map_err(|_| "URL de planilha inválida".to_string())?;
        sheets_url.query_pairs_mut().append_pair("includeGridData", "true");
        let sheets = self.get_json(sheets_url,deadline)?;
        if sheets.get("spreadsheetId").and_then(Value::as_str) != Some(sheet_id) || !sheets.get("sheets").is_some_and(Value::is_array) {
            return Err("Resposta Sheets não corresponde à planilha solicitada".into());
        }
        let mut drive_url = Url::parse(&format!("https://www.googleapis.com/drive/v3/files/{sheet_id}"))
            .map_err(|_| "URL Drive inválida".to_string())?;
        drive_url.query_pairs_mut().append_pair("fields", "*").append_pair("supportsAllDrives", "true");
        let drive = self.get_json(drive_url,deadline)?;
        if drive.get("id").and_then(Value::as_str) != Some(sheet_id)
            || !drive.get("name").is_some_and(Value::is_string)
            || drive.get("mimeType").and_then(Value::as_str) != Some("application/vnd.google-apps.spreadsheet") {
            return Err("Resposta Drive não corresponde à planilha solicitada".into());
        }
        let permissions = self.permissions(sheet_id,deadline)?;
        let mut scripts = BTreeMap::new();
        for id in script_ids {
            let url = Url::parse(&format!("https://script.googleapis.com/v1/projects/{id}/content"))
                .map_err(|_| "URL Apps Script inválida".to_string())?;
            let mut content = self.get_json(url,deadline)?;
            if content.get("scriptId").and_then(Value::as_str) != Some(id.as_str())
                || !content.get("files").is_some_and(Value::is_array) {
                return Err("Resposta Apps Script inválida; coleta descartada".into());
            }
            if content["files"].as_array().is_some_and(|files| files.iter().any(|file|
                !file.get("name").is_some_and(Value::is_string)
                || !file.get("source").is_some_and(Value::is_string)
                || !matches!(file.get("type").and_then(Value::as_str),Some("SERVER_JS"|"HTML"|"JSON")))) {
                return Err("Arquivo Apps Script incompleto; coleta descartada".into());
            }
            // API file order is not semantically meaningful; source text remains exact.
            if let Some(files) = content.get_mut("files").and_then(Value::as_array_mut) {
                files.sort_by_key(|f| (f.get("name").and_then(Value::as_str).unwrap_or("").to_owned(),
                    f.get("type").and_then(Value::as_str).unwrap_or("").to_owned()));
            }
            scripts.insert(id.clone(), content);
        }
        Ok(json!({"sheets":sheets,"drive":drive,"permissions":permissions,"scripts":scripts}))
    }

    fn permissions(&self, id: &str,deadline:Instant) -> Result<Vec<Value>, String> {
        let mut all = Vec::new();
        let mut page_token = None::<String>;
        let mut seen = BTreeSet::new();
        for _ in 0..1000 {
            let mut url = Url::parse(&format!("https://www.googleapis.com/drive/v3/files/{id}/permissions"))
                .map_err(|_| "URL de permissões inválida".to_string())?;
            url.query_pairs_mut().append_pair("fields", "*").append_pair("pageSize", "100").append_pair("supportsAllDrives", "true");
            if let Some(ref token) = page_token { url.query_pairs_mut().append_pair("pageToken", token); }
            let page = self.get_json(url,deadline)?;
            let permissions = page.get("permissions").and_then(Value::as_array)
                .ok_or("Página de permissões inválida; coleta descartada")?;
            all.extend(permissions.iter().cloned());
            match page.get("nextPageToken").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                None => {
                    all.sort_by_key(|p| p.get("id").and_then(Value::as_str).unwrap_or("").to_owned());
                    return Ok(all);
                },
                Some(token) => {
                    if !seen.insert(token.to_owned()) { return Err("Paginação Google repetida; coleta descartada".into()); }
                    page_token = Some(token.to_owned());
                }
            }
        }
        Err("Paginação excedeu limite de segurança; coleta descartada".into())
    }
}

fn rate_limited(body: &[u8]) -> bool {
    serde_json::from_slice::<Value>(body).ok()
        .and_then(|v| v.get("error")?.get("errors")?.as_array().cloned())
        .is_some_and(|errors| errors.iter().any(|e| matches!(e.get("reason").and_then(Value::as_str), Some("rateLimitExceeded" | "userRateLimitExceeded"))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, collections::VecDeque};
    struct Mock { replies: RefCell<VecDeque<HttpReply>>, urls: RefCell<Vec<Url>>, waits: RefCell<usize> }
    impl Mock {
        fn new(replies: Vec<(u16, Value)>) -> Self {
            Self { replies: RefCell::new(replies.into_iter().map(|(status,v)| HttpReply {status,body:serde_json::to_vec(&v).unwrap(),retry_after:None}).collect()), urls:RefCell::new(vec![]),waits:RefCell::new(0) }
        }
    }
    impl ReadTransport for Mock {
        fn get(&self,url:&Url,bearer:&str,timeout:Duration)->Result<HttpReply,String> {
            assert!(!timeout.is_zero());
            assert_eq!(bearer,"fixture-not-a-real-token");
            self.urls.borrow_mut().push(url.clone());
            self.replies.borrow_mut().pop_front().ok_or("fixture esgotada".into())
        }
        fn wait(&self,_duration:Duration) { *self.waits.borrow_mut() += 1; }
    }
    fn api(replies:Vec<(u16,Value)>)->GoogleApi<Mock> { GoogleApi {transport:Mock::new(replies),bearer:"fixture-not-a-real-token".into()} }
    #[test]
    fn paginates_and_fetches_explicit_script_without_writes() {
        let api=api(vec![(200,json!({"spreadsheetId":"sheet_1","sheets":[]})),(200,json!({"id":"sheet_1","name":"Fixture","mimeType":"application/vnd.google-apps.spreadsheet"})),
            (200,json!({"permissions":[{"id":"b"}],"nextPageToken":"page2"})),(200,json!({"permissions":[{"id":"a"}]})),
            (200,json!({"scriptId":"script_1","files":[{"name":"Z","type":"SERVER_JS","source":"x"},{"name":"A","type":"SERVER_JS","source":"y"}]}))]);
        let snapshot=api.fetch_snapshot("sheet_1", &["script_1".into()]).unwrap();
        assert_eq!(snapshot["permissions"][0]["id"],"a");
        assert_eq!(snapshot["scripts"]["script_1"]["files"][0]["name"],"A");
        let urls=api.transport.urls.borrow();
        assert!(urls[0].query().unwrap().contains("includeGridData=true"));
        assert!(urls[3].query().unwrap().contains("pageToken=page2"));
    }
    #[test]
    fn unavailable_authorized_script_fails_entire_collection() {
        let api=api(vec![(200,json!({"spreadsheetId":"x","sheets":[]})),(200,json!({"id":"x","name":"Fixture","mimeType":"application/vnd.google-apps.spreadsheet"})),(200,json!({"permissions":[]})),(403,json!({"error":{"message":"SECRET MUST NOT APPEAR"}}))]);
        let error=api.fetch_snapshot("x", &["s".into()]).unwrap_err();
        assert!(error.contains("403"));assert!(!error.contains("SECRET"));
    }
    #[test]
    fn retries_bounded_and_never_parses_failure_as_snapshot() {
        let api=api((0..4).map(|_|(429,json!({"error":"quota"}))).collect());
        assert!(api.fetch_snapshot("x", &[]).unwrap_err().contains("429"));
        assert_eq!(*api.transport.waits.borrow(),3);
        assert_eq!(api.transport.urls.borrow().len(),4);
    }
    #[test]
    fn rejects_bad_ids_before_network_and_bearer_exfiltration_endpoints() {
        let api=api(vec![]);
        for id in ["../secret","https://evil.invalid","x?fields=secret",""] { assert!(api.fetch_snapshot(id,&[]).is_err()); }
        for url in ["http://sheets.googleapis.com/v4/spreadsheets/x","https://evil.invalid/v4/spreadsheets/x","https://sheets.googleapis.com:444/v4/spreadsheets/x","https://user@sheets.googleapis.com/v4/spreadsheets/x"] {
            assert!(validate_api_url(&Url::parse(url).unwrap()).is_err());
        }
        assert!(api.transport.urls.borrow().is_empty());
    }
    #[test]
    fn rejects_repeated_page_token() {
        let api=api(vec![(200,json!({"spreadsheetId":"x","sheets":[]})),(200,json!({"id":"x","name":"Fixture","mimeType":"application/vnd.google-apps.spreadsheet"})),
            (200,json!({"permissions":[],"nextPageToken":"same"})),(200,json!({"permissions":[],"nextPageToken":"same"}))]);
        assert!(api.fetch_snapshot("x",&[]).unwrap_err().contains("repetida"));
    }
    #[test]
    fn account_identity_uses_only_readonly_about_resource() {
        let first=api(vec![(200,json!({"user":{"permissionId":"1234567890"}}))]);
        assert_eq!(first.account_identity().unwrap(),"1234567890");
        assert_eq!(first.transport.urls.borrow()[0].path(),"/drive/v3/about");
        let missing=api(vec![(200,json!({"user":{"displayName":"not-an-identity"}}))]);
        assert!(missing.account_identity().is_err());
    }
    #[test]
    fn expired_collection_budget_never_sends_another_request() {
        let api=api(vec![]);
        let url=Url::parse("https://sheets.googleapis.com/v4/spreadsheets/x").unwrap();
        assert!(api.get_json(url,Instant::now()-Duration::from_secs(1)).unwrap_err().contains("esgotado"));
        assert!(api.transport.urls.borrow().is_empty());
    }
}
