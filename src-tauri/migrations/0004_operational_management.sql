-- Sprint 04: local operational registry. Approved snapshots remain immutable.
CREATE TABLE registered_spreadsheets (
 account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
 spreadsheet_id TEXT NOT NULL,
 name TEXT NOT NULL,
 paused INTEGER NOT NULL DEFAULT 0 CHECK(paused IN(0,1)),
 removed INTEGER NOT NULL DEFAULT 0 CHECK(removed IN(0,1)),
 scope_json TEXT CHECK(scope_json IS NULL OR json_valid(scope_json)),
 scope_hash TEXT CHECK(scope_hash IS NULL OR length(scope_hash)=64),
 selected_sheet_ids_json TEXT NOT NULL DEFAULT '[]' CHECK(json_valid(selected_sheet_ids_json)),
 tabs_json TEXT NOT NULL CHECK(json_valid(tabs_json)),
 revision INTEGER NOT NULL DEFAULT 1 CHECK(revision>=1),
 authorization TEXT NOT NULL DEFAULT 'available' CHECK(authorization IN('unknown','available','error')),
 access_error TEXT,
 tabs_refreshed_at TEXT,
 created_at TEXT NOT NULL,
 updated_at TEXT NOT NULL,
 PRIMARY KEY(account_id,spreadsheet_id)
);
CREATE TRIGGER registry_capacity_insert BEFORE INSERT ON registered_spreadsheets
WHEN NEW.removed=0 AND (SELECT count(*) FROM registered_spreadsheets WHERE removed=0)>=30
BEGIN SELECT RAISE(ABORT,'registry_limit_30'); END;
CREATE TRIGGER registry_capacity_restore BEFORE UPDATE OF removed ON registered_spreadsheets
WHEN OLD.removed=1 AND NEW.removed=0 AND (SELECT count(*) FROM registered_spreadsheets WHERE removed=0)>=30
BEGIN SELECT RAISE(ABORT,'registry_limit_30'); END;
CREATE INDEX registry_account_active ON registered_spreadsheets(account_id,removed,name);
CREATE TABLE management_preferences (
 account_id TEXT PRIMARY KEY NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
 preferences_json TEXT NOT NULL CHECK(json_valid(preferences_json))
);
-- Archived files do not consume a technical capture slot. Existing scopes and baselines stay intact.
DROP TRIGGER integrity_capacity_insert;
CREATE TRIGGER integrity_capacity_insert BEFORE INSERT ON integrity_scopes
WHEN NOT EXISTS(SELECT 1 FROM integrity_scopes WHERE account_id=NEW.account_id AND spreadsheet_id=NEW.spreadsheet_id)
 AND (SELECT count(*) FROM (SELECT s.account_id,s.spreadsheet_id FROM integrity_scopes s WHERE NOT EXISTS(SELECT 1 FROM registered_spreadsheets r WHERE r.account_id=s.account_id AND r.spreadsheet_id=s.spreadsheet_id AND r.removed=1) GROUP BY s.account_id,s.spreadsheet_id))>=30
BEGIN SELECT RAISE(ABORT,'integrity_limit_30'); END;
