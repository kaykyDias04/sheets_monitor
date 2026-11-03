//! Windows notification submission. Plugin success proves preparation, not OS delivery.
use super::service::NotificationSink;
use crate::monitoring::{NotificationOutcome, NotificationRecord};
use std::sync::Arc;
use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

pub struct NativeNotifier {
    app: AppHandle,
}
impl NativeNotifier {
    /// Callback receives only an opaque outbox ID; the caller must recheck account.
    pub fn new(app: AppHandle, open: Arc<dyn Fn(String) + Send + Sync>) -> Self {
        let _ = app.notification().on_action(move |action| {
            if action.action_id() != "tap" {
                return;
            }
            if let Some(id) = action
                .notification()
                .and_then(|notification| notification.extra().get("outbox_id"))
                .and_then(serde_json::Value::as_str)
            {
                if id.len() <= 160 {
                    open(id.to_owned());
                }
            }
        });
        Self { app }
    }
}
fn clipped(value: &str, max: usize) -> String {
    value
        .chars()
        .filter(|c| !c.is_control())
        .take(max)
        .collect()
}
fn payload(record: &NotificationRecord) -> (String, String) {
    let title = match (record.source.as_str(), record.severity.as_str()) {
        ("operational", "info") => "Sheets Monitor — Monitoramento restabelecido",
        ("operational", _) => "Sheets Monitor — Monitoramento indisponível",
        (_, "critical") => "Sheets Monitor — Falha de integridade",
        (_, "warning") => "Sheets Monitor — Atenção",
        _ => "Sheets Monitor — Alteração detectada",
    }
    .into();
    let tab = record
        .sheet_title
        .as_deref()
        .map(|name| format!("\nAba: {}", clipped(name, 80)))
        .unwrap_or_default();
    (
        title,
        format!(
            "Planilha: {}{}\n{}\nHorário: {}",
            clipped(&record.spreadsheet_name, 100),
            tab,
            clipped(&record.diagnosis, 240),
            clipped(&record.detected_at, 40)
        ),
    )
}
impl NotificationSink for NativeNotifier {
    fn submit(&self, record: &NotificationRecord) -> NotificationOutcome {
        if !cfg!(windows) {
            return NotificationOutcome::Unsupported;
        }
        let (title, body) = payload(record);
        match self
            .app
            .notification()
            .builder()
            .title(title)
            .body(body)
            .extra("outbox_id", serde_json::Value::String(record.id.clone()))
            .show()
        {
            Ok(()) => NotificationOutcome::DispatchSubmitted,
            Err(_) => NotificationOutcome::Retryable {
                code: "native_prepare_failed".into(),
                next_attempt_ms: chrono::Utc::now().timestamp_millis().saturating_add(
                    crate::monitoring::retry_delay_ms(
                        record.attempt_count,
                        rand::random::<u64>() % 1001,
                        None,
                    ),
                ),
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_payload_is_bounded_and_contains_tab_when_known() {
        let record = NotificationRecord {
            id: "opaque".into(),
            event_id: "fixture".into(),
            source: "integrity".into(),
            spreadsheet_id: "private_id_not_for_toast".into(),
            spreadsheet_name: "Synthetic\nTest".into(),
            sheet_id: Some(7),
            sheet_title: Some("Dados".into()),
            severity: "critical".into(),
            diagnosis: "x".repeat(1000),
            detected_at: "2026-11-02T12:00:00Z".into(),
            state: "pending".into(),
            attempt_count: 1,
            last_attempt_ms: None,
            submitted_ms: None,
            group_id: "opaque".into(),
        };
        let (title, body) = payload(&record);
        assert!(title.contains("Falha de integridade"));
        assert!(body.contains("Aba: Dados"));
        assert!(body.contains("Planilha: SyntheticTest"));
        assert!(!body.contains(&record.spreadsheet_id));
        assert!(body.chars().count() < 500);
    }
}
