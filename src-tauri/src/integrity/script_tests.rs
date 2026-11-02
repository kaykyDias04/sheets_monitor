use super::*;
use serde_json::json;
fn file(source: &str) -> ScriptFile {
    ScriptFile {
        name: "Code".into(),
        file_type: ScriptFileType::ServerJs,
        source: source.into(),
    }
}
fn analyze(source: &str) -> ScriptAnalysis {
    analyze_sources(&[file(source)], "fixture_sheet", true).unwrap()
}
fn source(tail: &str) -> String {
    format!("const ss=SpreadsheetApp.getActiveSpreadsheet(); const sheet=ss.getSheetByName('Dados'); {tail}")
}
fn snapshot() -> StructuralSnapshot {
    serde_json::from_value(json!({
  "schema_version":1,"id":"snapshot_fixture","account_subject":"account_fixture","spreadsheet_id":"fixture_sheet","capture_started_at":"2026-11-02T00:00:00Z","captured_at":"2026-11-02T00:00:01Z","scope_hash":"fixture",
  "scope":{"schema_version":1,"sheets":[{"sheet_id":7,"protect_title":true,"protect_position":false,"regions":[{"id":"data","range":{"start_row":0,"end_row":10,"start_col":0,"end_col":3},"header_row":0,"columns":[{"id":"id","header":"ID","position":0,"protect_position":true,"required":true},{"id":"amount","header":"Valor","position":1,"protect_position":true,"required":true},{"id":"total","header":"Total","position":2,"protect_position":true,"required":true}],"key_column":"id","critical":true,"allow_record_deletion":true,"formulas":[]}]}],"scripts":[],"named_range_ids":[],"snapshot_retention_days":30,"history_retention_days":90},
  "sheets":[{"sheet_id":7,"title":"Dados","index":0,"row_count":20,"column_count":8,"regions":[{"region_id":"data","range":{"start_row":0,"end_row":10,"start_col":0,"end_col":3},"complete":true,"headers":[{"column":0,"header":"ID"},{"column":1,"header":"Valor"},{"column":2,"header":"Total"}],"rows":[],"formulas":[]}]}],"named_ranges":[],"scripts":[],"coverage":{"metadata_complete":true,"complete":true,"issues":[],"atomic_across_services":false},"connectivity":"reachable"
 })).unwrap()
}
#[test]
fn static_alias_chain_preserves_sheet_range_and_read_methods() {
    let a=analyze(&source("const alias=sheet; const range=alias.getRange('A2:B4'); range.getValues(); range.getDisplayValues(); range.getFormulas(); range.getColumn();"));
    assert!(a.complete, "{:?}", a.limitations);
    assert!(a
        .dependencies
        .iter()
        .any(|d| d.kind == DependencyKind::ReadValues));
    assert!(a
        .dependencies
        .iter()
        .any(|d| d.kind == DependencyKind::ReadDisplayValues));
    assert!(a
        .dependencies
        .iter()
        .any(|d| d.kind == DependencyKind::ReadFormulas));
    assert!(a
        .dependencies
        .iter()
        .all(|d| d.coverage == DependencyCoverage::Static));
    assert!(validate_dependencies(&a, &snapshot(), &snapshot(), true).is_empty());
}
#[test]
fn literal_open_by_id_is_proven_but_external_id_is_unresolved() {
    let a=analyze_sources(&[file("const s=SpreadsheetApp.openById('fixture_sheet').getSheetByName('Dados'); s.getRange(2,1,3,2).getValues();")],"fixture_sheet",false).unwrap();
    assert!(a.complete);
    let a=analyze_sources(&[file("SpreadsheetApp.openById('another_sheet').getSheetByName('Dados').getRange('A1').getValues();")],"fixture_sheet",false).unwrap();
    assert!(!a.complete);
    assert!(a
        .dependencies
        .iter()
        .any(|d| d.detail_code == "external_or_dynamic_spreadsheet"));
}
#[test]
fn standalone_active_spreadsheet_is_not_assumed_to_be_monitored() {
    let a = analyze_sources(
        &[file(&source("sheet.getRange('A1');"))],
        "fixture_sheet",
        false,
    )
    .unwrap();
    assert!(!a.complete);
    assert!(!a
        .dependencies
        .iter()
        .any(|d| d.coverage == DependencyCoverage::Static && d.sheet_name.is_some()));
}
#[test]
fn numeric_ranges_convert_one_based_to_zero_based() {
    let a = analyze(&source("sheet.getRange(2,1,3,2).getValues();"));
    let range = a
        .dependencies
        .iter()
        .find(|d| d.kind == DependencyKind::RangeNumeric)
        .unwrap()
        .range
        .as_ref()
        .unwrap();
    assert_eq!(
        *range,
        GridRect {
            start_row: 1,
            end_row: 4,
            start_col: 0,
            end_col: 2
        }
    );
    assert!(a.complete);
}
#[test]
fn invalid_numeric_bounds_are_confirmed_for_proven_receiver() {
    let a = analyze(&source("sheet.getRange(0,1);"));
    assert!(validate_dependencies(&a, &snapshot(), &snapshot(), true)
        .iter()
        .any(|f| f.code == "script_range_invalid" && f.outcome == FindingOutcome::Violation));
}
#[test]
fn valid_and_invalid_sheet_dependencies_are_distinguished() {
    let a = analyze(&source("sheet.getRange('A2:B4');"));
    let old = snapshot();
    let mut now = old.clone();
    now.sheets.clear();
    let findings = validate_dependencies(&a, &old, &now, true);
    assert!(findings
        .iter()
        .any(|f| f.code == "script_sheet_missing" && f.outcome == FindingOutcome::Violation));
    now.coverage.metadata_complete = false;
    assert!(validate_dependencies(&a, &old, &now, true)
        .iter()
        .all(|f| f.outcome == FindingOutcome::Inconclusive));
}
#[test]
fn moved_logical_fields_break_fixed_script_range_even_inside_grid() {
    let a = analyze(&source("sheet.getRange('A2:B4').getValues();"));
    let old = snapshot();
    let mut now = old.clone();
    now.sheets[0].regions[0].headers.swap(0, 1);
    now.sheets[0].regions[0].headers[0].column = 0;
    now.sheets[0].regions[0].headers[1].column = 1;
    let findings = validate_dependencies(&a, &old, &now, true);
    assert!(findings.iter().any(
        |f| f.code == "script_logical_column_changed" && f.outcome == FindingOutcome::Violation
    ));
}
#[test]
fn incomplete_region_never_confirms_missing_logical_column() {
    let a = analyze(&source("sheet.getRange('A2:B4');"));
    let old = snapshot();
    let mut now = old.clone();
    now.sheets[0].regions[0].complete = false;
    now.sheets[0].regions[0].headers.clear();
    let findings = validate_dependencies(&a, &old, &now, true);
    assert!(!findings.is_empty());
    assert!(findings
        .iter()
        .all(|f| f.outcome == FindingOutcome::Inconclusive));
}
#[test]
fn ranges_outside_grid_are_invalid_but_outside_scope_are_partial() {
    let a = analyze(&source("sheet.getRange('A2:B4');"));
    let old = snapshot();
    let mut now = old.clone();
    now.sheets[0].column_count = 1;
    assert!(validate_dependencies(&a, &old, &now, true)
        .iter()
        .any(|f| f.code == "script_range_outside_grid" && f.outcome == FindingOutcome::Violation));
    let a = analyze(&source("sheet.getRange('D2:F4');"));
    assert!(validate_dependencies(&a, &old, &old, true)
        .iter()
        .any(|f| f.code == "script_range_coverage_partial"
            && f.outcome == FindingOutcome::Inconclusive));
}
#[test]
fn dynamic_arguments_are_unproven_not_confirmed_damage() {
    let a = analyze(&source(
        "sheet.getRange(start,1,3,2).getValues(); ss.getSheetByName(name);",
    ));
    assert!(!a.complete);
    let findings = validate_dependencies(&a, &snapshot(), &snapshot(), true);
    assert!(!findings.is_empty());
    assert!(findings
        .iter()
        .all(|f| f.outcome == FindingOutcome::Inconclusive && f.severity == Severity::Critical));
}
#[test]
fn unknown_objects_do_not_become_proven_sheets() {
    let a = analyze("custom.getSheetByName('Missing').getRange('Z999').getValues();");
    assert!(!a.complete);
    assert!(!a
        .dependencies
        .iter()
        .any(|d| d.coverage == DependencyCoverage::Static));
    assert!(validate_dependencies(&a, &snapshot(), &snapshot(), true)
        .iter()
        .all(|f| f.outcome == FindingOutcome::Inconclusive));
}
#[test]
fn function_parameters_and_lexical_shadowing_do_not_reuse_outer_binding() {
    let a=analyze(&source("function test(sheet){sheet.getRange('Z999');} {const sheet=custom; sheet.getRange('Z999');} sheet.getRange('A2:B4');"));
    assert!(!a.complete);
    assert_eq!(
        a.dependencies
            .iter()
            .filter(
                |d| d.kind == DependencyKind::RangeA1 && d.coverage == DependencyCoverage::Static
            )
            .count(),
        1
    );
    assert!(a
        .dependencies
        .iter()
        .any(|d| d.coverage == DependencyCoverage::Partial));
}
#[test]
fn reassigned_aliases_and_shadowed_spreadsheet_app_are_not_proven() {
    for js in [source("sheet=custom; sheet.getRange('Z999');"),"function test(SpreadsheetApp){const s=SpreadsheetApp.getActiveSpreadsheet().getSheetByName('Missing');s.getRange('A1');}".into(),"function test(){if(false){var SpreadsheetApp=custom;} SpreadsheetApp.getActiveSpreadsheet().getSheetByName('Missing').getRange('A1');}".into(),"SpreadsheetApp=custom; SpreadsheetApp.getActiveSpreadsheet().getSheetByName('Missing').getRange('A1');".into()]{let a=analyze(&js);assert!(!a.complete);assert!(!a.dependencies.iter().any(|d|d.kind==DependencyKind::RangeA1&&d.coverage==DependencyCoverage::Static));}
}
#[test]
fn last_row_and_column_queries_alone_are_harmless() {
    let a = analyze(&source("sheet.getLastRow(); sheet.getLastColumn();"));
    assert!(a.complete);
    assert!(validate_dependencies(&a, &snapshot(), &snapshot(), true).is_empty());
}
#[test]
fn comments_and_string_contents_do_not_create_dependencies() {
    let a=analyze("// ss.getSheetByName('Missing').getRange('Z999')\nconst message=\"SpreadsheetApp.getActiveSpreadsheet().getRange('Z999')\";const pattern=/getSheetByName/;");
    assert!(a.complete);
    assert!(a.dependencies.is_empty());
}
#[test]
fn parse_errors_and_html_have_explicit_partial_coverage() {
    let a = analyze("function bad( { ss.getSheetByName('Missing');");
    assert!(!a.complete);
    assert!(a.limitations.contains(&"javascript_parse_error".into()));
    let a = analyze_sources(
        &[
            ScriptFile {
                name: "Index".into(),
                file_type: ScriptFileType::Html,
                source: "<script>danger()</script>".into(),
            },
            ScriptFile {
                name: "appsscript".into(),
                file_type: ScriptFileType::Json,
                source: "{}".into(),
            },
        ],
        "fixture_sheet",
        true,
    )
    .unwrap();
    assert!(!a.complete);
    assert!(a.dependencies.is_empty());
    assert!(a
        .limitations
        .contains(&"html_inline_javascript_unanalyzed".into()));
}
#[test]
fn unsupported_computed_and_detached_methods_are_partial() {
    for tail in [
        "const method=sheet.getRange; method('A1');",
        "sheet['getRange']('A1');",
        "sheet[method]('A1');",
        "sheet.getDataRange();",
    ] {
        let a = analyze(&source(tail));
        assert!(!a.complete, "{tail}");
    }
}
#[test]
fn eval_and_method_mutation_invalidate_static_proofs() {
    for tail in [
        "eval(code); sheet.getRange('A1');",
        "sheet.getRange=other; sheet.getRange('A1');",
        "new Function(code); sheet.getRange('A1');",
    ] {
        let a = analyze(&source(tail));
        assert!(!a.complete);
        assert!(a
            .dependencies
            .iter()
            .all(|d| d.coverage != DependencyCoverage::Static));
    }
}
#[test]
fn parser_limits_reject_unbounded_source() {
    let large = " ".repeat(MAX_FILE_BYTES + 1);
    assert_eq!(
        analyze_sources(&[file(&large)], "fixture_sheet", true),
        Err(EngineError::Limit)
    );
    let files = (0..MAX_FILES + 1).map(|_| file("")).collect::<Vec<_>>();
    assert_eq!(
        analyze_sources(&files, "fixture_sheet", true),
        Err(EngineError::Limit)
    );
}
#[test]
fn escaped_sheet_names_and_absolute_a1_are_supported_without_eval() {
    assert_eq!(
        parse_a1("'Dado''s'!$A$2:$B$4").unwrap(),
        (
            Some("Dado's".into()),
            GridRect {
                start_row: 1,
                end_row: 4,
                start_col: 0,
                end_col: 2
            }
        )
    );
    assert!(parse_a1("Bad Sheet!A1").is_none());
    assert!(parse_a1("R[1]C[2]").is_none());
    assert_eq!(decode_string(r#""Dado's""#).unwrap(), "Dado's");
}
#[test]
fn dependency_incident_key_survives_different_wrong_column_positions() {
    let analysis = analyze(&source("sheet.getRange('A2:C4').getValues();"));
    let baseline = snapshot();
    let mut b = baseline.clone();
    b.sheets[0].regions[0].headers[0].header = "Other".into();
    b.sheets[0].regions[0].headers[1].header = "ID".into();
    let mut c = baseline.clone();
    c.sheets[0].regions[0].headers[0].header = "Other".into();
    c.sheets[0].regions[0].headers[2].header = "ID".into();
    let field = |observed: &StructuralSnapshot| {
        validate_dependencies(&analysis, &baseline, observed, true)
            .into_iter()
            .find(|f| {
                f.code == "script_logical_column_changed" && f.expected.as_deref() == Some("id@0")
            })
            .unwrap()
    };
    let b = field(&b);
    let c = field(&c);
    assert_eq!(b.key, c.key);
    assert_ne!(b.found, c.found);
    assert_ne!(fingerprint(&b), fingerprint(&c));
}
#[test]
fn grid_bounds_incident_identity_does_not_depend_on_current_grid_size() {
    let analysis = analyze(&source("sheet.getRange('A2:B4');"));
    let baseline = snapshot();
    let mut b = baseline.clone();
    b.sheets[0].row_count = 2;
    let mut c = baseline.clone();
    c.sheets[0].row_count = 3;
    let problem = |observed: &StructuralSnapshot| {
        validate_dependencies(&analysis, &baseline, observed, true)
            .into_iter()
            .find(|f| f.code == "script_range_outside_grid")
            .unwrap()
    };
    let b = problem(&b);
    let c = problem(&c);
    assert_eq!(b.key, c.key);
    assert_ne!(b.expected, c.expected);
    assert_ne!(fingerprint(&b), fingerprint(&c));
}
#[test]
fn partial_range_retains_confirmed_header_violation_and_coverage_limitation() {
    let analysis = analyze(&source("sheet.getRange('A2:B15').getValues();"));
    let baseline = snapshot();
    let mut current = baseline.clone();
    current.sheets[0].regions[0].headers[0].header = "Other".into();
    current.sheets[0].regions[0].headers[1].header = "ID".into();
    let findings = validate_dependencies(&analysis, &baseline, &current, true);
    assert!(findings.iter().any(
        |f| f.code == "script_logical_column_changed" && f.outcome == FindingOutcome::Violation
    ));
    assert!(findings
        .iter()
        .any(|f| f.code == "script_range_coverage_partial"
            && f.outcome == FindingOutcome::Inconclusive));
    let analysis = analyze(&source("sheet.getRange('A12:B15');"));
    let findings = validate_dependencies(&analysis, &baseline, &current, true);
    assert!(findings
        .iter()
        .all(|f| f.outcome == FindingOutcome::Inconclusive));
}
