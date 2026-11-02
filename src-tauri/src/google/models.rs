use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpreadsheetMetadata {
    pub spreadsheet_id: String,
    pub spreadsheet_url: Option<String>,
    pub title: String,
    pub locale: Option<String>,
    pub time_zone: Option<String>,
    pub sheets: Vec<SheetMetadata>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SheetMetadata {
    pub sheet_id: u32,
    pub title: String,
    pub index: u32,
    pub sheet_type: Option<String>,
    pub row_count: Option<u32>,
    pub column_count: Option<u32>,
    pub hidden: bool,
}
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ValueRange {
    pub range: String,
    pub major_dimension: String,
    pub values: Vec<Vec<Value>>,
}
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct SheetValues {
    pub spreadsheet_id: String,
    pub values: Vec<ValueRange>,
    pub formulas: Vec<ValueRange>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DriveMetadata {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub modified_time: Option<String>,
    pub trashed: bool,
    pub web_view_link: Option<String>,
    pub version: Option<String>,
    pub drive_id: Option<String>,
    pub resource_key: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScriptMetadata {
    pub script_id: String,
    pub title: String,
    pub parent_id: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScriptFileType {
    #[serde(rename = "SERVER_JS")]
    ServerJs,
    #[serde(rename = "HTML")]
    Html,
    #[serde(rename = "JSON")]
    Json,
}
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ScriptFile {
    pub name: String,
    pub file_type: ScriptFileType,
    pub source: String,
}
impl std::fmt::Debug for ScriptFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptFile")
            .field("file_type", &self.file_type)
            .field("source", &"[redacted]")
            .finish()
    }
}
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ScriptProject {
    pub metadata: ScriptMetadata,
    pub files: Vec<ScriptFile>,
    pub source_hash: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawSpreadsheet {
    pub spreadsheet_id: String,
    pub spreadsheet_url: Option<String>,
    pub properties: RawSpreadsheetProperties,
    pub sheets: Vec<RawSheet>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawSpreadsheetProperties {
    pub title: String,
    pub locale: Option<String>,
    pub time_zone: Option<String>,
}
#[derive(Deserialize)]
pub(super) struct RawSheet {
    pub properties: RawSheetProperties,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawSheetProperties {
    pub sheet_id: u32,
    pub title: String,
    pub index: u32,
    pub sheet_type: Option<String>,
    pub grid_properties: Option<RawGrid>,
    #[serde(default)]
    pub hidden: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawGrid {
    pub row_count: u32,
    pub column_count: u32,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawValues {
    pub spreadsheet_id: String,
    #[serde(default)]
    pub value_ranges: Vec<RawValueRange>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawValueRange {
    pub range: String,
    #[serde(default = "rows")]
    pub major_dimension: String,
    #[serde(default)]
    pub values: Vec<Vec<Value>>,
}
fn rows() -> String {
    "ROWS".into()
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawDrive {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub modified_time: Option<String>,
    #[serde(default)]
    pub trashed: bool,
    pub web_view_link: Option<String>,
    pub version: Option<String>,
    pub drive_id: Option<String>,
    pub resource_key: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawScriptMetadata {
    pub script_id: String,
    pub title: String,
    pub parent_id: Option<String>,
    pub create_time: Option<String>,
    pub update_time: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawScriptContent {
    pub script_id: String,
    pub files: Vec<RawScriptFile>,
}
#[derive(Deserialize)]
pub(super) struct RawScriptFile {
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: ScriptFileType,
    pub source: String,
}
impl From<RawScriptFile> for ScriptFile {
    fn from(f: RawScriptFile) -> Self {
        Self {
            name: f.name,
            file_type: f.file_type,
            source: f.source,
        }
    }
}
