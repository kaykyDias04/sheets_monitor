use rusqlite::Connection;
use sheets_monitor_lib::database::{AppSettings, Database};

#[test]
fn database_reopens_with_same_data_settings_and_migration_ledger() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("app-data").join("foundation.db");
    let mut database = Database::open(&path).unwrap();
    assert_eq!(database.schema_version().unwrap(), 1);
    database.seed_demo().unwrap();
    let before = database.list_demo_spreadsheets().unwrap();
    let history = database.demo_history().unwrap();
    database
        .save_settings(&AppSettings {
            interval_seconds: 180,
            ..Default::default()
        })
        .unwrap();
    drop(database);
    let mut database = Database::open(&path).unwrap();
    database.seed_demo().unwrap();
    assert_eq!(database.list_demo_spreadsheets().unwrap(), before);
    assert_eq!(database.demo_history().unwrap(), history);
    assert_eq!(database.settings().unwrap().interval_seconds, 180);
    assert_eq!(database.table_names().unwrap().len(), 11);
    drop(database);
    let connection = Connection::open(&path).unwrap();
    assert_eq!(
        connection
            .query_row("SELECT count(*) FROM _schema_migrations", [], |row| row
                .get::<_, i64>(0))
            .unwrap(),
        1
    );
    let checksum: String = connection
        .query_row("SELECT checksum FROM _schema_migrations", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(checksum.len(), 64);
    let fk_errors: i64 = connection
        .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(fk_errors, 0);
}

#[test]
fn existing_future_database_is_not_downgraded() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("future.db");
    let connection = Connection::open(&path).unwrap();
    connection.pragma_update(None, "user_version", 999).unwrap();
    drop(connection);
    assert!(
        matches!(Database::open(&path), Err(error) if error == "database_version_newer_than_application")
    );
    let connection = Connection::open(&path).unwrap();
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
            .unwrap(),
        999
    );
}
