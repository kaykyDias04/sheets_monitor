//! Synthetic capacity evidence; no network, scheduler or real baseline approval.
use super::*;
#[test]
fn thirty_workbooks_with_one_thousand_records_have_bounded_payloads_and_precise_differences() {
    let started = std::time::Instant::now();
    let mut maximum_bytes = 0;
    let mut total_differences = 0;
    for index in 0..30 {
        let mut base = fixtures::baseline();
        base.id = format!("scale_baseline_{index}");
        base.snapshot.spreadsheet_id = format!("scale_sheet_{index}");
        base.snapshot.scope.sheets[0].regions[0].range.end_row = 1200;
        base.snapshot.scope.sheets[0].regions[0].formulas.clear();
        fixtures::rehash(&mut base.snapshot);
        base.snapshot.sheets[0].regions[0].range.end_row = 1200;
        base.snapshot.sheets[0].row_count = 1200;
        base.snapshot.sheets[0].regions[0].rows = (0..1000)
            .map(|r| fixtures::row(r + 1, &format!("record_{r}"), i64::from(r)))
            .collect();
        let mut current = base.snapshot.clone();
        current.id = format!("scale_observed_{index}");
        let row = &mut current.sheets[0].regions[0].rows[500];
        row.cell_hashes.insert("amount".into(), fingerprint(&9999));
        row.fingerprint = fingerprint(&row.cell_hashes);
        current.validate().unwrap();
        maximum_bytes = maximum_bytes.max(serde_json::to_vec(&current).unwrap().len());
        let report = evaluate(&base, Some(&base.snapshot), &current, "scale_check").unwrap();
        assert_eq!(report.status, IntegrityStatus::Online);
        assert_eq!(report.differences.len(), 1);
        total_differences += report.differences.len();
    }
    println!("synthetic_workbooks=30 records_per_workbook=1000 differences={} maximum_snapshot_bytes={} elapsed_ms={}",total_differences,maximum_bytes,started.elapsed().as_millis());
    assert!(maximum_bytes < 8 * 1024 * 1024);
}
#[test]
fn configured_cell_budget_is_enforced_before_collection() {
    let mut scope = fixtures::scope();
    scope.sheets[0].regions[0].range.end_col = 100;
    scope.sheets[0].regions[0].range.end_row = (MAX_CELLS / 100) as u32 + 1;
    assert_eq!(scope.validate(), Err(EngineError::Limit));
}
