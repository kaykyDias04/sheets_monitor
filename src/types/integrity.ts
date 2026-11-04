export interface GridRect {
  start_row: number;
  end_row: number;
  start_col: number;
  end_col: number;
}
export interface MonitorScope {
  schema_version: 1;
  sheets: {
    sheet_id: number;
    protect_title: boolean;
    protect_position: boolean;
    regions: {
      id: string;
      range: GridRect;
      header_row: number;
      columns: {
        id: string;
        header: string;
        position: number;
        protect_position: boolean;
        required: boolean;
      }[];
      key_column: string | null;
      critical: boolean;
      allow_record_deletion: boolean;
      formulas: {
        id: string;
        column_id: string;
        baseline_row: number;
        mode: "exact" | "structural";
        allow_row_shift: boolean;
      }[];
    }[];
  }[];
  scripts: { script_id: string; critical: boolean }[];
  named_range_ids: string[];
  snapshot_retention_days: number;
  history_retention_days: number;
}
export interface Coverage {
  metadata_complete: boolean;
  complete: boolean;
  atomic_across_services: boolean;
  issues: { sheet_id: number | null; rule_id: string | null; critical: boolean; code: string }[];
}
export interface StructuralSnapshot {
  schema_version: number;
  id: string;
  account_subject: string;
  spreadsheet_id: string;
  capture_started_at: string;
  captured_at: string;
  scope_hash: string;
  scope: MonitorScope;
  sheets: {
    sheet_id: number;
    title: string;
    index: number;
    row_count: number;
    column_count: number;
    regions: {
      region_id: string;
      complete: boolean;
      headers?: { column: number; header: string }[];
      rows: unknown[];
      formulas: unknown[];
    }[];
  }[];
  coverage: Coverage;
  scripts: {
    script_id: string;
    source_hash: string;
    files: { name: string; file_type: string; hash: string }[];
    analysis: { parser: string; complete: boolean; dependencies: unknown[]; limitations: string[] };
  }[];
  connectivity: string;
}
export interface EngineEnvelope<T> {
  generation: number;
  account_subject: string;
  data: T;
}
export interface BaselineSummary {
  id: string;
  version: number;
  active: boolean;
  approved_at: string;
  approval_origin: string;
  snapshot_id?: string;
  scope_hash?: string;
  snapshot?: StructuralSnapshot;
}
export interface CapturePreview {
  session_generation: number;
  snapshot: StructuralSnapshot;
  snapshot_fingerprint: string;
  active_baseline: BaselineSummary | null;
}
export interface BaselineVersion extends BaselineSummary {
  snapshot: StructuralSnapshot;
}
export type IntegrityStatus = "online" | "offline" | "indeterminate" | "checking";
export interface Finding {
  key: string;
  sheet_id: number | null;
  rule_id: string;
  code: string;
  severity: "info" | "warning" | "critical";
  outcome: "violation" | "inconclusive";
  expected: string | null;
  found: string | null;
  location: string | null;
}
export interface IntegrityReport {
  check_id: string;
  snapshot_id: string;
  baseline_id: string;
  account_subject: string;
  spreadsheet_id: string;
  detected_at: string;
  status: IntegrityStatus;
  connectivity: string;
  sheets: { sheet_id: number; status: IntegrityStatus }[];
  findings: Finding[];
  differences: {
    key: string;
    kind: string;
    category: string;
    location: string | null;
    before: string | null;
    after: string | null;
    ambiguous: boolean;
  }[];
  coverage: Coverage;
}
export interface IntegrityEvent {
  event_id: string;
  check_id: string;
  spreadsheet_id: string;
  sheet_id: number | null;
  kind: string;
  category: string;
  severity: string;
  rule_id: string | null;
  location: string | null;
  before: string | null;
  after: string | null;
  detected_at: string;
  fingerprint: string;
  diagnosis: string;
}
export interface ValidationResult {
  report: IntegrityReport;
  events: IntegrityEvent[];
}
export interface ScopeQuery {
  spreadsheet_id: string;
  scope_hash: string;
}
export interface ScopeProposal {
  scope: MonitorScope;
  available_sheets: { sheet_id: number; title: string; index: number }[];
}
