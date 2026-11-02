use crate::{
    auth::{AuthCode, AuthError, SHEETS_SCOPE},
    database::{Database, IntegrationQueryInput, QueryService},
    google::{parse_spreadsheet_id, ApiError, GoogleApi},
    integration::{CommandError, IntegrationRuntime},
    integrity::runtime::{BaselineSummary, CaptureInput, EngineEnvelope, ValidationResult},
    integrity::{fingerprint, BaselineVersion},
    management::*,
};
fn error(code: &str, message: &str) -> CommandError {
    CommandError {
        code: code.into(),
        message: message.into(),
        retryable: false,
    }
}
impl IntegrationRuntime {
    fn management_local<T>(
        &self,
        expected: Option<u64>,
        operation: impl FnOnce(&mut Database, &str) -> Result<T, CommandError>,
    ) -> Result<EngineEnvelope<T>, CommandError> {
        let state = self.auth.state()?;
        let account = state
            .auth
            .account
            .ok_or_else(|| AuthError::new(AuthCode::ReconnectRequired))?;
        if expected.is_some_and(|g| g != state.generation) {
            return Err(AuthError::new(AuthCode::OperationSuperseded).into());
        }
        self.auth.with_generation(state.generation, || {
            let mut db = Database::open(&self.database_path).map_err(|_| CommandError::local())?;
            db.upsert_account(
                &account.subject,
                account.email.as_deref(),
                account.name.as_deref(),
            )
            .map_err(|_| CommandError::local())?;
            operation(&mut db, &account.subject).map(|data| EngineEnvelope {
                generation: state.generation,
                account_subject: account.subject,
                data,
            })
        })?
    }
    pub fn list_registered_spreadsheets(
        &self,
    ) -> Result<EngineEnvelope<Vec<RegisteredSpreadsheet>>, CommandError> {
        self.management_local(None, |db, s| db.registered_spreadsheets(s))
    }
    pub fn get_dashboard_summary(&self) -> Result<EngineEnvelope<DashboardSummary>, CommandError> {
        self.management_local(None, |db, s| db.operational_dashboard(s))
    }
    pub fn register_spreadsheet(
        &self,
        input: RegisterInput,
    ) -> Result<EngineEnvelope<RegisteredSpreadsheet>, CommandError> {
        let id = parse_spreadsheet_id(&input.url)?;
        let mut session = self.auth.authorized()?;
        if session.generation != input.expected_generation {
            return Err(AuthError::new(AuthCode::OperationSuperseded).into());
        }
        if !session.has_scope(SHEETS_SCOPE) {
            return Err(AuthError::new(AuthCode::InsufficientScope).into());
        }
        let mut metadata = GoogleApi::new(session.access_token().into())?.sheet_metadata(&id);
        if matches!(metadata, Err(ApiError::Unauthorized)) {
            session = self.auth.force_refresh(&session)?;
            metadata = GoogleApi::new(session.access_token().into())?.sheet_metadata(&id);
        }
        let metadata = metadata?;
        self.management_local(Some(session.generation), |db, s| {
            db.record_query(
                s,
                &IntegrationQueryInput {
                    service: QueryService::SheetsMetadata,
                    resource_id: id,
                    resource_name: Some(metadata.title.clone()),
                    source_hash: None,
                    associated_spreadsheet_id: None,
                    error: None,
                },
            )
            .map_err(|_| CommandError::local())?;
            db.register_operational_spreadsheet(s, &metadata)
        })
    }
    pub fn list_spreadsheet_tabs(
        &self,
        input: SpreadsheetQuery,
    ) -> Result<EngineEnvelope<RegisteredSpreadsheet>, CommandError> {
        let existing = self.management_local(None, |db, s| {
            db.registered_spreadsheet(s, &input.spreadsheet_id)
        })?;
        let mut session = self.auth.authorized()?;
        if session.generation != existing.generation {
            return Err(AuthError::new(AuthCode::OperationSuperseded).into());
        }
        if !session.has_scope(SHEETS_SCOPE) {
            return Err(AuthError::new(AuthCode::InsufficientScope).into());
        }
        let mut metadata =
            GoogleApi::new(session.access_token().into())?.sheet_metadata(&input.spreadsheet_id);
        if matches!(metadata, Err(ApiError::Unauthorized)) {
            session = self.auth.force_refresh(&session)?;
            metadata = GoogleApi::new(session.access_token().into())?
                .sheet_metadata(&input.spreadsheet_id);
        }
        self.management_local(Some(session.generation), |db, s| match metadata {
            Ok(metadata) => db.refresh_registered_metadata(s, &metadata),
            Err(e) => db.record_registry_access_error(s, &input.spreadsheet_id, e.kind()),
        })
    }
    pub fn remove_spreadsheet(
        &self,
        input: RemoveInput,
    ) -> Result<EngineEnvelope<()>, CommandError> {
        self.management_local(Some(input.expected_generation), |db, s| {
            db.remove_registered_spreadsheet(s, &input)
        })
    }
    pub fn pause_spreadsheet(
        &self,
        input: Mutation,
    ) -> Result<EngineEnvelope<RegisteredSpreadsheet>, CommandError> {
        self.management_local(Some(input.expected_generation), |db, s| {
            db.set_registered_paused(s, &input, true)
        })
    }
    pub fn resume_spreadsheet(
        &self,
        input: Mutation,
    ) -> Result<EngineEnvelope<RegisteredSpreadsheet>, CommandError> {
        self.management_local(Some(input.expected_generation), |db, s| {
            db.set_registered_paused(s, &input, false)
        })
    }
    pub fn update_monitored_tabs(
        &self,
        input: TabsInput,
    ) -> Result<EngineEnvelope<RegisteredSpreadsheet>, CommandError> {
        self.management_local(Some(input.expected_generation), |db, s| {
            db.set_registered_tabs(s, &input.mutation(), &input.sheet_ids)
        })
    }
    pub fn update_integrity_rules(
        &self,
        input: RulesInput,
    ) -> Result<EngineEnvelope<RegisteredSpreadsheet>, CommandError> {
        self.management_local(Some(input.expected_generation), |db, s| {
            db.set_registered_rules(s, &input.mutation(), &input.scope)
        })
    }
    pub fn preview_baseline(
        &self,
        input: Mutation,
    ) -> Result<EngineEnvelope<OperationalPreview>, CommandError> {
        let row = self
            .management_local(Some(input.expected_generation), |db, s| {
                db.check_registry_mutation(s, &input)
            })?
            .data;
        let scope = row.scope.ok_or_else(|| {
            error(
                "configuration_pending",
                "Configure as regras antes de capturar a referência.",
            )
        })?;
        if scope
            .sheets
            .iter()
            .map(|s| s.sheet_id)
            .collect::<std::collections::BTreeSet<_>>()
            != row.selected_sheet_ids.iter().copied().collect()
        {
            return Err(error(
                "configuration_pending",
                "Salve regras para todas as abas selecionadas.",
            ));
        }
        let capture = self.capture_integrity_snapshot(CaptureInput {
            url: row.url,
            scope,
        })?;
        self.management_local(Some(input.expected_generation), |db, s| {
            db.check_registry_mutation(s, &input)?;
            Ok(OperationalPreview {
                revision: input.expected_revision,
                preview: capture.data,
            })
        })
    }
    pub fn approve_baseline(
        &self,
        input: ApprovalInput,
    ) -> Result<EngineEnvelope<BaselineVersion>, CommandError> {
        self.management_local(Some(input.expected_generation), |db, s| {
            let row = db.check_registry_mutation(s, &input.mutation())?;
            let snapshot = db.structural_snapshot(s, &input.snapshot_id)?;
            if snapshot.spreadsheet_id != row.spreadsheet_id
                || row.scope_hash.as_deref() != Some(&snapshot.scope_hash)
                || fingerprint(&snapshot) != input.snapshot_fingerprint
            {
                return Err(error(
                    "configuration_conflict",
                    "A prévia não corresponde à configuração atual.",
                ));
            }
            Ok(db.approve_registered_structural_baseline(
                s,
                &input.snapshot_id,
                input.expected_version,
                input.confirmed,
                "human:native",
                Some(input.expected_revision),
            )?)
        })
    }
    pub fn run_spreadsheet_check(
        &self,
        input: Mutation,
    ) -> Result<EngineEnvelope<ValidationResult>, CommandError> {
        let row = self
            .management_local(Some(input.expected_generation), |db, s| {
                db.check_registry_mutation(s, &input)
            })?
            .data;
        if row.paused || row.baseline_version.is_none() || row.status == "configuration_pending" {
            return Err(error(
                "configuration_pending",
                "Retome o cadastro e aprove uma baseline para verificar.",
            ));
        }
        let scope = row.scope.ok_or_else(|| {
            error(
                "configuration_pending",
                "Configure a proteção desta planilha.",
            )
        })?;
        let capture = self.capture_integrity_snapshot(CaptureInput {
            url: row.url,
            scope,
        })?;
        self.management_local(Some(input.expected_generation), |db, s| {
            db.check_registry_mutation(s, &input)
        })?;
        self.validate_integrity_snapshot_registered(
            capture.data.snapshot.id,
            true,
            Some(input.expected_revision),
        )
    }
    pub fn get_spreadsheet_diagnostics(
        &self,
        input: SpreadsheetQuery,
    ) -> Result<EngineEnvelope<Diagnostics>, CommandError> {
        self.management_local(None, |db, s| {
            db.operational_diagnostics(s, &input.spreadsheet_id)
        })
    }
    pub fn list_change_events(
        &self,
        input: EventFilter,
    ) -> Result<EngineEnvelope<Page<Activity>>, CommandError> {
        self.management_local(None, |db, s| db.operational_events(s, &input))
    }
    pub fn list_associated_scripts(
        &self,
        input: SpreadsheetQuery,
    ) -> Result<EngineEnvelope<Vec<ScriptSummary>>, CommandError> {
        self.management_local(None, |db, s| {
            db.operational_scripts(s, &input.spreadsheet_id)
        })
    }
    pub fn list_baseline_versions(
        &self,
        input: BaselinesQuery,
    ) -> Result<EngineEnvelope<Page<BaselineSummary>>, CommandError> {
        self.management_local(None, |db, s| db.operational_baselines(s, &input))
    }
    pub fn compare_baseline_versions(
        &self,
        input: CompareBaselinesInput,
    ) -> Result<EngineEnvelope<BaselineComparison>, CommandError> {
        self.management_local(None, |db, s| {
            let before =
                db.operational_baseline_by_id(s, &input.spreadsheet_id, &input.before_id)?;
            let after = db.operational_baseline_by_id(s, &input.spreadsheet_id, &input.after_id)?;
            let mut differences = Vec::new();
            let mut truncated = false;
            diff_json(
                "regras",
                Some(
                    &serde_json::to_value(&before.snapshot.scope)
                        .map_err(|_| CommandError::local())?,
                ),
                Some(
                    &serde_json::to_value(&after.snapshot.scope)
                        .map_err(|_| CommandError::local())?,
                ),
                &mut differences,
                &mut truncated,
            );
            diff_json(
                "estrutura",
                Some(
                    &serde_json::to_value(&before.snapshot.sheets)
                        .map_err(|_| CommandError::local())?,
                ),
                Some(
                    &serde_json::to_value(&after.snapshot.sheets)
                        .map_err(|_| CommandError::local())?,
                ),
                &mut differences,
                &mut truncated,
            );
            diff_json(
                "scripts",
                Some(
                    &serde_json::to_value(&before.snapshot.scripts)
                        .map_err(|_| CommandError::local())?,
                ),
                Some(
                    &serde_json::to_value(&after.snapshot.scripts)
                        .map_err(|_| CommandError::local())?,
                ),
                &mut differences,
                &mut truncated,
            );
            Ok(BaselineComparison {
                scope_changed: before.snapshot.scope_hash != after.snapshot.scope_hash,
                before: before.into(),
                after: after.into(),
                differences,
                truncated,
            })
        })
    }
    pub fn get_management_preferences(&self) -> Result<EngineEnvelope<Preferences>, CommandError> {
        self.management_local(None, |db, s| db.management_preferences(s))
    }
    pub fn save_management_preferences(
        &self,
        input: PreferencesInput,
    ) -> Result<EngineEnvelope<Preferences>, CommandError> {
        self.management_local(Some(input.expected_generation), |db, s| {
            db.save_management_preferences(s, &input.preferences)
        })
    }
}
fn diff_json(
    path: &str,
    before: Option<&serde_json::Value>,
    after: Option<&serde_json::Value>,
    out: &mut Vec<BaselineDifference>,
    truncated: &mut bool,
) {
    if before == after {
        return;
    }
    if out.len() >= 200 {
        *truncated = true;
        return;
    }
    match (before, after) {
        (Some(serde_json::Value::Object(a)), Some(serde_json::Value::Object(b))) => {
            let keys = a
                .keys()
                .chain(b.keys())
                .collect::<std::collections::BTreeSet<_>>();
            for key in keys {
                diff_json(
                    &format!("{path}.{key}"),
                    a.get(key),
                    b.get(key),
                    out,
                    truncated,
                );
            }
        }
        (Some(serde_json::Value::Array(a)), Some(serde_json::Value::Array(b))) => {
            for i in 0..a.len().max(b.len()) {
                diff_json(&format!("{path}[{i}]"), a.get(i), b.get(i), out, truncated);
            }
        }
        _ => {
            let text = |v: Option<&serde_json::Value>| {
                v.map(|v| {
                    let s = v.to_string();
                    if s.chars().count() > 1024 {
                        s.chars().take(1024).collect::<String>() + "…"
                    } else {
                        s
                    }
                })
            };
            out.push(BaselineDifference {
                path: path.into(),
                before: text(before),
                after: text(after),
            });
        }
    }
}
