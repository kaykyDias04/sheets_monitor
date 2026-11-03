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
