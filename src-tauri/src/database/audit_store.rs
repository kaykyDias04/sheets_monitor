use super::Database;
use crate::integration::CommandError;
use rusqlite::{params, OptionalExtension};
impl Database {
    pub(crate) fn audit_window(
        &self,
        subject: &str,
        event: &str,
    ) -> Result<(String, String, String), CommandError> {
        if event.is_empty() || event.len() > 256 || event.chars().any(char::is_control) {
            return Err(CommandError {
                code: "invalid_input".into(),
                message: "Evento inválido para auditoria.".into(),
                retryable: false,
            });
        }
        self.connection.query_row("SELECT s.spreadsheet_id, COALESCE((SELECT old.captured_at FROM structural_snapshots old WHERE old.scope_id=s.id AND old.account_id=a.id AND julianday(old.captured_at)<julianday(current.capture_started) ORDER BY julianday(old.captured_at) DESC LIMIT 1),base.captured_at), ss.captured_at FROM integrity_events e JOIN accounts a ON a.id=e.account_id JOIN integrity_scopes s ON s.id=e.scope_id JOIN integrity_reports r ON r.check_id=e.check_id AND r.account_id=a.id JOIN structural_snapshots ss ON ss.id=r.snapshot_id AND ss.account_id=a.id JOIN baseline_versions b ON b.id=r.baseline_id JOIN structural_snapshots base ON base.id=b.snapshot_id JOIN (SELECT id,json_extract(snapshot_json,'$.capture_started_at') AS capture_started FROM structural_snapshots) current ON current.id=ss.id WHERE a.provider_subject=?1 AND e.event_id=?2", params![subject,event], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(|_| CommandError::local())?.ok_or_else(|| CommandError { code: "not_found".into(), message: "Evento não encontrado nesta conta.".into(), retryable: false })
    }
}
