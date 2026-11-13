//! Workspace audit is file-level evidence, never proof of a particular cell/tab edit.
use crate::{
    auth::{AuthCode, AuthError, AUDIT_SCOPE},
    google::{ApiError, GoogleApi, ReadTransport},
    integration::{CommandError, IntegrationRuntime},
    integrity::runtime::EngineEnvelope,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use std::time::Instant;

#[derive(Debug, Serialize)]
pub struct AuditRecord {
    pub actor: Option<String>,
    pub occurred_at: String,
    pub action: String,
}
#[derive(Debug, Serialize)]
pub struct AuditReport {
    pub responsible: String,
    pub note: String,
    pub records: Vec<AuditRecord>,
    pub truncated: bool,
}
impl<T: ReadTransport> GoogleApi<T> {
    pub fn workspace_audit(&self, id: &str, from: &str, to: &str) -> Result<AuditReport, ApiError> {
        crate::google::validate_google_id(id)?;
        let from = DateTime::parse_from_rfc3339(from).map_err(|_| ApiError::InvalidInput)?;
        let to = DateTime::parse_from_rfc3339(to).map_err(|_| ApiError::InvalidInput)?;
        let now = Utc::now();
        if from >= to || to > now {
            return Err(ApiError::InvalidInput);
        }
        let mut result = AuditReport { responsible: "Responsável não identificado".into(), note: "O Google Workspace informa edições do arquivo, não a autoria da diferença estrutural específica. Proximidade temporal não comprova responsabilidade. Registros podem chegar com atraso ou omitir identidade.".into(), records: vec![], truncated: false };
        let retention_start = (now - chrono::Duration::days(180)).fixed_offset();
        let from = if from < retention_start {
            result.note.push_str(
                " Cobertura parcial: a auditoria Drive só disponibiliza os últimos 180 dias.",
            );
            if to <= retention_start {
                result.note.push_str(" A janela solicitada está inteiramente fora da retenção; nenhuma consulta foi enviada.");
                return Ok(result);
            }
            retention_start
        } else {
            from
        };
        let mut token: Option<String> = None;
        for _ in 0..5 {
            let mut url = url::Url::parse("https://admin.googleapis.com/admin/reports/v1/activity/users/all/applications/drive").map_err(|_| ApiError::InvalidInput)?;
            url.query_pairs_mut().append_pair("eventName", "edit").append_pair("filters", &format!("doc_id=={id}")).append_pair("startTime", &from.to_rfc3339()).append_pair("endTime", &to.to_rfc3339()).append_pair("maxResults", "100").append_pair("fields", "items(id(time),actor(email),events(name,parameters(name,value))),nextPageToken");
            if let Some(t) = &token {
                url.query_pairs_mut().append_pair("pageToken", t);
            }
            let page: Value =
                self.read(url, Instant::now() + std::time::Duration::from_secs(30))?;
            result.records.extend(parse_records(&page, id, from, to));
            token = page
                .get("nextPageToken")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_owned);
            if token.is_none() {
                return Ok(result);
            }
        }
        result.truncated = true;
        Ok(result)
    }
}
fn parse_records(
    page: &Value,
    id: &str,
    from: DateTime<chrono::FixedOffset>,
    to: DateTime<chrono::FixedOffset>,
) -> Vec<AuditRecord> {
    let mut records = vec![];
    for item in page
        .get("items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let Some(raw_time) = item.pointer("/id/time").and_then(Value::as_str) else {
            continue;
        };
        let Ok(time) = DateTime::parse_from_rfc3339(raw_time) else {
            continue;
        };
        if time <= from || time > to {
            continue;
        }
        let matches = item
            .get("events")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .any(|event| {
                event.get("name").and_then(Value::as_str) == Some("edit")
                    && event
                        .get("parameters")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .any(|p| {
                            p.get("name").and_then(Value::as_str) == Some("doc_id")
                                && p.get("value").and_then(Value::as_str) == Some(id)
                        })
            });
        if !matches {
            continue;
        }
        let actor = item
            .pointer("/actor/email")
            .and_then(Value::as_str)
            .filter(|s| s.len() <= 512 && !s.chars().any(char::is_control))
            .map(str::to_owned);
        records.push(AuditRecord {
            actor,
            occurred_at: time.to_rfc3339(),
            action: "Edição do arquivo (sem vínculo comprovado com esta regra)".into(),
        });
    }
    records
}
impl IntegrationRuntime {
    pub fn get_event_audit(
        &self,
        event_id: String,
    ) -> Result<EngineEnvelope<AuditReport>, CommandError> {
        let context =
            self.management_local(None, |db, subject| db.audit_window(subject, &event_id))?;
        let mut session = self.auth.authorized()?;
        if session.generation != context.generation {
            return Err(AuthError::new(AuthCode::OperationSuperseded).into());
        }
        if !self.auth.features()?.1 || !session.has_scope(AUDIT_SCOPE) {
            return Err(CommandError { code: "audit_not_authorized".into(), message: "Auditoria opcional não autorizada. Ative-a em Configurações e conceda consentimento com uma conta Workspace com privilégio de auditoria. A identidade não será presumida.".into(), retryable: false });
        }
        let (id, from, to) = context.data;
        let mut report =
            GoogleApi::new(session.access_token().into())?.workspace_audit(&id, &from, &to);
        if matches!(report, Err(ApiError::Unauthorized)) {
            session = self.auth.force_refresh(&session)?;
            report =
                GoogleApi::new(session.access_token().into())?.workspace_audit(&id, &from, &to);
        }
        let report = report?;
        self.auth.with_current(&session, || {
            Ok(EngineEnvelope {
                generation: session.generation,
                account_subject: session.account.subject.clone(),
                data: report,
            })
        })?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn audit_does_not_attribute_other_files_outside_window_or_missing_identity() {
        let from = DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z").unwrap();
        let to = from + chrono::Duration::minutes(10);
        let row = |id: &str, time: &str, actor: Value| serde_json::json!({"id":{"time":time},"actor":actor,"events":[{"name":"edit","parameters":[{"name":"doc_id","value":id}]}]});
        let page = serde_json::json!({"items":[row("fixture", "2026-01-01T00:05:00Z", serde_json::json!({"email":"editor@example.test"})),row("wrong", "2026-01-01T00:05:00Z", Value::Null), row("fixture", "2026-01-01T00:00:00Z", Value::Null), row("fixture", "2026-01-01T00:06:00Z", Value::Null)]});
        let records = parse_records(&page, "fixture", from, to);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].actor.as_deref(), Some("editor@example.test"));
        assert!(records[1].actor.is_none());
        assert!(records[0].action.contains("sem vínculo comprovado"));
    }

    #[test]
    fn audit_clamps_retention_and_does_not_query_expired_windows() {
        use crate::google::HttpReply;
        use reqwest::header::HeaderValue;
        use std::{cell::Cell, rc::Rc, time::Duration};
        use url::Url;

        struct RetainedWindow {
            calls: Rc<Cell<usize>>,
            cutoff: DateTime<Utc>,
        }
        impl ReadTransport for RetainedWindow {
            fn get(&self, url: &Url, _: &HeaderValue, _: Duration) -> Result<HttpReply, ApiError> {
                self.calls.set(self.calls.get() + 1);
                let start = url
                    .query_pairs()
                    .find(|(key, _)| key == "startTime")
                    .unwrap()
                    .1;
                let start = DateTime::parse_from_rfc3339(&start).unwrap();
                assert!(start >= self.cutoff);
                assert!(start < self.cutoff + chrono::Duration::minutes(1));
                Ok(HttpReply {
                    status: 200,
                    body: br#"{"items":[]}"#.to_vec(),
                    retry_after: None,
                })
            }
        }
        let now = Utc::now();
        let calls = Rc::new(Cell::new(0));
        let api = GoogleApi::with_transport(
            "synthetic-access".into(),
            RetainedWindow {
                calls: Rc::clone(&calls),
                cutoff: now - chrono::Duration::days(180),
            },
        )
        .unwrap();
        let from = (now - chrono::Duration::days(200)).to_rfc3339();
        let expired_to = (now - chrono::Duration::days(190)).to_rfc3339();
        let expired = api.workspace_audit("fixture", &from, &expired_to).unwrap();
        assert!(expired.note.contains("inteiramente fora da retenção"));
        assert_eq!(expired.responsible, "Responsável não identificado");
        let current_to = (now - chrono::Duration::hours(1)).to_rfc3339();
        let partial = api.workspace_audit("fixture", &from, &current_to).unwrap();
        assert!(partial.note.contains("Cobertura parcial"));
        assert!(partial.records.is_empty());
        assert!(!partial.truncated);
        // The expired query must not consume a request.
        assert_eq!(calls.get(), 1);
    }
}
