//! Synthetic fixtures only; no account/resource from Google is used here.
use super::*;
use std::collections::BTreeMap;

pub(crate) fn scope() -> MonitorScope {
    MonitorScope {
        schema_version: SNAPSHOT_VERSION,
        sheets: vec![SheetRule {
            sheet_id: 7,
            protect_title: true,
            protect_position: false,
            regions: vec![RegionRule {
                id: "records".into(),
                range: GridRect {
                    start_row: 0,
                    end_row: 8,
                    start_col: 0,
                    end_col: 4,
                },
                header_row: 0,
                columns: vec![
                    ColumnRule {
                        id: "id".into(),
                        header: "ID".into(),
                        position: 0,
                        protect_position: true,
                        required: true,
                    },
                    ColumnRule {
                        id: "amount".into(),
                        header: "Amount".into(),
                        position: 1,
                        protect_position: true,
                        required: true,
                    },
                    ColumnRule {
                        id: "total".into(),
                        header: "Total".into(),
                        position: 2,
                        protect_position: true,
                        required: true,
                    },
                ],
                key_column: Some("id".into()),
                critical: true,
                allow_record_deletion: true,
                formulas: vec![FormulaRule {
                    id: "total_formula".into(),
                    column_id: "total".into(),
                    baseline_row: 1,
                    mode: FormulaMode::Structural,
                    allow_row_shift: true,
                }],
            }],
        }],
        scripts: vec![],
        named_range_ids: vec![],
        snapshot_retention_days: 7,
        history_retention_days: 90,
    }
}
pub(crate) fn row(number: u32, key: &str, amount: i64) -> RowDigest {
    let cell_hashes = BTreeMap::from([
        ("id".into(), fingerprint(&key)),
        ("amount".into(), fingerprint(&amount)),
        ("total".into(), fingerprint(&(amount * 2))),
        ("column:3".into(), fingerprint(&"note")),
    ]);
    RowDigest {
        row: number,
        key_hash: Some(fingerprint(&key)),
        fingerprint: fingerprint(&cell_hashes),
        cell_hashes,
    }
}
pub(crate) fn snapshot() -> StructuralSnapshot {
    let scope = scope();
    let rows = vec![row(1, "record_a", 10), row(2, "record_b", 20)];
    StructuralSnapshot {
        schema_version: SNAPSHOT_VERSION,
        id: "snapshot_fixture_1".into(),
        account_subject: "fixture_account".into(),
        spreadsheet_id: "fixture_sheet".into(),
        capture_started_at: "2026-11-02T12:00:00Z".into(),
        captured_at: "2026-11-02T12:00:01Z".into(),
        scope_hash: scope.hash(),
        scope,
        sheets: vec![SheetSnapshot {
            sheet_id: 7,
            title: "Data".into(),
            index: 0,
            row_count: 100,
            column_count: 4,
            regions: vec![RegionSnapshot {
                region_id: "records".into(),
                range: GridRect {
                    start_row: 0,
                    end_row: 8,
                    start_col: 0,
                    end_col: 4,
                },
                complete: true,
                headers: vec![
                    ObservedColumn {
                        column: 0,
                        header: "ID".into(),
                    },
                    ObservedColumn {
                        column: 1,
                        header: "Amount".into(),
                    },
                    ObservedColumn {
                        column: 2,
                        header: "Total".into(),
                    },
                    ObservedColumn {
                        column: 3,
                        header: "Note".into(),
                    },
                ],
                formulas: rows
                    .iter()
                    .map(|row| ObservedFormula {
                        row: row.row,
                        column: 2,
                        formula: Some(format!("=B{}*2", row.row + 1)),
                        error: None,
                        value_hash: row.cell_hashes["total"].clone(),
                        key_hash: row.key_hash.clone(),
                    })
                    .collect(),
                rows,
            }],
        }],
        named_ranges: vec![],
        scripts: vec![],
        coverage: Coverage {
            metadata_complete: true,
            complete: true,
            issues: vec![],
            atomic_across_services: false,
        },
        connectivity: Connectivity::Reachable,
    }
}
pub(crate) fn baseline() -> BaselineVersion {
    BaselineVersion {
        id: "baseline_fixture_1".into(),
        version: 1,
        active: true,
        approved_at: "2026-11-02T12:01:00Z".into(),
        approval_origin: "synthetic_fixture".into(),
        snapshot: snapshot(),
    }
}
pub(crate) fn rehash(snapshot: &mut StructuralSnapshot) {
    snapshot.scope_hash = snapshot.scope.hash();
}
pub(crate) fn advance(snapshot: &mut StructuralSnapshot) {
    snapshot.id = "snapshot_fixture_2".into();
    snapshot.captured_at = "2026-11-02T12:02:00Z".into();
}
pub(crate) fn attach_script(snapshot: &mut StructuralSnapshot) {
    snapshot.scope.scripts = vec![ScriptRule {
        script_id: "fixture_script".into(),
        critical: false,
    }];
    snapshot.scripts = vec![ScriptSnapshot {
        script_id: "fixture_script".into(),
        source_hash: fingerprint(&"source_a"),
        files: vec![ScriptFileDigest {
            name: "Code".into(),
            file_type: "SERVER_JS".into(),
            hash: fingerprint(&"source_a"),
        }],
        analysis: ScriptAnalysis {
            parser: "fixture_ast".into(),
            complete: true,
            dependencies: vec![],
            limitations: vec![],
        },
    }];
    rehash(snapshot);
}
