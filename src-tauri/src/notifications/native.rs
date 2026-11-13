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
    let title = if record.event_type_label.trim().is_empty() {
        crate::management::events::notification_title(
            &record.source,
            record.event_kind.as_deref(),
            &record.severity,
            &record.diagnosis,
        )
    } else {
        format!("Sheets Monitor — {}", clipped(&record.event_type_label, 80))
    };
    let description =
        crate::management::events::diagnosis_pt_br(&record.diagnosis, record.event_kind.as_deref());
    let formatted_time = if record.formatted_detected_at.trim().is_empty() {
        crate::management::events::format_local_datetime(&record.detected_at)
    } else {
        record.formatted_detected_at.clone()
    };
    let spreadsheet = clipped(&record.spreadsheet_name, 90);
    let header_line = match record
        .sheet_title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
    {
        Some(tab) => format!("Planilha: {spreadsheet} · Aba: {}", clipped(tab, 60)),
        None => format!("Planilha: {spreadsheet}"),
    };
    (
        title,
        format!(
            "{header_line}\n{}\nHorário: {}",
            clipped(&description, 220),
            clipped(&formatted_time, 32)
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
    fn native_payload_is_bounded_pt_br_and_differentiates_problems_from_resolutions() {
        let problem = NotificationRecord {
            id: "opaque".into(),
            event_id: "fixture".into(),
            source: "integrity".into(),
            spreadsheet_id: "private_id_not_for_toast".into(),
            spreadsheet_name: "Synthetic\nTest".into(),
            sheet_id: Some(7),
            sheet_title: Some("Dados".into()),
            severity: "critical".into(),
            event_kind: Some("violation".into()),
            event_type_label: String::new(),
            diagnosis: "protected_header_changed_or_column_missing".into(),
            diagnostic_code: "protected_header_changed_or_column_missing".into(),
            detected_at: "2026-11-02T12:00:00Z".into(),
            formatted_detected_at: String::new(),
            state: "pending".into(),
            attempt_count: 1,
            last_attempt_ms: None,
            submitted_ms: None,
            group_id: "opaque".into(),
        };
        let (title, body) = payload(&problem);
        assert_eq!(title, "Sheets Monitor — Alteração estrutural detectada");
        assert!(body.contains("Planilha: SyntheticTest · Aba: Dados"));
        assert!(body.contains("Cabeçalho protegido alterado ou coluna obrigatória ausente."));
        assert!(!body.contains("protected_header_changed_or_column_missing"));
        assert!(!body.contains(&problem.spreadsheet_id));
        assert!(body.contains("Horário: 02/11/2026 "));
        assert!(body.chars().count() < 500);

        let mut recovered = problem.clone();
        recovered.severity = "info".into();
        recovered.event_kind = Some("recovered".into());
        recovered.diagnosis = "critical_rule_recovered".into();
        let (rec_title, rec_body) = payload(&recovered);
        assert_eq!(rec_title, "Sheets Monitor — Estrutura normalizada");
        assert!(
            rec_body.contains("A estrutura protegida voltou a corresponder à referência aprovada.")
        );
        assert!(!rec_body.contains("critical_rule_recovered"));

        let mut offline = problem.clone();
        offline.source = "operational".into();
        offline.sheet_title = None;
        offline.severity = "warning".into();
        offline.event_kind = Some("unavailable".into());
        offline.diagnosis = "network_failed".into();
        let (off_title, off_body) = payload(&offline);
        assert_eq!(off_title, "Sheets Monitor — Monitoramento indisponível");
        assert!(!off_body.contains("Aba:"));
        assert!(off_body.contains("Planilha: SyntheticTest"));
        assert!(off_body.contains("Horário: 02/11/2026 "));
        assert!(off_body.contains("Causa:"));
        assert!(off_body.contains("Impacto:"));
        assert!(off_body.contains("Solução:"));
        assert!(!off_body.contains("network_failed"));
        assert!(off_body.chars().count() < 500);

        let mut online = offline;
        online.severity = "info".into();
        online.event_kind = Some("recovered".into());
        online.diagnosis = "operational_recovered".into();
        let (on_title, on_body) = payload(&online);
        assert_eq!(on_title, "Sheets Monitor — Monitoramento restabelecido");
        assert!(on_body.contains("Monitoramento restabelecido"));
    }
}
