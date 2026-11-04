//! A reviewable proposal from bounded, authorized header reads. Nothing is saved.
use super::*;
use crate::{
    google::grid::GridDocument,
    integrity::{
        collector::a1_range, ColumnRule, GridRect, MonitorScope, RegionRule, SheetRule,
        SNAPSHOT_VERSION,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    time::{Duration, Instant},
};
const HEADER_COLUMNS: u32 = 50;
const PROPOSED_ROWS: u32 = 20_000;
const MAX_HEADER_CELLS: usize = 1_500;
const GET_BUDGET: Duration = Duration::from_secs(180);
fn pending() -> CommandError {
    error("configuration_pending","Não foi possível propor cabeçalhos únicos na linha âncora informada em todas as abas selecionadas. Confira a linha âncora, atualize as abas ou configure manualmente os cabeçalhos no editor de regras.")
}
fn invalid_anchor_row() -> CommandError {
    error(
        "invalid_anchor_row",
        "Informe uma linha âncora válida (entre 1 e 1.000.000) antes de sugerir as regras.",
    )
}
fn anchor_out_of_range() -> CommandError {
    error(
        "anchor_row_out_of_range",
        "A linha âncora informada ultrapassa o total de linhas de uma ou mais abas selecionadas. Informe uma linha existente ou atualize as abas.",
    )
}
fn resolve_anchor_row(header_row: Option<u32>) -> Result<(u32, bool), CommandError> {
    match header_row {
        None => Ok((0, false)),
        Some(0) => Err(invalid_anchor_row()),
        Some(row) if row > 1_000_000 => Err(invalid_anchor_row()),
        Some(row) => Ok((row - 1, true)),
    }
}
fn known_rule(row: &RegisteredSpreadsheet, id: u32, force_repropose: bool) -> Option<&SheetRule> {
    if force_repropose {
        return None;
    }
    row.scope
        .as_ref()
        .filter(|s| s.validate().is_ok())
        .and_then(|s| s.sheets.iter().find(|s| s.sheet_id == id))
}
fn proposal_ranges(
    row: &RegisteredSpreadsheet,
    anchor_row: u32,
    force_repropose: bool,
) -> Result<Vec<Vec<String>>, CommandError> {
    if row.selected_sheet_ids.is_empty() || row.selected_sheet_ids.len() > 30 {
        return Err(pending());
    }
    let mut ranges = Vec::new();
    for id in &row.selected_sheet_ids {
        let tab = row
            .tabs
            .iter()
            .find(|t| t.sheet_id == *id)
            .ok_or_else(pending)?;
        if tab.sheet_type.as_deref().is_some_and(|t| t != "GRID") {
            return Err(pending());
        }
        if known_rule(row, *id, force_repropose).is_some() {
            continue;
        }
        let rows = tab.row_count.filter(|r| *r > 0).ok_or_else(pending)?;
        if anchor_row >= rows {
            return Err(anchor_out_of_range());
        }
        let columns = tab
            .column_count
            .filter(|c| *c > 0)
            .ok_or_else(pending)?
            .min(HEADER_COLUMNS);
        ranges.push(a1_range(
            &tab.title,
            &GridRect {
                start_row: anchor_row,
                end_row: anchor_row + 1,
                start_col: 0,
                end_col: columns,
            },
        ));
    }
    // Each request has at most 20 ranges; at most 30 sheets are selected.
    // The grid reader additionally validates URI/response bounds.
    Ok(ranges.chunks(20).map(|r| r.to_vec()).collect())
}
fn proposal_from_documents(
    row: &RegisteredSpreadsheet,
    documents: &[GridDocument],
    anchor_row: u32,
    force_repropose: bool,
) -> Result<MonitorScope, CommandError> {
    let wanted = row
        .selected_sheet_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if wanted.is_empty() || wanted.len() != row.selected_sheet_ids.len() || wanted.len() > 30 {
        return Err(pending());
    }
    let requested = wanted
        .iter()
        .filter(|id| known_rule(row, **id, force_repropose).is_none())
        .copied()
        .collect::<BTreeSet<_>>();
    let mut cells = BTreeMap::new();
    let mut sizes = BTreeMap::new();
    for document in documents {
        if document.spreadsheet_id != row.spreadsheet_id {
            return Err(ApiError::InvalidResponse.into());
        }
        for sheet in &document.sheets {
            let id = sheet.properties.sheet_id;
            if requested.contains(&id) {
                let size = sheet
                    .properties
                    .grid_properties
                    .as_ref()
                    .ok_or_else(pending)?;
                if size.row_count == 0 || size.column_count == 0 {
                    return Err(pending());
                }
                if anchor_row >= size.row_count {
                    return Err(anchor_out_of_range());
                }
                sizes.insert(id, (size.row_count, size.column_count));
            }
            for block in &sheet.data {
                for (r, row_data) in block.row_data.iter().enumerate() {
                    for (c, cell) in row_data.values.iter().enumerate() {
                        let actual_row = block
                            .start_row
                            .checked_add(r as u32)
                            .ok_or(ApiError::InvalidResponse)?;
                        let actual_column = block
                            .start_column
                            .checked_add(c as u32)
                            .ok_or(ApiError::InvalidResponse)?;
                        if !requested.contains(&id)
                            || actual_row != anchor_row
                            || actual_column >= HEADER_COLUMNS
                            || cells.len() >= MAX_HEADER_CELLS
                            || cells.insert((id, actual_column), cell.clone()).is_some()
                        {
                            return Err(ApiError::InvalidResponse.into());
                        }
                    }
                }
            }
        }
    }
    let mut scope = row.scope.clone().unwrap_or(MonitorScope {
        schema_version: SNAPSHOT_VERSION,
        sheets: vec![],
        scripts: vec![],
        named_range_ids: vec![],
        snapshot_retention_days: 7,
        history_retention_days: 90,
    });
    scope.sheets.clear();
    for id in &row.selected_sheet_ids {
        if let Some(rule) = known_rule(row, *id, force_repropose) {
            scope.sheets.push(rule.clone());
            continue;
        }
        let (rows, columns) = sizes.get(id).copied().ok_or_else(pending)?;
        let columns = columns.min(HEADER_COLUMNS);
        let mut headers = BTreeSet::new();
        let mut rules = Vec::new();
        for column in 0..columns {
            let Some(cell) = cells.get(&(*id, column)) else {
                continue;
            };
            if cell.error().is_some() {
                return Err(pending());
            }
            let header = match cell.value() {
                serde_json::Value::String(s) => s,
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => continue,
            };
            if header.trim().is_empty() {
                continue;
            }
            if header.len() > 512
                || header.chars().any(char::is_control)
                || !headers.insert(header.clone())
            {
                return Err(pending());
            }
            rules.push(ColumnRule {
                id: format!("column_{column}"),
                header,
                position: column,
                protect_position: true,
                required: true,
            });
        }
        if rules.is_empty() {
            return Err(pending());
        }
        let end_row = rows
            .min(anchor_row.saturating_add(PROPOSED_ROWS))
            .min(1_000_000);
        scope.sheets.push(SheetRule {
            sheet_id: *id,
            protect_title: true,
            protect_position: false,
            regions: vec![RegionRule {
                id: "data".into(),
                range: GridRect {
                    start_row: anchor_row,
                    end_row,
                    start_col: 0,
                    end_col: columns,
                },
                header_row: anchor_row,
                columns: rules,
                key_column: None,
                critical: true,
                allow_record_deletion: true,
                formulas: vec![],
            }],
        });
    }
    scope.validate()?;
    Ok(scope)
}
fn proposal_with_history_default(
    row: &RegisteredSpreadsheet,
    documents: &[GridDocument],
    default_history_days: u16,
    anchor_row: u32,
    force_repropose: bool,
) -> Result<MonitorScope, CommandError> {
    let mut proposal = proposal_from_documents(row, documents, anchor_row, force_repropose)?;
    if row.scope.is_none() {
        proposal.history_retention_days = default_history_days;
    }
    proposal.validate()?;
    Ok(proposal)
}
impl IntegrationRuntime {
    pub fn propose_integrity_rules(
        &self,
        input: ProposeRulesInput,
    ) -> Result<EngineEnvelope<MonitorScope>, CommandError> {
        let (anchor_row, force_repropose) = resolve_anchor_row(input.header_row)?;
        let mutation = input.mutation();
        let (row, default_history_days) = self
            .management_local(Some(mutation.expected_generation), |db, s| {
                Ok((
                    db.check_registry_mutation(s, &mutation)?,
                    db.management_preferences(s)?.history_days,
                ))
            })?
            .data;
        let ranges = proposal_ranges(&row, anchor_row, force_repropose)?;
        let mut documents = Vec::new();
        if !ranges.is_empty() {
            let mut session = self.auth.authorized()?;
            if session.generation != mutation.expected_generation {
                return Err(AuthError::new(AuthCode::OperationSuperseded).into());
            }
            if !session.has_scope(SHEETS_SCOPE) {
                return Err(AuthError::new(AuthCode::InsufficientScope).into());
            }
            let start = Instant::now();
            let mut refreshed = false;
            let mut gets = 0;
            for chunk in ranges {
                if start.elapsed() >= GET_BUDGET || gets >= 4 {
                    return Err(ApiError::Timeout.into());
                }
                gets += 1;
                let mut result = GoogleApi::new(session.access_token().into())?
                    .structural_grid(&row.spreadsheet_id, &chunk);
                if matches!(result, Err(ApiError::Unauthorized)) && !refreshed {
                    session = self.auth.force_refresh(&session)?;
                    refreshed = true;
                    if start.elapsed() >= GET_BUDGET || gets >= 4 {
                        return Err(ApiError::Timeout.into());
                    }
                    gets += 1;
                    result = GoogleApi::new(session.access_token().into())?
                        .structural_grid(&row.spreadsheet_id, &chunk);
                }
                if start.elapsed() >= GET_BUDGET {
                    return Err(ApiError::Timeout.into());
                }
                documents.push(result?);
            }
        }
        let scope = proposal_with_history_default(
            &row,
            &documents,
            default_history_days,
            anchor_row,
            force_repropose,
        )?;
        self.management_local(Some(mutation.expected_generation), |db, s| {
            db.check_registry_mutation(s, &mutation)?;
            Ok(scope)
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::google::SheetMetadata;
    fn tab(id: u32) -> SheetMetadata {
        SheetMetadata {
            sheet_id: id,
            title: format!("Tab {id}"),
            index: id,
            sheet_type: Some("GRID".into()),
            row_count: Some(25_000),
            column_count: Some(60),
            hidden: false,
        }
    }
    fn row(ids: &[u32]) -> RegisteredSpreadsheet {
        RegisteredSpreadsheet {
            spreadsheet_id: "fixture_sheet".into(),
            name: "Fixture".into(),
            url: "https://docs.google.com/spreadsheets/d/fixture_sheet/edit".into(),
            account_email: None,
            paused: false,
            removed: false,
            status: "configuration_pending".into(),
            authorization: "available".into(),
            access_error: None,
            last_checked_at: None,
            baseline_version: None,
            scope: None,
            scope_hash: None,
            revision: 1,
            tabs: ids.iter().map(|id| tab(*id)).collect(),
            selected_sheet_ids: ids.to_vec(),
            tabs_refreshed_at: None,
        }
    }
    fn document(id: u32, headers: serde_json::Value) -> GridDocument {
        document_at_row(id, 0, headers)
    }
    fn document_at_row(id: u32, start_row: u32, headers: serde_json::Value) -> GridDocument {
        serde_json::from_value(serde_json::json!({"spreadsheetId":"fixture_sheet","sheets":[{"properties":{"sheetId":id,"title":format!("Tab {id}"),"index":0,"sheetType":"GRID","gridProperties":{"rowCount":25_000,"columnCount":60}},"data":[{"startRow":start_row,"startColumn":0,"rowData":[{"values":headers}]}]}]})).unwrap()
    }
    #[test]
    fn actual_headers_form_proposal_without_guessing_formulas_or_saving() {
        let row = row(&[7]);
        let doc = document(
            7,
            serde_json::json!([{"effectiveValue":{"stringValue":"ID"}},{"effectiveValue":{"stringValue":"Amount"}},{"effectiveValue":{"numberValue":2026}}]),
        );
        let scope = proposal_from_documents(&row, &[doc], 0, false).unwrap();
        assert_eq!(scope.sheets[0].sheet_id, 7);
        let region = &scope.sheets[0].regions[0];
        assert_eq!(region.columns[0].header, "ID");
        assert_eq!(region.columns[2].header, "2026");
        assert_eq!(region.range.start_row, 0);
        assert_eq!(region.header_row, 0);
        assert_eq!(region.range.end_row, 20_000);
        assert_eq!(region.range.end_col, 50);
        assert!(region.formulas.is_empty());
        assert!(scope.scripts.is_empty());
        assert!(row.scope.is_none());
    }
    #[test]
    fn custom_anchor_row_reads_headers_and_expands_past_thirty_columns_and_twenty_thousand_rows() {
        let (anchor, force) = resolve_anchor_row(Some(4)).unwrap();
        assert_eq!((anchor, force), (3, true));
        let mut row = row(&[7]);
        row.scope = Some(crate::integrity::fixtures::scope());
        let ranges = proposal_ranges(&row, anchor, force).unwrap();
        assert_eq!(ranges, vec![vec!["'Tab 7'!A4:AX4".to_string()]]);
        let values: Vec<_> = (1..=35)
            .map(|i| serde_json::json!({"effectiveValue":{"stringValue":format!("Col {i}")}}))
            .collect();
        let doc = document_at_row(7, anchor, serde_json::Value::Array(values));
        let scope = proposal_from_documents(&row, &[doc], anchor, force).unwrap();
        let region = &scope.sheets[0].regions[0];
        assert_eq!(region.columns.len(), 35);
        assert_eq!(region.range.start_row, 3);
        assert_eq!(region.header_row, 3);
        assert_eq!(region.range.end_row, 20_003);
        assert_eq!(region.range.end_col, 50);
        assert_eq!(
            resolve_anchor_row(Some(0)).unwrap_err().code,
            "invalid_anchor_row"
        );
        assert_eq!(
            proposal_ranges(&row, 25_000, true).unwrap_err().code,
            "anchor_row_out_of_range"
        );
    }
    #[test]
    fn missing_duplicate_or_error_headers_require_manual_configuration() {
        for headers in [
            serde_json::json!([]),
            serde_json::json!([{"effectiveValue":{"stringValue":" "}}]),
            serde_json::json!([{"effectiveValue":{"stringValue":"ID"}},{"effectiveValue":{"stringValue":"ID"}}]),
            serde_json::json!([{"effectiveValue":{"errorValue":{"type":"REF"}}}]),
        ] {
            let error =
                proposal_from_documents(&row(&[7]), &[document(7, headers)], 0, false).unwrap_err();
            assert_eq!(error.code, "configuration_pending");
        }
    }
    #[test]
    fn selected_stable_ids_limit_requests_and_reject_unrequested_data() {
        let selected = row(&(1..=30).collect::<Vec<_>>());
        let ranges = proposal_ranges(&selected, 0, false).unwrap();
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].len(), 20);
        assert_eq!(ranges[1].len(), 10);
        assert!(ranges.iter().flatten().all(|r| r.ends_with("!A1:AX1")));
        let error = proposal_from_documents(
            &row(&[7]),
            &[document(
                8,
                serde_json::json!([{"effectiveValue":{"stringValue":"ID"}}]),
            )],
            0,
            false,
        )
        .unwrap_err();
        assert_eq!(error.code, "invalid_response");
    }
    #[test]
    fn valid_existing_rules_and_globals_are_preserved() {
        let mut row = row(&[7]);
        let scope = crate::integrity::fixtures::scope();
        row.scope = Some(scope.clone());
        assert!(proposal_ranges(&row, 0, false).unwrap().is_empty());
        assert_eq!(proposal_from_documents(&row, &[], 0, false).unwrap(), scope);
    }
    #[test]
    fn response_offsets_cannot_turn_different_row_into_header() {
        let mut doc = document(
            7,
            serde_json::json!([{"effectiveValue":{"stringValue":"Not a header"}}]),
        );
        doc.sheets[0].data[0].start_row = 3;
        assert!(proposal_from_documents(&row(&[7]), &[doc], 0, false).is_err());
    }
    #[test]
    fn new_proposals_use_history_preference_without_overwriting_existing_policy() {
        let empty = row(&[7]);
        let doc = document(
            7,
            serde_json::json!([
                {"effectiveValue":{"stringValue":"ID"}}
            ]),
        );
        let fresh = proposal_with_history_default(&empty, &[doc], 365, 0, false).unwrap();
        assert_eq!(fresh.history_retention_days, 365);
        let mut existing = row(&[7]);
        let mut scope = crate::integrity::fixtures::scope();
        scope.history_retention_days = 30;
        existing.scope = Some(scope.clone());
        assert_eq!(
            proposal_with_history_default(&existing, &[], 365, 0, false).unwrap(),
            scope
        );
    }
}
