use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const MAX_MONITORS: usize = 30;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Monitor {
    pub id: String,
    pub name: String,
    pub spreadsheet_id: String,
    pub script_ids: Vec<String>,
    pub enabled: bool,
    pub archived: bool,
    pub baseline_snapshot_id: Option<i64>,
    pub latest_snapshot_id: Option<i64>,
    pub last_checked_at: Option<String>,
    pub last_error: Option<String>,
    pub created_at: String,
}

/// Exactly the read-only API responses. Collection times are stored separately.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Snapshot {
    pub sheets: Value,
    pub drive: Value,
    #[serde(default)]
    pub permissions: Vec<Value>,
    #[serde(default)]
    pub scripts: BTreeMap<String, Value>,
    #[serde(default)]
    pub findings: BTreeMap<String, Vec<crate::analysis::Finding>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DifferenceKind { Added, Removed, Changed }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Difference {
    /// RFC 6901 JSON pointer, including escaped / and ~ characters.
    pub path: String,
    pub kind: DifferenceKind,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunTicket {
    pub monitor_id: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResult {
    pub snapshot_id: i64,
    pub awaiting_baseline: bool,
    pub differences: Vec<Difference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineCandidate {
    pub snapshot_id: i64,
    pub fingerprint: String,
    pub snapshot: Snapshot,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub id: i64,
    pub monitor_id: String,
    pub snapshot_id: i64,
    pub difference: Difference,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i64,
    pub event_id: i64,
    pub monitor_id: String,
    pub monitor_name: String,
    pub difference: Difference,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub next_attempt_at: i64,
    pub delivered_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub monitor_id: String,
    pub action: String,
    pub created_at: String,
}

pub fn valid_google_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 200
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
