use super::*;
use super::{
    compare::{evaluate, validate_baseline_snapshot},
    fixtures::*,
};

fn report(current: &StructuralSnapshot) -> IntegrityReport {
    evaluate(&baseline(), Some(&snapshot()), current, "check_fixture").unwrap()
}
fn has(report: &IntegrityReport, code: &str) -> bool {
    report.findings.iter().any(|f| f.code == code)
}
fn update_row(snapshot: &mut StructuralSnapshot, index: usize, key: &str, amount: i64) {
    let number = snapshot.sheets[0].regions[0].rows[index].row;
    snapshot.sheets[0].regions[0].rows[index] = row(number, key, amount);
}

#[test]
fn unchanged_fixture_is_online() {
    let report = report(&snapshot());
    assert_eq!(report.status, IntegrityStatus::Online);
    assert!(report.differences.is_empty());
    assert_eq!(
        report.previous_snapshot_id.as_deref(),
        Some("snapshot_fixture_1")
    );
}
#[test]
fn inserted_keyed_record_is_informative_online() {
    let mut current = snapshot();
    current.sheets[0].regions[0]
        .rows
        .push(row(3, "record_c", 30));
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Online);
    assert!(r
        .differences
        .iter()
        .any(|d| d.kind == "record_added" && !d.ambiguous));
}
#[test]
fn deleted_noncritical_data_record_stays_online() {
    let mut current = snapshot();
    current.sheets[0].regions[0].rows.pop();
    current.sheets[0].regions[0].formulas.pop();
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Online);
    assert!(r.differences.iter().any(|d| d.kind == "record_removed"));
}
#[test]
fn deleted_key_with_per_record_formula_respects_deletion_policy() {
    let mut current = snapshot();
    current.sheets[0].regions[0].rows.remove(0);
    current.sheets[0].regions[0].formulas.remove(0);
    assert_eq!(report(&current).status, IntegrityStatus::Online);
}
#[test]
fn deleted_formula_record_disallowed_is_offline_without_fabricated_missing_formula() {
    let mut b = baseline();
    b.snapshot.scope.sheets[0].regions[0].allow_record_deletion = false;
    rehash(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    current.sheets[0].regions[0].rows.remove(0);
    current.sheets[0].regions[0].formulas.remove(0);
    let r = evaluate(&b, None, &current, "check").unwrap();
    assert_eq!(r.status, IntegrityStatus::Offline);
    assert!(has(&r, "protected_record_removed"));
    assert!(has(&r, "formula_anchor_ambiguous"));
}
#[test]
fn deletion_rule_protects_records_without_formulas_and_persists_until_restored() {
    let mut b = baseline();
    b.snapshot.scope.sheets[0].regions[0].formulas.clear();
    b.snapshot.scope.sheets[0].regions[0].allow_record_deletion = false;
    b.snapshot.sheets[0].regions[0].formulas.clear();
    rehash(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    current.sheets[0].regions[0].rows.pop();
    let r = evaluate(&b, None, &current, "deleted").unwrap();
    assert_eq!(r.status, IntegrityStatus::Offline);
    assert!(r
        .findings
        .iter()
        .any(|f| f.code == "protected_record_removed" && f.rule_id == "records"));
    let repeated = evaluate(&b, Some(&current), &current, "repeated").unwrap();
    assert!(has(&repeated, "protected_record_removed"));
    assert_eq!(
        evaluate(&b, Some(&current), &b.snapshot, "restored")
            .unwrap()
            .status,
        IntegrityStatus::Online
    );
    current.sheets[0].regions[0].complete = false;
    assert!(!has(
        &evaluate(&b, None, &current, "partial").unwrap(),
        "protected_record_removed"
    ));
}
#[test]
fn common_value_edit_generates_cell_difference_without_offline() {
    let mut current = snapshot();
    update_row(&mut current, 1, "record_b", 42);
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Online);
    assert!(r.differences.iter().any(|d| d.kind == "value_changed"));
    assert!(r
        .differences
        .iter()
        .all(|d| d.before.as_ref().is_none_or(|v| !v.contains("record_b"))));
}
#[test]
fn protected_column_reorder_is_offline() {
    let mut current = snapshot();
    current.sheets[0].regions[0].headers[0].column = 1;
    current.sheets[0].regions[0].headers[1].column = 0;
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Offline);
    assert!(has(&r, "protected_column_reordered"));
}
#[test]
fn protected_column_deletion_is_offline() {
    let mut current = snapshot();
    current.sheets[0].column_count = 3;
    current.sheets[0].regions[0].headers.remove(1);
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Offline);
    assert!(has(&r, "protected_column_missing"));
}
#[test]
fn critical_header_rename_is_offline() {
    let mut current = snapshot();
    current.sheets[0].regions[0].headers[1].header = "Unexpected".into();
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Offline);
    assert!(has(&r, "protected_header_changed_or_column_missing"));
}
#[test]
fn duplicate_header_is_uncertain_not_column_deletion() {
    let mut current = snapshot();
    current.sheets[0].regions[0].headers[3].header = "Amount".into();
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Indeterminate);
    assert!(has(&r, "column_identity_ambiguous"));
    assert!(!has(&r, "protected_column_missing"));
}
#[test]
fn deleted_monitored_sheet_is_offline_only_with_complete_metadata() {
    let mut current = snapshot();
    current.sheets.clear();
    assert_eq!(report(&current).status, IntegrityStatus::Offline);
    current.coverage.metadata_complete = false;
    current.coverage.complete = false;
    current.connectivity = Connectivity::NetworkFailed;
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Indeterminate);
    assert!(!has(&r, "monitored_sheet_missing"));
}
#[test]
fn protected_sheet_rename_is_offline() {
    let mut current = snapshot();
    current.sheets[0].title = "Other".into();
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Offline);
    assert!(has(&r, "protected_sheet_renamed"));
}
#[test]
fn protected_sheet_reordering_is_configurable() {
    let mut b = baseline();
    b.snapshot.scope.sheets[0].protect_position = true;
    rehash(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    current.sheets[0].index = 2;
    assert_eq!(
        evaluate(&b, None, &current, "check").unwrap().status,
        IntegrityStatus::Offline
    );
    assert_eq!(
        report(&{
            let mut s = snapshot();
            s.sheets[0].index = 2;
            s
        })
        .status,
        IntegrityStatus::Online
    );
}
#[test]
fn protected_formula_removed_or_replaced_is_offline() {
    for value in ["empty", "constant"] {
        let mut current = snapshot();
        current.sheets[0].regions[0].formulas[0].formula = None;
        current.sheets[0].regions[0].formulas[0].value_hash = fingerprint(&value);
        let r = report(&current);
        assert_eq!(r.status, IntegrityStatus::Offline);
        assert!(has(&r, "protected_formula_removed_or_replaced"));
    }
}
#[test]
fn effective_ref_error_is_confirmed_while_loading_is_uncertain() {
    for error in [
        "REF",
        "NAME",
        "DIVIDE_BY_ZERO",
        "VALUE",
        "NUM",
        "N_A",
        "ERROR",
    ] {
        let mut current = snapshot();
        current.sheets[0].regions[0].formulas[0].error = Some(error.into());
        assert_eq!(report(&current).status, IntegrityStatus::Offline);
    }
    for error in ["LOADING", "UNKNOWN"] {
        let mut current = snapshot();
        current.sheets[0].regions[0].formulas[0].error = Some(error.into());
        assert_eq!(report(&current).status, IntegrityStatus::Indeterminate);
    }
}
#[test]
fn literal_ref_text_outside_protected_formula_is_not_an_error() {
    let mut current = snapshot();
    current.sheets[0].regions[0].rows[1]
        .cell_hashes
        .insert("column:3".into(), fingerprint(&"#REF!"));
    assert_eq!(report(&current).status, IntegrityStatus::Online);
}
#[test]
fn formula_shifted_by_inserted_row_with_key_and_target_identity_is_online() {
    let mut current = snapshot();
    for r in &mut current.sheets[0].regions[0].rows {
        r.row += 1;
    }
    for f in &mut current.sheets[0].regions[0].formulas {
        f.row += 1;
        f.formula = Some(format!("=B{}*2", f.row + 1));
    }
    current.sheets[0].regions[0]
        .rows
        .insert(0, row(1, "record_new", 5));
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Online);
    assert!(r.differences.iter().any(|d| d.kind == "record_added"));
}
#[test]
fn formula_shifted_by_deleted_row_is_online() {
    let mut b = baseline();
    for r in &mut b.snapshot.sheets[0].regions[0].rows {
        r.row += 1;
    }
    for f in &mut b.snapshot.sheets[0].regions[0].formulas {
        f.row += 1;
        f.formula = Some(format!("=B{}*2", f.row + 1));
    }
    b.snapshot.scope.sheets[0].regions[0].formulas[0].baseline_row = 2;
    rehash(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    for r in &mut current.sheets[0].regions[0].rows {
        r.row -= 1;
    }
    for f in &mut current.sheets[0].regions[0].formulas {
        f.row -= 1;
        f.formula = Some(format!("=B{}*2", f.row + 1));
    }
    assert_eq!(
        evaluate(&b, None, &current, "check").unwrap().status,
        IntegrityStatus::Online
    );
}
#[test]
fn formula_without_unique_key_does_not_claim_valid_shift() {
    let mut current = snapshot();
    current.sheets[0].regions[0].rows[1].key_hash =
        current.sheets[0].regions[0].rows[0].key_hash.clone();
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Indeterminate);
    assert!(has(&r, "formula_anchor_ambiguous"));
}
#[test]
fn changed_formula_constant_proves_violation() {
    let mut current = snapshot();
    current.sheets[0].regions[0].formulas[0].formula = Some("=B2*3".into());
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Offline);
    assert!(has(&r, "protected_formula_changed"));
}
#[test]
fn changed_reference_without_mapping_is_inconclusive() {
    let mut current = snapshot();
    current.sheets[0].regions[0].formulas[0].formula = Some("=B7*2".into());
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Indeterminate);
    assert!(has(&r, "formula_equivalence_unproven"));
}
#[test]
fn dynamic_formula_is_inconclusive() {
    let mut current = snapshot();
    current.sheets[0].regions[0].formulas[0].formula = Some("=INDIRECT(\"B2\")*2".into());
    assert_eq!(report(&current).status, IntegrityStatus::Indeterminate);
}
#[test]
fn exact_mode_rejects_changed_constant_but_allows_proven_shift() {
    let mut b = baseline();
    b.snapshot.scope.sheets[0].regions[0].formulas[0].mode = FormulaMode::Exact;
    rehash(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    for row in &mut current.sheets[0].regions[0].rows {
        row.row += 1;
    }
    for f in &mut current.sheets[0].regions[0].formulas {
        f.row += 1;
        f.formula = Some(format!("=B{}*2", f.row + 1));
    }
    assert_eq!(
        evaluate(&b, None, &current, "check").unwrap().status,
        IntegrityStatus::Online
    );
}
#[test]
fn missing_formula_observation_is_uncertain_not_removed() {
    let mut current = snapshot();
    current.sheets[0].regions[0].formulas.clear();
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Indeterminate);
    assert!(!has(&r, "protected_formula_removed_or_replaced"));
}
#[test]
fn truncated_region_is_uncertain_without_content_differences() {
    let mut current = snapshot();
    current.sheets[0].regions[0].complete = false;
    current.sheets[0].regions[0].headers.clear();
    current.sheets[0].regions[0].rows.clear();
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Indeterminate);
    assert!(r.differences.is_empty());
    assert!(!has(&r, "protected_column_missing"));
}
#[test]
fn network_and_auth_failures_are_indeterminate_not_offline() {
    for connectivity in [
        Connectivity::NetworkFailed,
        Connectivity::AuthenticationFailed,
    ] {
        let mut current = snapshot();
        current.sheets.clear();
        current.coverage.complete = false;
        current.coverage.metadata_complete = false;
        current.connectivity = connectivity;
        let r = report(&current);
        assert_eq!(r.status, IntegrityStatus::Indeterminate);
        assert_eq!(r.connectivity, connectivity);
    }
}
#[test]
fn confirmed_violation_precedes_uncertainty_without_hiding_it() {
    let mut current = snapshot();
    current.sheets[0].title = "Renamed".into();
    current.sheets[0].regions[0].complete = false;
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Offline);
    assert!(r
        .findings
        .iter()
        .any(|f| f.outcome == FindingOutcome::Inconclusive));
}
#[test]
fn aggregation_ignores_unmonitored_tabs() {
    let mut current = snapshot();
    let mut extra = current.sheets[0].clone();
    extra.sheet_id = 99;
    extra.title = "Unmonitored".into();
    extra.regions.clear();
    current.sheets.push(extra);
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Online);
    assert_eq!(r.sheets.len(), 1);
}
#[test]
fn aggregation_classifies_each_monitored_tab() {
    let mut b = baseline();
    let mut second = b.snapshot.sheets[0].clone();
    second.sheet_id = 8;
    second.title = "Other".into();
    second.index = 1;
    b.snapshot.sheets.push(second);
    let mut rule = b.snapshot.scope.sheets[0].clone();
    rule.sheet_id = 8;
    b.snapshot.scope.sheets.push(rule);
    rehash(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    current.sheets[1].title = "Wrong".into();
    let r = evaluate(&b, None, &current, "check").unwrap();
    assert_eq!(r.status, IntegrityStatus::Offline);
    assert_eq!(r.sheets[0].status, IntegrityStatus::Online);
    assert_eq!(r.sheets[1].status, IntegrityStatus::Offline);
}
#[test]
fn unkeyed_edits_are_ambiguous_content_events() {
    let mut b = baseline();
    b.snapshot.scope.sheets[0].regions[0].formulas.clear();
    b.snapshot.scope.sheets[0].regions[0].key_column = None;
    for row in &mut b.snapshot.sheets[0].regions[0].rows {
        row.key_hash = None;
    }
    rehash(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    current.sheets[0].regions[0].rows[1].fingerprint = fingerprint(&"different");
    let r = evaluate(&b, Some(&b.snapshot), &current, "check").unwrap();
    assert_eq!(r.status, IntegrityStatus::Online);
    assert!(r
        .differences
        .iter()
        .any(|d| d.kind == "content_changed_unspecified" && d.ambiguous));
    assert!(!r
        .differences
        .iter()
        .any(|d| d.kind == "record_added" || d.kind == "record_removed"));
}
#[test]
fn duplicate_keys_fall_back_to_ambiguous_content_diff() {
    let mut b = baseline();
    b.snapshot.scope.sheets[0].regions[0].formulas.clear();
    rehash(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    current.sheets[0].regions[0].rows[1].key_hash =
        current.sheets[0].regions[0].rows[0].key_hash.clone();
    current.sheets[0].regions[0].rows[1].fingerprint = fingerprint(&"changed");
    let r = evaluate(&b, Some(&b.snapshot), &current, "check").unwrap();
    assert_eq!(r.status, IntegrityStatus::Online);
    assert!(r.differences.iter().all(|d| d.ambiguous));
}
#[test]
fn content_uses_previous_observation_not_frozen_baseline() {
    let b = baseline();
    let mut previous = b.snapshot.clone();
    update_row(&mut previous, 1, "record_b", 25);
    let current = previous.clone();
    let r = evaluate(&b, Some(&previous), &current, "check").unwrap();
    assert!(r.differences.is_empty());
    assert_eq!(r.status, IntegrityStatus::Online);
}
#[test]
fn first_check_uses_approved_snapshot_for_content_reference() {
    let b = baseline();
    let mut current = b.snapshot.clone();
    update_row(&mut current, 1, "record_b", 25);
    let r = evaluate(&b, None, &current, "check").unwrap();
    assert!(r.differences.iter().any(|d| d.kind == "value_changed"));
    assert_eq!(r.previous_snapshot_id, None);
}
#[test]
fn changed_scope_or_account_cannot_reuse_baseline() {
    let mut current = snapshot();
    current.scope.snapshot_retention_days = 8;
    rehash(&mut current);
    assert_eq!(
        evaluate(&baseline(), None, &current, "check").unwrap_err(),
        EngineError::BaselineConflict
    );
    let mut current = snapshot();
    current.account_subject = "other_account".into();
    assert_eq!(
        evaluate(&baseline(), None, &current, "check").unwrap_err(),
        EngineError::Forbidden
    );
}
#[test]
fn invalid_baseline_formula_or_capture_cannot_be_approved() {
    assert!(validate_baseline_snapshot(&snapshot()).is_ok());
    for case in 0..4 {
        let mut s = snapshot();
        match case {
            0 => s.coverage.complete = false,
            1 => s.sheets[0].regions[0].formulas[0].formula = None,
            2 => s.sheets[0].regions[0].formulas[0].error = Some("REF".into()),
            _ => s.sheets[0].regions[0].headers[3].header = "Amount".into(),
        };
        assert!(validate_baseline_snapshot(&s).is_err());
    }
}
#[test]
fn script_code_change_is_informative_without_false_offline() {
    let mut b = baseline();
    attach_script(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    current.scripts[0].source_hash = fingerprint(&"source_b");
    current.scripts[0].files[0].hash = fingerprint(&"source_b");
    let r = evaluate(&b, Some(&b.snapshot), &current, "check").unwrap();
    assert_eq!(r.status, IntegrityStatus::Online);
    assert!(r
        .differences
        .iter()
        .any(|d| d.kind == "script_file_changed"));
}
#[test]
fn script_file_addition_and_removal_each_have_events() {
    let mut b = baseline();
    attach_script(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    current.scripts[0].files[0].name = "New".into();
    let r = evaluate(&b, Some(&b.snapshot), &current, "check").unwrap();
    assert!(r.differences.iter().any(|d| d.kind == "script_file_added"));
    assert!(r
        .differences
        .iter()
        .any(|d| d.kind == "script_file_removed"));
}
#[test]
fn missing_critical_script_is_uncertain_not_proven_deleted() {
    let mut b = baseline();
    attach_script(&mut b.snapshot);
    b.snapshot.scope.scripts[0].critical = true;
    rehash(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    current.scripts.clear();
    let r = evaluate(&b, None, &current, "check").unwrap();
    assert_eq!(r.status, IntegrityStatus::Indeterminate);
    assert!(has(&r, "script_not_observed"));
}
#[test]
fn named_range_identity_geometry_and_rename_are_protected() {
    let mut b = baseline();
    b.snapshot.scope.named_range_ids = vec!["range_id".into()];
    b.snapshot.named_ranges = vec![NamedRangeSnapshot {
        id: "range_id".into(),
        name: "Required".into(),
        sheet_id: 7,
        range: GridRect {
            start_row: 0,
            end_row: 3,
            start_col: 0,
            end_col: 2,
        },
    }];
    rehash(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    current.named_ranges[0].range.end_col = 3;
    let r = evaluate(&b, None, &current, "check").unwrap();
    assert_eq!(r.status, IntegrityStatus::Offline);
    assert!(has(&r, "named_range_changed"));
}
#[test]
fn deterministic_report_and_fingerprints_ignore_check_clock_for_difference_identity() {
    let mut current = snapshot();
    advance(&mut current);
    update_row(&mut current, 1, "record_b", 80);
    let a = evaluate(&baseline(), Some(&snapshot()), &current, "check_a").unwrap();
    let b = evaluate(&baseline(), Some(&snapshot()), &current, "check_b").unwrap();
    assert_eq!(a.findings, b.findings);
    assert_eq!(a.differences, b.differences);
    assert_ne!(a.check_id, b.check_id);
}

#[test]
fn keyed_record_reordering_generates_one_order_event() {
    let mut current = snapshot();
    current.sheets[0].regions[0].rows.swap(0, 1);
    let r = report(&current);
    assert_eq!(r.status, IntegrityStatus::Online);
    assert_eq!(
        r.differences
            .iter()
            .filter(|d| d.kind == "record_order_changed")
            .count(),
        1
    );
}
#[test]
fn algebraic_rewrite_not_in_the_proven_subset_is_uncertain() {
    let mut b = baseline();
    b.snapshot.sheets[0].regions[0].formulas[0].formula = Some("=B2+B2".into());
    let mut current = b.snapshot.clone();
    current.sheets[0].regions[0].formulas[0].formula = Some("=B2*2".into());
    let r = evaluate(&b, None, &current, "check").unwrap();
    assert_eq!(r.status, IntegrityStatus::Indeterminate);
    assert!(has(&r, "formula_equivalence_unproven"));
}
#[test]
fn bounded_affine_commutation_is_a_deterministic_equivalence() {
    let mut current = snapshot();
    current.sheets[0].regions[0].formulas[0].formula = Some("=2*B2".into());
    assert_eq!(report(&current).status, IntegrityStatus::Online);
}
#[test]
fn absolute_reference_flags_are_not_erased_by_affine_normalization() {
    let mut b = baseline();
    b.snapshot.sheets[0].regions[0].formulas[0].formula = Some("=$B$2*2".into());
    let current = snapshot();
    assert_eq!(
        evaluate(&b, None, &current, "check").unwrap().status,
        IntegrityStatus::Indeterminate
    );
}
#[test]
fn exact_mode_preserves_nonreference_case_and_whitespace_policy() {
    let mut b = baseline();
    b.snapshot.scope.sheets[0].regions[0].formulas[0].mode = FormulaMode::Exact;
    rehash(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    current.sheets[0].regions[0].formulas[0].formula = Some("=B2 *2".into());
    assert_eq!(
        evaluate(&b, None, &current, "check").unwrap().status,
        IntegrityStatus::Indeterminate
    );
}

#[test]
fn critical_script_baseline_cannot_approve_a_proven_invalid_dependency() {
    let mut s = fixtures::snapshot();
    let script = crate::google::ScriptFile {
        name: "Code".into(),
        file_type: crate::google::ScriptFileType::ServerJs,
        source: "const ss=SpreadsheetApp.getActiveSpreadsheet(); ss.getSheetByName('Missing');"
            .into(),
    };
    s.scope.scripts.push(ScriptRule {
        script_id: "fixture_script".into(),
        critical: true,
    });
    fixtures::rehash(&mut s);
    let analysis = super::scripts::analyze_sources(&[script], &s.spreadsheet_id, true).unwrap();
    assert!(analysis.complete);
    s.scripts.push(ScriptSnapshot {
        script_id: "fixture_script".into(),
        source_hash: fingerprint(&"source"),
        files: vec![],
        analysis,
    });
    assert_eq!(
        validate_baseline_snapshot(&s),
        Err(EngineError::InvalidSnapshot)
    );
}

#[test]
fn bare_reference_and_numeric_coercion_are_not_proven_equivalent() {
    let mut b = baseline();
    b.snapshot.sheets[0].regions[0].formulas[0].formula = Some("=B2".into());
    let mut current = b.snapshot.clone();
    current.sheets[0].regions[0].formulas[0].formula = Some("=B2*1".into());
    assert_eq!(
        evaluate(&b, None, &current, "coercion").unwrap().status,
        IntegrityStatus::Indeterminate
    );
}
#[test]
fn known_null_error_in_a_protected_formula_is_a_confirmed_violation() {
    let mut current = snapshot();
    current.sheets[0].regions[0].formulas[0].error = Some("NULL_VALUE".into());
    assert_eq!(report(&current).status, IntegrityStatus::Offline);
}

#[test]
fn failed_optional_script_does_not_override_verified_critical_sheets() {
    for critical in [false, true] {
        let mut b = baseline();
        attach_script(&mut b.snapshot);
        b.snapshot.scope.scripts[0].critical = critical;
        rehash(&mut b.snapshot);
        let mut current = b.snapshot.clone();
        current.scripts.clear();
        current.coverage.complete = false;
        current.connectivity = Connectivity::Partial;
        current.coverage.issues.push(CoverageIssue {
            sheet_id: None,
            rule_id: Some("fixture_script".into()),
            critical,
            code: "forbidden".into(),
        });
        let report = evaluate(&b, None, &current, "optional").unwrap();
        assert_eq!(
            report.status,
            if critical {
                IntegrityStatus::Indeterminate
            } else {
                IntegrityStatus::Online
            }
        );
        assert!(report.findings.iter().any(|f| f.code == "forbidden"));
    }
}

#[test]
fn deletion_rule_without_stable_keys_does_not_invent_a_removed_identity() {
    let mut b = baseline();
    let rule = &mut b.snapshot.scope.sheets[0].regions[0];
    rule.allow_record_deletion = false;
    rule.key_column = None;
    rule.formulas.clear();
    b.snapshot.sheets[0].regions[0].formulas.clear();
    for row in &mut b.snapshot.sheets[0].regions[0].rows {
        row.key_hash = None;
    }
    rehash(&mut b.snapshot);
    let mut current = b.snapshot.clone();
    let changed = &mut current.sheets[0].regions[0].rows[0];
    changed
        .cell_hashes
        .insert("amount".into(), fingerprint(&999));
    changed.fingerprint = fingerprint(&changed.cell_hashes);
    let report = evaluate(&b, None, &current, "ambiguous").unwrap();
    assert_eq!(report.status, IntegrityStatus::Indeterminate);
    assert!(has(&report, "record_deletion_unproven"));
    assert!(!has(&report, "protected_record_removed"));
    current.sheets[0].regions[0].rows.pop();
    let report = evaluate(&b, None, &current, "fewer_records").unwrap();
    assert_eq!(report.status, IntegrityStatus::Offline);
    assert!(has(&report, "protected_record_removed"));
}
