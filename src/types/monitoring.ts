export interface MonitoringSchedule {
  spreadsheet_id: string;
  registry_revision: number;
  scope_hash: string;
  generation: number;
  last_started_ms: number | null;
  last_completed_ms: number | null;
  next_run_ms: number;
  failure_count: number;
  operational_code: string | null;
  lease_owner: string | null;
  lease_until_ms: number | null;
}
export interface MonitoringStatus {
  control: { enabled: boolean; generation: number };
  schedules: MonitoringSchedule[];
  eligible_count: number;
  interval_seconds: number;
  service: { running: boolean; active: number; queued: number; warning: string | null };
}
export interface NotificationRecord {
  id: string;
  event_id: string;
  source: string;
  spreadsheet_id: string;
  spreadsheet_name: string;
  sheet_id: number | null;
  sheet_title: string | null;
  severity: string;
  diagnosis: string;
  detected_at: string;
  state: string;
  attempt_count: number;
  last_attempt_ms: number | null;
  submitted_ms: number | null;
  group_id: string;
  event_kind?: string | null;
  event_type_label?: string;
  formatted_detected_at?: string;
}
export type InstallMode = "per_user_nsis" | "per_machine_nsis" | "portable_or_development";
export interface UpdaterStatus {
  state:
    | "idle"
    | "checking"
    | "up_to_date"
    | "downloading"
    | "ready_to_install"
    | "manual_required"
    | "installing"
    | "error"
    | string;
  current_version: string;
  available_version: string | null;
  release_url: string | null;
  release_notes: string | null;
  published_at: string | null;
  install_mode: InstallMode;
  can_auto_update: boolean;
  dismissed: boolean;
  last_checked_at: string | null;
  last_error_message: string | null;
}
export interface NativeReadiness {
  identity_mode: string;
  setting: string;
  error_code: string | null;
}
export interface DesktopStatus {
  tray_available: boolean;
  notification_capability: string;
  startup: {
    registered: boolean;
    managed: boolean;
    os_execution: string;
    error_code: string | null;
  };
  exiting: boolean;
}
