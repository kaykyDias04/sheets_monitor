//! Durable scheduler and notification contracts. Times are UTC milliseconds.
use crate::integrity::MonitorScope;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MonitoringControl {
    pub enabled: bool,
    pub generation: u64,
}
#[derive(Debug, Clone, Serialize)]
pub struct MonitoringCandidate {
    pub spreadsheet_id: String,
    pub name: String,
    pub scope: MonitorScope,
    pub scope_hash: String,
    pub registry_revision: u64,
    pub baseline_id: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct MonitoringSchedule {
    pub spreadsheet_id: String,
    pub registry_revision: u64,
    pub scope_hash: String,
    pub generation: u64,
    pub last_started_ms: Option<i64>,
    pub last_completed_ms: Option<i64>,
    pub next_run_ms: i64,
    pub failure_count: u32,
    pub operational_code: Option<String>,
    pub lease_owner: Option<String>,
    pub lease_until_ms: Option<i64>,
}
#[derive(Debug, Clone)]
pub struct MonitoringLease {
    pub spreadsheet_id: String,
    pub registry_revision: u64,
    pub generation: u64,
    pub owner: String,
    pub lease_id: String,
    pub started_ms: i64,
    pub manual: bool,
}
#[derive(Debug, Clone)]
pub enum CheckOutcome {
    Success,
    OperationalFailure { code: String },
    Cancelled,
}
#[derive(Debug, Clone, Serialize)]
pub struct OperationalEvent {
    pub id: String,
    pub spreadsheet_id: String,
    pub kind: String,
    pub code: String,
    pub detected_ms: i64,
}
#[derive(Debug, Clone, Serialize)]
pub struct NotificationRecord {
    pub id: String,
    pub event_id: String,
    pub source: String,
    pub spreadsheet_id: String,
    pub spreadsheet_name: String,
    pub sheet_id: Option<u32>,
    pub sheet_title: Option<String>,
    pub severity: String,
    #[serde(default)]
    pub event_kind: Option<String>,
    #[serde(default)]
    pub event_type_label: String,
    pub diagnosis: String,
    pub diagnostic_code: String,
    pub detected_at: String,
    #[serde(default)]
    pub formatted_detected_at: String,
    pub state: String,
    pub attempt_count: u32,
    pub last_attempt_ms: Option<i64>,
    pub submitted_ms: Option<i64>,
    pub group_id: String,
}
#[derive(Debug, Clone)]
pub struct NotificationAttempt {
    pub record: NotificationRecord,
    pub attempt_id: String,
    pub owner: String,
}
#[derive(Debug, Clone)]
pub enum NotificationOutcome {
    DispatchSubmitted,
    Retryable { code: String, next_attempt_ms: i64 },
    PermissionBlocked,
    Unsupported,
}
