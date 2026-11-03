//! Direct Windows submission. Native API success never proves display or reading.
use super::service::NotificationSink;
use crate::monitoring::{NotificationOutcome, NotificationRecord};
use std::sync::Arc;
use tauri::AppHandle;
#[cfg(windows)]
#[path = "windows.rs"]
mod windows;

#[derive(Clone, Debug, serde::Serialize)]
pub struct NativeReadiness {
    pub identity_mode: String,
    pub setting: String,
    pub error_code: Option<String>,
}

pub struct NativeNotifier {
    #[cfg(windows)]
    windows: windows::WindowsNotifier,
}
impl NativeNotifier {
    /// Callback receives only an opaque outbox ID; the caller must recheck account.
    pub fn new(app: AppHandle, open: Arc<dyn Fn(String) + Send + Sync>) -> Self {
        #[cfg(windows)]
        {
            Self {
                windows: windows::WindowsNotifier::new(app.config().identifier.clone(), open),
            }
        }
        #[cfg(not(windows))]
        {
            let _ = (app, open);
            Self {}
        }
    }
    pub fn readiness(&self) -> NativeReadiness {
        #[cfg(windows)]
        {
            self.windows.readiness()
        }
        #[cfg(not(windows))]
        {
            NativeReadiness {
                identity_mode: "unsupported".into(),
                setting: "unknown".into(),
                error_code: Some("unsupported".into()),
            }
        }
    }
}
/// Reads identity/settings without showing a toast or modifying Windows configuration.
pub fn readiness(app: &AppHandle) -> NativeReadiness {
    NativeNotifier::new(app.clone(), Arc::new(|_| {})).readiness()
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
        self.submit_with_failure(record, Arc::new(|_| {}))
    }
    fn submit_with_failure(
        &self,
        record: &NotificationRecord,
        failure: Arc<dyn Fn(String) + Send + Sync>,
    ) -> NotificationOutcome {
        let (title, body) = payload(record);
        #[cfg(windows)]
        {
            self.windows.submit(record, &title, &body, failure)
        }
        #[cfg(not(windows))]
        {
            let _ = (title, body, failure);
            NotificationOutcome::Unsupported
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
