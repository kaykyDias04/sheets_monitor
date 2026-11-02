use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};

pub(super) struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    // Applied in order; prior migrations retain their original checksums.
    Migration {
        version: 1,
        name: "0001_foundation",
        sql: include_str!("../../migrations/0001_foundation.sql"),
    },
    Migration {
        version: 2,
        name: "0002_google_integration",
        sql: include_str!("../../migrations/0002_google_integration.sql"),
    },
    Migration {
        version: 3,
        name: "0003_integrity_engine",
        sql: include_str!("../../migrations/0003_integrity_engine.sql"),
    },
    Migration {
        version: 4,
        name: "0004_operational_management",
        sql: include_str!("../../migrations/0004_operational_management.sql"),
    },
];

pub(super) fn apply(connection: &mut Connection) -> Result<(), String> {
    apply_list(connection, MIGRATIONS)
}

fn checksum(sql: &str) -> String {
    // Git may check out CRLF on Windows and LF in CI. A line-ending-only
    // conversion does not constitute a different migration.
    format!("{:x}", Sha256::digest(sql.replace("\r\n", "\n").as_bytes()))
}

/// Verify and migrate under one write transaction. Failure also rolls back the
/// ledger creation and the user_version; no half-applied migration is accepted.
pub(super) fn apply_list(
    connection: &mut Connection,
    migrations: &[Migration],
) -> Result<(), String> {
    if migrations
        .iter()
        .enumerate()
        .any(|(i, m)| m.version != i as i64 + 1)
    {
        return Err("migration_sequence_invalid".into());
    }
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|_| "migration_transaction_failed")?;
    let user_version: i64 = transaction
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| "migration_version_unreadable")?;
    let supported = migrations.last().map_or(0, |m| m.version);
    if user_version > supported {
        return Err("database_version_newer_than_application".into());
    }
    transaction
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS _schema_migrations (
            version INTEGER PRIMARY KEY CHECK (version >= 1),
            name TEXT NOT NULL UNIQUE,
            checksum TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );",
        )
        .map_err(|_| "migration_ledger_failed")?;
    let latest: i64 = transaction
        .query_row(
            "SELECT COALESCE(max(version), 0) FROM _schema_migrations",
            [],
            |row| row.get(0),
        )
        .map_err(|_| "migration_ledger_unreadable")?;
    if latest > supported {
        return Err("database_version_newer_than_application".into());
    }
    let count: i64 = transaction
        .query_row("SELECT count(*) FROM _schema_migrations", [], |row| {
            row.get(0)
        })
        .map_err(|_| "migration_ledger_unreadable")?;
    if count != latest || user_version != latest {
        return Err("migration_ledger_inconsistent".into());
    }
    for migration in migrations {
        let expected = checksum(migration.sql);
        let applied: Option<(String, String)> = transaction
            .query_row(
                "SELECT name, checksum FROM _schema_migrations WHERE version = ?1",
                [migration.version],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(|_| "migration_ledger_unreadable")?;
        if let Some((name, actual)) = applied {
            if name != migration.name || actual != expected {
                return Err("migration_checksum_mismatch".into());
            }
            continue;
        }
        transaction
            .execute_batch(migration.sql)
            .map_err(|_| "migration_execution_failed")?;
        transaction
            .execute(
                "INSERT INTO _schema_migrations(version, name, checksum, applied_at)
             VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
                params![migration.version, migration.name, expected],
            )
            .map_err(|_| "migration_ledger_write_failed")?;
        transaction
            .pragma_update(None, "user_version", migration.version)
            .map_err(|_| "migration_version_write_failed")?;
    }
    transaction
        .commit()
        .map_err(|_| "migration_commit_failed".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_failure_rolls_back_schema_ledger_and_version() {
        let mut connection = Connection::open_in_memory().unwrap();
        let migrations = [
            Migration {
                version: 1,
                name: "one",
                sql: "CREATE TABLE first_table(id INTEGER);",
            },
            Migration {
                version: 2,
                name: "two",
                sql: "CREATE TABLE second_table(id INTEGER); INVALID SQL;",
            },
        ];
        assert_eq!(
            apply_list(&mut connection, &migrations).unwrap_err(),
            "migration_execution_failed"
        );
        let count: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
        assert_eq!(
            connection
                .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn failed_upgrade_keeps_previous_successful_version() {
        let mut connection = Connection::open_in_memory().unwrap();
        let one = [Migration {
            version: 1,
            name: "one",
            sql: "CREATE TABLE stable(id INTEGER);",
        }];
        apply_list(&mut connection, &one).unwrap();
        let upgrade = [
            Migration {
                version: 1,
                name: "one",
                sql: "CREATE TABLE stable(id INTEGER);",
            },
            Migration {
                version: 2,
                name: "bad",
                sql: "DROP TABLE stable; INVALID SQL;",
            },
        ];
        assert!(apply_list(&mut connection, &upgrade).is_err());
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM stable", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            connection
                .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM _schema_migrations", [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn modified_migration_is_rejected() {
        let mut connection = Connection::open_in_memory().unwrap();
        apply(&mut connection).unwrap();
        connection
            .execute("UPDATE _schema_migrations SET checksum = 'tampered'", [])
            .unwrap();
        assert_eq!(
            apply(&mut connection).unwrap_err(),
            "migration_checksum_mismatch"
        );
    }

    #[test]
    fn migration_checksum_is_stable_across_windows_line_endings() {
        assert_eq!(
            checksum("SELECT 1;\nSELECT 2;\n"),
            checksum("SELECT 1;\r\nSELECT 2;\r\n")
        );
        assert_ne!(checksum("SELECT 1;\n"), checksum("SELECT 2;\n"));
    }

    #[test]
    fn future_version_and_ledger_are_rejected() {
        let mut connection = Connection::open_in_memory().unwrap();
        connection.pragma_update(None, "user_version", 99).unwrap();
        assert_eq!(
            apply(&mut connection).unwrap_err(),
            "database_version_newer_than_application"
        );
        connection.pragma_update(None, "user_version", 0).unwrap();
        apply(&mut connection).unwrap();
        connection
            .execute(
                "INSERT INTO _schema_migrations VALUES(5,'future','unknown','demo')",
                [],
            )
            .unwrap();
        assert_eq!(
            apply(&mut connection).unwrap_err(),
            "database_version_newer_than_application"
        );
    }
}
