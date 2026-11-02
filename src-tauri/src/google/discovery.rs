//! Bounded metadata-only Drive discovery. Each request fetches exactly one page;
//! registration and the local 30-workbook limit are separate operations.
use super::{endpoint, validate_google_id, ApiError, GoogleApi, ReadTransport, QUERY_BUDGET};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, time::Instant};

const SHEET_MIME: &str = "application/vnd.google-apps.spreadsheet";
const MAX_TOKEN_BYTES: usize = 4096;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryOrder {
    #[default]
    Name,
    NameDesc,
    ModifiedDesc,
    ModifiedAsc,
}
impl DiscoveryOrder {
    fn drive_key(self) -> &'static str {
        match self {
            Self::Name => "name_natural",
            Self::NameDesc => "name_natural desc",
            Self::ModifiedDesc => "modifiedTime desc,name_natural",
            Self::ModifiedAsc => "modifiedTime,name_natural",
        }
    }
}
fn default_page_size() -> u16 {
    50
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryQuery {
    #[serde(default)]
    pub search: String,
    pub page_token: Option<String>,
    #[serde(default)]
    pub order: DiscoveryOrder,
    pub drive_id: Option<String>,
    #[serde(default = "default_page_size")]
    pub page_size: u16,
}
impl Default for DiscoveryQuery {
    fn default() -> Self {
        Self {
            search: String::new(),
            page_token: None,
            order: DiscoveryOrder::default(),
            drive_id: None,
            page_size: default_page_size(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredSpreadsheet {
    pub id: String,
    pub name: String,
    pub mime_type: String,
    pub modified_time: Option<String>,
    pub shared: bool,
    pub drive_id: Option<String>,
    pub resource_key: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct DiscoveryPage {
    pub files: Vec<DiscoveredSpreadsheet>,
    pub next_page_token: Option<String>,
    pub incomplete_search: bool,
    pub complete: bool,
    pub corpus: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPage {
    files: Vec<RawFile>,
    next_page_token: Option<String>,
    #[serde(default)]
    incomplete_search: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFile {
    id: String,
    name: String,
    mime_type: String,
    modified_time: Option<String>,
    #[serde(default)]
    shared: bool,
    #[serde(default)]
    trashed: bool,
    drive_id: Option<String>,
    resource_key: Option<String>,
}
fn valid_token(token: &str) -> bool {
    !token.is_empty() && token.len() <= MAX_TOKEN_BYTES && !token.chars().any(char::is_control)
}
impl<T: ReadTransport> GoogleApi<T> {
    pub fn discover_spreadsheets(&self, query: &DiscoveryQuery) -> Result<DiscoveryPage, ApiError> {
        if !(1..=100).contains(&query.page_size)
            || query.search.len() > 256
            || query.search.chars().any(char::is_control)
            || query.page_token.as_deref().is_some_and(|t| !valid_token(t))
        {
            return Err(ApiError::InvalidInput);
        }
        if let Some(id) = &query.drive_id {
            validate_google_id(id)?;
        }
        let mut q = format!("mimeType = '{SHEET_MIME}' and trashed = false");
        if !query.search.trim().is_empty() {
            // Only the application controls q operators; user search is a Drive
            // string literal, then URL query serialization performs encoding.
            let escaped = query
                .search
                .trim()
                .replace('\\', "\\\\")
                .replace('\'', "\\'");
            q.push_str(&format!(" and name contains '{escaped}'"));
        }
        let corpus = if query.drive_id.is_some() {
            "drive"
        } else {
            "allDrives"
        };
        let mut url = endpoint("https://www.googleapis.com", "/drive/v3/files")?;
        url.query_pairs_mut()
            .append_pair("q", &q)
            .append_pair("spaces", "drive")
            .append_pair("corpora", corpus)
            .append_pair("supportsAllDrives", "true")
            .append_pair("includeItemsFromAllDrives", "true")
            .append_pair("pageSize", &query.page_size.to_string())
            .append_pair("orderBy", query.order.drive_key())
            .append_pair("fields", "nextPageToken,incompleteSearch,files(id,name,mimeType,modifiedTime,shared,trashed,driveId,resourceKey)");
        if let Some(token) = &query.page_token {
            url.query_pairs_mut().append_pair("pageToken", token);
        }
        if let Some(id) = &query.drive_id {
            url.query_pairs_mut().append_pair("driveId", id);
        }
        let raw: RawPage = self.read(url, Instant::now() + QUERY_BUDGET)?;
        if raw.files.len() > usize::from(query.page_size)
            || raw
                .next_page_token
                .as_deref()
                .is_some_and(|t| !valid_token(t))
            || (raw.next_page_token.is_some() && raw.next_page_token == query.page_token)
        {
            return Err(ApiError::InvalidResponse);
        }
        let mut seen = BTreeSet::new();
        let mut files = Vec::with_capacity(raw.files.len());
        for f in raw.files {
            if validate_google_id(&f.id).is_err()
                || !seen.insert(f.id.clone())
                || f.name.is_empty()
                || f.name.len() > 8192
                || f.mime_type != SHEET_MIME
                || f.trashed
                || f.drive_id
                    .as_deref()
                    .is_some_and(|d| validate_google_id(d).is_err())
                || f.resource_key.as_deref().is_some_and(|k| !valid_token(k))
                || f.modified_time
                    .as_ref()
                    .is_some_and(|v| chrono::DateTime::parse_from_rfc3339(v).is_err())
            {
                return Err(ApiError::InvalidResponse);
            }
            files.push(DiscoveredSpreadsheet {
                id: f.id,
                name: f.name,
                mime_type: f.mime_type,
                modified_time: f.modified_time,
                shared: f.shared || f.drive_id.is_some(),
                drive_id: f.drive_id,
                resource_key: f.resource_key,
            });
        }
        Ok(DiscoveryPage {
            complete: raw.next_page_token.is_none() && !raw.incomplete_search,
            files,
            next_page_token: raw.next_page_token,
            incomplete_search: raw.incomplete_search,
            corpus: corpus.into(),
        })
    }
}

#[cfg(test)]
mod tests;
