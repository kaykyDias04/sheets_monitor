export type IntegrityStatus = "online" | "offline" | "checking" | "indeterminate";
export interface AppSettings {
  interval_seconds: number;
  history_days: number;
  start_with_windows: boolean;
}
export interface DemoSpreadsheet {
  id: string;
  name: string;
  url: string;
  status: IntegrityStatus;
  selected_sheets: string[];
  last_checked_at: string | null;
  is_demo: true;
}
export interface DemoActivity {
  id: string;
  spreadsheet_id: string;
  spreadsheet_name: string;
  occurred_at: string;
  category: string;
  diagnosis: string;
  is_demo: true;
}
export interface FoundationDashboard {
  spreadsheets: DemoSpreadsheet[];
  activities: DemoActivity[];
  settings: AppSettings;
  is_demo: true;
  database_version: number;
}

export type Page = "dashboard" | "sheets" | "history" | "settings" | "google";
