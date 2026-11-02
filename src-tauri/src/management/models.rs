use crate::{
    google::SheetMetadata,
    integrity::runtime::{BaselineSummary, CapturePreview},
    integrity::{IntegrityEvent, IntegrityReport, MonitorScope, ScriptSnapshot},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mutation {
    pub spreadsheet_id: String,
    pub expected_generation: u64,
    pub expected_revision: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterInput {
    pub url: String,
    pub expected_generation: u64,
}
macro_rules! mutation_input {
 ($name:ident {$($field:ident : $kind:ty),* $(,)?}) => {
  #[derive(Deserialize)]
  #[serde(deny_unknown_fields)]
  pub struct $name {pub spreadsheet_id:String,pub expected_generation:u64,pub expected_revision:u64,$(pub $field:$kind),*}
  impl $name {pub fn mutation(&self)->Mutation{Mutation{spreadsheet_id:self.spreadsheet_id.clone(),expected_generation:self.expected_generation,expected_revision:self.expected_revision}}}
 };
}
mutation_input!(RemoveInput {
    confirmed: bool,
    history: String
});
mutation_input!(TabsInput {sheet_ids:Vec<u32>});
mutation_input!(RulesInput {
    scope: MonitorScope
});
mutation_input!(ApprovalInput {
    snapshot_id: String,
    snapshot_fingerprint: String,
    expected_version: u32,
    confirmed: bool
});
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpreadsheetQuery {
    pub spreadsheet_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselinesQuery {
    pub spreadsheet_id: String,
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "page_size")]
    pub limit: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompareBaselinesInput {
    pub spreadsheet_id: String,
    pub before_id: String,
    pub after_id: String,
}
#[derive(Debug, Serialize, Clone)]
pub struct RegisteredSpreadsheet {
    pub spreadsheet_id: String,
    pub name: String,
    pub url: String,
    pub account_email: Option<String>,
    pub paused: bool,
    pub removed: bool,
    pub status: String,
    pub authorization: String,
    pub access_error: Option<String>,
    pub last_checked_at: Option<String>,
    pub baseline_version: Option<u32>,
    pub scope: Option<MonitorScope>,
    pub scope_hash: Option<String>,
    pub revision: u64,
    pub tabs: Vec<SheetMetadata>,
    pub selected_sheet_ids: Vec<u32>,
    pub tabs_refreshed_at: Option<String>,
}
#[derive(Serialize)]
pub struct OperationalPreview {
    pub revision: u64,
    pub preview: CapturePreview,
}
#[derive(Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub offset: u32,
    pub limit: u32,
}
fn page_size() -> u32 {
    25
}
#[derive(Default, Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct EventFilter {
    pub spreadsheet_id: Option<String>,
    pub sheet_id: Option<u32>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub kind: Option<String>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub category: Option<String>,
    #[serde(default)]
    pub script_only: bool,
    #[serde(default)]
    pub query: String,
    #[serde(default)]
    pub offset: u32,
    #[serde(default = "page_size")]
    pub limit: u32,
}
#[derive(Serialize)]
pub struct Activity {
    #[serde(flatten)]
    pub event: IntegrityEvent,
    pub spreadsheet_name: String,
    pub sheet_title: Option<String>,
    pub status: String,
    pub removed: bool,
}
#[derive(Serialize)]
pub struct ScriptSummary {
    pub script_id: String,
    pub name: Option<String>,
    pub authorization: String,
    pub last_queried_at: Option<String>,
    pub critical: bool,
    pub analysis: Option<ScriptSnapshot>,
}
#[derive(Serialize)]
pub struct Diagnostics {
    pub spreadsheet: RegisteredSpreadsheet,
    pub report: Option<IntegrityReport>,
    pub baseline: Option<BaselineSummary>,
    pub scripts: Vec<ScriptSummary>,
}
#[derive(Serialize)]
pub struct DashboardSummary {
    pub registered: u32,
    pub online: u32,
    pub offline: u32,
    pub indeterminate: u32,
    pub paused: u32,
    pub pending: u32,
    pub not_checked: u32,
    pub spreadsheets: Vec<RegisteredSpreadsheet>,
    pub recent_events: Vec<Activity>,
    pub recent_failures: Vec<Activity>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    pub interval_seconds: u32,
    pub notifications_enabled: bool,
    pub history_days: u16,
    pub start_with_windows: bool,
    pub run_minimized: bool,
    pub theme: String,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            interval_seconds: 60,
            notifications_enabled: false,
            history_days: 90,
            start_with_windows: false,
            run_minimized: false,
            theme: "system".into(),
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferencesInput {
    pub expected_generation: u64,
    pub preferences: Preferences,
}
#[derive(Serialize)]
pub struct BaselineDifference {
    pub path: String,
    pub before: Option<String>,
    pub after: Option<String>,
}
#[derive(Serialize)]
pub struct BaselineComparison {
    pub before: BaselineSummary,
    pub after: BaselineSummary,
    pub scope_changed: bool,
    pub differences: Vec<BaselineDifference>,
    pub truncated: bool,
}
