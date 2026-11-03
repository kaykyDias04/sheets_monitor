//! Transactional integrity persistence. Raw cell values, script source and OAuth
//! credentials are excluded by the engine contracts.
use super::Database;
use crate::integrity::{
    fingerprint, BaselineVersion, Connectivity, ContentObservation, EngineError, EngineResult,
    Finding, FindingOutcome, IntegrityEvent, IntegrityReport, IntegrityStatus, NamedRangeSnapshot,
    RegionSnapshot, ScriptSnapshot, Severity, SheetSnapshot, StructuralSnapshot,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::BTreeSet;

fn guard_registered_snapshot(
    c: &Connection,
    snapshot: &StructuralSnapshot,
    revision: Option<u64>,
    checking: bool,
) -> EngineResult<()> {
    let aid = account(c, &snapshot.account_subject)?;
    let row: Option<(Option<String>, String, bool, bool, u64)> = c.query_row(
        "SELECT scope_hash,selected_sheet_ids_json,removed,paused,revision FROM registered_spreadsheets WHERE account_id=?1 AND spreadsheet_id=?2",
        params![aid,snapshot.spreadsheet_id],
        |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))
    ).optional().map_err(storage)?;
    match row {
        None if revision.is_some() => Err(EngineError::BaselineConflict),
        None => Ok(()),
        Some((hash, selected, removed, paused, current)) => {
            let selected: Vec<u32> = decode(&selected)?;
            if removed
                || checking && paused
                || hash.as_deref() != Some(&snapshot.scope_hash)
                || revision.is_some_and(|r| r != current)
                || selected.iter().copied().collect::<BTreeSet<_>>()
                    != snapshot
                        .scope
                        .sheets
                        .iter()
                        .map(|s| s.sheet_id)
                        .collect::<BTreeSet<_>>()
            {
                Err(EngineError::BaselineConflict)
            } else {
                Ok(())
            }
        }
    }
}

fn storage(_: rusqlite::Error) -> EngineError {
    EngineError::Storage
}
fn encode<T: Serialize>(value: &T) -> EngineResult<String> {
    serde_json::to_string(value).map_err(|_| EngineError::InvalidSnapshot)
}
fn decode<T: DeserializeOwned>(value: &str) -> EngineResult<T> {
    serde_json::from_str(value).map_err(|_| EngineError::InvalidSnapshot)
}
fn enum_key<T: Serialize>(value: &T) -> EngineResult<String> {
    serde_json::to_value(value)
        .map_err(|_| EngineError::InvalidState)?
        .as_str()
        .map(str::to_owned)
        .ok_or(EngineError::InvalidState)
}
fn account(connection: &Connection, subject: &str) -> EngineResult<String> {
    if crate::google::validate_google_id(subject).is_err() {
        return Err(EngineError::Forbidden);
    }
    connection
        .query_row(
            "SELECT id FROM accounts WHERE provider_subject=?1 AND is_demo=0",
            [subject],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage)?
        .ok_or(EngineError::Forbidden)
}
fn valid_hash(hash: &str) -> bool {
    hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit())
}
fn lookup_scope(
    connection: &Connection,
    subject: &str,
    spreadsheet_id: &str,
    scope_hash: &str,
) -> EngineResult<Option<String>> {
    if crate::google::validate_google_id(spreadsheet_id).is_err() || !valid_hash(scope_hash) {
        return Err(EngineError::InvalidScope);
    }
    let account_id = account(connection, subject)?;
    connection.query_row("SELECT id FROM integrity_scopes WHERE account_id=?1 AND spreadsheet_id=?2 AND scope_hash=?3",params![account_id,spreadsheet_id,scope_hash],|r|r.get(0)).optional().map_err(storage)
}
fn snapshot_from(
    connection: &Connection,
    subject: &str,
    id: &str,
) -> EngineResult<StructuralSnapshot> {
    let account_id = account(connection, subject)?;
    let row:Option<(String,String)>=connection.query_row("SELECT snapshot_json,fingerprint FROM structural_snapshots WHERE account_id=?1 AND id=?2",params![account_id,id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(storage)?;
    let (json, expected) = row.ok_or(EngineError::NotFound)?;
    let snapshot: StructuralSnapshot = decode(&json)?;
    snapshot.validate()?;
    if snapshot.account_subject != subject || fingerprint(&snapshot) != expected {
        return Err(EngineError::InvalidSnapshot);
    }
    Ok(snapshot)
}
fn capture_complete(snapshot: &StructuralSnapshot) -> bool {
    snapshot.coverage.complete
        && snapshot.coverage.metadata_complete
        && snapshot.connectivity == Connectivity::Reachable
        && !snapshot.coverage.issues.iter().any(|i| i.critical)
        && snapshot.scope.sheets.iter().all(|rule| {
            snapshot
                .sheets
                .iter()
                .find(|s| s.sheet_id == rule.sheet_id)
                .is_none_or(|sheet| {
                    rule.regions.iter().filter(|r| r.critical).all(|r| {
                        sheet
                            .regions
                            .iter()
                            .any(|observed| observed.region_id == r.id && observed.complete)
                    })
                })
        })
}
fn baseline_row(connection: &Connection, subject: &str, id: &str) -> EngineResult<BaselineVersion> {
    let account_id = account(connection, subject)?;
    let row:Option<(u32,String,String,String,bool)>=connection.query_row("SELECT b.version,b.approved_at,b.approval_origin,b.snapshot_id,s.active_baseline_id=b.id FROM baseline_versions b JOIN integrity_scopes s ON s.id=b.scope_id AND s.account_id=b.account_id WHERE b.account_id=?1 AND b.id=?2",params![account_id,id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().map_err(storage)?;
    let (version, approved_at, approval_origin, snapshot_id, active) =
        row.ok_or(EngineError::NotFound)?;
    Ok(BaselineVersion {
        id: id.into(),
        version,
        active,
        approved_at,
        approval_origin,
        snapshot: snapshot_from(connection, subject, &snapshot_id)?,
    })
}
fn event_for(
    report: &IntegrityReport,
    finding: &Finding,
    kind: &str,
    occurrence: u32,
) -> IntegrityEvent {
    IntegrityEvent {
        location: finding.location.clone(),
        event_id: uuid::Uuid::new_v4().to_string(),
        check_id: report.check_id.clone(),
        spreadsheet_id: report.spreadsheet_id.clone(),
        sheet_id: finding.sheet_id,
        kind: kind.into(),
        category: "integrity".into(),
        severity: if kind == "recovered" || kind == "analysis_concluded" {
            Severity::Info
        } else {
            finding.severity
        },
        rule_id: Some(finding.rule_id.clone()),
        before: if kind == "recovered" || kind == "analysis_concluded" {
            Some(fingerprint(finding))
        } else {
            finding.expected.clone()
        },
        after: if kind == "recovered" || kind == "analysis_concluded" {
            None
        } else {
            finding.found.clone()
        },
        detected_at: report.detected_at.clone(),
        fingerprint: fingerprint(&(kind, &finding.key, fingerprint(finding), occurrence)),
        diagnosis: if kind == "recovered" {
            "critical_rule_recovered".into()
        } else if kind == "analysis_concluded" {
            "analysis_concluded".into()
        } else {
            finding.code.clone()
        },
    }
}
fn insert_event(
    connection: &Connection,
    scope_id: &str,
    account_id: &str,
    event: &IntegrityEvent,
) -> EngineResult<()> {
    connection.execute("INSERT INTO integrity_events(event_id,check_id,scope_id,account_id,sheet_id,severity,detected_at,fingerprint,event_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![event.event_id,event.check_id,scope_id,account_id,event.sheet_id,enum_key(&event.severity)?,event.detected_at,event.fingerprint,encode(event)?]).map_err(storage)?;
    Ok(())
}
fn events_for_check(
    connection: &Connection,
    check_id: &str,
    account_id: &str,
) -> EngineResult<Vec<IntegrityEvent>> {
    let mut statement=connection.prepare("SELECT event_json FROM integrity_events WHERE check_id=?1 AND account_id=?2 ORDER BY detected_at,event_id").map_err(storage)?;
    let rows = statement
        .query_map(params![check_id, account_id], |r| r.get::<_, String>(0))
        .map_err(storage)?;
    rows.map(|row| decode(&row.map_err(storage)?)).collect()
}

fn update_observed_components(
    connection: &Connection,
    scope_id: &str,
    snapshot: &StructuralSnapshot,
) -> EngineResult<()> {
    let mut components = Vec::new();
    if snapshot.coverage.metadata_complete {
        let mut sheets = snapshot.sheets.clone();
        for sheet in &mut sheets {
            sheet.regions.clear();
        }
        components.push((
            "metadata".to_owned(),
            encode(&(sheets, &snapshot.named_ranges))?,
        ));
    }
    for sheet in &snapshot.sheets {
        for region in &sheet.regions {
            if region.complete {
                components.push((
                    format!("region:{}:{}", sheet.sheet_id, region.region_id),
                    encode(region)?,
                ));
            }
        }
    }
    for script in &snapshot.scripts {
        components.push((format!("script:{}", script.script_id), encode(script)?));
    }
    for (key, json) in components {
        connection.execute("INSERT INTO integrity_observation_components(scope_id,component_key,component_json,observed_at) VALUES(?1,?2,?3,?4) ON CONFLICT(scope_id,component_key) DO UPDATE SET component_json=excluded.component_json,observed_at=excluded.observed_at",params![scope_id,key,json,snapshot.captured_at]).map_err(storage)?;
    }
    connection
        .execute(
            "UPDATE integrity_scopes SET observation_revision=observation_revision+1 WHERE id=?1",
            [scope_id],
        )
        .map_err(storage)?;
    Ok(())
}
impl Database {
    pub fn save_structural_snapshot(&mut self, snapshot: &StructuralSnapshot) -> EngineResult<()> {
        snapshot.validate()?;
        if chrono::DateTime::parse_from_rfc3339(&snapshot.captured_at).is_err()
            || chrono::DateTime::parse_from_rfc3339(&snapshot.capture_started_at).is_err()
        {
            return Err(EngineError::InvalidSnapshot);
        }
        let json = encode(snapshot)?;
        let hash = fingerprint(snapshot);
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let account_id = account(&transaction, &snapshot.account_subject)?;
        let scope_id = format!(
            "scope-{}",
            fingerprint(&(&account_id, &snapshot.spreadsheet_id, &snapshot.scope_hash))
        );
        let existing: Option<String> = transaction
            .query_row(
                "SELECT fingerprint FROM structural_snapshots WHERE id=?1",
                [&snapshot.id],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage)?;
        if let Some(existing) = existing {
            return if existing == hash {
                Ok(())
            } else {
                Err(EngineError::InvalidSnapshot)
            };
        }
        let known_file:bool=transaction.query_row("SELECT EXISTS(SELECT 1 FROM integrity_scopes WHERE account_id=?1 AND spreadsheet_id=?2)",params![account_id,snapshot.spreadsheet_id],|r|r.get(0)).map_err(storage)?;
        if !known_file {
            let count:i64=transaction.query_row("SELECT count(*) FROM (SELECT s.account_id,s.spreadsheet_id FROM integrity_scopes s WHERE NOT EXISTS(SELECT 1 FROM registered_spreadsheets r WHERE r.account_id=s.account_id AND r.spreadsheet_id=s.spreadsheet_id AND r.removed=1) GROUP BY s.account_id,s.spreadsheet_id)",[],|r|r.get(0)).map_err(storage)?;
            if count >= 30 {
                return Err(EngineError::Limit);
            }
        }
        transaction.execute("INSERT INTO integrity_scopes(id,account_id,spreadsheet_id,scope_hash,scope_json,created_at) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(account_id,spreadsheet_id,scope_hash) DO NOTHING",params![scope_id,account_id,snapshot.spreadsheet_id,snapshot.scope_hash,encode(&snapshot.scope)?,snapshot.captured_at]).map_err(storage)?;
        transaction.execute("INSERT INTO structural_snapshots(id,scope_id,account_id,captured_at,fingerprint,snapshot_json,complete) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![snapshot.id,scope_id,account_id,snapshot.captured_at,hash,json,capture_complete(snapshot)]).map_err(storage)?;
        transaction.commit().map_err(storage)
    }

    pub fn structural_snapshot(&self, subject: &str, id: &str) -> EngineResult<StructuralSnapshot> {
        snapshot_from(&self.connection, subject, id)
    }

    pub fn approve_structural_baseline(
        &mut self,
        subject: &str,
        snapshot_id: &str,
        expected_version: u32,
        confirmed: bool,
        origin: &str,
    ) -> EngineResult<BaselineVersion> {
        self.approve_registered_structural_baseline(
            subject,
            snapshot_id,
            expected_version,
            confirmed,
            origin,
            None,
        )
    }
    pub fn approve_registered_structural_baseline(
        &mut self,
        subject: &str,
        snapshot_id: &str,
        expected_version: u32,
        confirmed: bool,
        origin: &str,
        registry_revision: Option<u64>,
    ) -> EngineResult<BaselineVersion> {
        if !confirmed {
            return Err(EngineError::ApprovalRequired);
        }
        let permitted = origin == "human:native"
            || cfg!(any(test, feature = "native-probe")) && origin == "synthetic:test";
        if !permitted {
            return Err(EngineError::ApprovalRequired);
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let snapshot = snapshot_from(&transaction, subject, snapshot_id)?;
        guard_registered_snapshot(&transaction, &snapshot, registry_revision, false)?;
        crate::integrity::compare::validate_baseline_snapshot(&snapshot)?;
        if !capture_complete(&snapshot) {
            return Err(EngineError::InvalidSnapshot);
        }
        let account_id = account(&transaction, subject)?;
        let scope_id = lookup_scope(
            &transaction,
            subject,
            &snapshot.spreadsheet_id,
            &snapshot.scope_hash,
        )?
        .ok_or(EngineError::NotFound)?;
        let current: u32 = transaction
            .query_row(
                "SELECT COALESCE(max(version),0) FROM baseline_versions WHERE scope_id=?1",
                [&scope_id],
                |r| r.get(0),
            )
            .map_err(storage)?;
        if current != expected_version {
            return Err(EngineError::BaselineConflict);
        }
        let version = current.checked_add(1).ok_or(EngineError::Limit)?;
        let id = uuid::Uuid::new_v4().to_string();
        let approved_at = chrono::Utc::now().to_rfc3339();
        transaction.execute("INSERT INTO baseline_versions(id,scope_id,account_id,snapshot_id,version,approved_at,approval_origin) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![id,scope_id,account_id,snapshot_id,version,approved_at,origin]).map_err(storage)?;
        let first_observation: bool = transaction
            .query_row(
                "SELECT previous_snapshot_id IS NULL FROM integrity_scopes WHERE id=?1",
                [&scope_id],
                |r| r.get(0),
            )
            .map_err(storage)?;
        transaction.execute("UPDATE integrity_scopes SET active_baseline_id=?1,previous_snapshot_id=COALESCE(previous_snapshot_id,?2) WHERE id=?3 AND account_id=?4",params![id,snapshot_id,scope_id,account_id]).map_err(storage)?;
        if first_observation {
            update_observed_components(&transaction, &scope_id, &snapshot)?;
        }
        transaction.commit().map_err(storage)?;
        Ok(BaselineVersion {
            id,
            version,
            active: true,
            approved_at,
            approval_origin: origin.into(),
            snapshot,
        })
    }

    pub fn structural_baselines(
        &self,
        subject: &str,
        spreadsheet_id: &str,
        scope_hash: &str,
    ) -> EngineResult<Vec<BaselineVersion>> {
        self.structural_baselines_page(subject, spreadsheet_id, scope_hash, 20, 0)
    }
    pub fn structural_baselines_page(
        &self,
        subject: &str,
        spreadsheet_id: &str,
        scope_hash: &str,
        limit: u32,
        offset: u32,
    ) -> EngineResult<Vec<BaselineVersion>> {
        if limit == 0 || limit > 20 || offset > 100000 {
            return Err(EngineError::Limit);
        }
        let Some(scope_id) = lookup_scope(&self.connection, subject, spreadsheet_id, scope_hash)?
        else {
            return Ok(vec![]);
        };
        let mut statement=self.connection.prepare("SELECT id FROM baseline_versions WHERE scope_id=?1 ORDER BY version DESC LIMIT ?2 OFFSET ?3").map_err(storage)?;
        let ids = statement
            .query_map(params![scope_id, limit, offset], |r| r.get::<_, String>(0))
            .map_err(storage)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage)?;
        ids.iter()
            .map(|id| baseline_row(&self.connection, subject, id))
            .collect()
    }
    pub fn content_observation(
        &self,
        subject: &str,
        spreadsheet_id: &str,
        scope_hash: &str,
    ) -> EngineResult<ContentObservation> {
        let Some(scope_id) = lookup_scope(&self.connection, subject, spreadsheet_id, scope_hash)?
        else {
            return Ok(ContentObservation {
                revision: 0,
                snapshot: None,
            });
        };
        let revision = self
            .connection
            .query_row(
                "SELECT observation_revision FROM integrity_scopes WHERE id=?1",
                [&scope_id],
                |r| r.get::<_, u64>(0),
            )
            .map_err(storage)?;
        let Some(mut snapshot) =
            self.previous_structural_snapshot(subject, spreadsheet_id, scope_hash)?
        else {
            return Ok(ContentObservation {
                revision,
                snapshot: None,
            });
        };
        let mut statement=self.connection.prepare("SELECT component_key,component_json FROM integrity_observation_components WHERE scope_id=?1 ORDER BY component_key").map_err(storage)?;
        let rows = statement
            .query_map([&scope_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })
            .map_err(storage)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(storage)?;
        for (key, json) in &rows {
            if key == "metadata" {
                let (sheets, names): (Vec<SheetSnapshot>, Vec<NamedRangeSnapshot>) = decode(json)?;
                snapshot.sheets = sheets;
                snapshot.named_ranges = names;
            }
        }
        snapshot.scripts.clear();
        for (key, json) in rows {
            if let Some(rest) = key.strip_prefix("region:") {
                if let Some((sheet_id, _)) = rest.split_once(':') {
                    if let Ok(id) = sheet_id.parse::<u32>() {
                        if let Some(sheet) = snapshot.sheets.iter_mut().find(|s| s.sheet_id == id) {
                            let region: RegionSnapshot = decode(&json)?;
                            sheet.regions.push(region);
                        }
                    }
                }
            } else if key.starts_with("script:") {
                snapshot.scripts.push(decode::<ScriptSnapshot>(&json)?);
            }
        }
        snapshot.id = format!("observation-{}-{revision}", &scope_hash[..16]);
        snapshot.validate()?;
        Ok(ContentObservation {
            revision,
            snapshot: Some(snapshot),
        })
    }
    pub fn active_structural_baseline(
        &self,
        subject: &str,
        spreadsheet_id: &str,
        scope_hash: &str,
    ) -> EngineResult<Option<BaselineVersion>> {
        let Some(scope_id) = lookup_scope(&self.connection, subject, spreadsheet_id, scope_hash)?
        else {
            return Ok(None);
        };
        let id: Option<String> = self
            .connection
            .query_row(
                "SELECT active_baseline_id FROM integrity_scopes WHERE id=?1",
                [scope_id],
                |r| r.get(0),
            )
            .map_err(storage)?;
        id.map(|id| baseline_row(&self.connection, subject, &id))
            .transpose()
    }

    pub fn previous_structural_snapshot(
        &self,
        subject: &str,
        spreadsheet_id: &str,
        scope_hash: &str,
    ) -> EngineResult<Option<StructuralSnapshot>> {
        let Some(scope_id) = lookup_scope(&self.connection, subject, spreadsheet_id, scope_hash)?
        else {
            return Ok(None);
        };
        let id: Option<String> = self
            .connection
            .query_row(
                "SELECT previous_snapshot_id FROM integrity_scopes WHERE id=?1",
                [scope_id],
                |r| r.get(0),
            )
            .map_err(storage)?;
        id.map(|id| snapshot_from(&self.connection, subject, &id))
            .transpose()
    }

    pub fn persist_integrity_report(
        &mut self,
        snapshot: &StructuralSnapshot,
        report: &IntegrityReport,
    ) -> EngineResult<Vec<IntegrityEvent>> {
        self.persist_registered_integrity_report(snapshot, report, None)
    }
    pub fn persist_registered_integrity_report(
        &mut self,
        snapshot: &StructuralSnapshot,
        report: &IntegrityReport,
        registry_revision: Option<u64>,
    ) -> EngineResult<Vec<IntegrityEvent>> {
        self.persist_integrity_report_guarded(snapshot, report, registry_revision, None)
    }
    pub fn persist_monitored_integrity_report(
        &mut self,
        snapshot: &StructuralSnapshot,
        report: &IntegrityReport,
        registry_revision: Option<u64>,
        lease: &crate::monitoring::models::MonitoringLease,
        now_ms: i64,
    ) -> EngineResult<Vec<IntegrityEvent>> {
        self.persist_integrity_report_guarded(
            snapshot,
            report,
            registry_revision,
            Some((lease, now_ms)),
        )
    }
    fn persist_integrity_report_guarded(
        &mut self,
        snapshot: &StructuralSnapshot,
        report: &IntegrityReport,
        registry_revision: Option<u64>,
        monitoring: Option<(&crate::monitoring::models::MonitoringLease, i64)>,
    ) -> EngineResult<Vec<IntegrityEvent>> {
        snapshot.validate()?;
        if report.snapshot_id != snapshot.id
            || report.account_subject != snapshot.account_subject
            || report.spreadsheet_id != snapshot.spreadsheet_id
            || report.check_id.is_empty()
            || report.check_id.len() > 128
            || report.status == IntegrityStatus::Checking
        {
            return Err(EngineError::InvalidState);
        }
        let violation = report
            .findings
            .iter()
            .any(|f| f.severity == Severity::Critical && f.outcome == FindingOutcome::Violation);
        let uncertainty = report
            .findings
            .iter()
            .any(|f| f.severity == Severity::Critical && f.outcome == FindingOutcome::Inconclusive);
        if violation && report.status != IntegrityStatus::Offline
            || !violation && report.status == IntegrityStatus::Offline
            || uncertainty && !violation && report.status != IntegrityStatus::Indeterminate
        {
            return Err(EngineError::InvalidState);
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let stored = snapshot_from(&transaction, &snapshot.account_subject, &snapshot.id)?;
        if let Some((lease, now_ms)) = monitoring {
            if lease.spreadsheet_id != snapshot.spreadsheet_id
                || !super::monitoring_store::lease_current(
                    &transaction,
                    &snapshot.account_subject,
                    lease,
                    now_ms,
                )
                .map_err(|_| EngineError::Storage)?
            {
                return Err(EngineError::BaselineConflict);
            }
        }
        guard_registered_snapshot(&transaction, snapshot, registry_revision, true)?;
        if fingerprint(&stored) != fingerprint(snapshot) {
            return Err(EngineError::InvalidSnapshot);
        }
        let account_id = account(&transaction, &snapshot.account_subject)?;
        let scope_id = lookup_scope(
            &transaction,
            &snapshot.account_subject,
            &snapshot.spreadsheet_id,
            &snapshot.scope_hash,
        )?
        .ok_or(EngineError::NotFound)?;
        let existing: Option<String> = transaction
            .query_row(
                "SELECT report_json FROM integrity_reports WHERE check_id=?1 AND account_id=?2",
                params![report.check_id, account_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage)?;
        if let Some(existing) = existing {
            if existing != encode(report)? {
                return Err(EngineError::InvalidState);
            }
            return events_for_check(&transaction, &report.check_id, &account_id);
        }
        let (active,previous,revision):(Option<String>,Option<String>,u64)=transaction.query_row("SELECT active_baseline_id,previous_snapshot_id,observation_revision FROM integrity_scopes WHERE id=?1",[&scope_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).map_err(storage)?;
        if active.as_deref() != Some(report.baseline_id.as_str())
            || previous != report.previous_snapshot_id
            || revision != report.observation_revision
        {
            return Err(EngineError::BaselineConflict);
        }
        transaction.execute("INSERT INTO integrity_reports(check_id,scope_id,account_id,snapshot_id,baseline_id,status,connectivity,detected_at,report_json) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![report.check_id,scope_id,account_id,snapshot.id,report.baseline_id,enum_key(&report.status)?,enum_key(&report.connectivity)?,report.detected_at,encode(report)?]).map_err(storage)?;
        let mut events = Vec::new();
        let mut seen = BTreeSet::new();
        for finding in &report.findings {
            let outcome = enum_key(&finding.outcome)?;
            let hash = fingerprint(finding);
            if !seen.insert((finding.key.clone(), outcome.clone())) {
                return Err(EngineError::InvalidState);
            }
            let existing:Option<(String,bool,u32)>=transaction.query_row("SELECT fingerprint,active,occurrence FROM integrity_incidents WHERE scope_id=?1 AND finding_key=?2 AND outcome=?3",params![scope_id,finding.key,outcome],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(storage)?;
            let (emit, occurrence) = match existing {
                Some((previous, true, n)) => (previous != hash, n),
                Some((_, false, n)) => (true, n.saturating_add(1)),
                None => (true, 1),
            };
            if emit {
                let kind = if finding.outcome == FindingOutcome::Violation {
                    "violation"
                } else {
                    "analysis_inconclusive"
                };
                let event = event_for(report, finding, kind, occurrence);
                insert_event(&transaction, &scope_id, &account_id, &event)?;
                events.push(event);
            }
            transaction.execute("INSERT INTO integrity_incidents(scope_id,account_id,finding_key,outcome,fingerprint,finding_json,active,occurrence,last_seen_at,resolved_at) VALUES(?1,?2,?3,?4,?5,?6,1,?7,?8,NULL) ON CONFLICT(scope_id,finding_key,outcome) DO UPDATE SET fingerprint=excluded.fingerprint,finding_json=excluded.finding_json,active=1,occurrence=excluded.occurrence,last_seen_at=excluded.last_seen_at,resolved_at=NULL",params![scope_id,account_id,finding.key,outcome,hash,encode(finding)?,occurrence,report.detected_at]).map_err(storage)?;
        }
        if capture_complete(snapshot) {
            let pending = {
                let mut statement=transaction.prepare("SELECT finding_key,outcome,finding_json,occurrence FROM integrity_incidents WHERE scope_id=?1 AND active=1").map_err(storage)?;
                let rows = statement
                    .query_map([&scope_id], |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, String>(2)?,
                            r.get::<_, u32>(3)?,
                        ))
                    })
                    .map_err(storage)?;
                rows.collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(storage)?
            };
            for (key, outcome, json, occurrence) in pending {
                if seen.contains(&(key.clone(), outcome.clone())) {
                    continue;
                }
                let finding: Finding = decode(&json)?;
                // An inconclusive current rule is not proof that a past violation recovered.
                if finding.outcome == FindingOutcome::Violation
                    && report.findings.iter().any(|f| {
                        f.outcome == FindingOutcome::Inconclusive
                            && f.severity == Severity::Critical
                            && (f.key == key
                                || f.rule_id == finding.rule_id
                                || f.sheet_id == finding.sheet_id)
                    })
                {
                    continue;
                }
                let event = event_for(
                    report,
                    &finding,
                    if finding.outcome == FindingOutcome::Violation {
                        "recovered"
                    } else {
                        "analysis_concluded"
                    },
                    occurrence,
                );
                insert_event(&transaction, &scope_id, &account_id, &event)?;
                events.push(event);
                transaction.execute("UPDATE integrity_incidents SET active=0,resolved_at=?1 WHERE scope_id=?2 AND finding_key=?3 AND outcome=?4",params![report.detected_at,scope_id,key,outcome]).map_err(storage)?;
            }
            transaction.execute("UPDATE integrity_scopes SET previous_snapshot_id=?1,latest_report_id=?2 WHERE id=?3 AND account_id=?4",params![snapshot.id,report.check_id,scope_id,account_id]).map_err(storage)?;
        } else {
            transaction
                .execute(
                    "UPDATE integrity_scopes SET latest_report_id=?1 WHERE id=?2 AND account_id=?3",
                    params![report.check_id, scope_id, account_id],
                )
                .map_err(storage)?;
        }
        for difference in &report.differences {
            let event = IntegrityEvent {
                location: difference.location.clone(),
                event_id: uuid::Uuid::new_v4().to_string(),
                check_id: report.check_id.clone(),
                spreadsheet_id: report.spreadsheet_id.clone(),
                sheet_id: difference.sheet_id,
                kind: difference.kind.clone(),
                category: difference.category.clone(),
                severity: Severity::Info,
                rule_id: None,
                before: difference.before.clone(),
                after: difference.after.clone(),
                detected_at: report.detected_at.clone(),
                fingerprint: fingerprint(difference),
                diagnosis: if difference.ambiguous {
                    "content_changed_operation_ambiguous".into()
                } else {
                    difference.kind.clone()
                },
            };
            insert_event(&transaction, &scope_id, &account_id, &event)?;
            events.push(event);
        }
        update_observed_components(&transaction, &scope_id, snapshot)?;
        transaction.commit().map_err(storage)?;
        Ok(events)
    }

    pub fn latest_integrity_report(
        &self,
        subject: &str,
        spreadsheet_id: &str,
        scope_hash: &str,
    ) -> EngineResult<Option<IntegrityReport>> {
        let Some(scope_id) = lookup_scope(&self.connection, subject, spreadsheet_id, scope_hash)?
        else {
            return Ok(None);
        };
        let json:Option<String>=self.connection.query_row("SELECT r.report_json FROM integrity_scopes s LEFT JOIN integrity_reports r ON r.check_id=s.latest_report_id AND r.account_id=s.account_id WHERE s.id=?1",[scope_id],|r|r.get(0)).map_err(storage)?;
        json.map(|value| decode(&value)).transpose()
    }

    pub fn integrity_events(
        &self,
        subject: &str,
        spreadsheet_id: &str,
        scope_hash: &str,
        limit: u32,
    ) -> EngineResult<Vec<IntegrityEvent>> {
        if !(1..=1000).contains(&limit) {
            return Err(EngineError::Limit);
        }
        let Some(scope_id) = lookup_scope(&self.connection, subject, spreadsheet_id, scope_hash)?
        else {
            return Ok(Vec::new());
        };
        let mut statement=self.connection.prepare("SELECT event_json FROM integrity_events WHERE scope_id=?1 ORDER BY detected_at DESC,event_id DESC LIMIT ?2").map_err(storage)?;
        let rows = statement
            .query_map(params![scope_id, limit], |r| r.get::<_, String>(0))
            .map_err(storage)?;
        rows.map(|row| decode(&row.map_err(storage)?)).collect()
    }

    pub fn prune_integrity_history(
        &mut self,
        subject: &str,
        spreadsheet_id: &str,
        scope_hash: &str,
        snapshot_days: u16,
        history_days: u16,
    ) -> EngineResult<usize> {
        if !(1..=365).contains(&snapshot_days) || !(1..=3650).contains(&history_days) {
            return Err(EngineError::InvalidScope);
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let Some(scope_id) = lookup_scope(&transaction, subject, spreadsheet_id, scope_hash)?
        else {
            return Ok(0);
        };
        let history_cutoff = format!("-{history_days} days");
        let snapshot_cutoff = format!("-{snapshot_days} days");
        // TTL expiration is explicit; pending delivery never becomes a false success.
        transaction.execute("UPDATE notification_attempts SET outcome='expired',completed_ms=CAST((julianday('now')-2440587.5)*86400000 AS INTEGER) WHERE outcome='dispatching' AND notification_id IN(SELECT n.id FROM notification_outbox n JOIN integrity_events e ON e.event_id=n.integrity_event_id WHERE e.scope_id=?1 AND julianday(e.detected_at)<julianday('now',?2))",params![scope_id,history_cutoff]).map_err(storage)?;
        transaction.execute("UPDATE notification_outbox SET state='expired',lease_owner=NULL,lease_until_ms=NULL WHERE integrity_event_id IN(SELECT event_id FROM integrity_events WHERE scope_id=?1 AND julianday(detected_at)<julianday('now',?2)) AND state NOT IN('dispatch_submitted','suppressed_by_preference','unsupported','expired')",params![scope_id,history_cutoff]).map_err(storage)?;
        let events=transaction.execute("DELETE FROM integrity_events WHERE scope_id=?1 AND julianday(detected_at)<julianday('now',?2)",params![scope_id,history_cutoff]).map_err(storage)?;
        let reports=transaction.execute("DELETE FROM integrity_reports WHERE scope_id=?1 AND check_id NOT IN(SELECT latest_report_id FROM integrity_scopes WHERE id=?1 AND latest_report_id IS NOT NULL) AND julianday(detected_at)<julianday('now',?2) AND NOT EXISTS(SELECT 1 FROM integrity_events e WHERE e.check_id=integrity_reports.check_id)",params![scope_id,history_cutoff]).map_err(storage)?;
        let snapshots=transaction.execute("DELETE FROM structural_snapshots WHERE scope_id=?1 AND julianday(captured_at)<julianday('now',?2) AND NOT EXISTS(SELECT 1 FROM baseline_versions b WHERE b.snapshot_id=structural_snapshots.id) AND NOT EXISTS(SELECT 1 FROM integrity_scopes s WHERE s.previous_snapshot_id=structural_snapshots.id) AND NOT EXISTS(SELECT 1 FROM integrity_reports r JOIN integrity_scopes s ON s.latest_report_id=r.check_id WHERE r.snapshot_id=structural_snapshots.id)",params![scope_id,snapshot_cutoff]).map_err(storage)?;
        transaction.commit().map_err(storage)?;
        Ok(events + reports + snapshots)
    }
}

#[cfg(test)]
#[path = "integrity_store_tests.rs"]
mod tests;
