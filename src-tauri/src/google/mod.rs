//! Authorized GET-only Google readers. Values and sources remain transient.
mod discovery;
mod models;
mod transport;
pub use discovery::*;
pub use models::*;
use reqwest::header::HeaderValue;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    time::{Duration, Instant},
};
pub(crate) use transport::{parse_retry_after, rate_limited};
pub use transport::{ApiError, HttpReply, HttpsTransport, ReadTransport, MAX_RESPONSE_BYTES};
use url::Url;
use zeroize::Zeroize;
pub const MAX_RANGES: usize = 20;
const QUERY_BUDGET: Duration = Duration::from_secs(60);
pub fn validate_google_id(id: &str) -> Result<(), ApiError> {
    if id.is_empty()
        || id.len() > 256
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(ApiError::InvalidInput);
    }
    Ok(())
}
/// No navigation: safe IDs and the documented docs.google.com Sheets shapes.
pub fn parse_spreadsheet_id(input: &str) -> Result<String, ApiError> {
    let input = input.trim();
    if validate_google_id(input).is_ok() {
        return Ok(input.into());
    }
    let url = Url::parse(input).map_err(|_| ApiError::InvalidInput)?;
    if url.scheme() != "https"
        || url.host_str() != Some("docs.google.com")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || input.contains('%')
        || input.contains('\\')
        || input.chars().any(char::is_control)
    {
        return Err(ApiError::InvalidInput);
    }
    let raw_path = input
        .split_once("://")
        .ok_or(ApiError::InvalidInput)?
        .1
        .split_once('/')
        .map(|(_, p)| p)
        .unwrap_or("")
        .split(['?', '#'])
        .next()
        .unwrap_or("");
    if raw_path.split('/').any(|p| p == "." || p == "..") {
        return Err(ApiError::InvalidInput);
    }
    let segments: Vec<_> = url.path_segments().ok_or(ApiError::InvalidInput)?.collect();
    let (id, tail) = match segments.as_slice() {
        ["spreadsheets", "d", id, rest @ ..] => (*id, rest),
        ["spreadsheets", "u", user, "d", id, rest @ ..]
            if !user.is_empty() && user.bytes().all(|b| b.is_ascii_digit()) =>
        {
            (*id, rest)
        }
        _ => return Err(ApiError::InvalidInput),
    };
    if !matches!(
        tail,
        [] | [""] | ["edit"] | ["edit", ""] | ["view"] | ["preview"]
    ) {
        return Err(ApiError::InvalidInput);
    }
    validate_google_id(id)?;
    Ok(id.into())
}
pub struct GoogleApi<T: ReadTransport = HttpsTransport> {
    transport: T,
    authorization: HeaderValue,
    quota: std::sync::Mutex<Option<ApiError>>,
}
impl GoogleApi<HttpsTransport> {
    pub fn new(access_token: String) -> Result<Self, ApiError> {
        Self::with_transport(access_token, HttpsTransport::new()?)
    }
}
impl<T: ReadTransport> GoogleApi<T> {
    /// Trusted fixture seam, never exposed as IPC or caller-controlled endpoints.
    pub fn with_transport(mut access_token: String, transport: T) -> Result<Self, ApiError> {
        let mut bearer = format!("Bearer {access_token}");
        let result = HeaderValue::from_str(&bearer).map_err(|_| ApiError::InvalidInput);
        access_token.zeroize();
        bearer.zeroize();
        let mut authorization = result?;
        authorization.set_sensitive(true);
        Ok(Self {
            transport,
            authorization,
            quota: std::sync::Mutex::new(None),
        })
    }
    pub(crate) fn read<D: DeserializeOwned>(
        &self,
        url: Url,
        deadline: Instant,
    ) -> Result<D, ApiError> {
        if let Some(error) = self.quota.lock().map_err(|_| ApiError::Cancelled)?.as_ref() {
            return Err(error.clone());
        }
        let bytes = transport::get_bounded(&self.transport, &url, &self.authorization, deadline)
            .inspect_err(|error| {
                if error.kind() == "rate_limited" {
                    if let Ok(mut quota) = self.quota.lock() {
                        *quota = Some(error.clone());
                    }
                }
            })?;
        serde_json::from_slice(&bytes).map_err(|_| ApiError::InvalidResponse)
    }
    pub fn sheet_metadata(&self, input: &str) -> Result<SpreadsheetMetadata, ApiError> {
        let id = parse_spreadsheet_id(input)?;
        let mut url = endpoint(
            "https://sheets.googleapis.com",
            &format!("/v4/spreadsheets/{id}"),
        )?;
        url.query_pairs_mut().append_pair("includeGridData","false").append_pair("fields","spreadsheetId,spreadsheetUrl,properties(title,locale,timeZone),sheets(properties(sheetId,title,index,sheetType,gridProperties(rowCount,columnCount),hidden))");
        let raw: RawSpreadsheet = self.read(url, Instant::now() + QUERY_BUDGET)?;
        if raw.spreadsheet_id != id {
            return Err(ApiError::InvalidResponse);
        }
        let mut seen = BTreeSet::new();
        if raw
            .sheets
            .iter()
            .any(|s| !seen.insert(s.properties.sheet_id))
        {
            return Err(ApiError::InvalidResponse);
        }
        Ok(SpreadsheetMetadata {
            spreadsheet_id: raw.spreadsheet_id,
            spreadsheet_url: raw.spreadsheet_url,
            title: raw.properties.title,
            locale: raw.properties.locale,
            time_zone: raw.properties.time_zone,
            sheets: raw
                .sheets
                .into_iter()
                .map(|s| SheetMetadata {
                    sheet_id: s.properties.sheet_id,
                    title: s.properties.title,
                    index: s.properties.index,
                    sheet_type: s.properties.sheet_type,
                    hidden: s.properties.hidden,
                    row_count: s.properties.grid_properties.as_ref().map(|g| g.row_count),
                    column_count: s
                        .properties
                        .grid_properties
                        .as_ref()
                        .map(|g| g.column_count),
                })
                .collect(),
        })
    }
    pub fn sheet_values(&self, id: &str, ranges: &[String]) -> Result<SheetValues, ApiError> {
        validate_google_id(id)?;
        if ranges.is_empty()
            || ranges.len() > MAX_RANGES
            || ranges
                .iter()
                .any(|r| r.trim().is_empty() || r.len() > 256 || r.chars().any(char::is_control))
            || ranges.iter().map(String::len).sum::<usize>() > 3000
        {
            return Err(ApiError::InvalidInput);
        }
        let deadline = Instant::now() + QUERY_BUDGET;
        let mut batches = Vec::with_capacity(2);
        for render in ["UNFORMATTED_VALUE", "FORMULA"] {
            let mut url = endpoint(
                "https://sheets.googleapis.com",
                &format!("/v4/spreadsheets/{id}/values:batchGet"),
            )?;
            for range in ranges {
                url.query_pairs_mut().append_pair("ranges", range);
            }
            url.query_pairs_mut()
                .append_pair("majorDimension", "ROWS")
                .append_pair("valueRenderOption", render)
                .append_pair("dateTimeRenderOption", "SERIAL_NUMBER");
            let batch: RawValues = self.read(url, deadline)?;
            if batch.spreadsheet_id != id || batch.value_ranges.len() != ranges.len() {
                return Err(ApiError::InvalidResponse);
            }
            let values: Vec<ValueRange> = batch
                .value_ranges
                .into_iter()
                .map(|r| ValueRange {
                    range: r.range,
                    major_dimension: r.major_dimension,
                    values: r.values,
                })
                .collect();
            if values.iter().any(|r| {
                r.range.is_empty()
                    || r.major_dimension != "ROWS"
                    || r.values
                        .iter()
                        .flatten()
                        .any(|v| v.is_array() || v.is_object())
            }) {
                return Err(ApiError::InvalidResponse);
            }
            batches.push(values);
        }
        let formulas = batches.pop().ok_or(ApiError::InvalidResponse)?;
        let values = batches.pop().ok_or(ApiError::InvalidResponse)?;
        Ok(SheetValues {
            spreadsheet_id: id.into(),
            values,
            formulas,
        })
    }
    pub fn drive_metadata(&self, id: &str) -> Result<DriveMetadata, ApiError> {
        validate_google_id(id)?;
        let mut url = endpoint(
            "https://www.googleapis.com",
            &format!("/drive/v3/files/{id}"),
        )?;
        url.query_pairs_mut()
            .append_pair(
                "fields",
                "id,name,mimeType,modifiedTime,trashed,webViewLink,version,driveId,resourceKey",
            )
            .append_pair("supportsAllDrives", "true");
        let raw: RawDrive = self.read(url, Instant::now() + QUERY_BUDGET)?;
        if raw.id != id {
            return Err(ApiError::InvalidResponse);
        }
        Ok(DriveMetadata {
            id: raw.id,
            name: raw.name,
            mime_type: raw.mime_type,
            modified_time: raw.modified_time,
            trashed: raw.trashed,
            web_view_link: raw.web_view_link,
            version: raw.version,
            drive_id: raw.drive_id,
            resource_key: raw.resource_key,
        })
    }
    pub fn script_project(&self, id: &str) -> Result<ScriptProject, ApiError> {
        validate_google_id(id)?;
        let deadline = Instant::now() + QUERY_BUDGET;
        let metadata: RawScriptMetadata = self.read(
            endpoint(
                "https://script.googleapis.com",
                &format!("/v1/projects/{id}"),
            )?,
            deadline,
        )?;
        let content: RawScriptContent = self.read(
            endpoint(
                "https://script.googleapis.com",
                &format!("/v1/projects/{id}/content"),
            )?,
            deadline,
        )?;
        if metadata.script_id != id || content.script_id != id {
            return Err(ApiError::InvalidResponse);
        }
        let mut files: Vec<ScriptFile> = content.files.into_iter().map(Into::into).collect();
        let mut seen = BTreeSet::new();
        if files
            .iter()
            .any(|f| f.name.is_empty() || !seen.insert((f.name.clone(), f.file_type.clone())))
        {
            return Err(ApiError::InvalidResponse);
        }
        files.sort_by(|a, b| (&a.name, &a.file_type).cmp(&(&b.name, &b.file_type)));
        let encoded = serde_json::to_vec(&files).map_err(|_| ApiError::InvalidResponse)?;
        let source_hash = format!("{:x}", Sha256::digest(encoded));
        Ok(ScriptProject {
            metadata: ScriptMetadata {
                script_id: metadata.script_id,
                title: metadata.title,
                parent_id: metadata.parent_id,
                create_time: metadata.create_time,
                update_time: metadata.update_time,
            },
            files,
            source_hash,
        })
    }
}
fn endpoint(origin: &str, path: &str) -> Result<Url, ApiError> {
    Url::parse(&format!("{origin}{path}")).map_err(|_| ApiError::InvalidInput)
}
pub mod grid;
#[cfg(test)]
mod tests;
