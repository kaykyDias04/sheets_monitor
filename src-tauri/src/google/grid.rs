//! Targeted grid reads: raw cells are transient and never logged.
use super::{ApiError, GoogleApi, ReadTransport, MAX_RANGES, QUERY_BUDGET};
use serde::Deserialize;
use serde_json::Value;
use std::time::Instant;
use url::Url;
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridDocument {
    pub spreadsheet_id: String,
    #[serde(default)]
    pub sheets: Vec<GridSheet>,
    #[serde(default)]
    pub named_ranges: Vec<GridNamedRange>,
}
#[derive(Clone, Deserialize)]
pub struct GridSheet {
    pub properties: GridProperties,
    #[serde(default)]
    pub data: Vec<GridBlock>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridProperties {
    pub sheet_id: u32,
    pub title: String,
    pub index: u32,
    pub sheet_type: Option<String>,
    pub grid_properties: Option<GridSize>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridSize {
    pub row_count: u32,
    pub column_count: u32,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridBlock {
    #[serde(default)]
    pub start_row: u32,
    #[serde(default)]
    pub start_column: u32,
    #[serde(default)]
    pub row_data: Vec<GridRow>,
}
#[derive(Clone, Deserialize)]
pub struct GridRow {
    #[serde(default)]
    pub values: Vec<GridCell>,
}
#[derive(Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridCell {
    pub user_entered_value: Option<Value>,
    pub effective_value: Option<Value>,
}
fn valid_extended(value: &Value, user: bool) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    if object.is_empty() {
        return true;
    }
    if object.len() != 1 {
        return false;
    }
    let (key, value) = object.iter().next().expect("one field");
    match key.as_str() {
        "numberValue" => value.is_number(),
        "boolValue" => value.is_boolean(),
        "stringValue" => value.is_string(),
        "formulaValue" => {
            user && value
                .as_str()
                .is_some_and(|s| s.starts_with('=') && s.len() <= 16384)
        }
        "errorValue" => {
            !user
                && value.as_object().is_some_and(|e| {
                    e.keys().all(|k| matches!(k.as_str(), "type" | "message"))
                        && e.get("type").and_then(Value::as_str).is_some_and(|s| {
                            matches!(
                                s,
                                "ERROR"
                                    | "NULL_VALUE"
                                    | "DIVIDE_BY_ZERO"
                                    | "VALUE"
                                    | "REF"
                                    | "NAME"
                                    | "NUM"
                                    | "N_A"
                                    | "LOADING"
                            )
                        })
                        && e.get("message").is_none_or(Value::is_string)
                })
        }
        _ => false,
    }
}
impl GridCell {
    pub fn valid(&self) -> bool {
        self.user_entered_value
            .as_ref()
            .is_none_or(|v| valid_extended(v, true))
            && self
                .effective_value
                .as_ref()
                .is_none_or(|v| valid_extended(v, false))
    }

    pub fn formula(&self) -> Option<&str> {
        self.user_entered_value
            .as_ref()?
            .get("formulaValue")?
            .as_str()
    }
    pub fn error(&self) -> Option<&str> {
        self.effective_value
            .as_ref()?
            .get("errorValue")?
            .get("type")?
            .as_str()
    }
    pub fn value(&self) -> Value {
        self.effective_value
            .as_ref()
            .and_then(|v| {
                v.get("stringValue")
                    .or_else(|| v.get("numberValue"))
                    .or_else(|| v.get("boolValue"))
            })
            .cloned()
            .unwrap_or(Value::Null)
    }
    pub fn is_empty(&self) -> bool {
        self.value().is_null() && self.formula().is_none() && self.error().is_none()
    }
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridNamedRange {
    pub named_range_id: String,
    pub name: String,
    pub range: GoogleGridRange,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleGridRange {
    pub sheet_id: u32,
    pub start_row_index: Option<u32>,
    pub end_row_index: Option<u32>,
    pub start_column_index: Option<u32>,
    pub end_column_index: Option<u32>,
}
impl<T: ReadTransport> GoogleApi<T> {
    pub fn structural_grid(&self, id: &str, ranges: &[String]) -> Result<GridDocument, ApiError> {
        super::validate_google_id(id)?;
        if ranges.len() > MAX_RANGES
            || ranges
                .iter()
                .any(|r| r.is_empty() || r.len() > 256 || r.chars().any(char::is_control))
            || ranges.iter().map(String::len).sum::<usize>() > 3000
        {
            return Err(ApiError::InvalidInput);
        }
        let mut url = Url::parse(&format!(
            "https://sheets.googleapis.com/v4/spreadsheets/{id}"
        ))
        .map_err(|_| ApiError::InvalidInput)?;
        let fields = if ranges.is_empty() {
            "spreadsheetId,sheets(properties(sheetId,title,index,sheetType,gridProperties(rowCount,columnCount))),namedRanges(namedRangeId,name,range)"
        } else {
            "spreadsheetId,sheets(properties(sheetId,title,index,sheetType,gridProperties(rowCount,columnCount)),data(startRow,startColumn,rowData(values(userEnteredValue,effectiveValue))))"
        };
        url.query_pairs_mut().append_pair("fields", fields);
        for range in ranges {
            url.query_pairs_mut().append_pair("ranges", range);
        }
        let result: GridDocument = self.read(url, Instant::now() + QUERY_BUDGET)?;
        if result.sheets.is_empty()
            || result.sheets.iter().any(|s| {
                s.properties
                    .sheet_type
                    .as_deref()
                    .is_none_or(|k| k == "GRID")
                    && s.properties.grid_properties.is_none()
            })
            || result.spreadsheet_id != id
            || result.sheets.iter().enumerate().any(|(i, s)| {
                result.sheets[..i]
                    .iter()
                    .any(|p| p.properties.sheet_id == s.properties.sheet_id)
            })
        {
            return Err(ApiError::InvalidResponse);
        }
        if result.sheets.iter().any(|s| {
            s.properties.grid_properties.as_ref().is_some_and(|g| {
                g.row_count == 0
                    || g.column_count == 0
                    || g.row_count > 1_000_000
                    || g.column_count > 18278
            }) || s.data.iter().any(|b| {
                b.row_data
                    .iter()
                    .any(|r| r.values.iter().any(|c| !c.valid()))
            })
        }) {
            return Err(ApiError::InvalidResponse);
        }
        if result.sheets.iter().any(|sheet| {
            sheet
                .properties
                .grid_properties
                .as_ref()
                .is_some_and(|size| {
                    sheet.data.iter().any(|block| {
                        block.start_row >= size.row_count
                            || block.start_column >= size.column_count
                            || block.row_data.len() as u64
                                > u64::from(size.row_count - block.start_row)
                            || block.row_data.iter().any(|row| {
                                row.values.len() as u64
                                    > u64::from(size.column_count - block.start_column)
                            })
                    })
                })
        }) {
            return Err(ApiError::InvalidResponse);
        }
        Ok(result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distinguishes_error_values_from_literal_error_text() {
        let literal: GridCell =
            serde_json::from_value(serde_json::json!({"effectiveValue":{"stringValue":"#REF!"}}))
                .unwrap();
        assert_eq!(literal.error(), None);
        let broken:GridCell=serde_json::from_value(serde_json::json!({"userEnteredValue":{"formulaValue":"=#REF!"},"effectiveValue":{"errorValue":{"type":"REF"}}})).unwrap();
        assert_eq!(broken.error(), Some("REF"));
        assert_eq!(broken.formula(), Some("=#REF!"));
    }
}
