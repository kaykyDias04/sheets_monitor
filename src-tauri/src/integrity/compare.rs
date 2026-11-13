//! Baseline integrity and previous-observation content differences are separate.
use super::formulas::{compare_formula, FormulaComparison, FormulaContext};
use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn evaluate(
    baseline: &BaselineVersion,
    previous: Option<&StructuralSnapshot>,
    current: &StructuralSnapshot,
    check_id: &str,
) -> EngineResult<IntegrityReport> {
    baseline.snapshot.validate()?;
    current.validate()?;
    let approved = &baseline.snapshot;
    let content_previous = previous.unwrap_or(approved);
    if current.account_subject != approved.account_subject
        || current.spreadsheet_id != approved.spreadsheet_id
    {
        return Err(EngineError::Forbidden);
    }
    if current.scope_hash != approved.scope_hash {
        return Err(EngineError::BaselineConflict);
    }
    if baseline.id.is_empty() || baseline.approved_at.is_empty() {
        return Err(EngineError::MissingBaseline);
    }
    if let Some(old) = previous {
        old.validate()?;
        if old.scope_hash != current.scope_hash
            || old.account_subject != current.account_subject
            || old.spreadsheet_id != current.spreadsheet_id
        {
            return Err(EngineError::BaselineConflict);
        }
    }
    let mut findings = Vec::new();
    let mut differences = Vec::new();
    for issue in &current.coverage.issues {
        findings.push(finding(
            issue.sheet_id,
            issue.rule_id.as_deref().unwrap_or("capture"),
            &format!("capture:{}", issue.code),
            &issue.code,
            issue.critical,
            FindingOutcome::Inconclusive,
            None,
            None,
            None,
        ));
    }
    if !current.coverage.metadata_complete {
        findings.push(finding(
            None,
            "metadata",
            "metadata",
            "metadata_not_observed",
            true,
            FindingOutcome::Inconclusive,
            None,
            None,
            None,
        ));
    }
    if !current.coverage.complete && current.coverage.issues.is_empty() {
        findings.push(finding(
            None,
            "connectivity",
            "connectivity",
            "capture_incomplete",
            true,
            FindingOutcome::Inconclusive,
            None,
            None,
            None,
        ));
    }
    for sheet_rule in &approved.scope.sheets {
        let id = sheet_rule.sheet_id;
        let critical = sheet_rule.regions.iter().any(|r| r.critical);
        let baseline_sheet = approved
            .sheets
            .iter()
            .find(|s| s.sheet_id == id)
            .ok_or(EngineError::InvalidSnapshot)?;
        let Some(sheet) = current.sheets.iter().find(|s| s.sheet_id == id) else {
            findings.push(finding(
                Some(id),
                "sheet",
                "existence",
                if current.coverage.metadata_complete {
                    "monitored_sheet_missing"
                } else {
                    "sheet_not_observed"
                },
                critical,
                if current.coverage.metadata_complete {
                    FindingOutcome::Violation
                } else {
                    FindingOutcome::Inconclusive
                },
                Some(baseline_sheet.title.clone()),
                None,
                None,
            ));
            continue;
        };
        if sheet_rule.protect_title && sheet.title != baseline_sheet.title {
            findings.push(finding(
                Some(id),
                "sheet",
                "title",
                "protected_sheet_renamed",
                critical,
                FindingOutcome::Violation,
                Some(baseline_sheet.title.clone()),
                Some(sheet.title.clone()),
                None,
            ));
        }
        if sheet_rule.protect_position && sheet.index != baseline_sheet.index {
            findings.push(finding(
                Some(id),
                "sheet",
                "position",
                "protected_sheet_reordered",
                critical,
                FindingOutcome::Violation,
                Some(baseline_sheet.index.to_string()),
                Some(sheet.index.to_string()),
                None,
            ));
        }
        for rule in &sheet_rule.regions {
            let expected = baseline_sheet
                .regions
                .iter()
                .find(|r| r.region_id == rule.id)
                .ok_or(EngineError::InvalidSnapshot)?;
            let Some(region) = sheet
                .regions
                .iter()
                .find(|r| r.region_id == rule.id)
                .filter(|r| r.complete)
            else {
                findings.push(finding(
                    Some(id),
                    &rule.id,
                    "coverage",
                    "region_not_observed",
                    rule.critical,
                    FindingOutcome::Inconclusive,
                    None,
                    None,
                    None,
                ));
                continue;
            };
            if !expected.complete {
                return Err(EngineError::InvalidSnapshot);
            }
            if region.range != expected.range {
                findings.push(finding(
                    Some(id),
                    &rule.id,
                    "range",
                    "protected_region_displaced",
                    rule.critical,
                    FindingOutcome::Violation,
                    Some(rect_label(&expected.range)),
                    Some(rect_label(&region.range)),
                    None,
                ));
            }
            let mut ambiguous_columns = BTreeSet::new();
            for column in &rule.columns {
                let matches: Vec<_> = region
                    .headers
                    .iter()
                    .filter(|h| h.header == column.header)
                    .collect();
                if matches.len() > 1 {
                    ambiguous_columns.insert(column.id.clone());
                    findings.push(finding(
                        Some(id),
                        &column.id,
                        &format!("{}:identity", rule.id),
                        "column_identity_ambiguous",
                        rule.critical,
                        FindingOutcome::Inconclusive,
                        Some(column.header.clone()),
                        Some("duplicate_header".into()),
                        None,
                    ));
                } else if matches.is_empty() {
                    ambiguous_columns.insert(column.id.clone());
                    if column.required {
                        let code = if sheet.column_count < baseline_sheet.column_count {
                            "protected_column_missing"
                        } else {
                            "protected_header_changed_or_column_missing"
                        };
                        findings.push(finding(
                            Some(id),
                            &column.id,
                            &format!("{}:header", rule.id),
                            code,
                            rule.critical,
                            FindingOutcome::Violation,
                            Some(column.header.clone()),
                            region
                                .headers
                                .iter()
                                .find(|h| h.column == column.position)
                                .map(|h| h.header.clone()),
                            Some(format!("column:{}", column.position)),
                        ));
                    }
                } else if column.protect_position && matches[0].column != column.position {
                    findings.push(finding(
                        Some(id),
                        &column.id,
                        &format!("{}:position", rule.id),
                        "protected_column_reordered",
                        rule.critical,
                        FindingOutcome::Violation,
                        Some(column.position.to_string()),
                        Some(matches[0].column.to_string()),
                        Some(format!("column:{}", matches[0].column)),
                    ));
                }
            }
            if !rule.allow_record_deletion && region.complete {
                check_record_deletion(id, rule, expected, region, &mut findings);
            }
            check_formulas(
                FormulaRegionContext {
                    sheet_id: id,
                    rule,
                    expected,
                    found: region,
                    ambiguous_columns: &ambiguous_columns,
                    baseline: approved,
                    current,
                },
                &mut findings,
            );
        }
        if let Some(old) = content_previous.sheets.iter().find(|s| s.sheet_id == id) {
            if old.title != sheet.title {
                differences.push(difference(
                    Some(id),
                    "sheet_title_changed",
                    "structure",
                    format!("sheet:{id}:title"),
                    None,
                    Some(old.title.clone()),
                    Some(sheet.title.clone()),
                    false,
                ));
            }
            if old.index != sheet.index {
                differences.push(difference(
                    Some(id),
                    "sheet_position_changed",
                    "structure",
                    format!("sheet:{id}:index"),
                    None,
                    Some(old.index.to_string()),
                    Some(sheet.index.to_string()),
                    false,
                ));
            }
            for rule in &sheet_rule.regions {
                if let (Some(before), Some(after)) = (
                    old.regions
                        .iter()
                        .find(|r| r.region_id == rule.id && r.complete),
                    sheet
                        .regions
                        .iter()
                        .find(|r| r.region_id == rule.id && r.complete),
                ) {
                    header_differences(id, rule, before, after, &mut differences);
                    row_differences(id, rule, before, after, &mut differences);
                }
            }
        }
    }
    for id in &approved.scope.named_range_ids {
        let expected = approved
            .named_ranges
            .iter()
            .find(|r| &r.id == id)
            .ok_or(EngineError::InvalidSnapshot)?;
        match current.named_ranges.iter().find(|r| &r.id == id) {
            Some(found) if found != expected => findings.push(finding(
                Some(expected.sheet_id),
                id,
                "named_range",
                "named_range_changed",
                true,
                FindingOutcome::Violation,
                Some(fingerprint(expected)),
                Some(fingerprint(found)),
                Some(expected.name.clone()),
            )),
            None => findings.push(finding(
                Some(expected.sheet_id),
                id,
                "named_range",
                if current.coverage.metadata_complete {
                    "named_range_missing"
                } else {
                    "named_range_not_observed"
                },
                true,
                if current.coverage.metadata_complete {
                    FindingOutcome::Violation
                } else {
                    FindingOutcome::Inconclusive
                },
                Some(expected.name.clone()),
                None,
                None,
            )),
            _ => {}
        }
        if let Some(old) = content_previous.named_ranges.iter().find(|r| &r.id == id) {
            let after = current.named_ranges.iter().find(|r| &r.id == id);
            if current.coverage.metadata_complete && after != Some(old) {
                differences.push(difference(
                    Some(old.sheet_id),
                    "named_range_changed",
                    "structure",
                    format!("named:{id}"),
                    None,
                    Some(fingerprint(old)),
                    after.map(fingerprint),
                    false,
                ));
            }
        }
    }
    for rule in &approved.scope.scripts {
        let Some(script) = current
            .scripts
            .iter()
            .find(|s| s.script_id == rule.script_id)
        else {
            findings.push(finding(
                None,
                &rule.script_id,
                "script_capture",
                "script_not_observed",
                rule.critical,
                FindingOutcome::Inconclusive,
                None,
                None,
                None,
            ));
            continue;
        };
        let mut dependencies = super::scripts::validate_dependencies(
            &script.analysis,
            approved,
            current,
            rule.critical,
        );
        for item in &mut dependencies {
            item.key = format!("script:{}:{}", rule.script_id, item.key);
            item.rule_id = format!("{}:{}", rule.script_id, item.rule_id);
        }
        findings.extend(dependencies);
        if !script.analysis.complete {
            findings.push(finding(
                None,
                &rule.script_id,
                "script_analysis",
                "script_analysis_incomplete",
                rule.critical,
                FindingOutcome::Inconclusive,
                None,
                None,
                None,
            ));
        }
        if let Some(old) = content_previous
            .scripts
            .iter()
            .find(|s| s.script_id == rule.script_id)
        {
            script_differences(old, script, &mut differences);
        }
    }
    // Stable keys make ordering deterministic and remove duplicate diagnoses from
    // overlapping capture and dependency checks without hiding distinct rules.
    let mut unique = BTreeMap::new();
    for f in findings {
        unique.insert((f.key.clone(), f.code.clone()), f);
    }
    let findings: Vec<_> = unique.into_values().collect();
    let mut unique = BTreeMap::new();
    for d in differences {
        unique.insert((d.key.clone(), d.kind.clone()), d);
    }
    let differences: Vec<_> = unique.into_values().collect();
    let sheets = current
        .scope
        .sheets
        .iter()
        .map(|s| SheetAssessment {
            sheet_id: s.sheet_id,
            status: status(
                findings
                    .iter()
                    .filter(|f| f.sheet_id == Some(s.sheet_id) || f.sheet_id.is_none()),
            ),
        })
        .collect();
    Ok(IntegrityReport {
        observation_revision: 0,
        check_id: check_id.into(),
        snapshot_id: current.id.clone(),
        previous_snapshot_id: previous.map(|s| s.id.clone()),
        baseline_id: baseline.id.clone(),
        account_subject: current.account_subject.clone(),
        spreadsheet_id: current.spreadsheet_id.clone(),
        detected_at: current.captured_at.clone(),
        status: status(findings.iter()),
        connectivity: current.connectivity,
        sheets,
        findings,
        differences,
        coverage: current.coverage.clone(),
    })
}

pub fn validate_baseline_snapshot(snapshot: &StructuralSnapshot) -> EngineResult<()> {
    snapshot.validate()?;
    if !snapshot.coverage.complete
        || !snapshot.coverage.metadata_complete
        || snapshot.connectivity != Connectivity::Reachable
        || snapshot.coverage.issues.iter().any(|i| i.critical)
    {
        return Err(EngineError::InvalidSnapshot);
    }
    for sheet in &snapshot.scope.sheets {
        let data = snapshot
            .sheets
            .iter()
            .find(|s| s.sheet_id == sheet.sheet_id)
            .ok_or(EngineError::InvalidSnapshot)?;
        for region in &sheet.regions {
            let data = data
                .regions
                .iter()
                .find(|r| r.region_id == region.id && r.complete)
                .ok_or(EngineError::InvalidSnapshot)?;
            for c in &region.columns {
                if c.required
                    && (data.headers.iter().filter(|h| h.header == c.header).count() != 1
                        || !data
                            .headers
                            .iter()
                            .any(|h| h.header == c.header && h.column == c.position))
                {
                    return Err(EngineError::InvalidSnapshot);
                }
            }
            for formula in &region.formulas {
                if formula.allow_row_shift
                    && (region.key_column.is_none() || !unique_keys(&data.rows))
                {
                    return Err(EngineError::InvalidSnapshot);
                }
                let column = region
                    .columns
                    .iter()
                    .find(|c| c.id == formula.column_id)
                    .ok_or(EngineError::InvalidScope)?;
                let formula = data
                    .formulas
                    .iter()
                    .find(|f| f.row == formula.baseline_row && f.column == column.position)
                    .ok_or(EngineError::InvalidSnapshot)?;
                if formula.formula.is_none() || formula.error.is_some() {
                    return Err(EngineError::InvalidSnapshot);
                }
            }
        }
    }
    for named in &snapshot.scope.named_range_ids {
        if !snapshot.named_ranges.iter().any(|n| &n.id == named) {
            return Err(EngineError::InvalidSnapshot);
        }
    }
    for script in &snapshot.scope.scripts {
        if let Some(observed) = snapshot
            .scripts
            .iter()
            .find(|s| s.script_id == script.script_id)
        {
            if script.critical
                && super::scripts::validate_dependencies(
                    &observed.analysis,
                    snapshot,
                    snapshot,
                    true,
                )
                .iter()
                .any(|f| f.severity == Severity::Critical)
            {
                return Err(EngineError::InvalidSnapshot);
            }
        }
        if !snapshot
            .scripts
            .iter()
            .any(|s| s.script_id == script.script_id && (!script.critical || s.analysis.complete))
        {
            return Err(EngineError::InvalidSnapshot);
        }
    }
    Ok(())
}
fn status<'a>(findings: impl Iterator<Item = &'a Finding>) -> IntegrityStatus {
    let mut uncertain = false;
    for f in findings.filter(|f| f.severity == Severity::Critical) {
        if f.outcome == FindingOutcome::Violation {
            return IntegrityStatus::Offline;
        }
        uncertain = true;
    }
    if uncertain {
        IntegrityStatus::Indeterminate
    } else {
        IntegrityStatus::Online
    }
}
#[allow(clippy::too_many_arguments)]
fn finding(
    sheet_id: Option<u32>,
    rule_id: &str,
    key: &str,
    code: &str,
    critical: bool,
    outcome: FindingOutcome,
    expected: Option<String>,
    found: Option<String>,
    location: Option<String>,
) -> Finding {
    Finding {
        key: format!(
            "{}:{rule_id}:{key}",
            sheet_id.map_or_else(|| "global".into(), |s| s.to_string())
        ),
        sheet_id,
        rule_id: rule_id.into(),
        code: code.into(),
        severity: if critical {
            Severity::Critical
        } else {
            Severity::Warning
        },
        outcome,
        expected,
        found,
        location,
    }
}
#[allow(clippy::too_many_arguments)]
fn difference(
    sheet_id: Option<u32>,
    kind: &str,
    category: &str,
    key: String,
    location: Option<String>,
    before: Option<String>,
    after: Option<String>,
    ambiguous: bool,
) -> ContentDifference {
    ContentDifference {
        key,
        sheet_id,
        kind: kind.into(),
        category: category.into(),
        location,
        before,
        after,
        ambiguous,
    }
}
fn rect_label(r: &GridRect) -> String {
    format!(
        "R{}:{}C{}:{}",
        r.start_row, r.end_row, r.start_col, r.end_col
    )
}

struct FormulaRegionContext<'a> {
    sheet_id: u32,
    rule: &'a RegionRule,
    expected: &'a RegionSnapshot,
    found: &'a RegionSnapshot,
    ambiguous_columns: &'a BTreeSet<String>,
    baseline: &'a StructuralSnapshot,
    current: &'a StructuralSnapshot,
}
fn check_formulas(context: FormulaRegionContext<'_>, findings: &mut Vec<Finding>) {
    let FormulaRegionContext {
        sheet_id,
        rule,
        expected,
        found,
        ambiguous_columns,
        baseline,
        current,
    } = context;
    for formula_rule in &rule.formulas {
        let Some(column) = rule.columns.iter().find(|c| c.id == formula_rule.column_id) else {
            continue;
        };
        let Some(before) = expected
            .formulas
            .iter()
            .find(|f| f.row == formula_rule.baseline_row && f.column == column.position)
        else {
            findings.push(finding(
                Some(sheet_id),
                &formula_rule.id,
                "baseline",
                "baseline_formula_not_observed",
                rule.critical,
                FindingOutcome::Inconclusive,
                None,
                None,
                None,
            ));
            continue;
        };
        if ambiguous_columns.contains(&column.id)
            || formula_rule.allow_row_shift
                && rule
                    .key_column
                    .as_ref()
                    .is_some_and(|key| ambiguous_columns.contains(key))
        {
            findings.push(finding(
                Some(sheet_id),
                &formula_rule.id,
                "identity",
                "formula_column_or_key_not_resolved",
                rule.critical,
                FindingOutcome::Inconclusive,
                None,
                None,
                None,
            ));
            continue;
        }
        let Some(current_column) = found
            .headers
            .iter()
            .find(|c| c.header == column.header)
            .map(|c| c.column)
        else {
            continue;
        };
        let mut identity_uncertain = false;
        let mut deleted_record = false;
        let row = if formula_rule.allow_row_shift {
            match &before.key_hash {
                Some(key)
                    if expected
                        .rows
                        .iter()
                        .filter(|r| r.key_hash.as_ref() == Some(key))
                        .count()
                        == 1 =>
                {
                    let rows: Vec<_> = found
                        .rows
                        .iter()
                        .filter(|r| r.key_hash.as_ref() == Some(key))
                        .collect();
                    if rows.len() == 1 {
                        Some(rows[0].row)
                    } else if rows.is_empty()
                        && rule.allow_record_deletion
                        && unique_keys(&expected.rows)
                        && unique_keys(&found.rows)
                    {
                        deleted_record = true;
                        None
                    } else {
                        identity_uncertain = true;
                        None
                    }
                }
                _ => {
                    identity_uncertain = true;
                    None
                }
            }
        } else {
            Some(formula_rule.baseline_row)
        };
        if deleted_record {
            continue;
        }
        if identity_uncertain {
            findings.push(finding(
                Some(sheet_id),
                &formula_rule.id,
                "anchor",
                "formula_anchor_ambiguous",
                rule.critical,
                FindingOutcome::Inconclusive,
                None,
                None,
                None,
            ));
            continue;
        }
        let row = row.unwrap_or(formula_rule.baseline_row);
        if !formula_rule.allow_row_shift {
            // A stable key changed under a fixed coordinate: do not confuse row
            // movement/deletion with deletion of the approved logical formula.
            if let Some(key) = &before.key_hash {
                if !found
                    .rows
                    .iter()
                    .any(|r| r.row == row && r.key_hash.as_ref() == Some(key))
                {
                    findings.push(finding(
                        Some(sheet_id),
                        &formula_rule.id,
                        "anchor",
                        "fixed_formula_anchor_changed",
                        rule.critical,
                        FindingOutcome::Inconclusive,
                        None,
                        None,
                        None,
                    ));
                    continue;
                }
            }
        }
        let Some(after) = found
            .formulas
            .iter()
            .find(|f| f.row == row && f.column == current_column)
        else {
            // Collector must record observed empty/constant critical cells too.
            findings.push(finding(
                Some(sheet_id),
                &formula_rule.id,
                "coverage",
                "formula_cell_not_observed",
                rule.critical,
                FindingOutcome::Inconclusive,
                None,
                None,
                Some(format!("R{}C{}", row + 1, current_column + 1)),
            ));
            continue;
        };
        let location = Some(format!("R{}C{}", row + 1, current_column + 1));
        if let Some(error) = &after.error {
            let confirmed = matches!(
                error.as_str(),
                "REF"
                    | "NAME"
                    | "DIVIDE_BY_ZERO"
                    | "VALUE"
                    | "NUM"
                    | "N_A"
                    | "ERROR"
                    | "NULL_VALUE"
            );
            findings.push(finding(
                Some(sheet_id),
                &formula_rule.id,
                "effective_error",
                if error == "LOADING" {
                    "formula_loading"
                } else if confirmed {
                    "formula_effective_error"
                } else {
                    "formula_error_unknown"
                },
                rule.critical,
                if confirmed {
                    FindingOutcome::Violation
                } else {
                    FindingOutcome::Inconclusive
                },
                None,
                Some(error.clone()),
                location,
            ));
            continue;
        }
        let Some(expected_formula) = before.formula.as_deref() else {
            findings.push(finding(
                Some(sheet_id),
                &formula_rule.id,
                "baseline",
                "baseline_formula_not_observed",
                rule.critical,
                FindingOutcome::Inconclusive,
                None,
                None,
                location,
            ));
            continue;
        };
        let Some(current_formula) = after.formula.as_deref() else {
            findings.push(finding(
                Some(sheet_id),
                &formula_rule.id,
                "presence",
                "protected_formula_removed_or_replaced",
                rule.critical,
                FindingOutcome::Violation,
                Some(fingerprint(&expected_formula)),
                Some(after.value_hash.clone()),
                location,
            ));
            continue;
        };
        match compare_formula(
            expected_formula,
            current_formula,
            FormulaContext {
                sheet_id,
                before: (before.row, before.column),
                after: (after.row, after.column),
                mode: formula_rule.mode,
                allow_shift: formula_rule.allow_row_shift,
                baseline,
                current,
            },
        ) {
            FormulaComparison::Equivalent => {}
            FormulaComparison::Changed => findings.push(finding(
                Some(sheet_id),
                &formula_rule.id,
                "expression",
                "protected_formula_changed",
                rule.critical,
                FindingOutcome::Violation,
                Some(fingerprint(&expected_formula)),
                Some(fingerprint(&current_formula)),
                location,
            )),
            FormulaComparison::BrokenReference => findings.push(finding(
                Some(sheet_id),
                &formula_rule.id,
                "reference",
                "formula_sheet_reference_missing",
                rule.critical,
                FindingOutcome::Violation,
                None,
                None,
                location,
            )),
            FormulaComparison::Inconclusive => findings.push(finding(
                Some(sheet_id),
                &formula_rule.id,
                "equivalence",
                "formula_equivalence_unproven",
                rule.critical,
                FindingOutcome::Inconclusive,
                Some(fingerprint(&expected_formula)),
                Some(fingerprint(&current_formula)),
                location,
            )),
        }
    }
}
fn unique_keys(rows: &[RowDigest]) -> bool {
    let mut keys = BTreeSet::new();
    rows.iter()
        .all(|r| r.key_hash.as_ref().is_some_and(|k| keys.insert(k)))
}
fn header_differences(
    sheet: u32,
    rule: &RegionRule,
    before: &RegionSnapshot,
    after: &RegionSnapshot,
    out: &mut Vec<ContentDifference>,
) {
    let old: BTreeMap<_, _> = before
        .headers
        .iter()
        .map(|c| (c.column, &c.header))
        .collect();
    let new: BTreeMap<_, _> = after
        .headers
        .iter()
        .map(|c| (c.column, &c.header))
        .collect();
    for column in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        if old.get(&column) != new.get(&column) {
            out.push(difference(
                Some(sheet),
                "header_changed",
                "structure",
                format!("sheet:{sheet}:region:{}:header:{column}", rule.id),
                Some(format!("column:{column}")),
                old.get(&column).map(|s| (*s).clone()),
                new.get(&column).map(|s| (*s).clone()),
                false,
            ));
        }
    }
    if before.range != after.range {
        out.push(difference(
            Some(sheet),
            "region_changed",
            "structure",
            format!("sheet:{sheet}:region:{}:range", rule.id),
            None,
            Some(rect_label(&before.range)),
            Some(rect_label(&after.range)),
            false,
        ));
    }
}
fn check_record_deletion(
    sheet: u32,
    rule: &RegionRule,
    before: &RegionSnapshot,
    after: &RegionSnapshot,
    findings: &mut Vec<Finding>,
) {
    let keyed = rule.key_column.is_some() && unique_keys(&before.rows) && unique_keys(&after.rows);
    let missing = if keyed {
        let keys: BTreeSet<_> = after
            .rows
            .iter()
            .filter_map(|row| row.key_hash.as_ref())
            .collect();
        before
            .rows
            .iter()
            .filter(|row| row.key_hash.as_ref().is_some_and(|key| !keys.contains(key)))
            .count()
    } else {
        before.rows.len().saturating_sub(after.rows.len())
    };
    let uncertain = !keyed
        && missing == 0
        && !before
            .rows
            .iter()
            .map(|row| &row.fingerprint)
            .eq(after.rows.iter().map(|row| &row.fingerprint));
    if missing > 0 || uncertain {
        findings.push(finding(
            Some(sheet),
            &rule.id,
            "record_deletion",
            if uncertain {
                "record_deletion_unproven"
            } else {
                "protected_record_removed"
            },
            rule.critical,
            if uncertain {
                FindingOutcome::Inconclusive
            } else {
                FindingOutcome::Violation
            },
            Some(before.rows.len().to_string()),
            Some(after.rows.len().to_string()),
            None,
        ));
    }
}

fn row_differences(
    sheet: u32,
    rule: &RegionRule,
    before: &RegionSnapshot,
    after: &RegionSnapshot,
    out: &mut Vec<ContentDifference>,
) {
    if rule.key_column.is_some() && unique_keys(&before.rows) && unique_keys(&after.rows) {
        let old: BTreeMap<_, _> = before
            .rows
            .iter()
            .map(|r| (r.key_hash.as_deref().unwrap_or_default(), r))
            .collect();
        let new: BTreeMap<_, _> = after
            .rows
            .iter()
            .map(|r| (r.key_hash.as_deref().unwrap_or_default(), r))
            .collect();
        let old_order: Vec<_> = before
            .rows
            .iter()
            .filter_map(|r| r.key_hash.as_ref())
            .filter(|k| new.contains_key(k.as_str()))
            .collect();
        let new_order: Vec<_> = after
            .rows
            .iter()
            .filter_map(|r| r.key_hash.as_ref())
            .filter(|k| old.contains_key(k.as_str()))
            .collect();
        if old_order != new_order {
            out.push(difference(
                Some(sheet),
                "record_order_changed",
                "content",
                format!("sheet:{sheet}:region:{}:order", rule.id),
                None,
                Some(fingerprint(&old_order)),
                Some(fingerprint(&new_order)),
                false,
            ));
        }
        for key in old
            .keys()
            .chain(new.keys())
            .copied()
            .collect::<BTreeSet<_>>()
        {
            let identity = format!("sheet:{sheet}:region:{}:record:{key}", rule.id);
            match (old.get(key), new.get(key)) {
                (None, Some(row)) => out.push(difference(
                    Some(sheet),
                    "record_added",
                    "content",
                    identity,
                    Some(format!("row:{}", row.row)),
                    None,
                    Some(row.fingerprint.clone()),
                    false,
                )),
                (Some(row), None) => out.push(difference(
                    Some(sheet),
                    "record_removed",
                    "content",
                    identity,
                    Some(format!("row:{}", row.row)),
                    Some(row.fingerprint.clone()),
                    None,
                    false,
                )),
                (Some(a), Some(b)) => {
                    for column in a
                        .cell_hashes
                        .keys()
                        .chain(b.cell_hashes.keys())
                        .collect::<BTreeSet<_>>()
                    {
                        if a.cell_hashes.get(column) != b.cell_hashes.get(column) {
                            out.push(difference(
                                Some(sheet),
                                "value_changed",
                                "content",
                                format!("{identity}:column:{column}"),
                                Some(format!("row:{}:column:{column}", b.row)),
                                a.cell_hashes.get(column).cloned(),
                                b.cell_hashes.get(column).cloned(),
                                false,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
    } else {
        sequence_differences(sheet, rule, &before.rows, &after.rows, out);
    }
}
fn sequence_differences(
    sheet: u32,
    rule: &RegionRule,
    before: &[RowDigest],
    after: &[RowDigest],
    out: &mut Vec<ContentDifference>,
) {
    if before
        .iter()
        .map(|r| &r.fingerprint)
        .eq(after.iter().map(|r| &r.fingerprint))
    {
        return;
    }
    let emit =
        |a: &[RowDigest], b: &[RowDigest], index: usize, out: &mut Vec<ContentDifference>| {
            out.push(difference(
                Some(sheet),
                "content_changed_unspecified",
                "content",
                format!("sheet:{sheet}:region:{}:sequence:{index}", rule.id),
                Some(format!(
                    "sequence_offset:{index}:before_rows:{}:after_rows:{}",
                    a.len(),
                    b.len()
                )),
                Some(fingerprint(
                    &a.iter().map(|r| &r.fingerprint).collect::<Vec<_>>(),
                )),
                Some(fingerprint(
                    &b.iter().map(|r| &r.fingerprint).collect::<Vec<_>>(),
                )),
                true,
            ));
        };
    if before.len().saturating_mul(after.len()) > 1_000_000 {
        for index in 0..before.len().max(after.len()).div_ceil(128) {
            let start = index * 128;
            let a = before
                .get(start..before.len().min(start + 128))
                .unwrap_or(&[]);
            let b = after
                .get(start..after.len().min(start + 128))
                .unwrap_or(&[]);
            if !a
                .iter()
                .map(|r| &r.fingerprint)
                .eq(b.iter().map(|r| &r.fingerprint))
            {
                emit(a, b, start, out);
            }
        }
        return;
    }
    let n = before.len();
    let m = after.len();
    let width = m + 1;
    let mut lcs = vec![0u32; (n + 1) * (m + 1)];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i * width + j] = if before[i].fingerprint == after[j].fingerprint {
                1 + lcs[(i + 1) * width + j + 1]
            } else {
                lcs[(i + 1) * width + j].max(lcs[i * width + j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let (mut a, mut b) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && before[i].fingerprint == after[j].fingerprint {
            if i > a || j > b {
                emit(&before[a..i], &after[b..j], a, out);
            }
            i += 1;
            j += 1;
            a = i;
            b = j;
        } else if j == m || (i < n && lcs[(i + 1) * width + j] >= lcs[i * width + j + 1]) {
            i += 1;
        } else {
            j += 1;
        }
    }
    if i > a || j > b {
        emit(&before[a..i], &after[b..j], a, out);
    }
}
fn script_differences(
    before: &ScriptSnapshot,
    after: &ScriptSnapshot,
    out: &mut Vec<ContentDifference>,
) {
    let old: BTreeMap<_, _> = before
        .files
        .iter()
        .map(|f| ((f.name.as_str(), f.file_type.as_str()), &f.hash))
        .collect();
    let new: BTreeMap<_, _> = after
        .files
        .iter()
        .map(|f| ((f.name.as_str(), f.file_type.as_str()), &f.hash))
        .collect();
    let start = out.len();
    for key in old
        .keys()
        .chain(new.keys())
        .copied()
        .collect::<BTreeSet<_>>()
    {
        if old.get(&key) == new.get(&key) {
            continue;
        }
        let kind = if !old.contains_key(&key) {
            "script_file_added"
        } else if !new.contains_key(&key) {
            "script_file_removed"
        } else {
            "script_file_changed"
        };
        out.push(difference(
            None,
            kind,
            "script",
            format!("script:{}:file:{}:{}", after.script_id, key.0, key.1),
            Some(format!("{}:{}", key.0, key.1)),
            old.get(&key).map(|h| (*h).clone()),
            new.get(&key).map(|h| (*h).clone()),
            false,
        ));
    }
    if before.source_hash != after.source_hash && out.len() == start {
        out.push(difference(
            None,
            "script_code_changed",
            "script",
            format!("script:{}:hash", after.script_id),
            None,
            Some(before.source_hash.clone()),
            Some(after.source_hash.clone()),
            false,
        ));
    }
}
