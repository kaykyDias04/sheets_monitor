-- SM-022..025. No historical alert backfill and no OAuth data.
CREATE TABLE monitoring_controls (
 account_id TEXT PRIMARY KEY REFERENCES accounts(id) ON DELETE RESTRICT,
 enabled INTEGER NOT NULL DEFAULT 0 CHECK(enabled IN(0,1)),
 generation INTEGER NOT NULL DEFAULT 0 CHECK(generation>=0)
);
CREATE TABLE monitoring_schedules (
 account_id TEXT NOT NULL,
 spreadsheet_id TEXT NOT NULL,
 registry_revision INTEGER NOT NULL,
 scope_hash TEXT NOT NULL,
 generation INTEGER NOT NULL DEFAULT 0,
 last_started_ms INTEGER,
 last_completed_ms INTEGER,
 next_run_ms INTEGER NOT NULL,
 failure_count INTEGER NOT NULL DEFAULT 0 CHECK(failure_count>=0),
 operational_code TEXT,
 lease_id TEXT,
 lease_owner TEXT,
 lease_until_ms INTEGER,
 lease_manual INTEGER NOT NULL DEFAULT 0 CHECK(lease_manual IN(0,1)),
 PRIMARY KEY(account_id,spreadsheet_id),
 FOREIGN KEY(account_id,spreadsheet_id) REFERENCES registered_spreadsheets(account_id,spreadsheet_id) ON DELETE CASCADE
);
CREATE TABLE monitoring_operational_events (
 id TEXT PRIMARY KEY,
 account_id TEXT NOT NULL,
 spreadsheet_id TEXT NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN('unavailable','recovered')),
 code TEXT NOT NULL,
 detected_ms INTEGER NOT NULL,
 FOREIGN KEY(account_id,spreadsheet_id) REFERENCES registered_spreadsheets(account_id,spreadsheet_id) ON DELETE CASCADE
);
CREATE INDEX operational_events_account_time ON monitoring_operational_events(account_id,detected_ms DESC,id);
CREATE TABLE notification_outbox (
 id TEXT PRIMARY KEY,
 account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
 spreadsheet_id TEXT NOT NULL,
 integrity_event_id TEXT UNIQUE REFERENCES integrity_events(event_id) ON DELETE CASCADE,
 operational_event_id TEXT UNIQUE REFERENCES monitoring_operational_events(id) ON DELETE CASCADE,
 source TEXT NOT NULL CHECK(source IN('integrity','operational')),
 spreadsheet_name TEXT NOT NULL,
 sheet_id INTEGER,
 severity TEXT NOT NULL CHECK(severity IN('info','warning','critical')),
 diagnosis TEXT NOT NULL,
 detected_at TEXT NOT NULL,
 created_ms INTEGER NOT NULL,
 expires_ms INTEGER NOT NULL,
 state TEXT NOT NULL CHECK(state IN('pending','dispatching','retry','permission_blocked','unsupported','dispatch_submitted','suppressed_by_preference','expired')),
 next_attempt_ms INTEGER NOT NULL,
 attempt_count INTEGER NOT NULL DEFAULT 0,
 attempt_id TEXT,
 lease_owner TEXT,
 lease_until_ms INTEGER,
 last_attempt_ms INTEGER,
 submitted_ms INTEGER,
 group_id TEXT NOT NULL,
 CHECK((integrity_event_id IS NOT NULL)+(operational_event_id IS NOT NULL)=1)
);
CREATE INDEX notification_outbox_due ON notification_outbox(account_id,state,next_attempt_ms);
CREATE TABLE notification_attempts (
 id TEXT PRIMARY KEY,
 notification_id TEXT NOT NULL REFERENCES notification_outbox(id) ON DELETE CASCADE,
 owner TEXT NOT NULL,
 started_ms INTEGER NOT NULL,
 completed_ms INTEGER,
 outcome TEXT NOT NULL CHECK(outcome IN('dispatching','ambiguous','dispatch_submitted','retry','permission_blocked','unsupported','suppressed_by_preference','expired')),
 code TEXT
);
CREATE TRIGGER integrity_event_notification AFTER INSERT ON integrity_events
BEGIN
 INSERT INTO notification_outbox(id,account_id,spreadsheet_id,integrity_event_id,source,spreadsheet_name,sheet_id,severity,diagnosis,detected_at,created_ms,expires_ms,state,next_attempt_ms,group_id)
 SELECT 'integrity:'||NEW.event_id,NEW.account_id,s.spreadsheet_id,NEW.event_id,'integrity',COALESCE(r.name,s.spreadsheet_id),NEW.sheet_id,NEW.severity,COALESCE(json_extract(NEW.event_json,'$.diagnosis'),'Alteração de integridade detectada.'),NEW.detected_at,
 CAST((julianday(NEW.detected_at)-2440587.5)*86400000 AS INTEGER),
 CAST((julianday(NEW.detected_at)-2440587.5)*86400000 AS INTEGER)+CAST(COALESCE(json_extract(s.scope_json,'$.history_retention_days'),90) AS INTEGER)*86400000,
 CASE WHEN COALESCE((SELECT json_extract(preferences_json,'$.notifications_enabled') FROM management_preferences WHERE account_id=NEW.account_id),0)=1 THEN 'pending' ELSE 'suppressed_by_preference' END,
 CAST((julianday(NEW.detected_at)-2440587.5)*86400000 AS INTEGER),'integrity:'||NEW.event_id
 FROM integrity_scopes s LEFT JOIN registered_spreadsheets r ON r.account_id=s.account_id AND r.spreadsheet_id=s.spreadsheet_id WHERE s.id=NEW.scope_id;
END;
