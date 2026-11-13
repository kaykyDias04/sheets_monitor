-- Configuration sync only: never credentials, observations, events or local startup flags.
ALTER TABLE app_settings RENAME TO app_settings_before_cloud_sync;
CREATE TABLE app_settings (
 id INTEGER PRIMARY KEY CHECK(id=1),
 interval_seconds INTEGER NOT NULL DEFAULT 60 CHECK(interval_seconds BETWEEN 60 AND 600),
 history_days INTEGER NOT NULL DEFAULT 90 CHECK(history_days=90),
 start_with_windows INTEGER NOT NULL DEFAULT 0 CHECK(start_with_windows=0),
 demo_seeded INTEGER NOT NULL DEFAULT 0 CHECK(demo_seeded IN(0,1)),
 updated_at TEXT NOT NULL
);
INSERT INTO app_settings SELECT id,interval_seconds,history_days,start_with_windows,demo_seeded,updated_at FROM app_settings_before_cloud_sync;
DROP TABLE app_settings_before_cloud_sync;
CREATE TABLE cloud_sync_device(id INTEGER PRIMARY KEY CHECK(id=1), device_id TEXT NOT NULL);
CREATE TABLE cloud_sync_state (
 account_subject TEXT PRIMARY KEY NOT NULL,
 base_commit TEXT,
 base_payload_hash TEXT,
 last_synced_at TEXT
);
CREATE TABLE cloud_sync_commits (
 account_subject TEXT NOT NULL,
 commit_id TEXT NOT NULL,
 commit_json TEXT NOT NULL CHECK(json_valid(commit_json)),
 PRIMARY KEY(account_subject,commit_id)
);
CREATE TABLE cloud_sync_backups (
 id TEXT PRIMARY KEY NOT NULL,
 account_subject TEXT NOT NULL,
 created_at TEXT NOT NULL,
 payload_hash TEXT NOT NULL,
 payload_json TEXT NOT NULL CHECK(json_valid(payload_json))
);
CREATE TABLE cloud_sync_status (
 account_subject TEXT PRIMARY KEY NOT NULL,
 status_json TEXT NOT NULL CHECK(json_valid(status_json))
);

-- Free-form findings remain historical records, not actionable notification work.
DROP TRIGGER integrity_event_notification;
CREATE TRIGGER integrity_event_notification AFTER INSERT ON integrity_events
WHEN json_extract(NEW.event_json,'$.category')='integrity'
 AND length(trim(COALESCE(json_extract(NEW.event_json,'$.rule_id'),'')))>0
BEGIN
 INSERT INTO notification_outbox(id,account_id,spreadsheet_id,integrity_event_id,source,spreadsheet_name,sheet_id,severity,diagnosis,detected_at,created_ms,expires_ms,state,next_attempt_ms,group_id)
 SELECT 'integrity:'||NEW.event_id,NEW.account_id,s.spreadsheet_id,NEW.event_id,'integrity',COALESCE(r.name,s.spreadsheet_id),NEW.sheet_id,NEW.severity,COALESCE(json_extract(NEW.event_json,'$.diagnosis'),'Alteração de integridade detectada.'),NEW.detected_at,
 CAST((julianday(NEW.detected_at)-2440587.5)*86400000 AS INTEGER),
 CAST((julianday(NEW.detected_at)-2440587.5)*86400000 AS INTEGER)+CAST(COALESCE(json_extract(s.scope_json,'$.history_retention_days'),90) AS INTEGER)*86400000,
 CASE WHEN COALESCE((SELECT json_extract(preferences_json,'$.notifications_enabled') FROM management_preferences WHERE account_id=NEW.account_id),0)=1 THEN 'pending' ELSE 'suppressed_by_preference' END,
 CAST((julianday(NEW.detected_at)-2440587.5)*86400000 AS INTEGER),'integrity:'||NEW.event_id
 FROM integrity_scopes s LEFT JOIN registered_spreadsheets r ON r.account_id=s.account_id AND r.spreadsheet_id=s.spreadsheet_id WHERE s.id=NEW.scope_id;
END;
UPDATE notification_outbox
SET state='suppressed_by_preference',lease_owner=NULL,lease_until_ms=NULL,attempt_id=NULL
WHERE source='integrity' AND state IN('pending','retry','permission_blocked')
 AND EXISTS(
  SELECT 1 FROM integrity_events e WHERE e.event_id=notification_outbox.integrity_event_id
   AND (COALESCE(json_extract(e.event_json,'$.category'),'')!='integrity'
    OR length(trim(COALESCE(json_extract(e.event_json,'$.rule_id'),'')))=0)
 );
