use super::{database_error, Database};
use crate::google::{validate_google_id, ApiError};
use rusqlite::{params, OptionalExtension, TransactionBehavior};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationConfig {
    pub client_id: String,
    pub drive_metadata_enabled: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountMetadata {
    pub subject: String,
    pub email: Option<String>,
    pub name: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryService {
    SheetsMetadata,
    SheetsValues,
    Drive,
    AppsScript,
}
impl QueryService {
    fn key(&self) -> &'static str {
        match self {
            Self::SheetsMetadata => "sheets_metadata",
            Self::SheetsValues => "sheets_values",
            Self::Drive => "drive",
            Self::AppsScript => "apps_script",
        }
    }
    fn resource_kind(&self) -> &'static str {
        match self {
            Self::SheetsMetadata | Self::SheetsValues => "spreadsheet",
            Self::Drive => "file",
            Self::AppsScript => "script",
        }
    }
}
#[derive(Debug, Clone)]
pub struct IntegrationQueryInput {
    pub service: QueryService,
    pub resource_id: String,
    pub resource_name: Option<String>,
    pub associated_spreadsheet_id: Option<String>,
    pub source_hash: Option<String>,
    pub error: Option<ApiError>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationQuery {
    pub service: String,
    pub resource_id: String,
    pub resource_name: Option<String>,
    pub status: String,
    pub error_kind: Option<String>,
    pub queried_at: String,
    pub source_hash: Option<String>,
    pub associated_spreadsheet_ids: Vec<String>,
    pub is_demo: bool,
}
fn local_id(prefix: &str, raw: &str) -> String {
    format!("{prefix}-{:x}", Sha256::digest(raw.as_bytes()))
}
fn clean_metadata(value: Option<&str>, max: usize) -> Result<Option<String>, String> {
    match value {
        None => Ok(None),
        Some(text) if text.chars().count() <= max && !text.chars().any(char::is_control) => {
            Ok(Some(text.into()))
        }
        _ => Err("invalid_account_metadata".into()),
    }
}
impl Database {
    pub fn integration_config(&self) -> Result<Option<IntegrationConfig>, String> {
        self.connection
            .query_row(
                "SELECT client_id,drive_metadata_enabled FROM integration_config WHERE id=1",
                [],
                |r| {
                    Ok(IntegrationConfig {
                        client_id: r.get(0)?,
                        drive_metadata_enabled: r.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(database_error)
    }
    pub fn save_integration_config(&mut self, config: &IntegrationConfig) -> Result<(), String> {
        if config.client_id.len() > 256
            || !config.client_id.ends_with(".apps.googleusercontent.com")
            || !config
                .client_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        {
            return Err("invalid_desktop_client_id".into());
        }
        self.connection.execute("INSERT INTO integration_config(id,client_id,drive_metadata_enabled,updated_at) VALUES(1,?1,?2,strftime('%Y-%m-%dT%H:%M:%fZ','now')) ON CONFLICT(id) DO UPDATE SET client_id=excluded.client_id,drive_metadata_enabled=excluded.drive_metadata_enabled,updated_at=excluded.updated_at",params![config.client_id,config.drive_metadata_enabled]).map_err(database_error)?;
        Ok(())
    }
    pub fn upsert_account(
        &mut self,
        subject: &str,
        email: Option<&str>,
        name: Option<&str>,
    ) -> Result<String, String> {
        validate_google_id(subject).map_err(|_| "invalid_account_subject")?;
        let email = clean_metadata(email, 512)?;
        let name = clean_metadata(name, 1024)?;
        let id = local_id("google", subject);
        self.connection.execute("INSERT INTO accounts(id,provider_subject,display_name,is_demo,created_at,email,last_connected_at) VALUES(?1,?2,?3,0,strftime('%Y-%m-%dT%H:%M:%fZ','now'),?4,strftime('%Y-%m-%dT%H:%M:%fZ','now')) ON CONFLICT(id) DO UPDATE SET display_name=excluded.display_name,email=excluded.email,last_connected_at=excluded.last_connected_at",params![id,subject,name.as_deref().unwrap_or("Conta Google"),email]).map_err(database_error)?;
        Ok(id)
    }
    pub fn integration_account(&self, subject: &str) -> Result<Option<AccountMetadata>, String> {
        self.connection.query_row("SELECT provider_subject,email,display_name FROM accounts WHERE provider_subject=?1 AND is_demo=0",[subject],|r|Ok(AccountMetadata{subject:r.get(0)?,email:r.get(1)?,name:r.get(2)?})).optional().map_err(database_error)
    }
    pub fn record_query(
        &mut self,
        subject: &str,
        input: &IntegrationQueryInput,
    ) -> Result<(), String> {
        validate_google_id(&input.resource_id).map_err(|_| "invalid_resource_id")?;
        if input
            .source_hash
            .as_ref()
            .is_some_and(|h| h.len() != 64 || !h.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err("invalid_source_hash".into());
        }
        if input.source_hash.is_some() && input.service != QueryService::AppsScript {
            return Err("source_hash_requires_script".into());
        }
        let name = clean_metadata(input.resource_name.as_deref(), 1024)?;
        if input.associated_spreadsheet_id.is_some() && input.service != QueryService::AppsScript {
            return Err("association_requires_script".into());
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(database_error)?;
        let account_id: Option<String> = transaction
            .query_row(
                "SELECT id FROM accounts WHERE provider_subject=?1 AND is_demo=0",
                [subject],
                |r| r.get(0),
            )
            .optional()
            .map_err(database_error)?;
        let account_id = account_id.ok_or("account_not_found")?;
        let resource_id = local_id(
            "resource",
            &format!(
                "{account_id}:{}:{}",
                input.service.resource_kind(),
                input.resource_id
            ),
        );
        let error = input.error.as_ref().map(ApiError::kind);
        let status = if error.is_some() { "error" } else { "success" };
        transaction.execute("INSERT INTO integration_resources(id,account_id,kind,external_id,name,source_hash,last_status,last_error_kind,last_queried_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,strftime('%Y-%m-%dT%H:%M:%fZ','now')) ON CONFLICT(account_id,kind,external_id) DO UPDATE SET name=COALESCE(excluded.name,integration_resources.name),source_hash=COALESCE(excluded.source_hash,integration_resources.source_hash),last_status=excluded.last_status,last_error_kind=excluded.last_error_kind,last_queried_at=excluded.last_queried_at",params![resource_id,account_id,input.service.resource_kind(),input.resource_id,name,input.source_hash,status,error]).map_err(database_error)?;
        let query_id = local_id("query", &format!("{resource_id}:{}", input.service.key()));
        transaction.execute("INSERT INTO integration_queries(id,account_id,resource_id,service,status,error_kind,queried_at) VALUES(?1,?2,?3,?4,?5,?6,strftime('%Y-%m-%dT%H:%M:%fZ','now')) ON CONFLICT(account_id,service,resource_id) DO UPDATE SET status=excluded.status,error_kind=excluded.error_kind,queried_at=excluded.queried_at",params![query_id,account_id,resource_id,input.service.key(),status,error]).map_err(database_error)?;
        if let Some(sheet_id) = &input.associated_spreadsheet_id {
            validate_google_id(sheet_id).map_err(|_| "invalid_associated_spreadsheet")?;
            let sheet_resource:Option<String>=transaction.query_row("SELECT r.id FROM integration_resources r JOIN integration_queries q ON q.resource_id=r.id AND q.account_id=r.account_id WHERE r.account_id=?1 AND r.kind='spreadsheet' AND r.external_id=?2 AND q.service='sheets_metadata' AND q.status='success'",params![account_id,sheet_id],|r|r.get(0)).optional().map_err(database_error)?;
            let sheet_resource =
                sheet_resource.ok_or("associated_spreadsheet_not_consulted_by_account")?;
            // Only an explicitly requested, successful script read creates a link.
            if input.error.is_none() {
                transaction.execute("INSERT INTO integration_associations(account_id,spreadsheet_resource_id,script_resource_id,created_at) VALUES(?1,?2,?3,strftime('%Y-%m-%dT%H:%M:%fZ','now')) ON CONFLICT DO NOTHING",params![account_id,sheet_resource,resource_id]).map_err(database_error)?;
            }
        }
        transaction.commit().map_err(database_error)
    }
    pub fn integration_queries(&self, subject: &str) -> Result<Vec<IntegrationQuery>, String> {
        let mut statement=self.connection.prepare("SELECT q.service,r.external_id,r.name,q.status,q.error_kind,q.queried_at,r.source_hash,r.id FROM integration_queries q JOIN accounts a ON a.id=q.account_id JOIN integration_resources r ON r.id=q.resource_id AND r.account_id=q.account_id WHERE a.provider_subject=?1 AND a.is_demo=0 ORDER BY q.queried_at DESC,q.id").map_err(database_error)?;
        let rows = statement
            .query_map([subject], |r| {
                Ok((
                    IntegrationQuery {
                        service: r.get(0)?,
                        resource_id: r.get(1)?,
                        resource_name: r.get(2)?,
                        status: r.get(3)?,
                        error_kind: r.get(4)?,
                        queried_at: r.get(5)?,
                        source_hash: r.get(6)?,
                        associated_spreadsheet_ids: Vec::new(),
                        is_demo: false,
                    },
                    r.get::<_, String>(7)?,
                ))
            })
            .map_err(database_error)?;
        let mut result = Vec::new();
        for row in rows {
            let (mut query, id) = row.map_err(database_error)?;
            let mut links=self.connection.prepare("SELECT s.external_id FROM integration_associations l JOIN integration_resources s ON s.id=l.spreadsheet_resource_id AND s.account_id=l.account_id WHERE l.script_resource_id=?1 ORDER BY s.external_id").map_err(database_error)?;
            query.associated_spreadsheet_ids = links
                .query_map([id], |r| r.get(0))
                .map_err(database_error)?
                .collect::<Result<_, _>>()
                .map_err(database_error)?;
            result.push(query);
        }
        Ok(result)
    }
}

#[cfg(test)]
#[path = "integration_tests.rs"]
mod tests;
