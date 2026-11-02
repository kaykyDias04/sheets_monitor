//! Local persistence. Demonstration fixtures and authorized integration metadata
//! have separate tables. Cell values/source/tokens are not stored; approved
//! structural snapshots retain protected formulas needed for deterministic checks.
mod integration;
mod integrity_store;
mod migrations;
pub use integration::*;

use rusqlite::{params, Connection, OpenFlags, TransactionBehavior};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, time::Duration};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppSettings {
    pub interval_seconds: u32,
    pub history_days: u32,
    pub start_with_windows: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            interval_seconds: 60,
            history_days: 90,
            start_with_windows: false,
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !(60..=300).contains(&self.interval_seconds) {
            return Err("interval_seconds_must_be_between_60_and_300".into());
        }
        if self.history_days != 90 {
            return Err("history_days_fixed_at_90_in_sprint_01".into());
        }
        if self.start_with_windows {
            return Err("windows_startup_not_implemented_in_sprint_01".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DemoStatus {
    Online,
    Offline,
    Checking,
    Indeterminate,
}

fn status_from_sql(value: String) -> rusqlite::Result<DemoStatus> {
    match value.as_str() {
        "online" => Ok(DemoStatus::Online),
        "offline" => Ok(DemoStatus::Offline),
        "checking" => Ok(DemoStatus::Checking),
        "indeterminate" => Ok(DemoStatus::Indeterminate),
        _ => Err(rusqlite::Error::InvalidQuery),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DemoSpreadsheet {
    pub id: String,
    pub name: String,
    pub url: String,
    pub status: DemoStatus,
    pub selected_sheets: Vec<String>,
    pub last_checked_at: Option<String>,
    pub is_demo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DemoActivity {
    pub id: String,
    pub spreadsheet_id: String,
    pub spreadsheet_name: String,
    pub occurred_at: String,
    pub category: String,
    pub diagnosis: String,
    pub is_demo: bool,
}

pub struct Database {
    connection: Connection,
}

fn database_error(_: rusqlite::Error) -> String {
    "local_database_error".into()
}

impl Database {
    /// Native code supplies its resolved Tauri app_data_dir/foundation.db.
    /// Relative paths and a symbolic-link database leaf are rejected.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        if !path.is_absolute() {
            return Err("database_path_must_be_absolute".into());
        }
        if fs::symlink_metadata(path).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err("database_symlink_not_allowed".into());
        }
        let parent = path.parent().ok_or("database_parent_missing")?;
        fs::create_dir_all(parent).map_err(|_| "database_directory_unavailable")?;
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(database_error)?;
        Self::initialize(connection)
    }

    pub fn memory() -> Result<Self, String> {
        Self::initialize(Connection::open_in_memory().map_err(database_error)?)
    }

    fn initialize(mut connection: Connection) -> Result<Self, String> {
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(database_error)?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA trusted_schema = OFF; PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;")
            .map_err(database_error)?;
        migrations::apply(&mut connection)?;
        Ok(Self { connection })
    }

    pub fn schema_version(&self) -> Result<i64, String> {
        self.connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(database_error)
    }

    /// Diagnostic schema inventory, excluding SQLite and migration internals.
    pub fn table_names(&self) -> Result<Vec<String>, String> {
        let mut statement = self.connection.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' AND name != '_schema_migrations' ORDER BY name")
            .map_err(database_error)?;
        let rows = statement
            .query_map([], |row| row.get(0))
            .map_err(database_error)?;
        rows.collect::<Result<_, _>>().map_err(database_error)
    }

    pub fn settings(&self) -> Result<AppSettings, String> {
        self.connection.query_row("SELECT interval_seconds, history_days, start_with_windows FROM app_settings WHERE id = 1", [], |row| {
            Ok(AppSettings { interval_seconds: row.get(0)?, history_days: row.get(1)?, start_with_windows: row.get(2)? })
        }).map_err(database_error)
    }

    /// Persists preferences only. It never starts a timer or configures Windows.
    pub fn save_settings(&mut self, settings: &AppSettings) -> Result<(), String> {
        settings.validate()?;
        let changed = self.connection.execute(
            "UPDATE app_settings SET interval_seconds = ?1, history_days = ?2, start_with_windows = ?3,
             updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now') WHERE id = 1",
            params![settings.interval_seconds, settings.history_days, settings.start_with_windows],
        ).map_err(database_error)?;
        if changed != 1 {
            return Err("app_settings_missing".into());
        }
        Ok(())
    }

    pub fn list_demo_spreadsheets(&self) -> Result<Vec<DemoSpreadsheet>, String> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, url, status, last_checked_at FROM spreadsheets WHERE is_demo = 1 AND archived_at IS NULL ORDER BY name, id",
        ).map_err(database_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok(DemoSpreadsheet {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    status: status_from_sql(row.get(3)?)?,
                    selected_sheets: Vec::new(),
                    last_checked_at: row.get(4)?,
                    is_demo: true,
                })
            })
            .map_err(database_error)?;
        let mut spreadsheets = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(database_error)?;
        let mut sheets = self.connection.prepare("SELECT title FROM monitored_sheets WHERE spreadsheet_id = ?1 AND is_demo = 1 ORDER BY position, external_sheet_id")
            .map_err(database_error)?;
        for spreadsheet in &mut spreadsheets {
            let rows = sheets
                .query_map([&spreadsheet.id], |row| row.get(0))
                .map_err(database_error)?;
            spreadsheet.selected_sheets = rows.collect::<Result<_, _>>().map_err(database_error)?;
        }
        Ok(spreadsheets)
    }

    pub fn demo_history(&self) -> Result<Vec<DemoActivity>, String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT e.id, e.spreadsheet_id, s.name, e.occurred_at, e.category, e.diagnosis
             FROM change_events e JOIN spreadsheets s ON s.id = e.spreadsheet_id
             WHERE e.is_demo = 1 AND s.is_demo = 1 ORDER BY e.occurred_at DESC, e.id",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok(DemoActivity {
                    id: row.get(0)?,
                    spreadsheet_id: row.get(1)?,
                    spreadsheet_name: row.get(2)?,
                    occurred_at: row.get(3)?,
                    category: row.get(4)?,
                    diagnosis: row.get(5)?,
                    is_demo: true,
                })
            })
            .map_err(database_error)?;
        rows.collect::<Result<_, _>>().map_err(database_error)
    }

    /// Explicit local fixtures, seeded once atomically. No Google URL, authorized
    /// account, approved baseline, executed check or sent notification is invented.
    pub fn seed_demo(&mut self) -> Result<(), String> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(database_error)?;
        let seeded: bool = transaction
            .query_row(
                "SELECT demo_seeded FROM app_settings WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .map_err(database_error)?;
        if seeded {
            return Ok(());
        }
        let timestamp = "2026-11-01T12:00:00Z";
        transaction.execute("INSERT INTO accounts(id, display_name, is_demo, created_at) VALUES ('demo-account', 'Demonstração local — sem login', 1, ?1)", [timestamp]).map_err(database_error)?;
        for (id, name, status, category, diagnosis) in [
            (
                "demo-financeiro",
                "Financeiro · Demonstração",
                "online",
                "content",
                "Exemplo simulado: valor atualizado; integridade preservada.",
            ),
            (
                "demo-vendas",
                "Vendas · Demonstração",
                "offline",
                "structure",
                "Exemplo simulado: coluna protegida ausente.",
            ),
            (
                "demo-estoque",
                "Estoque · Demonstração",
                "checking",
                "operational",
                "Exemplo simulado do estado Verificando; nenhuma consulta está em execução.",
            ),
            (
                "demo-operacoes",
                "Operações · Demonstração",
                "indeterminate",
                "operational",
                "Exemplo simulado: não foi possível confirmar a integridade.",
            ),
        ] {
            transaction.execute(
                "INSERT INTO spreadsheets(id, account_id, name, url, status, scope, is_demo, last_checked_at, created_at)
                 VALUES (?1, 'demo-account', ?2, ?3, ?4, 'selected', 1, ?5, ?5)",
                params![id, name, format!("https://example.invalid/{id}"), status, timestamp],
            ).map_err(database_error)?;
            transaction.execute("INSERT INTO monitored_sheets(id, spreadsheet_id, external_sheet_id, title, is_demo) VALUES (?1, ?2, 0, 'Resumo', 1)", params![format!("{id}-tab"), id]).map_err(database_error)?;
            transaction.execute(
                "INSERT INTO monitoring_checks(id, spreadsheet_id, status, started_at, completed_at, diagnosis, is_demo) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
                params![format!("{id}-check"), id, status, timestamp, if status == "checking" { None } else { Some(timestamp) }, diagnosis],
            ).map_err(database_error)?;
            transaction.execute(
                "INSERT INTO change_events(id, spreadsheet_id, monitoring_check_id, category, diagnosis, details_json, occurred_at, is_demo)
                 VALUES (?1, ?2, ?3, ?4, ?5, '{\"demo\":true}', ?6, 1)",
                params![format!("{id}-event"), id, format!("{id}-check"), category, diagnosis, timestamp],
            ).map_err(database_error)?;
            transaction.execute("INSERT INTO notifications(id, change_event_id, status, created_at, is_demo) VALUES (?1, ?2, 'demo', ?3, 1)", params![format!("{id}-notice"), format!("{id}-event"), timestamp]).map_err(database_error)?;
        }
        transaction.execute(
            "INSERT INTO baselines(id, spreadsheet_id, version, snapshot_json, fingerprint, created_at, is_demo)
             VALUES ('demo-baseline', 'demo-financeiro', 1, '{\"demo\":true,\"sheets\":[\"Resumo\"]}', 'demo-not-an-approved-digest', ?1, 1)", [timestamp],
        ).map_err(database_error)?;
        transaction.execute(
            "INSERT INTO integrity_rules(id, spreadsheet_id, baseline_id, kind, definition_json, is_demo)
             VALUES ('demo-rule', 'demo-financeiro', 'demo-baseline', 'protected_column', '{\"demo\":true,\"column\":\"Receita\"}', 1)", [],
        ).map_err(database_error)?;
        transaction.execute(
            "INSERT INTO script_projects(id, spreadsheet_id, name, is_demo) VALUES ('demo-project', 'demo-financeiro', 'Projeto fictício — não autorizado', 1)", [],
        ).map_err(database_error)?;
        transaction.execute(
            "INSERT INTO script_dependencies(id, script_project_id, kind, reference_json, is_demo)
             VALUES ('demo-dependency', 'demo-project', 'sheet_name', '{\"demo\":true,\"sheet\":\"Resumo\"}', 1)", [],
        ).map_err(database_error)?;
        transaction
            .execute("UPDATE app_settings SET demo_seeded = 1 WHERE id = 1", [])
            .map_err(database_error)?;
        transaction.commit().map_err(database_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_eleven_tables_are_created_and_read_with_explicit_demo_seed() {
        let mut database = Database::memory().unwrap();
        let expected = [
            "accounts",
            "app_settings",
            "baselines",
            "change_events",
            "integrity_rules",
            "monitored_sheets",
            "monitoring_checks",
            "notifications",
            "script_dependencies",
            "script_projects",
            "spreadsheets",
        ];
        let tables = database.table_names().unwrap();
        assert!(expected
            .iter()
            .all(|name| tables.iter().any(|table| table == name)));
        assert!(database.list_demo_spreadsheets().unwrap().is_empty());
        assert!(database.demo_history().unwrap().is_empty());
        database.seed_demo().unwrap();
        for table in expected {
            let count: i64 = database
                .connection
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert!(
                count > 0,
                "fixture table {table} must be readable and populated"
            );
        }
        assert_eq!(database.list_demo_spreadsheets().unwrap().len(), 4);
        assert_eq!(database.demo_history().unwrap().len(), 4);
        assert!(database
            .list_demo_spreadsheets()
            .unwrap()
            .iter()
            .all(|s| s.is_demo
                && s.url.starts_with("https://example.invalid/")
                && s.selected_sheets == ["Resumo"]));
        assert_eq!(
            database
                .connection
                .query_row(
                    "SELECT count(*) FROM baselines WHERE approved_at IS NOT NULL",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        assert_eq!(
            database
                .connection
                .query_row(
                    "SELECT count(*) FROM script_projects WHERE authorized_at IS NOT NULL",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
        assert_eq!(
            database
                .connection
                .query_row(
                    "SELECT count(*) FROM notifications WHERE status = 'sent'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
    }

    #[test]
    fn defaults_bounds_and_unavailable_startup_are_enforced() {
        let mut database = Database::memory().unwrap();
        assert_eq!(database.settings().unwrap(), AppSettings::default());
        for interval_seconds in [60, 120, 300] {
            let settings = AppSettings {
                interval_seconds,
                ..Default::default()
            };
            database.save_settings(&settings).unwrap();
            assert_eq!(database.settings().unwrap(), settings);
        }
        for interval_seconds in [0, 59, 301, u32::MAX] {
            assert!(database
                .save_settings(&AppSettings {
                    interval_seconds,
                    ..Default::default()
                })
                .is_err());
        }
        assert!(database
            .save_settings(&AppSettings {
                start_with_windows: true,
                ..Default::default()
            })
            .is_err());
        assert!(database
            .save_settings(&AppSettings {
                history_days: 1,
                ..Default::default()
            })
            .is_err());
        assert!(database
            .connection
            .execute("UPDATE app_settings SET interval_seconds = 59", [])
            .is_err());
    }

    #[test]
    fn foreign_keys_and_schema_constraints_reject_invalid_records() {
        let mut database = Database::memory().unwrap();
        assert_eq!(
            database
                .connection
                .pragma_query_value(None, "foreign_keys", |row| row.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert!(database.connection.execute("INSERT INTO monitored_sheets(id, spreadsheet_id, external_sheet_id, title) VALUES ('bad','missing',0,'Tab')", []).is_err());
        database.seed_demo().unwrap();
        assert!(database
            .connection
            .execute("UPDATE spreadsheets SET status = 'invalid'", [])
            .is_err());
        assert!(database
            .connection
            .execute("UPDATE baselines SET snapshot_json = '{broken'", [])
            .is_err());
        assert!(database.connection.execute("INSERT INTO baselines SELECT 'duplicate', spreadsheet_id, version, snapshot_json, fingerprint, approved_at, created_at, is_demo FROM baselines", []).is_err());
        assert!(database.connection.execute("INSERT INTO integrity_rules(id,spreadsheet_id,baseline_id,kind,definition_json,is_demo) VALUES('cross-rule','demo-vendas','demo-baseline','demo','{}',1)", []).is_err());
        assert!(database.connection.execute("INSERT INTO change_events(id,spreadsheet_id,monitoring_check_id,category,diagnosis,details_json,occurred_at,is_demo) VALUES('cross-event','demo-vendas','demo-financeiro-check','demo','demo','{}','demo',1)", []).is_err());
    }

    #[test]
    fn fixture_capacity_is_enforced_for_insert_and_restore() {
        let database = Database::memory().unwrap();
        for index in 0..30 {
            database.connection.execute("INSERT INTO spreadsheets(id,name,url,is_demo,created_at) VALUES (?1,'Demo','https://example.invalid',1,'demo')", [format!("fixture-{index}")]).unwrap();
        }
        assert!(database.connection.execute("INSERT INTO spreadsheets(id,name,url,is_demo,created_at) VALUES ('31','Demo','https://example.invalid',1,'demo')", []).is_err());
        database
            .connection
            .execute(
                "UPDATE spreadsheets SET archived_at = 'demo' WHERE id = 'fixture-0'",
                [],
            )
            .unwrap();
        database.connection.execute("INSERT INTO spreadsheets(id,name,url,is_demo,created_at) VALUES ('31','Demo','https://example.invalid',1,'demo')", []).unwrap();
        assert!(database
            .connection
            .execute(
                "UPDATE spreadsheets SET archived_at = NULL WHERE id = 'fixture-0'",
                []
            )
            .is_err());
    }

    #[test]
    fn seed_failure_rolls_back_every_fixture_and_can_be_retried() {
        let mut database = Database::memory().unwrap();
        database.connection.execute_batch("CREATE TRIGGER reject_fixture BEFORE INSERT ON script_projects BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        assert!(database.seed_demo().is_err());
        assert!(database.list_demo_spreadsheets().unwrap().is_empty());
        assert!(database.demo_history().unwrap().is_empty());
        assert!(!database
            .connection
            .query_row("SELECT demo_seeded FROM app_settings", [], |row| row
                .get::<_, bool>(0))
            .unwrap());
        database
            .connection
            .execute_batch("DROP TRIGGER reject_fixture;")
            .unwrap();
        database.seed_demo().unwrap();
        database.seed_demo().unwrap();
        assert_eq!(database.demo_history().unwrap().len(), 4);
    }

    #[test]
    fn unsafe_relative_database_path_is_rejected() {
        assert!(Database::open("relative.db").is_err());
    }
}
