export interface GoogleAccount {
  subject: string;
  email: string | null;
  name: string | null;
}
export interface IntegrationState {
  generation: number;
  configured: boolean;
  client_id: string | null;
  auth: {
    status: "unconfigured" | "disconnected" | "connected" | "reconnect_required";
    account: GoogleAccount | null;
    granted_scopes: string[];
    expires_at: number | null;
  };
  drive_metadata_opt_in: boolean;
  real_validation: "pending";
}
export interface OAuthConfiguration {
  client_id: string;
  client_secret?: string;
  drive_metadata_opt_in: boolean;
}
export interface IntegrationFailure {
  code?: string;
  kind?: string;
  message?: string;
  retryable?: boolean;
}
export interface ServiceResult<T> {
  status: "success" | "error" | "not_authorized" | "not_requested";
  data: T | null;
  error: IntegrationFailure | null;
}
export interface SpreadsheetMetadata {
  spreadsheet_id: string;
  spreadsheet_url?: string | null;
  title: string;
  locale?: string | null;
  time_zone?: string | null;
  sheets: {
    sheet_id: number;
    title: string;
    index: number;
    sheet_type?: string | null;
    row_count?: number | null;
    column_count?: number | null;
    hidden: boolean;
  }[];
}
export interface ValueRange {
  range: string;
  major_dimension: string;
  values: unknown[][];
}
export interface SheetValues {
  spreadsheet_id: string;
  values: ValueRange[];
  formulas: ValueRange[];
}
export interface DriveMetadata {
  id: string;
  name: string;
  mime_type: string;
  modified_time?: string | null;
  trashed: boolean;
  web_view_link?: string | null;
  version?: string | null;
}
export interface ScriptProject {
  metadata: {
    script_id: string;
    title: string;
    parent_id?: string | null;
    create_time?: string | null;
    update_time?: string | null;
  };
  files: { name: string; file_type: "SERVER_JS" | "HTML" | "JSON"; source: string }[];
  source_hash: string;
}
export interface SheetAccessResult {
  generation: number;
  account_subject: string;
  spreadsheet_id: string;
  sheets: ServiceResult<SpreadsheetMetadata>;
  values: ServiceResult<SheetValues>;
  drive: ServiceResult<DriveMetadata>;
}
export interface ScriptAccessResult {
  generation: number;
  account_subject: string;
  script: ServiceResult<ScriptProject>;
}
