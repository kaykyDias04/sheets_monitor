-- Sprint 02: metadata only. Never store OAuth credentials or response content.
ALTER TABLE accounts ADD COLUMN email TEXT;
ALTER TABLE accounts ADD COLUMN last_connected_at TEXT;

CREATE TABLE integration_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    client_id TEXT NOT NULL CHECK (length(client_id) BETWEEN 1 AND 256),
    drive_metadata_enabled INTEGER NOT NULL DEFAULT 0 CHECK (drive_metadata_enabled IN (0, 1)),
    updated_at TEXT NOT NULL
);

CREATE TABLE integration_resources (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('spreadsheet', 'file', 'script')),
    external_id TEXT NOT NULL,
    name TEXT,
    source_hash TEXT CHECK (source_hash IS NULL OR length(source_hash) = 64),
    last_status TEXT NOT NULL CHECK (last_status IN ('success', 'error')),
    last_error_kind TEXT,
    last_queried_at TEXT NOT NULL,
    is_demo INTEGER NOT NULL DEFAULT 0 CHECK (is_demo = 0),
    UNIQUE (account_id, kind, external_id),
    UNIQUE (id, account_id)
);

CREATE TABLE integration_queries (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    resource_id TEXT NOT NULL,
    service TEXT NOT NULL CHECK (service IN ('sheets_metadata', 'sheets_values', 'drive', 'apps_script')),
    status TEXT NOT NULL CHECK (status IN ('success', 'error')),
    error_kind TEXT,
    queried_at TEXT NOT NULL,
    is_demo INTEGER NOT NULL DEFAULT 0 CHECK (is_demo = 0),
    FOREIGN KEY (resource_id, account_id) REFERENCES integration_resources(id, account_id) ON DELETE CASCADE,
    UNIQUE (account_id, service, resource_id)
);

CREATE TABLE integration_associations (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    spreadsheet_resource_id TEXT NOT NULL,
    script_resource_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (account_id, spreadsheet_resource_id, script_resource_id),
    FOREIGN KEY (spreadsheet_resource_id, account_id) REFERENCES integration_resources(id, account_id) ON DELETE CASCADE,
    FOREIGN KEY (script_resource_id, account_id) REFERENCES integration_resources(id, account_id) ON DELETE CASCADE
);
