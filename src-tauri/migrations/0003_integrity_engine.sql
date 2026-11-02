-- Sprint 03: normalized, account-scoped integrity state. No OAuth credentials or script source.
CREATE TABLE integrity_scopes (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    spreadsheet_id TEXT NOT NULL,
    scope_hash TEXT NOT NULL CHECK(length(scope_hash)=64),
    scope_json TEXT NOT NULL CHECK(json_valid(scope_json)),
    previous_snapshot_id TEXT,
    active_baseline_id TEXT,
    latest_report_id TEXT,
    created_at TEXT NOT NULL,
    UNIQUE(account_id,spreadsheet_id,scope_hash),
    UNIQUE(id,account_id),
    FOREIGN KEY(previous_snapshot_id,id,account_id) REFERENCES structural_snapshots(id,scope_id,account_id) ON DELETE RESTRICT,
    FOREIGN KEY(active_baseline_id,id,account_id) REFERENCES baseline_versions(id,scope_id,account_id) ON DELETE RESTRICT,
    FOREIGN KEY(latest_report_id,id,account_id) REFERENCES integrity_reports(check_id,scope_id,account_id) ON DELETE RESTRICT
);
CREATE TRIGGER integrity_capacity_insert BEFORE INSERT ON integrity_scopes
WHEN NOT EXISTS(SELECT 1 FROM integrity_scopes WHERE account_id=NEW.account_id AND spreadsheet_id=NEW.spreadsheet_id)
 AND (SELECT count(*) FROM (SELECT account_id,spreadsheet_id FROM integrity_scopes GROUP BY account_id,spreadsheet_id))>=30
BEGIN SELECT RAISE(ABORT,'integrity_limit_30'); END;
CREATE TRIGGER integrity_scope_immutable BEFORE UPDATE OF account_id,spreadsheet_id,scope_hash,scope_json ON integrity_scopes
BEGIN SELECT RAISE(ABORT,'integrity_scope_immutable'); END;

CREATE TABLE structural_snapshots (
    id TEXT PRIMARY KEY NOT NULL,
    scope_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    captured_at TEXT NOT NULL,
    fingerprint TEXT NOT NULL CHECK(length(fingerprint)=64),
    snapshot_json TEXT NOT NULL CHECK(json_valid(snapshot_json)),
    complete INTEGER NOT NULL CHECK(complete IN(0,1)),
    FOREIGN KEY(scope_id,account_id) REFERENCES integrity_scopes(id,account_id) ON DELETE RESTRICT,
    UNIQUE(id,scope_id,account_id)
);
CREATE INDEX structural_snapshots_scope_time ON structural_snapshots(scope_id,captured_at DESC,id);
CREATE TRIGGER structural_snapshots_immutable BEFORE UPDATE ON structural_snapshots
BEGIN SELECT RAISE(ABORT,'structural_snapshot_immutable'); END;

CREATE TABLE baseline_versions (
    id TEXT PRIMARY KEY NOT NULL,
    scope_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    snapshot_id TEXT NOT NULL,
    version INTEGER NOT NULL CHECK(version>=1),
    approved_at TEXT NOT NULL,
    approval_origin TEXT NOT NULL CHECK(approval_origin IN('human:native','synthetic:test')),
    FOREIGN KEY(scope_id,account_id) REFERENCES integrity_scopes(id,account_id) ON DELETE RESTRICT,
    FOREIGN KEY(snapshot_id,scope_id,account_id) REFERENCES structural_snapshots(id,scope_id,account_id) ON DELETE RESTRICT,
    UNIQUE(scope_id,version),
    UNIQUE(id,scope_id,account_id)
);
CREATE TRIGGER baseline_versions_immutable_update BEFORE UPDATE ON baseline_versions
BEGIN SELECT RAISE(ABORT,'approved_baseline_immutable'); END;
CREATE TRIGGER baseline_versions_immutable_delete BEFORE DELETE ON baseline_versions
BEGIN SELECT RAISE(ABORT,'approved_baseline_immutable'); END;

CREATE TABLE integrity_reports (
    check_id TEXT PRIMARY KEY NOT NULL,
    scope_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    snapshot_id TEXT NOT NULL,
    baseline_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN('online','offline','indeterminate','checking')),
    connectivity TEXT NOT NULL,
    detected_at TEXT NOT NULL,
    report_json TEXT NOT NULL CHECK(json_valid(report_json)),
    FOREIGN KEY(scope_id,account_id) REFERENCES integrity_scopes(id,account_id) ON DELETE RESTRICT,
    FOREIGN KEY(baseline_id,scope_id,account_id) REFERENCES baseline_versions(id,scope_id,account_id) ON DELETE RESTRICT,
    UNIQUE(check_id,scope_id,account_id)
);
CREATE INDEX integrity_reports_scope_state_time ON integrity_reports(scope_id,status,detected_at DESC);

CREATE TABLE integrity_events (
    event_id TEXT PRIMARY KEY NOT NULL,
    check_id TEXT NOT NULL,
    scope_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    sheet_id INTEGER,
    severity TEXT NOT NULL CHECK(severity IN('info','warning','critical')),
    detected_at TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    event_json TEXT NOT NULL CHECK(json_valid(event_json)),
    FOREIGN KEY(check_id,scope_id,account_id) REFERENCES integrity_reports(check_id,scope_id,account_id) ON DELETE RESTRICT,
    UNIQUE(check_id,fingerprint)
);
CREATE INDEX integrity_events_scope_sheet_time ON integrity_events(scope_id,sheet_id,detected_at DESC);
CREATE INDEX integrity_events_scope_severity_time ON integrity_events(scope_id,severity,detected_at DESC);

CREATE TABLE integrity_incidents (
    scope_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    finding_key TEXT NOT NULL,
    outcome TEXT NOT NULL CHECK(outcome IN('violation','inconclusive')),
    fingerprint TEXT NOT NULL,
    finding_json TEXT NOT NULL CHECK(json_valid(finding_json)),
    active INTEGER NOT NULL CHECK(active IN(0,1)),
    occurrence INTEGER NOT NULL DEFAULT 1 CHECK(occurrence>=1),
    last_seen_at TEXT NOT NULL,
    resolved_at TEXT,
    PRIMARY KEY(scope_id,finding_key,outcome),
    FOREIGN KEY(scope_id,account_id) REFERENCES integrity_scopes(id,account_id) ON DELETE RESTRICT
);
CREATE INDEX integrity_incidents_scope_active ON integrity_incidents(scope_id,active);
-- Last fully observed components are separate from last fully successful capture.
-- This journal survives retention and avoids losing content events on partial reads.
CREATE TABLE integrity_observation_components (
 scope_id TEXT NOT NULL REFERENCES integrity_scopes(id) ON DELETE RESTRICT,
 component_key TEXT NOT NULL,
 component_json TEXT NOT NULL CHECK(json_valid(component_json)),
 observed_at TEXT NOT NULL,
 PRIMARY KEY(scope_id,component_key)
);
ALTER TABLE integrity_scopes ADD COLUMN observation_revision INTEGER NOT NULL DEFAULT 0;
