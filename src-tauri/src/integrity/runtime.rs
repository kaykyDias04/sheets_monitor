//! Authenticated local operations; no remote mutations or automatic approvals.
use crate::{
    auth::{AuthorizedSession, SHEETS_SCOPE},
    database::Database,
    google::{parse_spreadsheet_id, GoogleApi},
    integration::{CommandError, IntegrationRuntime},
    integrity::*,
};
use serde::{Deserialize, Serialize};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureInput {
    pub url: String,
    pub scope: MonitorScope,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApproveInput {
    pub expected_generation: u64,
    pub snapshot_id: String,
    pub snapshot_fingerprint: String,
    pub expected_version: u32,
    pub confirmed: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopeQuery {
    pub spreadsheet_id: String,
    pub scope_hash: String,
}
#[derive(Serialize)]
pub struct EngineEnvelope<T> {
    pub generation: u64,
    pub account_subject: String,
    pub data: T,
}
#[derive(Serialize)]
pub struct CapturePreview {
    pub session_generation: u64,
    pub snapshot: StructuralSnapshot,
    pub snapshot_fingerprint: String,
    pub active_baseline: Option<BaselineSummary>,
}
#[derive(Serialize)]
pub struct ValidationResult {
    pub report: IntegrityReport,
    pub events: Vec<IntegrityEvent>,
}
#[derive(Serialize)]
pub struct ScopeProposal {
    pub scope: MonitorScope,
    pub available_sheets: Vec<crate::google::SheetMetadata>,
}
impl From<EngineError> for CommandError {
    fn from(error: EngineError) -> Self {
        let kind = serde_json::to_value(&error)
            .ok()
            .and_then(|v| v.as_str().map(str::to_owned))
            .unwrap_or_else(|| "invalid_state".into());
        Self {
            code: format!("integrity_{kind}"),
            message: match error {
                EngineError::ApprovalRequired => {
                    "A aprovação explícita da referência é necessária."
                }
                EngineError::BaselineConflict => {
                    "A referência ou observação mudou. Atualize a prévia antes de continuar."
                }
                EngineError::MissingBaseline => "Capture e aprove uma referência para este escopo.",
                EngineError::InvalidScope => {
                    "Revise as regras, intervalos e identificadores do escopo."
                }
                EngineError::InvalidSnapshot => {
                    "O snapshot não permite esta operação. Revise sua cobertura."
                }
                EngineError::Limit => "O escopo excedeu os limites de captura ou análise.",
                EngineError::Forbidden => "A operação não pertence à conta conectada.",
                _ => "Não foi possível concluir a operação local de integridade.",
            }
            .into(),
            retryable: matches!(error, EngineError::Busy | EngineError::BaselineConflict),
        }
    }
}
#[derive(Serialize)]
pub struct BaselineSummary {
    pub id: String,
    pub version: u32,
    pub active: bool,
    pub approved_at: String,
    pub approval_origin: String,
    pub snapshot_id: String,
    pub scope_hash: String,
}
impl From<BaselineVersion> for BaselineSummary {
    fn from(b: BaselineVersion) -> Self {
        Self {
            id: b.id,
            version: b.version,
            active: b.active,
            approved_at: b.approved_at,
            approval_origin: b.approval_origin,
            snapshot_id: b.snapshot.id,
            scope_hash: b.snapshot.scope_hash,
        }
    }
}
static CAPTURES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
struct CapturePermit;
impl CapturePermit {
    fn acquire() -> Result<Self, CommandError> {
        use std::sync::atomic::Ordering;
        let mut count = CAPTURES.load(Ordering::SeqCst);
        loop {
            if count >= 3 {
                return Err(EngineError::Busy.into());
            }
            match CAPTURES.compare_exchange_weak(
                count,
                count + 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return Ok(Self),
                Err(actual) => count = actual,
            }
        }
    }
}
impl Drop for CapturePermit {
    fn drop(&mut self) {
        CAPTURES.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}
impl IntegrationRuntime {
    fn engine_database(&self) -> Result<Database, CommandError> {
        Database::open(&self.database_path).map_err(|_| CommandError::local())
    }
    fn engine_envelope<T>(
        &self,
        session: &AuthorizedSession,
        operation: impl FnOnce() -> Result<T, CommandError>,
    ) -> Result<EngineEnvelope<T>, CommandError> {
        self.auth.with_current(session, || {
            operation().map(|data| EngineEnvelope {
                generation: session.generation,
                account_subject: session.account.subject.clone(),
                data,
            })
        })?
    }
    pub fn propose_integrity_scope(
        &self,
        url: String,
    ) -> Result<EngineEnvelope<ScopeProposal>, CommandError> {
        let _permit = CapturePermit::acquire()?;
        let id = parse_spreadsheet_id(&url)?;
        let session = self.auth.authorized()?;
        if !session.has_scope(SHEETS_SCOPE) {
            return Err(
                crate::auth::AuthError::new(crate::auth::AuthCode::InsufficientScope).into(),
            );
        }
        let api = GoogleApi::new(session.access_token().into())?;
        let metadata = api.sheet_metadata(&id)?;
        let first = metadata
            .sheets
            .iter()
            .filter(|s| s.sheet_type.as_deref().is_none_or(|k| k == "GRID"))
            .min_by_key(|s| s.index)
            .ok_or(EngineError::InvalidScope)?;
        let rect = GridRect {
            start_row: 0,
            end_row: first.row_count.unwrap_or(1).min(100),
            start_col: 0,
            end_col: first.column_count.unwrap_or(1).min(8),
        };
        let grid = api.structural_grid(
            &id,
            &[collector::a1_range(
                &first.title,
                &GridRect {
                    end_row: rect.end_row.min(20),
                    ..rect.clone()
                },
            )],
        )?;
        let sheet = grid
            .sheets
            .iter()
            .find(|s| s.properties.sheet_id == first.sheet_id)
            .ok_or(EngineError::InvalidSnapshot)?;
        let mut columns = Vec::new();
        let mut formulas = Vec::new();
        for block in &sheet.data {
            for (i, row) in block.row_data.iter().enumerate() {
                let r = block
                    .start_row
                    .checked_add(i as u32)
                    .ok_or(EngineError::InvalidSnapshot)?;
                for (j, cell) in row.values.iter().enumerate() {
                    let c = block
                        .start_column
                        .checked_add(j as u32)
                        .ok_or(EngineError::InvalidSnapshot)?;
                    if !rect.contains(r, c) {
                        return Err(EngineError::InvalidSnapshot.into());
                    }
                    if r == 0 {
                        if let serde_json::Value::String(header) = cell.value() {
                            if !header.trim().is_empty() {
                                columns.push(ColumnRule {
                                    id: format!("field_{c}"),
                                    header,
                                    position: c,
                                    protect_position: true,
                                    required: true,
                                });
                            }
                        }
                    } else if cell.formula().is_some()
                        && columns.iter().any(|x| x.position == c)
                        && !formulas
                            .iter()
                            .any(|f: &FormulaRule| f.column_id == format!("field_{c}"))
                    {
                        formulas.push(FormulaRule {
                            id: format!("formula_{r}_{c}"),
                            column_id: format!("field_{c}"),
                            baseline_row: r,
                            mode: FormulaMode::Structural,
                            allow_row_shift: false,
                        });
                    }
                }
            }
        }
        let scope = MonitorScope {
            schema_version: SNAPSHOT_VERSION,
            sheets: vec![SheetRule {
                sheet_id: first.sheet_id,
                protect_title: true,
                protect_position: false,
                regions: vec![RegionRule {
                    id: "data".into(),
                    range: rect,
                    header_row: 0,
                    columns,
                    key_column: None,
                    critical: true,
                    allow_record_deletion: true,
                    formulas,
                }],
            }],
            scripts: vec![],
            named_range_ids: vec![],
            snapshot_retention_days: 7,
            history_retention_days: 90,
        };
        scope.validate()?;
        self.engine_envelope(&session, || {
            Ok(ScopeProposal {
                scope,
                available_sheets: metadata.sheets,
            })
        })
    }
    pub fn capture_integrity_snapshot(
        &self,
        input: CaptureInput,
    ) -> Result<EngineEnvelope<CapturePreview>, CommandError> {
        let _permit = CapturePermit::acquire()?;
        input.scope.validate()?;
        let id = parse_spreadsheet_id(&input.url)?;
        let mut session = self.auth.authorized()?;
        if !session.has_scope(SHEETS_SCOPE) {
            return Err(
                crate::auth::AuthError::new(crate::auth::AuthCode::InsufficientScope).into(),
            );
        }
        let mut snapshot = collector::capture_with(
            &GoogleApi::new(session.access_token().into())?,
            &session.account.subject,
            &id,
            input.scope.clone(),
        )?;
        if snapshot
            .coverage
            .issues
            .iter()
            .any(|e| e.code == "unauthorized")
        {
            session = self.auth.force_refresh(&session)?;
            snapshot = collector::capture_with(
                &GoogleApi::new(session.access_token().into())?,
                &session.account.subject,
                &id,
                input.scope,
            )?;
        }
        self.engine_envelope(&session, || {
            let mut db = self.engine_database()?;
            db.upsert_account(
                &session.account.subject,
                session.account.email.as_deref(),
                session.account.name.as_deref(),
            )
            .map_err(|_| CommandError::local())?;
            db.save_structural_snapshot(&snapshot)?;
            let active_baseline =
                db.active_structural_baseline(&session.account.subject, &id, &snapshot.scope_hash)?;
            Ok(CapturePreview {
                session_generation: session.generation,
                snapshot_fingerprint: fingerprint(&snapshot),
                snapshot,
                active_baseline: active_baseline.map(BaselineSummary::from),
            })
        })
    }
    pub fn approve_integrity_baseline(
        &self,
        input: ApproveInput,
    ) -> Result<EngineEnvelope<BaselineVersion>, CommandError> {
        if !input.confirmed {
            return Err(EngineError::ApprovalRequired.into());
        }
        let session = self.auth.authorized()?;
        if input.expected_generation != session.generation {
            return Err(EngineError::BaselineConflict.into());
        }
        self.engine_envelope(&session, || {
            let mut db = self.engine_database()?;
            let snapshot = db.structural_snapshot(&session.account.subject, &input.snapshot_id)?;
            if fingerprint(&snapshot) != input.snapshot_fingerprint {
                return Err(EngineError::BaselineConflict.into());
            }
            Ok(db.approve_structural_baseline(
                &session.account.subject,
                &input.snapshot_id,
                input.expected_version,
                true,
                "human:native",
            )?)
        })
    }
    pub fn integrity_baselines(
        &self,
        input: ScopeQuery,
        offset: u32,
    ) -> Result<EngineEnvelope<Vec<BaselineSummary>>, CommandError> {
        self.engine_local_query(input, |db, subject, q| {
            Ok(db
                .structural_baselines_page(subject, &q.spreadsheet_id, &q.scope_hash, 5, offset)?
                .into_iter()
                .map(BaselineSummary::from)
                .collect())
        })
    }
    pub fn integrity_active_baseline(
        &self,
        input: ScopeQuery,
    ) -> Result<EngineEnvelope<Option<BaselineVersion>>, CommandError> {
        self.engine_local_query(input, |db, subject, q| {
            Ok(db.active_structural_baseline(subject, &q.spreadsheet_id, &q.scope_hash)?)
        })
    }
    pub fn integrity_report(
        &self,
        input: ScopeQuery,
    ) -> Result<EngineEnvelope<Option<IntegrityReport>>, CommandError> {
        self.engine_local_query(input, |db, subject, q| {
            Ok(db.latest_integrity_report(subject, &q.spreadsheet_id, &q.scope_hash)?)
        })
    }
    pub fn integrity_event_list(
        &self,
        input: ScopeQuery,
    ) -> Result<EngineEnvelope<Vec<IntegrityEvent>>, CommandError> {
        self.engine_local_query(input, |db, subject, q| {
            Ok(db.integrity_events(subject, &q.spreadsheet_id, &q.scope_hash, 200)?)
        })
    }
    fn engine_local_query<T>(
        &self,
        input: ScopeQuery,
        operation: impl FnOnce(&mut Database, &str, &ScopeQuery) -> Result<T, CommandError>,
    ) -> Result<EngineEnvelope<T>, CommandError> {
        crate::google::validate_google_id(&input.spreadsheet_id)?;
        if input.scope_hash.len() != 64 || !input.scope_hash.bytes().all(|c| c.is_ascii_hexdigit())
        {
            return Err(EngineError::InvalidScope.into());
        }
        let session = self.auth.authorized()?;
        self.engine_envelope(&session, || {
            operation(
                &mut self.engine_database()?,
                &session.account.subject,
                &input,
            )
        })
    }
    pub fn validate_integrity_snapshot(
        &self,
        snapshot_id: String,
        persist: bool,
    ) -> Result<EngineEnvelope<ValidationResult>, CommandError> {
        self.validate_integrity_snapshot_registered(snapshot_id, persist, None)
    }
    pub fn validate_integrity_snapshot_registered(
        &self,
        snapshot_id: String,
        persist: bool,
        registry_revision: Option<u64>,
    ) -> Result<EngineEnvelope<ValidationResult>, CommandError> {
        let session = self.auth.authorized()?;
        self.engine_envelope(&session, || {
            let mut db = self.engine_database()?;
            let snapshot = db.structural_snapshot(&session.account.subject, &snapshot_id)?;
            let baseline = db
                .active_structural_baseline(
                    &session.account.subject,
                    &snapshot.spreadsheet_id,
                    &snapshot.scope_hash,
                )?
                .ok_or(EngineError::MissingBaseline)?;
            let previous = db.previous_structural_snapshot(
                &session.account.subject,
                &snapshot.spreadsheet_id,
                &snapshot.scope_hash,
            )?;
            let observation = db.content_observation(
                &session.account.subject,
                &snapshot.spreadsheet_id,
                &snapshot.scope_hash,
            )?;
            let mut report = compare::evaluate(
                &baseline,
                observation.snapshot.as_ref(),
                &snapshot,
                &uuid::Uuid::new_v4().to_string(),
            )?;
            report.detected_at = chrono::Utc::now().to_rfc3339();
            report.previous_snapshot_id = previous.as_ref().map(|s| s.id.clone());
            report.observation_revision = observation.revision;
            let events = if persist {
                db.persist_registered_integrity_report(&snapshot, &report, registry_revision)?
            } else {
                vec![]
            };
            Ok(ValidationResult { report, events })
        })
    }
    pub fn integrity_script_analysis(
        &self,
        snapshot_id: String,
    ) -> Result<EngineEnvelope<Vec<ScriptSnapshot>>, CommandError> {
        let session = self.auth.authorized()?;
        self.engine_envelope(&session, || {
            Ok(self
                .engine_database()?
                .structural_snapshot(&session.account.subject, &snapshot_id)?
                .scripts)
        })
    }
    pub fn prune_integrity(
        &self,
        input: ScopeQuery,
    ) -> Result<EngineEnvelope<usize>, CommandError> {
        self.engine_local_query(input, |db, subject, q| {
            let snapshot = db
                .previous_structural_snapshot(subject, &q.spreadsheet_id, &q.scope_hash)?
                .ok_or(EngineError::NotFound)?;
            Ok(db.prune_integrity_history(
                subject,
                &q.spreadsheet_id,
                &q.scope_hash,
                snapshot.scope.snapshot_retention_days,
                snapshot.scope.history_retention_days,
            )?)
        })
    }
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
