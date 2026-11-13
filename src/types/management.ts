import type {
  MonitorScope,
  CapturePreview,
  IntegrityReport,
  IntegrityEvent,
  BaselineSummary,
  StructuralSnapshot,
} from "./integrity";
import type { SpreadsheetMetadata } from "./google";
type SheetMetadata = SpreadsheetMetadata["sheets"][number];
export interface Mutation {
  spreadsheet_id: string;
  expected_generation: number;
  expected_revision: number;
}
export interface RegisteredSpreadsheet {
  spreadsheet_id: string;
  name: string;
  url: string;
  account_email: string | null;
  paused: boolean;
  removed: boolean;
  status:
    "configuration_pending" | "not_checked" | "online" | "offline" | "indeterminate" | "paused";
  authorization: "unknown" | "available" | "error";
  access_error: string | null;
  last_checked_at: string | null;
  baseline_version: number | null;
  scope: MonitorScope | null;
  scope_hash: string | null;
  revision: number;
  tabs: SheetMetadata[];
  selected_sheet_ids: number[];
  tabs_refreshed_at: string | null;
}
export interface RegisterInput {
  url: string;
  expected_generation: number;
}
export interface RemoveInput extends Mutation {
  confirmed: boolean;
  history: "preserve" | "delete_activity";
}
export interface TabsInput extends Mutation {
  sheet_ids: number[];
}
export interface RulesInput extends Mutation {
  scope: MonitorScope;
}
export interface ProposeRulesInput extends Mutation {
  header_row?: number;
}
export interface ApprovalInput extends Mutation {
  snapshot_id: string;
  snapshot_fingerprint: string;
  expected_version: number;
  confirmed: boolean;
}
export interface OperationalPreview {
  revision: number;
  preview: CapturePreview;
}
export interface Page<T> {
  items: T[];
  total: number;
  offset: number;
  limit: number;
}
export interface EventFilter {
  spreadsheet_id?: string | null;
  sheet_id?: number | null;
  from?: string | null;
  to?: string | null;
  kind?: string | null;
  severity?: string | null;
  status?: string | null;
  category?: string | null;
  event_type?: "failure" | "modification" | "resolution" | null;
  issue_status?: "active" | "resolved" | null;
  script_only?: boolean;
  query?: string;
  offset?: number;
  limit?: number;
}
export type ExportFormat = "csv" | "json" | "xlsx";
export interface ExportEventsInput {
  filter: EventFilter;
  format: ExportFormat;
}
export interface ExportEventsResult {
  exported_count: number;
  file_path: string | null;
  format: string;
}
export interface Activity extends IntegrityEvent {
  spreadsheet_name: string;
  sheet_title: string | null;
  status: string;
  removed: boolean;
  event_type?: "failure" | "modification" | "resolution" | string;
  tone?: "failure" | "resolved" | "neutral" | string;
  issue_status?: "active" | "resolved" | string | null;
  description?: string;
  formatted_detected_at?: string;
}
export interface ScriptSummary {
  script_id: string;
  name: string | null;
  authorization: string;
  last_queried_at: string | null;
  critical: boolean;
  analysis: StructuralSnapshot["scripts"][number] | null;
}
export interface Diagnostics {
  spreadsheet: RegisteredSpreadsheet;
  report: IntegrityReport | null;
  baseline: BaselineSummary | null;
  scripts: ScriptSummary[];
}
export interface DashboardSummary {
  registered: number;
  online: number;
  offline: number;
  indeterminate: number;
  paused: number;
  pending: number;
  not_checked: number;
  spreadsheets: RegisteredSpreadsheet[];
  recent_events: Activity[];
  recent_failures: Activity[];
}
export interface Preferences {
  interval_seconds: number;
  notifications_enabled: boolean;
  history_days: number;
  start_with_windows: boolean;
  run_minimized: boolean;
  theme: "system" | "light" | "dark";
}
export interface BaselineDifference {
  path: string;
  before: string | null;
  after: string | null;
}
export interface BaselineComparison {
  before: BaselineSummary;
  after: BaselineSummary;
  scope_changed: boolean;
  differences: BaselineDifference[];
  truncated: boolean;
}
