//! Manual reads only. Session generations guard every local write and response.
use crate::{
    auth::{
        AuthError, AuthManager, AuthorizedSession, IntegrationState, DRIVE_SCOPE, SCRIPT_SCOPE,
        SHEETS_SCOPE,
    },
    database::{Database, IntegrationConfig, IntegrationQueryInput, QueryService},
    google::{
        parse_spreadsheet_id, validate_google_id, ApiError, DriveMetadata, GoogleApi,
        ScriptProject, SheetValues, SpreadsheetMetadata,
    },
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone)]
pub struct IntegrationRuntime {
    pub auth: Arc<AuthManager>,
    pub database_path: PathBuf,
}
#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}
impl From<AuthError> for CommandError {
    fn from(e: AuthError) -> Self {
        Self {
            code: serde_json::to_value(e.code)
                .unwrap_or_default()
                .as_str()
                .unwrap_or("authorization_failed")
                .into(),
            message: e.message.into(),
            retryable: e.retryable,
        }
    }
}
impl From<ApiError> for CommandError {
    fn from(e: ApiError) -> Self {
        Self {
            code: e.kind().into(),
            message: e.to_string(),
            retryable: matches!(
                e,
                ApiError::Network
                    | ApiError::Timeout
                    | ApiError::RateLimited
                    | ApiError::Server { .. }
            ),
        }
    }
}
impl CommandError {
    pub fn local() -> Self {
        Self {
            code: "local_storage".into(),
            message: "Não foi possível salvar os metadados locais da integração.".into(),
            retryable: false,
        }
    }
}
#[derive(Serialize)]
pub struct ServiceResult<T> {
    pub status: &'static str,
    pub data: Option<T>,
    pub error: Option<CommandError>,
}
impl<T> ServiceResult<T> {
    fn unavailable(status: &'static str) -> Self {
        Self {
            status,
            data: None,
            error: None,
        }
    }
    fn result(result: Result<T, ApiError>) -> Self {
        match result {
            Ok(data) => Self {
                status: "success",
                data: Some(data),
                error: None,
            },
            Err(error) => Self {
                status: "error",
                data: None,
                error: Some(error.into()),
            },
        }
    }
}
#[derive(Deserialize)]
pub struct SheetInput {
    pub url: String,
    pub ranges: Vec<String>,
}
#[derive(Deserialize)]
pub struct ScriptInput {
    pub script_id: String,
    pub spreadsheet_id: Option<String>,
}
#[derive(Serialize)]
pub struct SheetAccessResult {
    pub generation: u64,
    pub account_subject: String,
    pub spreadsheet_id: String,
    pub sheets: ServiceResult<SpreadsheetMetadata>,
    pub values: ServiceResult<SheetValues>,
    pub drive: ServiceResult<DriveMetadata>,
}
#[derive(Serialize)]
pub struct ScriptAccessResult {
    pub generation: u64,
    pub account_subject: String,
    pub script: ServiceResult<ScriptProject>,
}
#[derive(Default)]
struct ResourceMetadata<'a> {
    name: Option<&'a str>,
    hash: Option<&'a str>,
    association: Option<String>,
}
impl IntegrationRuntime {
    pub fn persist_state(&self, state: &IntegrationState) -> Result<(), CommandError> {
        // Configuration is public; account writes require a current authenticated generation.
        if let Some(client_id) = &state.client_id {
            let mut db = Database::open(&self.database_path).map_err(|_| CommandError::local())?;
            db.save_integration_config(&IntegrationConfig {
                client_id: client_id.clone(),
                drive_metadata_enabled: state.drive_metadata_opt_in,
            })
            .map_err(|_| CommandError::local())?;
        }
        if let Some(account) = &state.auth.account {
            self.auth.with_generation(state.generation, || {
                let mut db =
                    Database::open(&self.database_path).map_err(|_| CommandError::local())?;
                db.upsert_account(
                    &account.subject,
                    account.email.as_deref(),
                    account.name.as_deref(),
                )
                .map_err(|_| CommandError::local())?;
                Ok::<_, CommandError>(())
            })??;
        }
        Ok(())
    }
    fn read<T>(
        &self,
        session: &mut AuthorizedSession,
        scope: &str,
        operation: impl Fn(&GoogleApi) -> Result<T, ApiError>,
    ) -> Result<Result<T, ApiError>, CommandError> {
        if !session.has_scope(scope) {
            return Ok(Err(ApiError::Forbidden));
        }
        let result = operation(&GoogleApi::new(session.access_token().to_owned())?);
        if matches!(result, Err(ApiError::Unauthorized)) {
            *session = self.auth.force_refresh(session)?;
            if !session.has_scope(scope) {
                return Ok(Err(ApiError::Forbidden));
            }
            return Ok(operation(&GoogleApi::new(
                session.access_token().to_owned(),
            )?));
        }
        Ok(result)
    }
    fn record<T>(
        &self,
        db: &mut Database,
        session: &AuthorizedSession,
        service: QueryService,
        id: &str,
        result: &Result<T, ApiError>,
        metadata: ResourceMetadata<'_>,
    ) -> Result<(), CommandError> {
        db.record_query(
            &session.account.subject,
            &IntegrationQueryInput {
                service,
                resource_id: id.into(),
                resource_name: metadata.name.map(str::to_owned),
                associated_spreadsheet_id: metadata.association,
                source_hash: metadata.hash.map(str::to_owned),
                error: result.as_ref().err().cloned(),
            },
        )
        .map_err(|_| CommandError::local())
    }
    pub fn sheet(&self, input: SheetInput) -> Result<SheetAccessResult, CommandError> {
        let id = parse_spreadsheet_id(&input.url)?;
        let mut session = self.auth.authorized()?;
        let sheets = if session.has_scope(SHEETS_SCOPE) {
            Some(self.read(&mut session, SHEETS_SCOPE, |api| api.sheet_metadata(&id))?)
        } else {
            None
        };
        let values = if session.has_scope(SHEETS_SCOPE) {
            Some(self.read(&mut session, SHEETS_SCOPE, |api| {
                api.sheet_values(&id, &input.ranges)
            })?)
        } else {
            None
        };
        let drive_requested = self.auth.state()?.drive_metadata_opt_in;
        let drive = if drive_requested && session.has_scope(DRIVE_SCOPE) {
            Some(self.read(&mut session, DRIVE_SCOPE, |api| api.drive_metadata(&id))?)
        } else {
            None
        };
        self.auth.with_current(&session, || {
            let mut db = Database::open(&self.database_path).map_err(|_| CommandError::local())?;
            db.upsert_account(
                &session.account.subject,
                session.account.email.as_deref(),
                session.account.name.as_deref(),
            )
            .map_err(|_| CommandError::local())?;
            if let Some(result) = &sheets {
                self.record(
                    &mut db,
                    &session,
                    QueryService::SheetsMetadata,
                    &id,
                    result,
                    ResourceMetadata {
                        name: result.as_ref().ok().map(|s| s.title.as_str()),
                        ..Default::default()
                    },
                )?;
            }
            if let Some(result) = &values {
                self.record(
                    &mut db,
                    &session,
                    QueryService::SheetsValues,
                    &id,
                    result,
                    ResourceMetadata::default(),
                )?;
            }
            if let Some(result) = &drive {
                self.record(
                    &mut db,
                    &session,
                    QueryService::Drive,
                    &id,
                    result,
                    ResourceMetadata {
                        name: result.as_ref().ok().map(|s| s.name.as_str()),
                        ..Default::default()
                    },
                )?;
            }
            Ok(SheetAccessResult {
                generation: session.generation,
                account_subject: session.account.subject.clone(),
                spreadsheet_id: id,
                sheets: sheets
                    .map(ServiceResult::result)
                    .unwrap_or_else(|| ServiceResult::unavailable("not_authorized")),
                values: values
                    .map(ServiceResult::result)
                    .unwrap_or_else(|| ServiceResult::unavailable("not_authorized")),
                drive: drive.map(ServiceResult::result).unwrap_or_else(|| {
                    ServiceResult::unavailable(if drive_requested {
                        "not_authorized"
                    } else {
                        "not_requested"
                    })
                }),
            })
        })?
    }
    pub fn script(&self, input: ScriptInput) -> Result<ScriptAccessResult, CommandError> {
        validate_google_id(&input.script_id)?;
        if let Some(id) = &input.spreadsheet_id {
            validate_google_id(id)?;
        }
        let mut session = self.auth.authorized()?;
        let result = if session.has_scope(SCRIPT_SCOPE) {
            Some(self.read(&mut session, SCRIPT_SCOPE, |api| {
                api.script_project(&input.script_id)
            })?)
        } else {
            None
        };
        self.auth.with_current(&session, || {
            let mut db = Database::open(&self.database_path).map_err(|_| CommandError::local())?;
            db.upsert_account(
                &session.account.subject,
                session.account.email.as_deref(),
                session.account.name.as_deref(),
            )
            .map_err(|_| CommandError::local())?;
            if let Some(result) = &result {
                self.record(
                    &mut db,
                    &session,
                    QueryService::AppsScript,
                    &input.script_id,
                    result,
                    ResourceMetadata {
                        name: result.as_ref().ok().map(|s| s.metadata.title.as_str()),
                        hash: result.as_ref().ok().map(|s| s.source_hash.as_str()),
                        association: input.spreadsheet_id,
                    },
                )?;
            }
            Ok(ScriptAccessResult {
                generation: session.generation,
                account_subject: session.account.subject.clone(),
                script: result
                    .map(ServiceResult::result)
                    .unwrap_or_else(|| ServiceResult::unavailable("not_authorized")),
            })
        })?
    }
}

#[cfg(test)]
mod tests;

impl IntegrationRuntime {
    /// Metadata discovery is independent of registration and does not fetch cells.
    pub fn discover_spreadsheets(
        &self,
        input: crate::google::DiscoveryQuery,
    ) -> Result<crate::integrity::runtime::EngineEnvelope<crate::google::DiscoveryPage>, CommandError>
    {
        let requested = self.auth.state()?;
        if !requested.drive_metadata_opt_in {
            return Err(
                crate::auth::AuthError::new(crate::auth::AuthCode::InsufficientScope).into(),
            );
        }
        let mut session = self.auth.authorized()?;
        if session.generation != requested.generation {
            return Err(
                crate::auth::AuthError::new(crate::auth::AuthCode::OperationSuperseded).into(),
            );
        }
        let page = self.read(&mut session, DRIVE_SCOPE, |api| {
            api.discover_spreadsheets(&input)
        })??;
        self.auth.with_current(&session, || {
            Ok(crate::integrity::runtime::EngineEnvelope {
                generation: session.generation,
                account_subject: session.account.subject.clone(),
                data: page,
            })
        })?
    }
}

impl IntegrationRuntime {
    pub fn open_spreadsheet(
        &self,
        input: crate::management::SpreadsheetQuery,
    ) -> Result<crate::integrity::runtime::EngineEnvelope<()>, CommandError> {
        use crate::auth::OAuthTransport;
        let result = self.get_spreadsheet_diagnostics(input)?;
        let url =
            url::Url::parse(&result.data.spreadsheet.url).map_err(|_| CommandError::local())?;
        self.auth.with_generation(result.generation, || {
            crate::auth::HttpsOAuthTransport::new()?.open_browser(&url)?;
            Ok(crate::integrity::runtime::EngineEnvelope {
                generation: result.generation,
                account_subject: result.account_subject,
                data: (),
            })
        })?
    }
}
