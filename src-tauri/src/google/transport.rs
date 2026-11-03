use reqwest::{
    blocking::Client,
    header::{HeaderValue, AUTHORIZATION},
};
use serde::Serialize;
use std::{
    io::Read,
    thread,
    time::{Duration, Instant},
};
use url::Url;
pub const MAX_RESPONSE_BYTES: u64 = 8 * 1024 * 1024;
const MAX_ATTEMPTS: u32 = 3;
const MAX_INLINE_RETRY: Duration = Duration::from_secs(1);
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ApiError {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    RateLimited,
    Server { status: u16 },
    Network,
    Timeout,
    InvalidResponse,
    ResponseTooLarge,
    InvalidInput,
    Cancelled,
}
impl ApiError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::BadRequest => "bad_request",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not_found",
            Self::RateLimited => "rate_limited",
            Self::Server { .. } => "server",
            Self::Network => "network",
            Self::Timeout => "timeout",
            Self::InvalidResponse => "invalid_response",
            Self::ResponseTooLarge => "response_too_large",
            Self::InvalidInput => "invalid_input",
            Self::Cancelled => "cancelled",
        }
    }
}
impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::BadRequest => "Google recusou o intervalo ou parâmetro da consulta (400).",
            Self::Unauthorized => "A sessão não foi autorizada (401); reconecte a conta.",
            Self::Forbidden => {
                "Acesso negado (403). Verifique escopos, API habilitada e permissão no recurso."
            }
            Self::NotFound => "Recurso inexistente, removido ou sem acesso (404).",
            Self::RateLimited => "Limite de consultas atingido; tente novamente mais tarde.",
            Self::Server { .. } => "O serviço Google está temporariamente indisponível.",
            Self::Network => "Falha de rede; nenhum resultado de sucesso foi produzido.",
            Self::Timeout => "A consulta excedeu o prazo. Tente um intervalo menor.",
            Self::InvalidResponse => "A resposta Google não corresponde ao formato esperado.",
            Self::ResponseTooLarge => "A resposta excedeu 8 MiB; reduza os intervalos consultados.",
            Self::InvalidInput => "ID, URL ou intervalos inválidos.",
            Self::Cancelled => "Consulta cancelada; resultado descartado.",
        })
    }
}
impl std::error::Error for ApiError {}
pub struct HttpReply {
    pub status: u16,
    pub body: Vec<u8>,
    pub retry_after: Option<Duration>,
}
pub trait ReadTransport {
    fn get(
        &self,
        url: &Url,
        authorization: &HeaderValue,
        timeout: Duration,
    ) -> Result<HttpReply, ApiError>;
    fn wait(&self, duration: Duration) {
        let deadline = Instant::now().checked_add(duration);
        while !crate::monitoring::work::cancelled() {
            let Some(remaining) = deadline.and_then(|d| d.checked_duration_since(Instant::now()))
            else {
                break;
            };
            if remaining.is_zero() {
                break;
            }
            thread::sleep(remaining.min(Duration::from_millis(50)));
        }
    }
    fn jitter(&self) -> Duration {
        Duration::from_millis(rand::random::<u64>() % 101)
    }
}
pub struct HttpsTransport {
    client: Client,
}
impl HttpsTransport {
    pub fn new() -> Result<Self, ApiError> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| ApiError::Network)?;
        Ok(Self { client })
    }
}
impl ReadTransport for HttpsTransport {
    fn get(
        &self,
        url: &Url,
        authorization: &HeaderValue,
        timeout: Duration,
    ) -> Result<HttpReply, ApiError> {
        validate_api_url(url)?;
        let response = self
            .client
            .get(url.clone())
            .header(AUTHORIZATION, authorization.clone())
            .timeout(timeout.min(Duration::from_secs(30)))
            .send()
            .map_err(|error| {
                if error.is_timeout() {
                    ApiError::Timeout
                } else {
                    ApiError::Network
                }
            })?;
        let status = response.status().as_u16();
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|h| h.to_str().ok())
            .and_then(parse_retry_after);
        if response
            .content_length()
            .is_some_and(|n| n > MAX_RESPONSE_BYTES)
        {
            return Err(ApiError::ResponseTooLarge);
        }
        let mut body = Vec::new();
        response
            .take(MAX_RESPONSE_BYTES + 1)
            .read_to_end(&mut body)
            .map_err(|error| {
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) {
                    ApiError::Timeout
                } else {
                    ApiError::Network
                }
            })?;
        if body.len() as u64 > MAX_RESPONSE_BYTES {
            return Err(ApiError::ResponseTooLarge);
        }
        Ok(HttpReply {
            status,
            body,
            retry_after,
        })
    }
}
pub(crate) fn parse_retry_after(raw: &str) -> Option<Duration> {
    if raw.len() > 128 {
        return None;
    }
    if let Ok(seconds) = raw.trim().parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let date = chrono::DateTime::parse_from_rfc2822(raw).ok()?;
    let seconds = (date.timestamp() - chrono::Utc::now().timestamp()).max(0) as u64;
    Some(Duration::from_secs(seconds))
}
pub(super) fn validate_api_url(url: &Url) -> Result<(), ApiError> {
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || url.port().is_some()
    {
        return Err(ApiError::InvalidInput);
    }
    let parts: Vec<_> = url.path_segments().ok_or(ApiError::InvalidInput)?.collect();
    let id = match (url.host_str(), parts.as_slice()) {
        (Some("sheets.googleapis.com"), ["v4", "spreadsheets", id]) => *id,
        (Some("sheets.googleapis.com"), ["v4", "spreadsheets", id, "values:batchGet"]) => *id,
        (Some("www.googleapis.com"), ["drive", "v3", "files", id]) => *id,
        (Some("www.googleapis.com"), ["drive", "v3", "files"]) => return Ok(()),
        (Some("script.googleapis.com"), ["v1", "projects", id]) => *id,
        (Some("script.googleapis.com"), ["v1", "projects", id, "content"]) => *id,
        _ => return Err(ApiError::InvalidInput),
    };
    super::validate_google_id(id)
}
pub(super) fn get_bounded<T: ReadTransport>(
    transport: &T,
    url: &Url,
    authorization: &HeaderValue,
    deadline: Instant,
) -> Result<Vec<u8>, ApiError> {
    validate_api_url(url)?;
    for attempt in 0..MAX_ATTEMPTS {
        if crate::monitoring::work::cancelled() {
            return Err(ApiError::Cancelled);
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|d| !d.is_zero())
            .ok_or(ApiError::Timeout)?;
        let reply = transport.get(url, authorization, remaining.min(Duration::from_secs(30)));
        let (result, retry_after) = match reply {
            Ok(reply) => {
                if reply.body.len() as u64 > MAX_RESPONSE_BYTES {
                    return Err(ApiError::ResponseTooLarge);
                }
                if reply.status == 200 {
                    return Ok(reply.body);
                }
                let error = match reply.status {
                    400 => ApiError::BadRequest,
                    401 => ApiError::Unauthorized,
                    403 if rate_limited(&reply.body) => ApiError::RateLimited,
                    403 => ApiError::Forbidden,
                    404 => ApiError::NotFound,
                    429 => ApiError::RateLimited,
                    500..=599 => ApiError::Server {
                        status: reply.status,
                    },
                    _ => ApiError::InvalidResponse,
                };
                (error, reply.retry_after)
            }
            Err(error) => (error, None),
        };
        let retry = matches!(
            result,
            ApiError::RateLimited | ApiError::Server { .. } | ApiError::Network | ApiError::Timeout
        );
        if let Some(delay) = retry_after {
            crate::monitoring::work::defer_retry(delay);
            // Preserve the full server embargo without blocking a worker for minutes.
            if retry && delay > MAX_INLINE_RETRY {
                return Err(result);
            }
        }
        if !retry || attempt + 1 == MAX_ATTEMPTS {
            return Err(result);
        }
        let delay =
            retry_after.unwrap_or(Duration::from_millis(250 * (1 << attempt)) + transport.jitter());
        if Instant::now()
            .checked_add(delay)
            .is_none_or(|end| end >= deadline)
        {
            return Err(ApiError::Timeout);
        }
        transport.wait(delay);
    }
    Err(ApiError::Timeout)
}
fn rate_limited(body: &[u8]) -> bool {
    serde_json::from_slice::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("error")?.get("errors")?.as_array().cloned())
        .is_some_and(|errors| {
            errors.iter().any(|e| {
                matches!(
                    e.get("reason").and_then(serde_json::Value::as_str),
                    Some(
                        "rateLimitExceeded"
                            | "userRateLimitExceeded"
                            | "dailyLimitExceeded"
                            | "quotaExceeded"
                    )
                )
            })
        })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn endpoints_reject_arbitrary_bearer_targets_and_unsafe_paths() {
        for raw in [
            "http://sheets.googleapis.com/v4/spreadsheets/a",
            "https://evil.invalid/v4/spreadsheets/a",
            "https://www.googleapis.com/drive/v3/files/a/permissions",
            "https://sheets.googleapis.com:444/v4/spreadsheets/a",
            "https://x@sheets.googleapis.com/v4/spreadsheets/a",
            "https://script.googleapis.com/v1/projects/a:run",
            "https://www.googleapis.com/drive/v3/files/",
            "https://www.googleapis.com/drive/v3/drives",
        ] {
            assert_eq!(
                validate_api_url(&Url::parse(raw).unwrap()),
                Err(ApiError::InvalidInput)
            );
        }
    }
    #[test]
    fn exact_drive_discovery_endpoint_is_authorized_without_arbitrary_paths() {
        assert_eq!(
            validate_api_url(
                &Url::parse("https://www.googleapis.com/drive/v3/files?q=fixture").unwrap()
            ),
            Ok(())
        );
    }
    #[test]
    fn cancellation_stops_before_the_next_network_request() {
        struct NoNetwork;
        impl ReadTransport for NoNetwork {
            fn get(&self, _: &Url, _: &HeaderValue, _: Duration) -> Result<HttpReply, ApiError> {
                panic!("cancelled GET must not run")
            }
        }
        let token = crate::monitoring::work::Cancellation::default();
        token.cancel();
        let (result, _) = crate::monitoring::work::in_operation(token, None, || {
            get_bounded(
                &NoNetwork,
                &Url::parse("https://sheets.googleapis.com/v4/spreadsheets/fixture").unwrap(),
                &HeaderValue::from_static("Bearer synthetic"),
                Instant::now() + Duration::from_secs(60),
            )
        });
        assert_eq!(result, Err(ApiError::Cancelled));
    }
    #[test]
    fn long_retry_after_defers_without_sleep_and_survives_operation_scope() {
        use std::cell::Cell;
        struct Limited(Cell<usize>);
        impl ReadTransport for Limited {
            fn get(&self, _: &Url, _: &HeaderValue, _: Duration) -> Result<HttpReply, ApiError> {
                self.0.set(self.0.get() + 1);
                Ok(HttpReply {
                    status: 429,
                    body: b"SENSITIVE".to_vec(),
                    retry_after: Some(Duration::from_secs(3600)),
                })
            }
            fn wait(&self, _: Duration) {
                panic!("long Retry-After must defer, not block")
            }
        }
        let transport = Limited(Cell::new(0));
        let (result, hint) = crate::monitoring::work::in_operation(
            crate::monitoring::work::Cancellation::default(),
            None,
            || {
                get_bounded(
                    &transport,
                    &Url::parse("https://sheets.googleapis.com/v4/spreadsheets/fixture").unwrap(),
                    &HeaderValue::from_static("Bearer synthetic"),
                    Instant::now() + Duration::from_secs(60),
                )
            },
        );
        assert_eq!(result, Err(ApiError::RateLimited));
        assert_eq!(hint, Some(3_600_000));
        assert_eq!(transport.0.get(), 1);
        assert_eq!(
            crate::monitoring::work::in_operation(
                crate::monitoring::work::Cancellation::default(),
                None,
                || ()
            )
            .1,
            None
        );
    }
    #[test]
    fn retry_after_is_preserved_and_errors_have_no_provider_body() {
        assert_eq!(
            parse_retry_after("999999"),
            Some(Duration::from_secs(999999))
        );
        assert_eq!(parse_retry_after("invalid"), None);
        assert!(!ApiError::Forbidden.to_string().contains("SECRET"));
        assert_eq!(
            serde_json::to_value(ApiError::Forbidden).unwrap()["kind"],
            "forbidden"
        );
    }
}
