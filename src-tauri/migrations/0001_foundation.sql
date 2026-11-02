-- Sprint 01: local schema only. No OAuth credentials, API access or execution.
CREATE TABLE accounts (
    id TEXT PRIMARY KEY NOT NULL,
    provider_subject TEXT UNIQUE,
    display_name TEXT NOT NULL,
    is_demo INTEGER NOT NULL DEFAULT 0 CHECK (is_demo IN (0, 1)),
    created_at TEXT NOT NULL
);

CREATE TABLE spreadsheets (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT REFERENCES accounts(id) ON DELETE RESTRICT,
    external_id TEXT,
    name TEXT NOT NULL CHECK (length(trim(name)) BETWEEN 1 AND 120),
    url TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'indeterminate'
        CHECK (status IN ('online', 'offline', 'checking', 'indeterminate')),
    scope TEXT NOT NULL DEFAULT 'all' CHECK (scope IN ('all', 'selected')),
    paused INTEGER NOT NULL DEFAULT 0 CHECK (paused IN (0, 1)),
    is_demo INTEGER NOT NULL DEFAULT 0 CHECK (is_demo IN (0, 1)),
    last_checked_at TEXT,
    archived_at TEXT,
    created_at TEXT NOT NULL,
    UNIQUE (account_id, external_id)
);
CREATE TRIGGER spreadsheets_capacity_insert BEFORE INSERT ON spreadsheets
WHEN NEW.archived_at IS NULL AND (SELECT count(*) FROM spreadsheets WHERE archived_at IS NULL) >= 30
BEGIN SELECT RAISE(ABORT, 'spreadsheet_limit_30'); END;
CREATE TRIGGER spreadsheets_capacity_restore BEFORE UPDATE OF archived_at ON spreadsheets
WHEN OLD.archived_at IS NOT NULL AND NEW.archived_at IS NULL
    AND (SELECT count(*) FROM spreadsheets WHERE archived_at IS NULL) >= 30
BEGIN SELECT RAISE(ABORT, 'spreadsheet_limit_30'); END;

CREATE TABLE monitored_sheets (
    id TEXT PRIMARY KEY NOT NULL,
    spreadsheet_id TEXT NOT NULL REFERENCES spreadsheets(id) ON DELETE CASCADE,
    external_sheet_id INTEGER NOT NULL CHECK (external_sheet_id >= 0),
    title TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0 CHECK (position >= 0),
    is_demo INTEGER NOT NULL DEFAULT 0 CHECK (is_demo IN (0, 1)),
    UNIQUE (spreadsheet_id, external_sheet_id)
);

CREATE TABLE baselines (
    id TEXT PRIMARY KEY NOT NULL,
    spreadsheet_id TEXT NOT NULL REFERENCES spreadsheets(id) ON DELETE CASCADE,
    version INTEGER NOT NULL CHECK (version >= 1),
    snapshot_json TEXT NOT NULL CHECK (json_valid(snapshot_json)),
    fingerprint TEXT NOT NULL,
    approved_at TEXT,
    created_at TEXT NOT NULL,
    is_demo INTEGER NOT NULL DEFAULT 0 CHECK (is_demo IN (0, 1)),
    UNIQUE (spreadsheet_id, version),
    UNIQUE (id, spreadsheet_id)
);

CREATE TABLE integrity_rules (
    id TEXT PRIMARY KEY NOT NULL,
    spreadsheet_id TEXT NOT NULL REFERENCES spreadsheets(id) ON DELETE CASCADE,
    baseline_id TEXT,
    kind TEXT NOT NULL,
    definition_json TEXT NOT NULL CHECK (json_valid(definition_json)),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    is_demo INTEGER NOT NULL DEFAULT 0 CHECK (is_demo IN (0, 1)),
    FOREIGN KEY (baseline_id, spreadsheet_id) REFERENCES baselines(id, spreadsheet_id) ON DELETE RESTRICT
);

CREATE TABLE script_projects (
    id TEXT PRIMARY KEY NOT NULL,
    spreadsheet_id TEXT NOT NULL REFERENCES spreadsheets(id) ON DELETE CASCADE,
    external_script_id TEXT,
    name TEXT NOT NULL,
    authorized_at TEXT,
    is_demo INTEGER NOT NULL DEFAULT 0 CHECK (is_demo IN (0, 1)),
    UNIQUE (spreadsheet_id, external_script_id)
);

CREATE TABLE script_dependencies (
    id TEXT PRIMARY KEY NOT NULL,
    script_project_id TEXT NOT NULL REFERENCES script_projects(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    reference_json TEXT NOT NULL CHECK (json_valid(reference_json)),
    is_dynamic INTEGER NOT NULL DEFAULT 0 CHECK (is_dynamic IN (0, 1)),
    is_demo INTEGER NOT NULL DEFAULT 0 CHECK (is_demo IN (0, 1))
);

CREATE TABLE monitoring_checks (
    id TEXT PRIMARY KEY NOT NULL,
    spreadsheet_id TEXT NOT NULL REFERENCES spreadsheets(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK (status IN ('online', 'offline', 'checking', 'indeterminate')),
    started_at TEXT NOT NULL,
    completed_at TEXT,
    diagnosis TEXT NOT NULL,
    is_demo INTEGER NOT NULL DEFAULT 0 CHECK (is_demo IN (0, 1)),
    UNIQUE (id, spreadsheet_id)
);

CREATE TABLE change_events (
    id TEXT PRIMARY KEY NOT NULL,
    spreadsheet_id TEXT NOT NULL REFERENCES spreadsheets(id) ON DELETE CASCADE,
    monitoring_check_id TEXT,
    category TEXT NOT NULL,
    diagnosis TEXT NOT NULL,
    details_json TEXT NOT NULL CHECK (json_valid(details_json)),
    occurred_at TEXT NOT NULL,
    is_demo INTEGER NOT NULL DEFAULT 0 CHECK (is_demo IN (0, 1)),
    FOREIGN KEY (monitoring_check_id, spreadsheet_id) REFERENCES monitoring_checks(id, spreadsheet_id) ON DELETE RESTRICT
);
CREATE INDEX change_events_history ON change_events(occurred_at DESC, id);

CREATE TABLE notifications (
    id TEXT PRIMARY KEY NOT NULL,
    change_event_id TEXT NOT NULL REFERENCES change_events(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'sent', 'failed', 'demo')),
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    created_at TEXT NOT NULL,
    delivered_at TEXT,
    is_demo INTEGER NOT NULL DEFAULT 0 CHECK (is_demo IN (0, 1)),
    CHECK (status != 'demo' OR is_demo = 1)
);

CREATE TABLE app_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    interval_seconds INTEGER NOT NULL DEFAULT 60 CHECK (interval_seconds BETWEEN 60 AND 300),
    history_days INTEGER NOT NULL DEFAULT 90 CHECK (history_days = 90),
    start_with_windows INTEGER NOT NULL DEFAULT 0 CHECK (start_with_windows = 0),
    demo_seeded INTEGER NOT NULL DEFAULT 0 CHECK (demo_seeded IN (0, 1)),
    updated_at TEXT NOT NULL
);
INSERT INTO app_settings (id, updated_at) VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
