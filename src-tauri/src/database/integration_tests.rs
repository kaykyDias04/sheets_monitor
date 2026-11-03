use super::*;
#[test]
fn metadata_is_account_scoped_without_payload_or_baseline_writes() {
    let mut db = Database::memory().unwrap();
    db.seed_demo().unwrap();
    db.upsert_account(
        "account_a",
        Some("fixture@example.invalid"),
        Some("Fixture"),
    )
    .unwrap();
    db.upsert_account("account_b", None, None).unwrap();
    let mut input = IntegrationQueryInput {
        service: QueryService::SheetsMetadata,
        resource_id: "sheet_a".into(),
        resource_name: Some("Fixture sheet".into()),
        associated_spreadsheet_id: None,
        source_hash: None,
        error: None,
    };
    db.record_query("account_a", &input).unwrap();
    assert!(db.integration_queries("account_b").unwrap().is_empty());
    input.service = QueryService::AppsScript;
    input.resource_id = "script_a".into();
    input.source_hash = Some("a".repeat(64));
    input.associated_spreadsheet_id = Some("sheet_a".into());
    assert!(db.record_query("account_b", &input).is_err());
    db.record_query("account_a", &input).unwrap();
    let rows = db.integration_queries("account_a").unwrap();
    assert_eq!(rows.len(), 2);
    let script = rows.iter().find(|q| q.service == "apps_script").unwrap();
    assert_eq!(script.associated_spreadsheet_ids, vec!["sheet_a"]);
    assert!(rows.iter().all(|q| !q.is_demo));
    assert_eq!(
        db.connection
            .query_row("SELECT count(*) FROM baselines WHERE is_demo=0", [], |r| {
                r.get::<_, i64>(0)
            })
            .unwrap(),
        0
    );
    let encoded = serde_json::to_string(&rows).unwrap();
    for forbidden in [
        "access_token",
        "refresh_token",
        "client_secret",
        "source\"",
        "values\"",
    ] {
        assert!(!encoded.contains(forbidden));
    }
}
#[test]
fn second_migration_preserves_first_schema_and_preferences() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("upgrade.db");
    {
        let mut connection = rusqlite::Connection::open(&path).unwrap();
        super::super::migrations::apply_list(
            &mut connection,
            &[super::super::migrations::Migration {
                version: 1,
                name: "0001_foundation",
                sql: include_str!("../../migrations/0001_foundation.sql"),
            }],
        )
        .unwrap();
        connection
            .execute(
                "UPDATE app_settings SET interval_seconds=180 WHERE id=1",
                [],
            )
            .unwrap();
    }
    let db = Database::open(&path).unwrap();
    assert_eq!(db.schema_version().unwrap(), 5);
    assert_eq!(db.settings().unwrap().interval_seconds, 180);
    assert_eq!(db.integration_queries("account_a").unwrap().len(), 0);
}
