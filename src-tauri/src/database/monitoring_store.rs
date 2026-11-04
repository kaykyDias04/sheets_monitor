//! Account-scoped scheduling, operational transitions and transactional outbox.
use super::Database;
use crate::{integration::CommandError, monitoring::models::*};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, CommandError>;
fn sql(_: rusqlite::Error) -> CommandError {
    CommandError::local()
}
fn invalid() -> CommandError {
    CommandError {
        code: "monitoring_conflict".into(),
        message: "O estado do monitoramento mudou. Atualize e tente novamente.".into(),
        retryable: false,
    }
}
fn account(c: &Connection, subject: &str) -> Result<String> {
    crate::google::validate_google_id(subject)?;
    c.query_row(
        "SELECT id FROM accounts WHERE provider_subject=?1 AND is_demo=0",
        [subject],
        |r| r.get(0),
    )
    .optional()
    .map_err(sql)?
    .ok_or_else(invalid)
}
fn control(c: &Connection, aid: &str) -> Result<MonitoringControl> {
    Ok(c.query_row(
        "SELECT enabled,generation FROM monitoring_controls WHERE account_id=?1",
        [aid],
        |r| {
            Ok(MonitoringControl {
                enabled: r.get(0)?,
                generation: r.get(1)?,
            })
        },
    )
    .optional()
    .map_err(sql)?
    .unwrap_or(MonitoringControl {
        enabled: false,
        generation: 0,
    }))
}
fn candidate(c: &Connection, aid: &str, id: &str) -> Result<Option<MonitoringCandidate>> {
    let row=c.query_row("SELECT r.name,r.scope_json,r.scope_hash,r.selected_sheet_ids_json,r.revision,b.id FROM registered_spreadsheets r JOIN integrity_scopes s ON s.account_id=r.account_id AND s.spreadsheet_id=r.spreadsheet_id AND s.scope_hash=r.scope_hash JOIN baseline_versions b ON b.id=s.active_baseline_id AND b.account_id=r.account_id AND b.scope_id=s.id WHERE r.account_id=?1 AND r.spreadsheet_id=?2 AND r.removed=0 AND r.paused=0",params![aid,id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,u64>(4)?,r.get::<_,String>(5)?))).optional().map_err(sql)?;
    let Some((name, raw, hash, selected, revision, baseline_id)) = row else {
        return Ok(None);
    };
    let scope: crate::integrity::MonitorScope =
        serde_json::from_str(&raw).map_err(|_| invalid())?;
    let selected: Vec<u32> = serde_json::from_str(&selected).map_err(|_| invalid())?;
    if scope.validate().is_err()
        || scope.hash() != hash
        || selected.is_empty()
        || selected.into_iter().collect::<BTreeSet<_>>()
            != scope.sheets.iter().map(|s| s.sheet_id).collect()
    {
        return Ok(None);
    }
    Ok(Some(MonitoringCandidate {
        spreadsheet_id: id.into(),
        name,
        scope,
        scope_hash: hash,
        registry_revision: revision,
        baseline_id,
    }))
}
pub(super) fn lease_current(
    c: &Connection,
    subject: &str,
    lease: &MonitoringLease,
    now_ms: i64,
) -> Result<bool> {
    let aid = account(c, subject)?;
    let ctl = control(c, &aid)?;
    if ctl.generation != lease.generation || (!ctl.enabled && !lease.manual) {
        return Ok(false);
    }
    let Some(can) = candidate(c, &aid, &lease.spreadsheet_id)? else {
        return Ok(false);
    };
    if can.registry_revision != lease.registry_revision {
        return Ok(false);
    }
    c.query_row("SELECT EXISTS(SELECT 1 FROM monitoring_schedules WHERE account_id=?1 AND spreadsheet_id=?2 AND registry_revision=?3 AND generation=?4 AND lease_owner=?5 AND lease_id=?6 AND lease_until_ms>?7 AND scope_hash=?8)",params![aid,lease.spreadsheet_id,lease.registry_revision,lease.generation,lease.owner,lease.lease_id,now_ms,can.scope_hash],|r|r.get(0)).map_err(sql)
}
fn clean_code(code: &str) -> Result<()> {
    if code.is_empty()
        || code.len() > 96
        || !code
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        Err(invalid())
    } else {
        Ok(())
    }
}
impl Database {
    pub fn monitoring_control(&self, subject: &str) -> Result<MonitoringControl> {
        control(&self.connection, &account(&self.connection, subject)?)
    }
    pub fn set_monitoring_enabled(
        &mut self,
        subject: &str,
        enabled: bool,
        _now_ms: i64,
    ) -> Result<MonitoringControl> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let aid = account(&tx, subject)?;
        tx.execute("INSERT INTO monitoring_controls(account_id,enabled,generation) VALUES(?1,?2,1) ON CONFLICT(account_id) DO UPDATE SET enabled=excluded.enabled,generation=generation+1",params![aid,enabled]).map_err(sql)?;
        tx.execute("UPDATE monitoring_schedules SET lease_id=NULL,lease_owner=NULL,lease_until_ms=NULL WHERE account_id=?1",[&aid]).map_err(sql)?;
        let result = control(&tx, &aid)?;
        tx.commit().map_err(sql)?;
        Ok(result)
    }
    pub fn monitoring_candidates(&self, subject: &str) -> Result<Vec<MonitoringCandidate>> {
        let aid = account(&self.connection, subject)?;
        let mut stmt=self.connection.prepare("SELECT spreadsheet_id FROM registered_spreadsheets WHERE account_id=?1 AND removed=0 AND paused=0 ORDER BY spreadsheet_id").map_err(sql)?;
        let ids = stmt
            .query_map([&aid], |r| r.get::<_, String>(0))
            .map_err(sql)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql)?;
        ids.into_iter()
            .filter_map(|id| candidate(&self.connection, &aid, &id).transpose())
            .collect()
    }
    pub fn ensure_monitoring_schedule(
        &mut self,
        subject: &str,
        can: &MonitoringCandidate,
        initial_next_ms: i64,
    ) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let aid = account(&tx, subject)?;
        let current = candidate(&tx, &aid, &can.spreadsheet_id)?.ok_or_else(invalid)?;
        if current.registry_revision != can.registry_revision
            || current.scope_hash != can.scope_hash
            || current.baseline_id != can.baseline_id
        {
            return Err(invalid());
        }
        tx.execute("INSERT INTO monitoring_schedules(account_id,spreadsheet_id,registry_revision,scope_hash,next_run_ms) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(account_id,spreadsheet_id) DO UPDATE SET registry_revision=excluded.registry_revision,scope_hash=excluded.scope_hash,next_run_ms=excluded.next_run_ms,lease_id=NULL,lease_owner=NULL,lease_until_ms=NULL WHERE registry_revision!=excluded.registry_revision OR scope_hash!=excluded.scope_hash",params![aid,can.spreadsheet_id,can.registry_revision,can.scope_hash,initial_next_ms]).map_err(sql)?;
        tx.commit().map_err(sql)
    }
    pub fn monitoring_schedules(&self, subject: &str) -> Result<Vec<MonitoringSchedule>> {
        let aid = account(&self.connection, subject)?;
        let mut stmt=self.connection.prepare("SELECT spreadsheet_id,registry_revision,scope_hash,generation,last_started_ms,last_completed_ms,next_run_ms,failure_count,operational_code,lease_owner,lease_until_ms FROM monitoring_schedules WHERE account_id=?1 ORDER BY next_run_ms,spreadsheet_id").map_err(sql)?;
        let result = stmt
            .query_map([aid], |r| {
                Ok(MonitoringSchedule {
                    spreadsheet_id: r.get(0)?,
                    registry_revision: r.get(1)?,
                    scope_hash: r.get(2)?,
                    generation: r.get(3)?,
                    last_started_ms: r.get(4)?,
                    last_completed_ms: r.get(5)?,
                    next_run_ms: r.get(6)?,
                    failure_count: r.get(7)?,
                    operational_code: r.get(8)?,
                    lease_owner: r.get(9)?,
                    lease_until_ms: r.get(10)?,
                })
            })
            .map_err(sql)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql)?;
        Ok(result)
    }
    pub fn defer_monitoring_schedule(
        &mut self,
        subject: &str,
        id: &str,
        next_ms: i64,
    ) -> Result<bool> {
        let aid = account(&self.connection, subject)?;
        Ok(self.connection.execute("UPDATE monitoring_schedules SET next_run_ms=?3 WHERE account_id=?1 AND spreadsheet_id=?2 AND lease_id IS NULL",params![aid,id,next_ms]).map_err(sql)?==1)
    }
    #[allow(clippy::too_many_arguments)]
    pub fn claim_monitoring_check(
        &mut self,
        subject: &str,
        id: &str,
        expected_revision: u64,
        generation: u64,
        owner: &str,
        now_ms: i64,
        lease_ms: i64,
        manual: bool,
    ) -> Result<Option<MonitoringLease>> {
        if owner.is_empty() || owner.len() > 128 || !(1000..=600_000).contains(&lease_ms) {
            return Err(invalid());
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let aid = account(&tx, subject)?;
        let ctl = control(&tx, &aid)?;
        if ctl.generation != generation || (!ctl.enabled && !manual) {
            return Ok(None);
        }
        let Some(can) = candidate(&tx, &aid, id)? else {
            return Ok(None);
        };
        if can.registry_revision != expected_revision {
            return Ok(None);
        }
        let lease_id = uuid::Uuid::new_v4().to_string();
        let changed=tx.execute("UPDATE monitoring_schedules SET generation=?3,lease_id=?4,lease_owner=?5,lease_until_ms=?6,lease_manual=?7,last_started_ms=?8 WHERE account_id=?1 AND spreadsheet_id=?2 AND registry_revision=?9 AND scope_hash=?10 AND (lease_id IS NULL OR lease_until_ms<=?8) AND ((?7 AND failure_count=0) OR next_run_ms<=?8)",params![aid,id,generation,lease_id,owner,now_ms.saturating_add(lease_ms),manual,now_ms,expected_revision,can.scope_hash]).map_err(sql)?;
        tx.commit().map_err(sql)?;
        Ok((changed == 1).then(|| MonitoringLease {
            spreadsheet_id: id.into(),
            registry_revision: expected_revision,
            generation,
            owner: owner.into(),
            lease_id,
            started_ms: now_ms,
            manual,
        }))
    }
    pub fn is_monitoring_lease_current(
        &self,
        subject: &str,
        lease: &MonitoringLease,
        now_ms: i64,
    ) -> Result<bool> {
        lease_current(&self.connection, subject, lease, now_ms)
    }
    pub fn recover_monitoring_leases(
        &mut self,
        subject: &str,
        owner: &str,
        now_ms: i64,
    ) -> Result<usize> {
        let aid = account(&self.connection, subject)?;
        self.connection.execute("UPDATE monitoring_schedules SET lease_id=NULL,lease_owner=NULL,lease_until_ms=NULL WHERE account_id=?1 AND lease_id IS NOT NULL AND (lease_owner!=?2 OR lease_until_ms<=?3)",params![aid,owner,now_ms]).map_err(sql)
    }
    pub fn finish_monitoring_check(
        &mut self,
        subject: &str,
        lease: &MonitoringLease,
        outcome: CheckOutcome,
        next_ms: i64,
        now_ms: i64,
    ) -> Result<bool> {
        if let CheckOutcome::OperationalFailure { code } = &outcome {
            clean_code(code)?
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        if !lease_current(&tx, subject, lease, now_ms)? {
            return Ok(false);
        }
        let aid = account(&tx, subject)?;
        let previous:Option<String>=tx.query_row("SELECT operational_code FROM monitoring_schedules WHERE account_id=?1 AND spreadsheet_id=?2",params![aid,lease.spreadsheet_id],|r|r.get(0)).map_err(sql)?;
        let (new_code, failed, transition) = match &outcome {
            CheckOutcome::Success => (
                None,
                false,
                previous
                    .as_ref()
                    .map(|_| ("recovered", "operational_recovered".to_string())),
            ),
            CheckOutcome::OperationalFailure { code } => (
                Some(code.clone()),
                true,
                (previous.as_deref() != Some(code.as_str())).then(|| ("unavailable", code.clone())),
            ),
            CheckOutcome::Cancelled => (previous.clone(), false, None),
        };
        if let Some((kind, code)) = transition {
            insert_operational(&tx, &aid, &lease.spreadsheet_id, kind, &code, now_ms)?;
        }
        tx.execute("UPDATE monitoring_schedules SET lease_id=NULL,lease_owner=NULL,lease_until_ms=NULL,last_completed_ms=CASE WHEN ?6 THEN last_completed_ms ELSE ?3 END,next_run_ms=?4,operational_code=?5,failure_count=CASE WHEN ?6 THEN failure_count WHEN ?7 THEN failure_count+1 ELSE 0 END WHERE account_id=?1 AND spreadsheet_id=?2",params![aid,lease.spreadsheet_id,now_ms,next_ms,new_code,matches!(outcome,CheckOutcome::Cancelled),failed]).map_err(sql)?;
        tx.commit().map_err(sql)?;
        Ok(true)
    }
}
fn insert_operational(
    c: &Connection,
    aid: &str,
    id: &str,
    kind: &str,
    code: &str,
    now_ms: i64,
) -> Result<()> {
    let eid = uuid::Uuid::new_v4().to_string();
    c.execute("INSERT INTO monitoring_operational_events(id,account_id,spreadsheet_id,kind,code,detected_ms) VALUES(?1,?2,?3,?4,?5,?6)",params![eid,aid,id,kind,code,now_ms]).map_err(sql)?;
    let diagnosis = crate::management::events::normalize_notification_diagnosis(
        "operational",
        Some(kind),
        Some(code),
        "",
    );
    c.execute("INSERT INTO notification_outbox(id,account_id,spreadsheet_id,operational_event_id,source,spreadsheet_name,severity,diagnosis,detected_at,created_ms,expires_ms,state,next_attempt_ms,group_id) SELECT 'operational:'||?1,?2,?3,?1,'operational',r.name,?4,?5,?6,?7,?7+CAST(COALESCE(json_extract(r.scope_json,'$.history_retention_days'),90) AS INTEGER)*86400000,CASE WHEN COALESCE((SELECT json_extract(preferences_json,'$.notifications_enabled') FROM management_preferences WHERE account_id=?2),0)=1 THEN 'pending' ELSE 'suppressed_by_preference' END,?7,'operational:'||?1 FROM registered_spreadsheets r WHERE r.account_id=?2 AND r.spreadsheet_id=?3",params![eid,aid,id,if kind=="recovered"{"info"}else{"warning"},diagnosis,chrono::DateTime::from_timestamp_millis(now_ms).ok_or_else(invalid)?.to_rfc3339(),now_ms]).map_err(sql)?;
    Ok(())
}

const NOTICE_SELECT: &str = "SELECT id,COALESCE(integrity_event_id,operational_event_id),source,spreadsheet_id,spreadsheet_name,sheet_id,severity,diagnosis,detected_at,state,attempt_count,last_attempt_ms,submitted_ms,group_id,COALESCE((SELECT json_extract(j.value,'$.title') FROM registered_spreadsheets r,json_each(r.tabs_json) j WHERE r.account_id=notification_outbox.account_id AND r.spreadsheet_id=notification_outbox.spreadsheet_id AND json_extract(j.value,'$.sheet_id')=notification_outbox.sheet_id LIMIT 1),(SELECT json_extract(j.value,'$.title') FROM integrity_events e JOIN integrity_scopes s ON s.id=e.scope_id AND s.account_id=e.account_id JOIN baseline_versions b ON b.id=s.active_baseline_id AND b.account_id=e.account_id JOIN structural_snapshots ss ON ss.id=b.snapshot_id AND ss.account_id=e.account_id,json_each(ss.snapshot_json,'$.sheets') j WHERE e.account_id=notification_outbox.account_id AND e.event_id=notification_outbox.integrity_event_id AND json_extract(j.value,'$.sheet_id')=notification_outbox.sheet_id LIMIT 1)),COALESCE((SELECT json_extract(e.event_json,'$.kind') FROM integrity_events e WHERE e.account_id=notification_outbox.account_id AND e.event_id=notification_outbox.integrity_event_id LIMIT 1),(SELECT m.kind FROM monitoring_operational_events m WHERE m.account_id=notification_outbox.account_id AND m.id=notification_outbox.operational_event_id LIMIT 1)),(SELECT m.code FROM monitoring_operational_events m WHERE m.account_id=notification_outbox.account_id AND m.id=notification_outbox.operational_event_id LIMIT 1) FROM notification_outbox";
fn notice(r: &rusqlite::Row<'_>) -> rusqlite::Result<NotificationRecord> {
    let source: String = r.get(2)?;
    let severity: String = r.get(6)?;
    let raw_diagnosis: String = r.get(7)?;
    let detected_at: String = r.get(8)?;
    let event_kind: Option<String> = r.get(15)?;
    let operational_code: Option<String> = r.get(16)?;
    let event_type_label = crate::management::events::notification_event_label(
        &source,
        event_kind.as_deref(),
        &severity,
        &raw_diagnosis,
    )
    .to_owned();
    let diagnosis = crate::management::events::normalize_notification_diagnosis(
        &source,
        event_kind.as_deref(),
        operational_code.as_deref(),
        &raw_diagnosis,
    );
    let formatted_detected_at = crate::management::events::format_local_datetime(&detected_at);
    Ok(NotificationRecord {
        id: r.get(0)?,
        event_id: r.get(1)?,
        source,
        spreadsheet_id: r.get(3)?,
        spreadsheet_name: r.get(4)?,
        sheet_id: r.get(5)?,
        sheet_title: r.get(14)?,
        severity,
        event_kind,
        event_type_label,
        diagnosis,
        detected_at,
        formatted_detected_at,
        state: r.get(9)?,
        attempt_count: r.get(10)?,
        last_attempt_ms: r.get(11)?,
        submitted_ms: r.get(12)?,
        group_id: r.get(13)?,
    })
}
fn enabled(c: &Connection, aid: &str) -> Result<bool> {
    c.query_row("SELECT COALESCE((SELECT json_extract(preferences_json,'$.notifications_enabled') FROM management_preferences WHERE account_id=?1),0)",[aid],|r|r.get(0)).map_err(sql)
}
fn expire(c: &Connection, aid: &str, now_ms: i64) -> Result<usize> {
    c.execute("UPDATE notification_attempts SET outcome='expired',completed_ms=?2 WHERE outcome='dispatching' AND notification_id IN(SELECT id FROM notification_outbox WHERE account_id=?1 AND expires_ms<=?2)",params![aid,now_ms]).map_err(sql)?;
    c.execute("UPDATE notification_outbox SET state='expired',lease_owner=NULL,lease_until_ms=NULL WHERE account_id=?1 AND expires_ms<=?2 AND state IN('pending','retry','dispatching','permission_blocked')",params![aid,now_ms]).map_err(sql)
}
pub(super) fn suppress(c: &Connection, aid: &str, now_ms: i64) -> Result<usize> {
    if enabled(c, aid)? {
        return Ok(0);
    }
    c.execute("UPDATE notification_attempts SET outcome='suppressed_by_preference',completed_ms=?2 WHERE outcome='dispatching' AND notification_id IN(SELECT id FROM notification_outbox WHERE account_id=?1)",params![aid,now_ms]).map_err(sql)?;
    c.execute("UPDATE notification_outbox SET state='suppressed_by_preference',lease_owner=NULL,lease_until_ms=NULL WHERE account_id=?1 AND state IN('pending','retry','dispatching','permission_blocked')",[aid]).map_err(sql)
}
impl Database {
    pub fn notification_history(
        &self,
        subject: &str,
        offset: u32,
        limit: u32,
    ) -> Result<crate::management::Page<NotificationRecord>> {
        if limit == 0 || limit > 100 || offset > 1_000_000 {
            return Err(invalid());
        }
        let aid = account(&self.connection, subject)?;
        let total = self
            .connection
            .query_row(
                "SELECT count(*) FROM notification_outbox WHERE account_id=?1",
                [&aid],
                |r| r.get(0),
            )
            .map_err(sql)?;
        let mut stmt=self.connection.prepare(&format!("{NOTICE_SELECT} WHERE account_id=?1 ORDER BY created_ms DESC,id DESC LIMIT ?2 OFFSET ?3")).map_err(sql)?;
        let items = stmt
            .query_map(params![aid, limit, offset], notice)
            .map_err(sql)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql)?;
        Ok(crate::management::Page {
            items,
            total,
            offset,
            limit,
        })
    }
    pub fn notification_by_id(
        &self,
        subject: &str,
        id: &str,
    ) -> Result<Option<NotificationRecord>> {
        let aid = account(&self.connection, subject)?;
        self.connection
            .query_row(
                &format!("{NOTICE_SELECT} WHERE account_id=?1 AND id=?2"),
                params![aid, id],
                notice,
            )
            .optional()
            .map_err(sql)
    }
    pub fn monitoring_operational_history(
        &self,
        subject: &str,
        offset: u32,
        limit: u32,
    ) -> Result<crate::management::Page<OperationalEvent>> {
        if limit == 0 || limit > 100 || offset > 1_000_000 {
            return Err(invalid());
        }
        let aid = account(&self.connection, subject)?;
        let total = self
            .connection
            .query_row(
                "SELECT count(*) FROM monitoring_operational_events WHERE account_id=?1",
                [&aid],
                |r| r.get(0),
            )
            .map_err(sql)?;
        let mut stmt=self.connection.prepare("SELECT id,spreadsheet_id,kind,code,detected_ms FROM monitoring_operational_events WHERE account_id=?1 ORDER BY detected_ms DESC,id DESC LIMIT ?2 OFFSET ?3").map_err(sql)?;
        let items = stmt
            .query_map(params![aid, limit, offset], |r| {
                Ok(OperationalEvent {
                    id: r.get(0)?,
                    spreadsheet_id: r.get(1)?,
                    kind: r.get(2)?,
                    code: r.get(3)?,
                    detected_ms: r.get(4)?,
                })
            })
            .map_err(sql)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql)?;
        Ok(crate::management::Page {
            items,
            total,
            offset,
            limit,
        })
    }
    pub fn suppress_disabled_notifications(&mut self, subject: &str, now_ms: i64) -> Result<usize> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let aid = account(&tx, subject)?;
        let count = suppress(&tx, &aid, now_ms)?;
        tx.commit().map_err(sql)?;
        Ok(count)
    }
    pub fn expire_notifications(&mut self, subject: &str, now_ms: i64) -> Result<usize> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let aid = account(&tx, subject)?;
        let count = expire(&tx, &aid, now_ms)?;
        tx.commit().map_err(sql)?;
        Ok(count)
    }
    pub fn claim_notification(
        &mut self,
        subject: &str,
        owner: &str,
        now_ms: i64,
    ) -> Result<Option<NotificationAttempt>> {
        if owner.is_empty() || owner.len() > 128 {
            return Err(invalid());
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let aid = account(&tx, subject)?;
        expire(&tx, &aid, now_ms)?;
        if !enabled(&tx, &aid)? {
            suppress(&tx, &aid, now_ms)?;
            tx.commit().map_err(sql)?;
            return Ok(None);
        }
        // Dispatch in progress at interruption is ambiguous; keep its immutable attempt.
        tx.execute("UPDATE notification_attempts SET outcome='ambiguous',completed_ms=?2 WHERE outcome='dispatching' AND notification_id IN(SELECT id FROM notification_outbox WHERE account_id=?1 AND state='dispatching' AND (lease_owner!=?3 OR lease_until_ms<=?2))",params![aid,now_ms,owner]).map_err(sql)?;
        tx.execute("UPDATE notification_outbox SET state='retry',lease_owner=NULL,lease_until_ms=NULL WHERE account_id=?1 AND state='dispatching' AND (lease_owner!=?2 OR lease_until_ms<=?3)",params![aid,owner,now_ms]).map_err(sql)?;
        let id:Option<String>=tx.query_row("SELECT id FROM notification_outbox WHERE account_id=?1 AND state IN('pending','retry') AND next_attempt_ms<=?2 AND expires_ms>?2 ORDER BY created_ms,id LIMIT 1",params![aid,now_ms],|r|r.get(0)).optional().map_err(sql)?;
        let Some(id) = id else {
            tx.commit().map_err(sql)?;
            return Ok(None);
        };
        let attempt_id = uuid::Uuid::new_v4().to_string();
        tx.execute("INSERT INTO notification_attempts(id,notification_id,owner,started_ms,outcome) VALUES(?1,?2,?3,?4,'dispatching')",params![attempt_id,id,owner,now_ms]).map_err(sql)?;
        tx.execute("UPDATE notification_outbox SET state='dispatching',attempt_id=?2,lease_owner=?3,lease_until_ms=?4,last_attempt_ms=?5,attempt_count=attempt_count+1 WHERE id=?1",params![id,attempt_id,owner,now_ms.saturating_add(60_000),now_ms]).map_err(sql)?;
        let record = tx
            .query_row(&format!("{NOTICE_SELECT} WHERE id=?1"), [id], notice)
            .map_err(sql)?;
        tx.commit().map_err(sql)?;
        Ok(Some(NotificationAttempt {
            record,
            attempt_id,
            owner: owner.into(),
        }))
    }
    pub fn finish_notification_attempt(
        &mut self,
        subject: &str,
        id: &str,
        attempt_id: &str,
        outcome: NotificationOutcome,
        now_ms: i64,
    ) -> Result<bool> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let aid = account(&tx, subject)?;
        expire(&tx, &aid, now_ms)?;
        if !enabled(&tx, &aid)? {
            suppress(&tx, &aid, now_ms)?;
            tx.commit().map_err(sql)?;
            return Ok(false);
        }
        let (state, code, next, submitted) = match outcome {
            NotificationOutcome::DispatchSubmitted => {
                ("dispatch_submitted", None, now_ms, Some(now_ms))
            }
            NotificationOutcome::Retryable {
                code,
                next_attempt_ms,
            } => {
                clean_code(&code)?;
                (
                    "retry",
                    Some(code),
                    next_attempt_ms.max(now_ms.saturating_add(1000)),
                    None,
                )
            }
            NotificationOutcome::PermissionBlocked => (
                "permission_blocked",
                Some("permission_denied".into()),
                now_ms,
                None,
            ),
            NotificationOutcome::Unsupported => {
                ("unsupported", Some("unsupported".into()), now_ms, None)
            }
        };
        let changed=tx.execute("UPDATE notification_outbox SET state=?4,next_attempt_ms=?5,submitted_ms=?6,lease_owner=NULL,lease_until_ms=NULL WHERE account_id=?1 AND id=?2 AND attempt_id=?3 AND state='dispatching'",params![aid,id,attempt_id,state,next,submitted]).map_err(sql)?;
        if changed == 1 {
            tx.execute("UPDATE notification_attempts SET outcome=?2,code=?3,completed_ms=?4 WHERE id=?1 AND outcome='dispatching'",params![attempt_id,state,code,now_ms]).map_err(sql)?;
        }
        tx.commit().map_err(sql)?;
        Ok(changed == 1)
    }
    /// Reconcile an asynchronous native failure with its exact durable attempt.
    /// A prior submission timestamp remains evidence of that request, even when
    /// the adapter later reports failure. True means the current row changed.
    #[allow(clippy::too_many_arguments)]
    pub fn retry_native_submission(
        &mut self,
        subject: &str,
        id: &str,
        attempt_id: &str,
        code: &str,
        next_ms: i64,
        now_ms: i64,
    ) -> Result<bool> {
        clean_code(code)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let aid = account(&tx, subject)?;
        // Inspect only this current attempt: an old callback must not alter a
        // replacement attempt, including its terminal housekeeping decisions.
        let expires: Option<i64> = tx.query_row(
            "SELECT expires_ms FROM notification_outbox WHERE account_id=?1 AND id=?2 AND attempt_id=?3 AND state IN('dispatching','dispatch_submitted')",
            params![aid,id,attempt_id],|r|r.get(0)
        ).optional().map_err(sql)?;
        let Some(expires) = expires else {
            return Ok(false);
        };
        let state = if expires <= now_ms {
            "expired"
        } else if !enabled(&tx, &aid)? {
            "suppressed_by_preference"
        } else {
            "retry"
        };
        let changed=tx.execute(
            "UPDATE notification_outbox SET state=?4,next_attempt_ms=?5,lease_owner=NULL,lease_until_ms=NULL WHERE account_id=?1 AND id=?2 AND attempt_id=?3 AND state IN('dispatching','dispatch_submitted')",
            params![aid,id,attempt_id,state,next_ms.max(now_ms.saturating_add(1000))]
        ).map_err(sql)?;
        if changed == 1 {
            tx.execute(
                "UPDATE notification_attempts SET outcome=?3,code=?4,completed_ms=?5 WHERE id=?1 AND notification_id=?2 AND outcome IN('dispatching','dispatch_submitted')",
                params![attempt_id,id,state,code,now_ms]
            ).map_err(sql)?;
        }
        tx.commit().map_err(sql)?;
        Ok(changed == 1)
    }
    /// Only an explicit permission/preference action requeues denied deliveries.
    pub fn retry_blocked_notifications(&mut self, subject: &str, now_ms: i64) -> Result<usize> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let aid = account(&tx, subject)?;
        expire(&tx, &aid, now_ms)?;
        let count = if enabled(&tx, &aid)? {
            tx.execute("UPDATE notification_outbox SET state='retry',next_attempt_ms=?2 WHERE account_id=?1 AND state='permission_blocked' AND expires_ms>?2",params![aid,now_ms]).map_err(sql)?
        } else {
            0
        };
        tx.commit().map_err(sql)?;
        Ok(count)
    }
    pub fn prune_monitoring_history(&mut self, subject: &str, now_ms: i64) -> Result<usize> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let aid = account(&tx, subject)?;
        expire(&tx, &aid, now_ms)?;
        let mut count=tx.execute("DELETE FROM monitoring_operational_events WHERE account_id=?1 AND id IN(SELECT operational_event_id FROM notification_outbox WHERE account_id=?1 AND expires_ms<=?2)",params![aid,now_ms]).map_err(sql)?;
        let policies = {
            let mut stmt=tx.prepare("SELECT spreadsheet_id,scope_hash,scope_json FROM integrity_scopes WHERE account_id=?1").map_err(sql)?;
            let rows = stmt
                .query_map([&aid], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                    ))
                })
                .map_err(sql)?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(sql)?;
            rows
        };
        tx.commit().map_err(sql)?;
        // Include paused, archived and older configured scopes; eligibility does
        // not grant indefinite history retention to inactive registrations.
        for (id, hash, raw) in policies {
            let scope: crate::integrity::MonitorScope =
                serde_json::from_str(&raw).map_err(|_| invalid())?;
            count += self
                .prune_integrity_history(
                    subject,
                    &id,
                    &hash,
                    scope.snapshot_retention_days,
                    scope.history_retention_days,
                )
                .map_err(CommandError::from)?;
        }
        Ok(count)
    }
}

#[cfg(test)]
#[path = "monitoring_store_tests.rs"]
mod tests;
