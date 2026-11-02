//! Bounded Google collection. Values/source are reduced before persistence.
use super::*;
use crate::google::{
    grid::{GridCell, GridDocument},
    ApiError, GoogleApi, ReadTransport,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::{Duration, Instant},
};
pub fn column_name(mut index: u32) -> String {
    let mut s = String::new();
    loop {
        s.insert(0, (b'A' + (index % 26) as u8) as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    s
}
pub fn a1_range(title: &str, r: &GridRect) -> String {
    format!(
        "'{}'!{}{}:{}{}",
        title.replace('\'', "''"),
        column_name(r.start_col),
        r.start_row + 1,
        column_name(r.end_col - 1),
        r.end_row
    )
}
fn issue(
    s: &mut StructuralSnapshot,
    sheet_id: Option<u32>,
    rule_id: Option<String>,
    critical: bool,
    code: &str,
) {
    s.coverage.complete = false;
    s.coverage.issues.push(CoverageIssue {
        sheet_id,
        rule_id,
        critical,
        code: code.into(),
    });
}
fn api_issue(s: &mut StructuralSnapshot, sheet: Option<u32>, code: ApiError) {
    s.connectivity = match code {
        ApiError::Unauthorized => Connectivity::AuthenticationFailed,
        ApiError::Network | ApiError::Timeout => Connectivity::NetworkFailed,
        _ => Connectivity::Partial,
    };
    issue(s, sheet, None, true, code.kind());
}
fn region_critical(s: &StructuralSnapshot, sheet_id: u32, region_id: &str) -> bool {
    s.scope
        .sheets
        .iter()
        .find(|sheet| sheet.sheet_id == sheet_id)
        .and_then(|sheet| sheet.regions.iter().find(|r| r.id == region_id))
        .is_none_or(|r| r.critical)
}
fn request_issue(
    s: &mut StructuralSnapshot,
    batch: &[(u32, String, String, GridRect)],
    code: &str,
) {
    s.connectivity = Connectivity::Partial;
    for (id, region, _, _) in batch {
        let critical = region_critical(s, *id, region);
        issue(s, Some(*id), Some(region.clone()), critical, code);
    }
}
pub fn failed_snapshot(
    subject: &str,
    id: &str,
    scope: MonitorScope,
    code: &str,
    connectivity: Connectivity,
) -> EngineResult<StructuralSnapshot> {
    scope.validate()?;
    let at = chrono::Utc::now().to_rfc3339();
    Ok(StructuralSnapshot {
        schema_version: SNAPSHOT_VERSION,
        id: uuid::Uuid::new_v4().to_string(),
        account_subject: subject.into(),
        spreadsheet_id: id.into(),
        capture_started_at: at.clone(),
        captured_at: at,
        scope_hash: scope.hash(),
        scope,
        sheets: Vec::new(),
        named_ranges: Vec::new(),
        scripts: Vec::new(),
        coverage: Coverage {
            metadata_complete: false,
            complete: false,
            issues: vec![CoverageIssue {
                sheet_id: None,
                rule_id: None,
                critical: true,
                code: code.into(),
            }],
            atomic_across_services: false,
        },
        connectivity,
    })
}
pub fn capture_with<T: ReadTransport>(
    api: &GoogleApi<T>,
    subject: &str,
    id: &str,
    scope: MonitorScope,
) -> EngineResult<StructuralSnapshot> {
    scope.validate()?;
    crate::google::validate_google_id(id).map_err(|_| EngineError::InvalidScope)?;
    let mut snapshot = failed_snapshot(
        subject,
        id,
        scope,
        "capture_pending",
        Connectivity::Reachable,
    )?;
    snapshot.coverage.issues.clear();
    snapshot.coverage.complete = true;
    let started = Instant::now();
    let metadata = match api.structural_grid(id, &[]) {
        Ok(m) => m,
        Err(e) => {
            api_issue(&mut snapshot, None, e);
            return Ok(snapshot);
        }
    };
    snapshot.coverage.metadata_complete = true;
    snapshot.sheets = metadata
        .sheets
        .iter()
        .map(|s| SheetSnapshot {
            sheet_id: s.properties.sheet_id,
            title: s.properties.title.clone(),
            index: s.properties.index,
            row_count: s
                .properties
                .grid_properties
                .as_ref()
                .map_or(0, |g| g.row_count),
            column_count: s
                .properties
                .grid_properties
                .as_ref()
                .map_or(0, |g| g.column_count),
            regions: Vec::new(),
        })
        .collect();
    snapshot.named_ranges = metadata
        .named_ranges
        .iter()
        .filter_map(|n| {
            let sheet = snapshot
                .sheets
                .iter()
                .find(|s| s.sheet_id == n.range.sheet_id)?;
            Some(NamedRangeSnapshot {
                id: n.named_range_id.clone(),
                name: n.name.clone(),
                sheet_id: n.range.sheet_id,
                range: GridRect {
                    start_row: n.range.start_row_index.unwrap_or(0),
                    end_row: n.range.end_row_index.unwrap_or(sheet.row_count),
                    start_col: n.range.start_column_index.unwrap_or(0),
                    end_col: n.range.end_column_index.unwrap_or(sheet.column_count),
                },
            })
        })
        .collect();
    let rules = snapshot.scope.sheets.clone();
    let mut requests = Vec::new();
    for sheet_rule in &rules {
        let Some(sheet) = snapshot
            .sheets
            .iter()
            .find(|s| s.sheet_id == sheet_rule.sheet_id)
        else {
            continue;
        };
        let source = metadata
            .sheets
            .iter()
            .find(|s| s.properties.sheet_id == sheet.sheet_id)
            .ok_or(EngineError::InvalidSnapshot)?;
        if source
            .properties
            .sheet_type
            .as_deref()
            .is_some_and(|kind| kind != "GRID")
        {
            issue(
                &mut snapshot,
                Some(sheet_rule.sheet_id),
                None,
                true,
                "unsupported_sheet_type",
            );
            continue;
        }
        for rule in &sheet_rule.regions {
            let r = GridRect {
                start_row: rule.range.start_row,
                end_row: rule.range.end_row.min(sheet.row_count),
                start_col: rule.range.start_col,
                end_col: rule.range.end_col.min(sheet.column_count),
            };
            if r.start_row < r.end_row && r.start_col < r.end_col {
                requests.push((
                    sheet.sheet_id,
                    rule.id.clone(),
                    a1_range(&sheet.title, &r),
                    r,
                ));
            }
        }
    }
    let mut cells: BTreeMap<(u32, u32, u32), GridCell> = BTreeMap::new();
    let mut retained_bytes: usize = 0;
    let mut unavailable = BTreeSet::new();
    let mut offset = 0;
    while offset < requests.len() {
        let begin = offset;
        let mut length = 0;
        while offset < requests.len()
            && offset - begin < 20
            && length + requests[offset].2.len() <= 2800
        {
            length += requests[offset].2.len();
            offset += 1;
        }
        if offset == begin {
            return Err(EngineError::Limit);
        }
        let batch = &requests[begin..offset];
        if started.elapsed() > Duration::from_secs(180) {
            for (id, r, _, _) in batch {
                unavailable.insert((*id, r.clone()));
            }
            request_issue(&mut snapshot, batch, ApiError::Timeout.kind());
            continue;
        }
        match api.structural_grid(id, &batch.iter().map(|r| r.2.clone()).collect::<Vec<_>>()) {
            Ok(grid) => {
                merge_cells(
                    &grid,
                    batch,
                    &mut cells,
                    &mut snapshot,
                    &mut unavailable,
                    &mut retained_bytes,
                );
            }
            Err(e) => {
                for (id, r, _, _) in batch {
                    unavailable.insert((*id, r.clone()));
                }
                request_issue(&mut snapshot, batch, e.kind());
            }
        }
    }
    for sheet_rule in &rules {
        if let Some(sheet) = snapshot
            .sheets
            .iter_mut()
            .find(|s| s.sheet_id == sheet_rule.sheet_id)
        {
            for rule in &sheet_rule.regions {
                if metadata
                    .sheets
                    .iter()
                    .find(|s| s.properties.sheet_id == sheet.sheet_id)
                    .and_then(|s| s.properties.sheet_type.as_deref())
                    .is_some_and(|s| s != "GRID")
                {
                    continue;
                }
                let observed = normalize_region(
                    sheet,
                    rule,
                    &cells,
                    !unavailable.contains(&(sheet.sheet_id, rule.id.clone())),
                );
                sheet.regions.push(observed);
            }
        }
    }
    let scripts = snapshot.scope.scripts.clone();
    for rule in scripts {
        if started.elapsed() > Duration::from_secs(180) {
            issue(
                &mut snapshot,
                None,
                Some(rule.script_id.clone()),
                rule.critical,
                "capture_timeout",
            );
            continue;
        }
        match api.script_project(&rule.script_id) {
            Ok(project) => {
                match super::scripts::analyze_sources(
                    &project.files,
                    id,
                    project.metadata.parent_id.as_deref() == Some(id),
                ) {
                    Ok(analysis) => {
                        let files = project
                            .files
                            .iter()
                            .map(|f| ScriptFileDigest {
                                name: f.name.clone(),
                                file_type: format!("{:?}", f.file_type),
                                hash: fingerprint(&f.source),
                            })
                            .collect();
                        snapshot.scripts.push(ScriptSnapshot {
                            script_id: rule.script_id,
                            source_hash: project.source_hash,
                            files,
                            analysis,
                        });
                    }
                    Err(_) => {
                        let files = project
                            .files
                            .iter()
                            .map(|f| ScriptFileDigest {
                                name: f.name.clone(),
                                file_type: format!("{:?}", f.file_type),
                                hash: fingerprint(&f.source),
                            })
                            .collect();
                        snapshot.scripts.push(ScriptSnapshot {
                            script_id: rule.script_id.clone(),
                            source_hash: project.source_hash,
                            files,
                            analysis: ScriptAnalysis {
                                parser: "tree-sitter-javascript".into(),
                                complete: false,
                                dependencies: vec![],
                                limitations: vec!["analysis_limit".into()],
                            },
                        });
                        issue(
                            &mut snapshot,
                            None,
                            Some(rule.script_id),
                            rule.critical,
                            "script_analysis_limit",
                        );
                    }
                }
            }
            Err(e) => {
                let code = e.kind();
                issue(
                    &mut snapshot,
                    None,
                    Some(rule.script_id),
                    rule.critical,
                    code,
                );
                snapshot.connectivity = Connectivity::Partial;
            }
        }
    }
    snapshot.captured_at = chrono::Utc::now().to_rfc3339();
    snapshot.validate()?;
    Ok(snapshot)
}
fn merge_cells(
    grid: &GridDocument,
    batch: &[(u32, String, String, GridRect)],
    cells: &mut BTreeMap<(u32, u32, u32), GridCell>,
    snapshot: &mut StructuralSnapshot,
    unavailable: &mut BTreeSet<(u32, String)>,
    retained_bytes: &mut usize,
) {
    for (index, (id, region, _, _)) in batch.iter().enumerate() {
        if !grid.sheets.iter().any(|s| s.properties.sheet_id == *id) {
            unavailable.insert((*id, region.clone()));
            request_issue(snapshot, &batch[index..index + 1], "grid_sheet_missing");
        }
    }
    for sheet in &grid.sheets {
        for block in &sheet.data {
            for (i, row) in block.row_data.iter().enumerate() {
                for (j, cell) in row.values.iter().enumerate() {
                    let Some(r) = block.start_row.checked_add(i as u32) else {
                        continue;
                    };
                    let Some(c) = block.start_column.checked_add(j as u32) else {
                        continue;
                    };
                    if !batch.iter().any(|(id, _, _, rect)| {
                        *id == sheet.properties.sheet_id && rect.contains(r, c)
                    }) {
                        request_issue(snapshot, batch, "grid_outside_requested_range");
                        for (id, region, _, _) in batch {
                            unavailable.insert((*id, region.clone()));
                        }
                        continue;
                    }
                    let key = (sheet.properties.sheet_id, r, c);
                    if let Some(old) = cells.get(&key) {
                        if old.value() != cell.value()
                            || old.formula() != cell.formula()
                            || old.error() != cell.error()
                        {
                            request_issue(snapshot, batch, "inconsistent_grid_read");
                            for (id, region, _, _) in batch {
                                unavailable.insert((*id, region.clone()));
                            }
                        }
                    } else {
                        let estimated = cell.value().to_string().len()
                            + cell.formula().map_or(0, str::len)
                            + cell.error().map_or(0, str::len)
                            + 64;
                        if retained_bytes.saturating_add(estimated) > 16 * 1024 * 1024 {
                            if !snapshot
                                .coverage
                                .issues
                                .iter()
                                .any(|i| i.code == "capture_memory_limit")
                            {
                                request_issue(snapshot, batch, "capture_memory_limit");
                            }
                            for (id, region, _, _) in batch {
                                unavailable.insert((*id, region.clone()));
                            }
                            continue;
                        }
                        *retained_bytes += estimated;
                        cells.insert(key, cell.clone());
                    }
                }
            }
        }
    }
}
fn normalize_region(
    sheet: &SheetSnapshot,
    rule: &RegionRule,
    cells: &BTreeMap<(u32, u32, u32), GridCell>,
    complete: bool,
) -> RegionSnapshot {
    let blank = GridCell::default();
    let cell = |r, c| cells.get(&(sheet.sheet_id, r, c)).unwrap_or(&blank);
    let end_col = rule.range.end_col.min(sheet.column_count);
    let end_row = rule.range.end_row.min(sheet.row_count);
    let headers: Vec<_> = (rule.range.start_col..end_col)
        .map(|c| {
            let v = cell(rule.header_row, c).value();
            ObservedColumn {
                column: c,
                header: match v {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Null => String::new(),
                    other => other.to_string(),
                },
            }
        })
        .collect();
    let logical_column = |id: &str| {
        rule.columns.iter().find(|c| c.id == id).and_then(|c| {
            let found: Vec<_> = headers.iter().filter(|h| h.header == c.header).collect();
            if found.len() == 1 {
                Some(found[0].column)
            } else {
                None
            }
        })
    };
    let formula_columns: BTreeSet<_> = rule
        .formulas
        .iter()
        .filter_map(|f| logical_column(&f.column_id))
        .collect();
    let mut rows = Vec::new();
    let mut formulas = Vec::new();
    for r in rule.header_row + 1..end_row {
        let key_hash = rule
            .key_column
            .as_ref()
            .and_then(|id| logical_column(id))
            .and_then(|c| {
                let v = cell(r, c).value();
                if v.is_null() || v == serde_json::Value::String(String::new()) {
                    None
                } else {
                    Some(fingerprint(&v))
                }
            });
        let mut hashes = BTreeMap::new();
        let mut any = false;
        for c in rule.range.start_col..end_col {
            let value = cell(r, c);
            any |= !value.is_empty();
            let logical = rule
                .columns
                .iter()
                .find(|col| logical_column(&col.id) == Some(c))
                .map(|col| col.id.clone())
                .unwrap_or_else(|| format!("column_{c}"));
            hashes.insert(
                logical,
                fingerprint(&(value.value(), value.formula(), value.error())),
            );
            if formula_columns.contains(&c) {
                formulas.push(ObservedFormula {
                    row: r,
                    column: c,
                    formula: value.formula().map(str::to_owned),
                    error: value.error().map(str::to_owned),
                    value_hash: fingerprint(&value.value()),
                    key_hash: key_hash.clone(),
                });
            }
        }
        if any {
            rows.push(RowDigest {
                row: r,
                key_hash,
                fingerprint: fingerprint(&hashes),
                cell_hashes: hashes,
            });
        }
    }
    RegionSnapshot {
        region_id: rule.id.clone(),
        range: rule.range.clone(),
        complete,
        headers,
        rows,
        formulas,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::google::{HttpReply, ReadTransport};
    use reqwest::header::HeaderValue;
    use serde_json::{json, Value};
    use std::cell::RefCell;
    use std::collections::VecDeque;
    struct Mock(RefCell<VecDeque<HttpReply>>);
    impl ReadTransport for Mock {
        fn get(&self, _: &url::Url, _: &HeaderValue, _: Duration) -> Result<HttpReply, ApiError> {
            self.0.borrow_mut().pop_front().ok_or(ApiError::Network)
        }
        fn wait(&self, _: Duration) {}
    }
    fn document(rows: u32, data: Option<Value>) -> Value {
        let mut sheet = json!({"properties":{"sheetId":7,"title":"Data","index":0,"sheetType":"GRID","gridProperties":{"rowCount":rows,"columnCount":4}}});
        if let Some(data) = data {
            sheet["data"] = data;
        }
        json!({"spreadsheetId":"fixture_sheet","sheets":[sheet]})
    }
    fn raw_data() -> Value {
        json!([{"startRow":0,"startColumn":0,"rowData":[{"values":[{"effectiveValue":{"stringValue":"ID"}},{"effectiveValue":{"stringValue":"Amount"}},{"effectiveValue":{"stringValue":"Total"}}]},{"values":[{"effectiveValue":{"stringValue":"record-a"}},{"effectiveValue":{"numberValue":12}},{"userEnteredValue":{"formulaValue":"=B2*2"},"effectiveValue":{"numberValue":24}},{"effectiveValue":{"stringValue":"#REF!"}}]}]}])
    }
    fn capture(rows: u32, data: Value) -> StructuralSnapshot {
        let replies = [document(rows, None), document(rows, Some(data))]
            .into_iter()
            .map(|body| HttpReply {
                status: 200,
                body: serde_json::to_vec(&body).unwrap(),
                retry_after: None,
            })
            .collect();
        let api = GoogleApi::with_transport("synthetic-access".into(), Mock(RefCell::new(replies)))
            .unwrap();
        capture_with(
            &api,
            "fixture_subject",
            "fixture_sheet",
            super::super::fixtures::scope(),
        )
        .unwrap()
    }
    #[test]
    fn deleting_physical_data_rows_does_not_shrink_logical_window_or_mark_offline() {
        let base = capture(8, raw_data());
        let current = capture(7, raw_data());
        assert!(base.coverage.complete && current.coverage.complete);
        let baseline = BaselineVersion {
            id: "synthetic".into(),
            version: 1,
            active: true,
            approved_at: base.captured_at.clone(),
            approval_origin: "synthetic:test".into(),
            snapshot: base.clone(),
        };
        assert_eq!(
            super::super::evaluate(&baseline, Some(&base), &current, "check")
                .unwrap()
                .status,
            IntegrityStatus::Online
        );
        assert_eq!(current.sheets[0].regions[0].range.end_row, 8);
    }
    #[test]
    fn raw_values_reduce_to_hashes_and_literal_error_text_has_no_error() {
        let snapshot = capture(8, raw_data());
        let encoded = serde_json::to_string(&snapshot).unwrap();
        assert!(!encoded.contains("record-a"));
        assert!(!encoded.contains("#REF!"));
        assert_eq!(snapshot.sheets[0].regions[0].formulas[0].error, None);
        assert_eq!(
            snapshot.sheets[0].regions[0].formulas[0].formula.as_deref(),
            Some("=B2*2")
        );
    }
    #[test]
    fn malformed_cell_is_a_technical_failure_not_a_confirmed_missing_formula() {
        let base = capture(8, raw_data());
        let mut data = raw_data();
        data[0]["rowData"][1]["values"][2]["effectiveValue"] = json!({"numberValue":{"bad":true}});
        let current = capture(8, data);
        assert!(!current.coverage.complete);
        assert!(!current.sheets[0].regions[0].complete);
        let baseline = BaselineVersion {
            id: "synthetic".into(),
            version: 1,
            active: true,
            approved_at: base.captured_at.clone(),
            approval_origin: "synthetic:test".into(),
            snapshot: base,
        };
        assert_eq!(
            super::super::evaluate(&baseline, None, &current, "bad-capture")
                .unwrap()
                .status,
            IntegrityStatus::Indeterminate
        );
    }
    #[test]
    fn grid_offsets_are_applied_when_normalizing_a_region() {
        let mut scope = super::super::fixtures::scope();
        let region = &mut scope.sheets[0].regions[0];
        region.range.start_row = 4;
        region.range.end_row = 10;
        region.header_row = 4;
        region.formulas[0].baseline_row = 5;
        let mut data = raw_data();
        data[0]["startRow"] = json!(4);
        let replies = [document(20, None), document(20, Some(data))]
            .into_iter()
            .map(|body| HttpReply {
                status: 200,
                body: serde_json::to_vec(&body).unwrap(),
                retry_after: None,
            })
            .collect();
        let api =
            GoogleApi::with_transport("synthetic".into(), Mock(RefCell::new(replies))).unwrap();
        let snap = capture_with(&api, "fixture_subject", "fixture_sheet", scope).unwrap();
        assert_eq!(snap.sheets[0].regions[0].headers[0].header, "ID");
        assert_eq!(snap.sheets[0].regions[0].rows[0].row, 5);
        assert_eq!(snap.sheets[0].regions[0].formulas[0].row, 5);
    }
    #[test]
    fn overflowing_grid_offsets_are_rejected_as_incomplete_observation() {
        let mut data = raw_data();
        data[0]["startRow"] = serde_json::json!(u32::MAX);
        let snapshot = capture(8, data);
        assert!(!snapshot.coverage.complete);
        assert!(!snapshot.sheets[0].regions[0].complete);
    }
}
