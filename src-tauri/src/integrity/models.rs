use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
pub const SNAPSHOT_VERSION: u32 = 1;
pub const MAX_CELLS: u64 = 30_000_000;
/// Workbook tab count is independent from the 30 registered-file capacity.
pub const MAX_MONITORED_TABS: usize = 1024;
pub type EngineResult<T> = Result<T, EngineError>;
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EngineError {
    InvalidScope,
    InvalidSnapshot,
    VersionUnsupported,
    MissingBaseline,
    BaselineConflict,
    ApprovalRequired,
    NotFound,
    Forbidden,
    Storage,
    Limit,
    Busy,
    InvalidState,
}
impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "integrity_{self:?}")
    }
}
impl std::error::Error for EngineError {}
pub fn fingerprint<T: Serialize>(value: &T) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(value).expect("serializable integrity contracts"))
    )
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GridRect {
    pub start_row: u32,
    pub end_row: u32,
    pub start_col: u32,
    pub end_col: u32,
}
impl GridRect {
    pub fn valid(&self) -> bool {
        self.start_row < self.end_row
            && self.start_col < self.end_col
            && self.end_row <= 1_000_000
            && self.end_col <= 18_278
    }
    pub fn cells(&self) -> u64 {
        u64::from(self.end_row.saturating_sub(self.start_row))
            * u64::from(self.end_col.saturating_sub(self.start_col))
    }
    pub fn contains(&self, row: u32, col: u32) -> bool {
        row >= self.start_row && row < self.end_row && col >= self.start_col && col < self.end_col
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MonitorScope {
    pub schema_version: u32,
    pub sheets: Vec<SheetRule>,
    pub scripts: Vec<ScriptRule>,
    pub named_range_ids: Vec<String>,
    pub snapshot_retention_days: u16,
    pub history_retention_days: u16,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SheetRule {
    pub sheet_id: u32,
    pub protect_title: bool,
    pub protect_position: bool,
    pub regions: Vec<RegionRule>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RegionRule {
    pub id: String,
    pub range: GridRect,
    pub header_row: u32,
    pub columns: Vec<ColumnRule>,
    pub key_column: Option<String>,
    pub critical: bool,
    pub allow_record_deletion: bool,
    pub formulas: Vec<FormulaRule>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ColumnRule {
    pub id: String,
    pub header: String,
    pub position: u32,
    pub protect_position: bool,
    pub required: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FormulaRule {
    pub id: String,
    pub column_id: String,
    pub baseline_row: u32,
    pub mode: FormulaMode,
    pub allow_row_shift: bool,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FormulaMode {
    Exact,
    Structural,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ScriptRule {
    pub script_id: String,
    pub critical: bool,
}
fn safe_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
}
impl MonitorScope {
    pub fn validate(&self) -> EngineResult<()> {
        if self.schema_version != SNAPSHOT_VERSION {
            return Err(EngineError::VersionUnsupported);
        }
        if self.sheets.is_empty()
            || self.sheets.len() > MAX_MONITORED_TABS
            || self.scripts.len() > 4
            || self.named_range_ids.len() > 50
            || !(1..=365).contains(&self.snapshot_retention_days)
            || !(1..=3650).contains(&self.history_retention_days)
        {
            return Err(EngineError::InvalidScope);
        }
        let mut sheets = BTreeSet::new();
        let mut cells = 0;
        for s in &self.sheets {
            if !sheets.insert(s.sheet_id) || s.regions.is_empty() || s.regions.len() > 8 {
                return Err(EngineError::InvalidScope);
            }
            let mut ids = BTreeSet::new();
            for r in &s.regions {
                if !safe_id(&r.id)
                    || !ids.insert(&r.id)
                    || !r.range.valid()
                    || r.header_row < r.range.start_row
                    || r.header_row >= r.range.end_row
                    || r.columns.is_empty()
                    || r.columns.len() > 128
                    || r.formulas.len() > 128
                {
                    return Err(EngineError::InvalidScope);
                }
                cells += r.range.cells();
                let mut columns = BTreeSet::new();
                let mut positions = BTreeSet::new();
                let mut headers = BTreeSet::new();
                for c in &r.columns {
                    if !safe_id(&c.id)
                        || !columns.insert(&c.id)
                        || !positions.insert(c.position)
                        || !headers.insert(&c.header)
                        || c.header.is_empty()
                        || c.header.len() > 512
                        || c.header.chars().any(char::is_control)
                        || c.position < r.range.start_col
                        || c.position >= r.range.end_col
                    {
                        return Err(EngineError::InvalidScope);
                    }
                }
                if r.key_column
                    .as_ref()
                    .is_some_and(|key| !columns.contains(key))
                {
                    return Err(EngineError::InvalidScope);
                }
                let mut formulas = BTreeSet::new();
                for f in &r.formulas {
                    if !safe_id(&f.id)
                        || !formulas.insert(&f.id)
                        || !columns.contains(&f.column_id)
                        || f.baseline_row <= r.header_row
                        || f.baseline_row >= r.range.end_row
                    {
                        return Err(EngineError::InvalidScope);
                    }
                }
            }
        }
        if cells > MAX_CELLS {
            return Err(EngineError::Limit);
        }
        let mut ids = BTreeSet::new();
        for s in &self.scripts {
            if crate::google::validate_google_id(&s.script_id).is_err() || !ids.insert(&s.script_id)
            {
                return Err(EngineError::InvalidScope);
            }
        }
        let mut names = BTreeSet::new();
        for n in &self.named_range_ids {
            if n.is_empty() || n.len() > 256 || !names.insert(n) {
                return Err(EngineError::InvalidScope);
            }
        }
        Ok(())
    }
    pub fn hash(&self) -> String {
        fingerprint(self)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoverageIssue {
    pub sheet_id: Option<u32>,
    pub rule_id: Option<String>,
    pub critical: bool,
    pub code: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Coverage {
    pub metadata_complete: bool,
    pub complete: bool,
    pub issues: Vec<CoverageIssue>,
    pub atomic_across_services: bool,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Connectivity {
    Reachable,
    AuthenticationFailed,
    NetworkFailed,
    Partial,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StructuralSnapshot {
    pub schema_version: u32,
    pub id: String,
    pub account_subject: String,
    pub spreadsheet_id: String,
    pub capture_started_at: String,
    pub captured_at: String,
    pub scope_hash: String,
    pub scope: MonitorScope,
    pub sheets: Vec<SheetSnapshot>,
    pub named_ranges: Vec<NamedRangeSnapshot>,
    pub scripts: Vec<ScriptSnapshot>,
    pub coverage: Coverage,
    pub connectivity: Connectivity,
}
impl StructuralSnapshot {
    pub fn validate(&self) -> EngineResult<()> {
        self.scope.validate()?;
        if self.schema_version != SNAPSHOT_VERSION {
            return Err(EngineError::VersionUnsupported);
        }
        if self.scope.hash() != self.scope_hash
            || self.id.len() > 128
            || self.id.is_empty()
            || crate::google::validate_google_id(&self.spreadsheet_id).is_err()
            || crate::google::validate_google_id(&self.account_subject).is_err()
        {
            return Err(EngineError::InvalidSnapshot);
        }
        if serde_json::to_vec(self)
            .map_err(|_| EngineError::InvalidSnapshot)?
            .len()
            > 256 * 1024 * 1024
        {
            return Err(EngineError::Limit);
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SheetSnapshot {
    pub sheet_id: u32,
    pub title: String,
    pub index: u32,
    pub row_count: u32,
    pub column_count: u32,
    pub regions: Vec<RegionSnapshot>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionSnapshot {
    pub region_id: String,
    pub range: GridRect,
    pub complete: bool,
    pub headers: Vec<ObservedColumn>,
    pub rows: Vec<RowDigest>,
    pub formulas: Vec<ObservedFormula>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObservedColumn {
    pub column: u32,
    pub header: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RowDigest {
    pub row: u32,
    pub key_hash: Option<String>,
    pub cell_hashes: BTreeMap<String, String>,
    pub fingerprint: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObservedFormula {
    pub row: u32,
    pub column: u32,
    pub formula: Option<String>,
    pub error: Option<String>,
    pub value_hash: String,
    pub key_hash: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NamedRangeSnapshot {
    pub id: String,
    pub name: String,
    pub sheet_id: u32,
    pub range: GridRect,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptSnapshot {
    pub script_id: String,
    pub source_hash: String,
    pub files: Vec<ScriptFileDigest>,
    pub analysis: ScriptAnalysis,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptFileDigest {
    pub name: String,
    pub file_type: String,
    pub hash: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptAnalysis {
    pub parser: String,
    pub complete: bool,
    pub dependencies: Vec<ScriptDependency>,
    pub limitations: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptDependency {
    pub file: String,
    pub line: u32,
    pub kind: DependencyKind,
    pub sheet_name: Option<String>,
    pub range: Option<GridRect>,
    pub coverage: DependencyCoverage,
    pub detail_code: String,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DependencyCoverage {
    Static,
    Partial,
    Dynamic,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    SheetByName,
    RangeA1,
    RangeNumeric,
    Column,
    LastRow,
    LastColumn,
    ReadValues,
    ReadDisplayValues,
    ReadFormulas,
    Dynamic,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaselineVersion {
    pub id: String,
    pub version: u32,
    pub active: bool,
    pub approved_at: String,
    pub approval_origin: String,
    pub snapshot: StructuralSnapshot,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityStatus {
    Online,
    Offline,
    Indeterminate,
    Checking,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FindingOutcome {
    Violation,
    Inconclusive,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Finding {
    pub key: String,
    pub sheet_id: Option<u32>,
    pub rule_id: String,
    pub code: String,
    pub severity: Severity,
    pub outcome: FindingOutcome,
    pub expected: Option<String>,
    pub found: Option<String>,
    pub location: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContentDifference {
    pub key: String,
    pub sheet_id: Option<u32>,
    pub kind: String,
    pub category: String,
    pub location: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub ambiguous: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SheetAssessment {
    pub sheet_id: u32,
    pub status: IntegrityStatus,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrityReport {
    pub observation_revision: u64,
    pub previous_snapshot_id: Option<String>,
    pub check_id: String,
    pub snapshot_id: String,
    pub baseline_id: String,
    pub account_subject: String,
    pub spreadsheet_id: String,
    pub detected_at: String,
    pub status: IntegrityStatus,
    pub connectivity: Connectivity,
    pub sheets: Vec<SheetAssessment>,
    pub findings: Vec<Finding>,
    pub differences: Vec<ContentDifference>,
    pub coverage: Coverage,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrityEvent {
    pub location: Option<String>,
    pub event_id: String,
    pub check_id: String,
    pub spreadsheet_id: String,
    pub sheet_id: Option<u32>,
    pub kind: String,
    pub category: String,
    pub severity: Severity,
    pub rule_id: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub detected_at: String,
    pub fingerprint: String,
    pub diagnosis: String,
}

#[derive(Debug, Clone)]
pub struct ContentObservation {
    pub revision: u64,
    pub snapshot: Option<StructuralSnapshot>,
}
