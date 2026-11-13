use super::Database;
use crate::{
    google::{validate_google_id, SpreadsheetMetadata},
    integration::CommandError,
    integrity::runtime::BaselineSummary,
    integrity::{fingerprint, BaselineVersion, IntegrityEvent, MonitorScope, Severity},
    management::*,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};
type Result<T> = std::result::Result<T, CommandError>;
fn error(code: &str, message: &str) -> CommandError {
    CommandError {
        code: code.into(),
        message: message.into(),
        retryable: false,
    }
}
fn sql(e: rusqlite::Error) -> CommandError {
    if e.to_string().contains("registry_limit_30") {
        error(
            "capacity",
            "O limite de 30 planilhas cadastradas foi atingido.",
        )
    } else {
        CommandError::local()
    }
}
fn encode<T: Serialize>(v: &T) -> Result<String> {
    serde_json::to_string(v).map_err(|_| CommandError::local())
}
fn decode<T: DeserializeOwned>(v: &str) -> Result<T> {
    serde_json::from_str(v).map_err(|_| CommandError::local())
}
fn account(c: &Connection, subject: &str) -> Result<String> {
    validate_google_id(subject)?;
    c.query_row(
        "SELECT id FROM accounts WHERE provider_subject=?1 AND is_demo=0",
        [subject],
        |r| r.get(0),
    )
    .optional()
    .map_err(sql)?
    .ok_or_else(|| error("forbidden", "Conecte uma conta para consultar seus dados."))
}
fn conflict() -> CommandError {
    error(
        "configuration_conflict",
        "A configuração mudou. Atualize a tela antes de continuar.",
    )
}
fn raw_row(
    c: &Connection,
    subject: &str,
    id: &str,
    include_removed: bool,
) -> Result<RegisteredSpreadsheet> {
    validate_google_id(id)?;
    let account_id = account(c, subject)?;
    let row=c.query_row("SELECT r.name,a.email,r.paused,r.removed,r.scope_json,r.scope_hash,r.revision,r.tabs_json,r.selected_sheet_ids_json,r.authorization,r.access_error,r.tabs_refreshed_at FROM registered_spreadsheets r JOIN accounts a ON a.id=r.account_id WHERE r.account_id=?1 AND r.spreadsheet_id=?2 AND (?3 OR r.removed=0)",params![account_id,id,include_removed],|r|Ok((r.get::<_,String>(0)?,r.get::<_,Option<String>>(1)?,r.get::<_,bool>(2)?,r.get::<_,bool>(3)?,r.get::<_,Option<String>>(4)?,r.get::<_,Option<String>>(5)?,r.get::<_,u64>(6)?,r.get::<_,String>(7)?,r.get::<_,String>(8)?,r.get::<_,String>(9)?,r.get::<_,Option<String>>(10)?,r.get::<_,Option<String>>(11)?))).optional().map_err(sql)?.ok_or_else(||error("not_registered","A planilha não está cadastrada nesta conta."))?;
    let scope: Option<MonitorScope> = row.4.as_deref().map(decode).transpose()?;
    let selected: Vec<u32> = decode(&row.8)?;
    let baseline: Option<(u32, String, Option<String>)> = if let Some(hash) = row.5.as_ref() {
        c.query_row("SELECT b.version,b.id,s.latest_report_id FROM integrity_scopes s JOIN baseline_versions b ON b.id=s.active_baseline_id WHERE s.account_id=?1 AND s.spreadsheet_id=?2 AND s.scope_hash=?3",params![account_id,id,hash],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(sql)?
    } else {
        None
    };
    let mut report_status = None;
    let mut last_checked_at = None;
    if let Some((_, baseline_id, Some(check))) = &baseline {
        let report=c.query_row("SELECT status,detected_at,baseline_id FROM integrity_reports WHERE check_id=?1 AND account_id=?2",params![check,account_id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).optional().map_err(sql)?;
        if let Some((status, time, bid)) = report {
            last_checked_at = Some(time);
            if &bid == baseline_id {
                report_status = Some(status);
            }
        }
    }
    let configured = scope.as_ref().is_some_and(|s| {
        s.validate().is_ok()
            && s.sheets.iter().map(|v| v.sheet_id).collect::<BTreeSet<_>>()
                == selected.iter().copied().collect()
            && !selected.is_empty()
    });
    let status = if row.2 {
        "paused".into()
    } else if !configured || baseline.is_none() {
        "configuration_pending".into()
    } else if row.9 == "error" {
        "indeterminate".into()
    } else {
        report_status.unwrap_or_else(|| "not_checked".into())
    };
    Ok(RegisteredSpreadsheet {
        spreadsheet_id: id.into(),
        name: row.0,
        url: format!("https://docs.google.com/spreadsheets/d/{id}/edit"),
        account_email: row.1,
        paused: row.2,
        removed: row.3,
        status,
        authorization: row.9,
        access_error: row.10,
        last_checked_at,
        baseline_version: baseline.map(|b| b.0),
        scope,
        scope_hash: row.5,
        revision: row.6,
        tabs: decode(&row.7)?,
        selected_sheet_ids: selected,
        tabs_refreshed_at: row.11,
    })
}
fn current(c: &Connection, subject: &str, m: &Mutation) -> Result<RegisteredSpreadsheet> {
    let row = raw_row(c, subject, &m.spreadsheet_id, false)?;
    if row.revision != m.expected_revision {
        return Err(conflict());
    }
    Ok(row)
}
fn configured_hash(row: &RegisteredSpreadsheet) -> Option<&str> {
    row.scope
        .as_ref()
        .filter(|scope| {
            scope
                .sheets
                .iter()
                .map(|s| s.sheet_id)
                .collect::<BTreeSet<_>>()
                == row.selected_sheet_ids.iter().copied().collect()
                && !row.selected_sheet_ids.is_empty()
        })
        .and(row.scope_hash.as_deref())
}
impl Database {
    pub fn registered_spreadsheets(&self, subject: &str) -> Result<Vec<RegisteredSpreadsheet>> {
        let aid = account(&self.connection, subject)?;
        let mut stmt=self.connection.prepare("SELECT spreadsheet_id FROM registered_spreadsheets WHERE account_id=?1 AND removed=0 ORDER BY name COLLATE NOCASE,spreadsheet_id").map_err(sql)?;
        let ids = stmt
            .query_map([aid], |r| r.get::<_, String>(0))
            .map_err(sql)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql)?;
        ids.iter()
            .map(|id| raw_row(&self.connection, subject, id, false))
            .collect()
    }
    pub fn registered_spreadsheet(&self, subject: &str, id: &str) -> Result<RegisteredSpreadsheet> {
        raw_row(&self.connection, subject, id, false)
    }
    pub fn register_operational_spreadsheet(
        &mut self,
        subject: &str,
        metadata: &SpreadsheetMetadata,
    ) -> Result<RegisteredSpreadsheet> {
        validate_google_id(&metadata.spreadsheet_id)?;
        if metadata.title.is_empty()
            || metadata.title.len() > 4096
            || metadata.sheets.len() > crate::integrity::MAX_MONITORED_TABS
        {
            return Err(error(
                "invalid_metadata",
                "Os metadados da planilha são inválidos.",
            ));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let aid = account(&tx, subject)?;
        let exists:Option<bool>=tx.query_row("SELECT removed FROM registered_spreadsheets WHERE account_id=?1 AND spreadsheet_id=?2",params![aid,metadata.spreadsheet_id],|r|r.get(0)).optional().map_err(sql)?;
        if exists == Some(false) {
            return Err(error(
                "duplicate",
                "Esta planilha já está cadastrada nesta conta.",
            ));
        }
        let tabs = encode(&metadata.sheets)?;
        if exists == Some(true) {
            tx.execute("UPDATE registered_spreadsheets SET removed=0,paused=0,name=?3,tabs_json=?4,authorization='available',access_error=NULL,revision=revision+1,tabs_refreshed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE account_id=?1 AND spreadsheet_id=?2",params![aid,metadata.spreadsheet_id,metadata.title,tabs]).map_err(sql)?;
        } else {
            tx.execute("INSERT INTO registered_spreadsheets(account_id,spreadsheet_id,name,tabs_json,tabs_refreshed_at,created_at,updated_at) VALUES(?1,?2,?3,?4,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'))",params![aid,metadata.spreadsheet_id,metadata.title,tabs]).map_err(sql)?;
        }
        tx.commit().map_err(sql)?;
        self.registered_spreadsheet(subject, &metadata.spreadsheet_id)
    }
    pub fn check_registry_mutation(
        &self,
        subject: &str,
        m: &Mutation,
    ) -> Result<RegisteredSpreadsheet> {
        current(&self.connection, subject, m)
    }
    pub fn refresh_registered_metadata(
        &mut self,
        subject: &str,
        metadata: &SpreadsheetMetadata,
    ) -> Result<RegisteredSpreadsheet> {
        let aid = account(&self.connection, subject)?;
        let tabs = encode(&metadata.sheets)?;
        let updated=self.connection.execute("UPDATE registered_spreadsheets SET name=?3,tabs_json=?4,authorization='available',access_error=NULL,revision=revision+1,tabs_refreshed_at=strftime('%Y-%m-%dT%H:%M:%fZ','now'),updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE account_id=?1 AND spreadsheet_id=?2 AND removed=0",params![aid,metadata.spreadsheet_id,metadata.title,tabs]).map_err(sql)?;
        if updated != 1 {
            return Err(conflict());
        }
        self.registered_spreadsheet(subject, &metadata.spreadsheet_id)
    }
    pub fn record_registry_access_error(
        &mut self,
        subject: &str,
        id: &str,
        kind: &str,
    ) -> Result<RegisteredSpreadsheet> {
        let aid = account(&self.connection, subject)?;
        self.connection.execute("UPDATE registered_spreadsheets SET authorization='error',access_error=?3,revision=revision+1 WHERE account_id=?1 AND spreadsheet_id=?2 AND removed=0",params![aid,id,kind]).map_err(sql)?;
        self.registered_spreadsheet(subject, id)
    }
    pub fn set_registered_tabs(
        &mut self,
        subject: &str,
        m: &Mutation,
        sheet_ids: &[u32],
    ) -> Result<RegisteredSpreadsheet> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let row = current(&tx, subject, m)?;
        let set: BTreeSet<_> = sheet_ids.iter().copied().collect();
        if set.len() != sheet_ids.len()
            || set.len() > crate::integrity::MAX_MONITORED_TABS
            || set.iter().any(|id| {
                !row.tabs.iter().any(|s| {
                    s.sheet_id == *id && s.sheet_type.as_deref().is_none_or(|k| k == "GRID")
                })
            })
        {
            return Err(error(
                "invalid_tabs",
                "Selecione abas de grade disponíveis, respeitando o limite de 1024 abas e 30.000.000 de células por escopo.",
            ));
        }
        let aid = account(&tx, subject)?;
        tx.execute("UPDATE registered_spreadsheets SET selected_sheet_ids_json=?3,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE account_id=?1 AND spreadsheet_id=?2",params![aid,m.spreadsheet_id,encode(&sheet_ids)?]).map_err(sql)?;
        tx.commit().map_err(sql)?;
        self.registered_spreadsheet(subject, &m.spreadsheet_id)
    }
    pub fn set_registered_rules(
        &mut self,
        subject: &str,
        m: &Mutation,
        scope: &MonitorScope,
    ) -> Result<RegisteredSpreadsheet> {
        scope.validate()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let row = current(&tx, subject, m)?;
        let expected: BTreeSet<_> = row.selected_sheet_ids.iter().copied().collect();
        if scope
            .sheets
            .iter()
            .map(|s| s.sheet_id)
            .collect::<BTreeSet<_>>()
            != expected
        {
            return Err(error(
                "tabs_scope_mismatch",
                "As regras devem corresponder às abas selecionadas.",
            ));
        }
        let aid = account(&tx, subject)?;
        for script in &scope.scripts {
            let found:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM integration_associations a JOIN integration_resources s ON s.id=a.script_resource_id JOIN integration_resources f ON f.id=a.spreadsheet_resource_id WHERE a.account_id=?1 AND f.external_id=?2 AND s.external_id=?3 AND s.last_status='success')",params![aid,m.spreadsheet_id,script.script_id],|r|r.get(0)).map_err(sql)?;
            if !found {
                return Err(error(
                    "script_not_authorized",
                    "Consulte e associe o script nesta conta antes de protegê-lo.",
                ));
            }
        }
        tx.execute("UPDATE registered_spreadsheets SET scope_json=?3,scope_hash=?4,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE account_id=?1 AND spreadsheet_id=?2",params![aid,m.spreadsheet_id,encode(scope)?,fingerprint(scope)]).map_err(sql)?;
        tx.commit().map_err(sql)?;
        self.registered_spreadsheet(subject, &m.spreadsheet_id)
    }
    pub fn set_registered_paused(
        &mut self,
        subject: &str,
        m: &Mutation,
        paused: bool,
    ) -> Result<RegisteredSpreadsheet> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let row = current(&tx, subject, m)?;
        if !paused
            && (row.scope.is_none()
                || row.baseline_version.is_none()
                || row.scope.as_ref().is_some_and(|s| {
                    s.sheets.iter().map(|v| v.sheet_id).collect::<BTreeSet<_>>()
                        != row.selected_sheet_ids.iter().copied().collect()
                }))
        {
            return Err(error(
                "configuration_pending",
                "Configure regras e aprove uma baseline antes de retomar.",
            ));
        }
        let aid = account(&tx, subject)?;
        tx.execute("UPDATE registered_spreadsheets SET paused=?3,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE account_id=?1 AND spreadsheet_id=?2",params![aid,m.spreadsheet_id,paused]).map_err(sql)?;
        tx.commit().map_err(sql)?;
        self.registered_spreadsheet(subject, &m.spreadsheet_id)
    }
    pub fn remove_registered_spreadsheet(
        &mut self,
        subject: &str,
        input: &RemoveInput,
    ) -> Result<()> {
        if !input.confirmed {
            return Err(error(
                "confirmation_required",
                "Confirme a remoção do cadastro local.",
            ));
        }
        if !matches!(input.history.as_str(), "preserve" | "delete_activity") {
            return Err(error(
                "invalid_history_choice",
                "Escolha se deseja preservar ou excluir as atividades.",
            ));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        current(&tx, subject, &input.mutation())?;
        let aid = account(&tx, subject)?;
        let id = &input.spreadsheet_id;
        if input.history == "delete_activity" {
            tx.execute("DELETE FROM monitoring_operational_events WHERE account_id=?1 AND spreadsheet_id=?2",params![aid,id]).map_err(sql)?;
            tx.execute("UPDATE monitoring_schedules SET operational_code=NULL,failure_count=0,lease_id=NULL,lease_owner=NULL,lease_until_ms=NULL WHERE account_id=?1 AND spreadsheet_id=?2",params![aid,id]).map_err(sql)?;
            tx.execute("UPDATE integrity_scopes SET latest_report_id=NULL,previous_snapshot_id=NULL,observation_revision=observation_revision+1 WHERE account_id=?1 AND spreadsheet_id=?2",params![aid,id]).map_err(sql)?;
            for table in [
                "integrity_events",
                "integrity_incidents",
                "integrity_observation_components",
                "integrity_reports",
            ] {
                tx.execute(&format!("DELETE FROM {table} WHERE scope_id IN(SELECT id FROM integrity_scopes WHERE account_id=?1 AND spreadsheet_id=?2)"),params![aid,id]).map_err(sql)?;
            }
        }
        tx.execute("UPDATE registered_spreadsheets SET removed=1,paused=1,revision=revision+1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE account_id=?1 AND spreadsheet_id=?2",params![aid,id]).map_err(sql)?;
        tx.commit().map_err(sql)?;
        Ok(())
    }
    pub fn management_preferences(&self, subject: &str) -> Result<Preferences> {
        let aid = account(&self.connection, subject)?;
        let raw: Option<String> = self
            .connection
            .query_row(
                "SELECT preferences_json FROM management_preferences WHERE account_id=?1",
                [aid],
                |r| r.get(0),
            )
            .optional()
            .map_err(sql)?;
        raw.map(|v| decode(&v))
            .transpose()
            .map(|v| v.unwrap_or_default())
    }
    pub fn save_management_preferences(
        &mut self,
        subject: &str,
        p: &Preferences,
    ) -> Result<Preferences> {
        if !(60..=600).contains(&p.interval_seconds)
            || !(1..=3650).contains(&p.history_days)
            || !matches!(p.theme.as_str(), "system" | "light" | "dark")
            || p.start_with_windows
        {
            return Err(error("invalid_preferences", "Revise intervalo, retenção e aparência. A inicialização com Windows é configurada separadamente neste dispositivo."));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let aid = account(&tx, subject)?;
        tx.execute("INSERT INTO management_preferences(account_id,preferences_json) VALUES(?1,?2) ON CONFLICT(account_id) DO UPDATE SET preferences_json=excluded.preferences_json",params![aid,encode(p)?]).map_err(sql)?;
        if !p.notifications_enabled {
            super::monitoring_store::suppress(&tx, &aid, chrono::Utc::now().timestamp_millis())?;
        }
        tx.commit().map_err(sql)?;
        Ok(p.clone())
    }
}
const UNIFIED_EVENTS_CTE: &str = "WITH unified AS (
SELECT e.event_id AS event_id,'integrity' AS source,s.spreadsheet_id AS spreadsheet_id,r.name AS spreadsheet_name,e.sheet_id AS sheet_id,
COALESCE(
  (SELECT json_extract(j.value,'$.title') FROM structural_snapshots ss,json_each(ss.snapshot_json,'$.sheets') j WHERE ss.id=p.snapshot_id AND json_extract(j.value,'$.sheet_id')=e.sheet_id LIMIT 1),
  (SELECT json_extract(j.value,'$.title') FROM baseline_versions b JOIN structural_snapshots ss ON ss.id=b.snapshot_id,json_each(ss.snapshot_json,'$.sheets') j WHERE b.id=s.active_baseline_id AND json_extract(j.value,'$.sheet_id')=e.sheet_id LIMIT 1),
  (SELECT json_extract(j.value,'$.title') FROM json_each(r.tabs_json) j WHERE json_extract(j.value,'$.sheet_id')=e.sheet_id LIMIT 1)
) AS sheet_title,
e.detected_at AS detected_at,julianday(e.detected_at) AS detected_jd,
COALESCE(json_extract(e.event_json,'$.kind'),'') AS kind,
COALESCE(json_extract(e.event_json,'$.category'),'integrity') AS category,
e.severity AS severity,
COALESCE(json_extract(e.event_json,'$.diagnosis'),'') AS diagnosis,
json_extract(e.event_json,'$.rule_id') AS rule_id,
p.status AS status,r.removed AS removed,
CASE
  WHEN json_extract(e.event_json,'$.kind') IN ('recovered','analysis_concluded') THEN 'resolution'
  WHEN json_extract(e.event_json,'$.kind') IN ('violation','analysis_inconclusive') OR e.severity IN ('critical','warning') THEN 'failure'
  ELSE 'modification'
END AS event_type,
CASE
  WHEN json_extract(e.event_json,'$.kind') IN ('recovered','analysis_concluded') THEN 'resolved'
  WHEN json_extract(e.event_json,'$.kind') IN ('violation','analysis_inconclusive') OR e.severity IN ('critical','warning') THEN (
    CASE
      WHEN EXISTS(SELECT 1 FROM integrity_events er WHERE er.scope_id=e.scope_id AND er.account_id=e.account_id AND json_extract(er.event_json,'$.kind') IN ('recovered','analysis_concluded') AND COALESCE(json_extract(er.event_json,'$.rule_id'),'')=COALESCE(json_extract(e.event_json,'$.rule_id'),'') AND COALESCE(er.sheet_id,-1)=COALESCE(e.sheet_id,-1) AND julianday(er.detected_at)>julianday(e.detected_at)) THEN 'resolved'
      WHEN EXISTS(SELECT 1 FROM integrity_incidents ii WHERE ii.scope_id=e.scope_id AND ii.account_id=e.account_id AND ii.active=1 AND json_extract(ii.finding_json,'$.rule_id')=json_extract(e.event_json,'$.rule_id') AND COALESCE(json_extract(ii.finding_json,'$.sheet_id'),-1)=COALESCE(e.sheet_id,-1)) THEN 'active'
      ELSE 'resolved'
    END
  )
  ELSE NULL
END AS issue_status,
e.event_json AS event_json
FROM integrity_events e
JOIN integrity_reports p ON p.check_id=e.check_id AND p.account_id=e.account_id
JOIN integrity_scopes s ON s.id=e.scope_id AND s.account_id=e.account_id
JOIN registered_spreadsheets r ON r.account_id=e.account_id AND r.spreadsheet_id=s.spreadsheet_id
WHERE e.account_id=?1 AND json_extract(e.event_json,'$.category')='integrity' AND length(trim(COALESCE(json_extract(e.event_json,'$.rule_id'),'')))>0
)";

const UNIFIED_EVENTS_WHERE: &str = " FROM unified u WHERE (?2 IS NULL OR u.spreadsheet_id=?2) AND (?3 IS NULL OR u.sheet_id=?3) AND (?4 IS NULL OR u.detected_jd>=julianday(?4)) AND (?5 IS NULL OR u.detected_jd<=julianday(?5)) AND (?6 IS NULL OR u.kind=?6) AND (?7 IS NULL OR u.severity=?7) AND (?8 IS NULL OR u.status=?8) AND (?9 IS NULL OR u.category=?9) AND (NOT ?10 OR u.category='script' OR u.kind LIKE '%script%' OR COALESCE(u.rule_id,'') LIKE 'script%') AND (?11='' OR instr(lower(u.spreadsheet_name||' '||COALESCE(u.sheet_title,'')||' '||u.diagnosis||' '||COALESCE(u.rule_id,'')),lower(?11))>0) AND (?12 IS NULL OR u.event_type=?12) AND (?13 IS NULL OR u.issue_status=?13)";

fn validate_event_filter(f: &EventFilter, check_pagination: bool) -> Result<()> {
    if check_pagination && (f.limit == 0 || f.limit > 100 || f.offset > 1_000_000) {
        return Err(error(
            "invalid_filter",
            "Revise os filtros, datas e paginação.",
        ));
    }
    if f.query.len() > 256
        || f.from
            .as_ref()
            .is_some_and(|d| chrono::DateTime::parse_from_rfc3339(d).is_err())
        || f.to
            .as_ref()
            .is_some_and(|d| chrono::DateTime::parse_from_rfc3339(d).is_err())
    {
        return Err(error(
            "invalid_filter",
            "Revise os filtros, datas e paginação.",
        ));
    }
    if let (Some(from), Some(to)) = (&f.from, &f.to) {
        if let (Ok(a), Ok(b)) = (
            chrono::DateTime::parse_from_rfc3339(from),
            chrono::DateTime::parse_from_rfc3339(to),
        ) {
            if a > b {
                return Err(error(
                    "invalid_filter",
                    "A data inicial não pode ser posterior à data final.",
                ));
            }
        }
    }
    if let Some(id) = &f.spreadsheet_id {
        validate_google_id(id)?;
    }
    if let Some(et) = &f.event_type {
        if !matches!(et.as_str(), "failure" | "modification" | "resolution") {
            return Err(error("invalid_filter", "Tipo de evento inválido."));
        }
    }
    if let Some(st) = &f.issue_status {
        if !matches!(st.as_str(), "active" | "resolved") {
            return Err(error("invalid_filter", "Status de ocorrência inválido."));
        }
    }
    for value in [&f.kind, &f.severity, &f.status, &f.category]
        .into_iter()
        .flatten()
    {
        if value.len() > 128 {
            return Err(error("invalid_filter", "Filtro de atividade inválido."));
        }
    }
    Ok(())
}

type UnifiedRow = (
    String,
    String,
    String,
    String,
    Option<String>,
    String,
    String,
    String,
    String,
    String,
    bool,
    String,
    Option<String>,
    Option<String>,
);

fn map_unified_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<UnifiedRow> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        r.get(2)?,
        r.get(3)?,
        r.get(4)?,
        r.get(5)?,
        r.get(6)?,
        r.get(7)?,
        r.get(8)?,
        r.get(9)?,
        r.get(10)?,
        r.get(11)?,
        r.get(12)?,
        r.get(13)?,
    ))
}

fn build_activity(row: UnifiedRow) -> Result<Activity> {
    let (
        event_id,
        source,
        spreadsheet_id,
        spreadsheet_name,
        sheet_title,
        detected_at,
        kind,
        severity,
        diagnosis,
        status,
        removed,
        event_type,
        issue_status,
        event_json,
    ) = row;
    let event = match event_json {
        Some(raw) => decode::<IntegrityEvent>(&raw)?,
        None => IntegrityEvent {
            event_id: event_id.clone(),
            check_id: format!("operational:{event_id}"),
            spreadsheet_id,
            sheet_id: None,
            kind: kind.clone(),
            category: "operational".into(),
            severity: if severity == "info" {
                Severity::Info
            } else {
                Severity::Warning
            },
            rule_id: None,
            before: None,
            after: None,
            location: None,
            detected_at: detected_at.clone(),
            fingerprint: event_id,
            diagnosis: diagnosis.clone(),
        },
    };
    let (_, tone) = events::classify_event(&source, Some(&kind), &severity, &diagnosis);
    let description = events::diagnosis_pt_br(&event.diagnosis, Some(&event.kind));
    let formatted_detected_at = events::format_local_datetime(&event.detected_at);
    Ok(Activity {
        event,
        spreadsheet_name,
        sheet_title,
        status,
        removed,
        event_type,
        tone: tone.into(),
        issue_status,
        description,
        formatted_detected_at,
    })
}

fn format_location_pt_br(location: Option<&str>) -> String {
    let Some(loc) = location.map(str::trim).filter(|l| !l.is_empty()) else {
        return String::new();
    };
    if let Some(col_raw) = loc.strip_prefix("column:") {
        if let Ok(col) = col_raw.parse::<u32>() {
            return format!("Coluna {}", col + 1);
        }
    }
    if let Some(rest) = loc.strip_prefix("row:") {
        if let Some((row_raw, col_name)) = rest.split_once(":column:") {
            if let Ok(row) = row_raw.parse::<u32>() {
                return format!("Linha {} · Campo {col_name}", row + 1);
            }
        }
        if let Ok(row) = rest.parse::<u32>() {
            return format!("Linha {}", row + 1);
        }
    }
    loc.chars().filter(|c| !c.is_control()).collect()
}

fn severity_pt_br(severity: &Severity) -> &'static str {
    match severity {
        Severity::Critical => "Crítico",
        Severity::Warning => "Aviso",
        Severity::Info => "Informativo",
    }
}

fn category_pt_br(category: &str) -> &'static str {
    match category {
        "integrity" => "Integridade",
        "content" => "Conteúdo",
        "structure" => "Estrutura",
        "script" => "Apps Script",
        "operational" => "Operacional",
        _ => "Monitoramento",
    }
}

fn spreadsheet_status_pt_br(status: &str) -> &str {
    match status {
        "online" => "Online",
        "offline" => "Offline",
        "indeterminate" => "Indeterminada",
        "paused" => "Pausada",
        "configuration_pending" => "Configuração pendente",
        "not_checked" => "Não verificada",
        _ => status,
    }
}

impl Database {
    pub fn operational_events(&self, subject: &str, f: &EventFilter) -> Result<Page<Activity>> {
        validate_event_filter(f, true)?;
        let aid = account(&self.connection, subject)?;
        let values = params![
            aid,
            f.spreadsheet_id,
            f.sheet_id,
            f.from,
            f.to,
            f.kind,
            f.severity,
            f.status,
            f.category,
            f.script_only,
            f.query,
            f.event_type,
            f.issue_status
        ];
        let total = self
            .connection
            .query_row(
                &format!("{UNIFIED_EVENTS_CTE} SELECT count(*){UNIFIED_EVENTS_WHERE}"),
                values,
                |r| r.get::<_, u64>(0),
            )
            .map_err(sql)?;
        let select = "SELECT u.event_id,u.source,u.spreadsheet_id,u.spreadsheet_name,u.sheet_title,u.detected_at,u.kind,u.severity,u.diagnosis,u.status,u.removed,u.event_type,u.issue_status,u.event_json";
        let mut stmt = self
            .connection
            .prepare(&format!(
                "{UNIFIED_EVENTS_CTE} {select}{UNIFIED_EVENTS_WHERE} ORDER BY u.detected_jd DESC,u.event_id DESC LIMIT ?14 OFFSET ?15"
            ))
            .map_err(sql)?;
        let rows = stmt
            .query_map(
                params![
                    aid,
                    f.spreadsheet_id,
                    f.sheet_id,
                    f.from,
                    f.to,
                    f.kind,
                    f.severity,
                    f.status,
                    f.category,
                    f.script_only,
                    f.query,
                    f.event_type,
                    f.issue_status,
                    f.limit,
                    f.offset
                ],
                map_unified_row,
            )
            .map_err(sql)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql)?;
        let items = rows
            .into_iter()
            .map(build_activity)
            .collect::<Result<Vec<_>>>()?;
        Ok(Page {
            items,
            total,
            limit: f.limit,
            offset: f.offset,
        })
    }

    pub fn export_operational_events(
        &self,
        subject: &str,
        f: &EventFilter,
        format: &str,
        destination_dir: &Path,
    ) -> Result<ExportEventsResult> {
        let normalized_format = format.trim().to_ascii_lowercase();
        if !matches!(normalized_format.as_str(), "csv" | "json") {
            return Err(error(
                "invalid_format",
                "Formato de exportação inválido. Escolha CSV ou JSON.",
            ));
        }
        validate_event_filter(f, false)?;
        let aid = account(&self.connection, subject)?;
        let values = params![
            aid,
            f.spreadsheet_id,
            f.sheet_id,
            f.from,
            f.to,
            f.kind,
            f.severity,
            f.status,
            f.category,
            f.script_only,
            f.query,
            f.event_type,
            f.issue_status
        ];
        let total = self
            .connection
            .query_row(
                &format!("{UNIFIED_EVENTS_CTE} SELECT count(*){UNIFIED_EVENTS_WHERE}"),
                values,
                |r| r.get::<_, u64>(0),
            )
            .map_err(sql)?;
        if total == 0 {
            return Ok(ExportEventsResult {
                exported_count: 0,
                file_path: None,
                format: normalized_format,
            });
        }
        fs::create_dir_all(destination_dir).map_err(|_| {
            error(
                "export_io_error",
                "Não foi possível preparar a pasta de destino para exportar os logs.",
            )
        })?;
        let now_local = chrono::Local::now();
        let stamp = now_local.format("%Y%m%d-%H%M%S-%3f");
        let file_name = format!("sheets-monitor-logs-{stamp}.{normalized_format}");
        let final_path = destination_dir.join(&file_name);
        let temp_path = destination_dir.join(format!("{file_name}.part"));
        let select = "SELECT u.event_id,u.source,u.spreadsheet_id,u.spreadsheet_name,u.sheet_title,u.detected_at,u.kind,u.severity,u.diagnosis,u.status,u.removed,u.event_type,u.issue_status,u.event_json";
        let mut stmt = self
            .connection
            .prepare(&format!(
                "{UNIFIED_EVENTS_CTE} {select}{UNIFIED_EVENTS_WHERE} ORDER BY u.detected_jd ASC,u.event_id ASC"
            ))
            .map_err(sql)?;
        let mut exported_count = 0u64;
        let write_result = (|| -> Result<()> {
            let file = File::create(&temp_path).map_err(|_| {
                error(
                    "export_io_error",
                    "Não foi possível criar o arquivo de exportação.",
                )
            })?;
            let mut writer = BufWriter::new(file);
            if normalized_format == "csv" {
                writer
                    .write_all("\u{FEFF}".as_bytes())
                    .map_err(|_| CommandError::local())?;
                events::write_csv_row(
                    &mut writer,
                    &[
                        "Data/Hora",
                        "Planilha",
                        "Aba",
                        "Tipo",
                        "Status",
                        "Severidade",
                        "Categoria",
                        "Descrição",
                        "Localização",
                        "Estado da Planilha",
                    ],
                )
                .map_err(|_| CommandError::local())?;
            } else {
                let header = serde_json::json!({
                    "schema_version": 1,
                    "exported_at": now_local.format("%d/%m/%Y %H:%M:%S").to_string(),
                    "total": total,
                });
                let header_text =
                    serde_json::to_string(&header).map_err(|_| CommandError::local())?;
                let prefix = header_text.trim_end_matches('}');
                write!(writer, "{prefix},\"items\":[").map_err(|_| CommandError::local())?;
            }
            let iter = stmt
                .query_map(
                    params![
                        aid,
                        f.spreadsheet_id,
                        f.sheet_id,
                        f.from,
                        f.to,
                        f.kind,
                        f.severity,
                        f.status,
                        f.category,
                        f.script_only,
                        f.query,
                        f.event_type,
                        f.issue_status
                    ],
                    map_unified_row,
                )
                .map_err(sql)?;
            for raw_row in iter {
                let activity = build_activity(raw_row.map_err(sql)?)?;
                let tab = activity.sheet_title.as_deref().unwrap_or("Geral");
                let kind_label = events::event_type_pt_br(&activity.event_type);
                let status_label = events::issue_status_pt_br(activity.issue_status.as_deref());
                let sev_label = severity_pt_br(&activity.event.severity);
                let cat_label = category_pt_br(&activity.event.category);
                let loc_label = format_location_pt_br(activity.event.location.as_deref());
                let sheet_status = spreadsheet_status_pt_br(&activity.status);
                if normalized_format == "csv" {
                    events::write_csv_row(
                        &mut writer,
                        &[
                            &activity.formatted_detected_at,
                            &activity.spreadsheet_name,
                            tab,
                            kind_label,
                            status_label,
                            sev_label,
                            cat_label,
                            &activity.description,
                            &loc_label,
                            sheet_status,
                        ],
                    )
                    .map_err(|_| CommandError::local())?;
                } else {
                    if exported_count > 0 {
                        writer.write_all(b",").map_err(|_| CommandError::local())?;
                    }
                    let record = serde_json::json!({
                        "detected_at_local": activity.formatted_detected_at,
                        "detected_at_utc": activity.event.detected_at,
                        "spreadsheet_name": activity.spreadsheet_name,
                        "sheet_title": activity.sheet_title,
                        "event_type": kind_label,
                        "status": status_label,
                        "severity": sev_label,
                        "category": cat_label,
                        "description": activity.description,
                        "location": if loc_label.is_empty() { None } else { Some(loc_label) },
                        "spreadsheet_status": sheet_status,
                    });
                    serde_json::to_writer(&mut writer, &record)
                        .map_err(|_| CommandError::local())?;
                }
                exported_count += 1;
            }
            if normalized_format == "json" {
                writer
                    .write_all(b"]}\n")
                    .map_err(|_| CommandError::local())?;
            }
            writer.flush().map_err(|_| CommandError::local())?;
            Ok(())
        })();
        if let Err(err) = write_result {
            let _ = fs::remove_file(&temp_path);
            return Err(err);
        }
        fs::rename(&temp_path, &final_path).map_err(|_| {
            let _ = fs::remove_file(&temp_path);
            error(
                "export_io_error",
                "Não foi possível finalizar o arquivo de exportação.",
            )
        })?;
        Ok(ExportEventsResult {
            exported_count,
            file_path: Some(final_path.to_string_lossy().into_owned()),
            format: normalized_format,
        })
    }
    pub fn operational_baselines(
        &self,
        subject: &str,
        q: &BaselinesQuery,
    ) -> Result<Page<BaselineSummary>> {
        if q.limit == 0 || q.limit > 20 || q.offset > 100000 {
            return Err(error("invalid_page", "Página de versões inválida."));
        }
        let registry = self.registered_spreadsheet(subject, &q.spreadsheet_id)?;
        let aid = account(&self.connection, subject)?;
        let total=self.connection.query_row("SELECT count(*) FROM baseline_versions b JOIN integrity_scopes s ON s.id=b.scope_id WHERE b.account_id=?1 AND s.spreadsheet_id=?2",params![aid,q.spreadsheet_id],|r|r.get(0)).map_err(sql)?;
        let mut stmt=self.connection.prepare("SELECT b.id,b.version,b.approved_at,b.approval_origin,b.snapshot_id,s.scope_hash,s.active_baseline_id=b.id FROM baseline_versions b JOIN integrity_scopes s ON s.id=b.scope_id WHERE b.account_id=?1 AND s.spreadsheet_id=?2 ORDER BY b.approved_at DESC,b.id DESC LIMIT ?3 OFFSET ?4").map_err(sql)?;
        let mut items = stmt
            .query_map(params![aid, q.spreadsheet_id, q.limit, q.offset], |r| {
                Ok(BaselineSummary {
                    id: r.get(0)?,
                    version: r.get(1)?,
                    approved_at: r.get(2)?,
                    approval_origin: r.get(3)?,
                    snapshot_id: r.get(4)?,
                    scope_hash: r.get(5)?,
                    active: r.get(6)?,
                })
            })
            .map_err(sql)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql)?;
        for item in &mut items {
            item.active =
                item.active && configured_hash(&registry) == Some(item.scope_hash.as_str());
        }
        Ok(Page {
            items,
            total,
            offset: q.offset,
            limit: q.limit,
        })
    }
    pub fn operational_baseline_by_id(
        &self,
        subject: &str,
        spreadsheet_id: &str,
        id: &str,
    ) -> Result<BaselineVersion> {
        let registry = self.registered_spreadsheet(subject, spreadsheet_id)?;
        let aid = account(&self.connection, subject)?;
        let row=self.connection.query_row("SELECT b.snapshot_id,b.version,b.approved_at,b.approval_origin,s.active_baseline_id=b.id FROM baseline_versions b JOIN integrity_scopes s ON s.id=b.scope_id WHERE b.id=?1 AND b.account_id=?2 AND s.spreadsheet_id=?3",params![id,aid,spreadsheet_id],|r|Ok((r.get::<_,String>(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().map_err(sql)?.ok_or_else(||error("baseline_not_found","A versão não pertence a esta planilha."))?;
        let snapshot = self.structural_snapshot(subject, &row.0)?;
        let active = row.4 && configured_hash(&registry) == Some(snapshot.scope_hash.as_str());
        Ok(BaselineVersion {
            id: id.into(),
            snapshot,
            version: row.1,
            approved_at: row.2,
            approval_origin: row.3,
            active,
        })
    }
    pub fn operational_scripts(&self, subject: &str, id: &str) -> Result<Vec<ScriptSummary>> {
        let row = self.registered_spreadsheet(subject, id)?;
        let mut analyses = Vec::new();
        if let Some(hash) = &row.scope_hash {
            if let Some(report) = self.latest_integrity_report(subject, id, hash)? {
                if let Ok(snapshot) = self.structural_snapshot(subject, &report.snapshot_id) {
                    analyses = snapshot.scripts;
                }
            } else if let Some(baseline) = self.active_structural_baseline(subject, id, hash)? {
                analyses = baseline.snapshot.scripts;
            }
        }
        let aid = account(&self.connection, subject)?;
        let mut stmt=self.connection.prepare("SELECT s.external_id,s.name,s.last_status,s.last_queried_at FROM integration_associations a JOIN integration_resources s ON s.id=a.script_resource_id JOIN integration_resources f ON f.id=a.spreadsheet_resource_id WHERE a.account_id=?1 AND f.external_id=?2 ORDER BY s.name,s.external_id").map_err(sql)?;
        let mut rows = stmt
            .query_map(params![aid, id], |r| {
                Ok(ScriptSummary {
                    script_id: r.get(0)?,
                    name: r.get(1)?,
                    authorization: r.get(2)?,
                    last_queried_at: r.get(3)?,
                    critical: false,
                    analysis: None,
                })
            })
            .map_err(sql)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql)?;
        for s in &mut rows {
            s.critical = row.scope.as_ref().is_some_and(|scope| {
                scope
                    .scripts
                    .iter()
                    .any(|r| r.script_id == s.script_id && r.critical)
            });
            s.analysis = analyses
                .iter()
                .find(|a| a.script_id == s.script_id)
                .cloned();
        }
        Ok(rows)
    }
    pub fn operational_diagnostics(&self, subject: &str, id: &str) -> Result<Diagnostics> {
        let spreadsheet = self.registered_spreadsheet(subject, id)?;
        let (report, baseline) = if let Some(hash) = &spreadsheet.scope_hash {
            (
                self.latest_integrity_report(subject, id, hash)?,
                self.active_structural_baseline(subject, id, hash)?
                    .map(BaselineSummary::from),
            )
        } else {
            (None, None)
        };
        let report = report.filter(|r| {
            baseline.as_ref().is_some_and(|b| b.id == r.baseline_id)
                && spreadsheet.status != "configuration_pending"
        });
        let scripts = self.operational_scripts(subject, id)?;
        Ok(Diagnostics {
            spreadsheet,
            report,
            baseline,
            scripts,
        })
    }
    pub fn operational_dashboard(&self, subject: &str) -> Result<DashboardSummary> {
        let spreadsheets = self.registered_spreadsheets(subject)?;
        let count =
            |status: &str| spreadsheets.iter().filter(|s| s.status == status).count() as u32;
        let summary = DashboardSummary {
            registered: spreadsheets.len() as u32,
            online: count("online"),
            offline: count("offline"),
            indeterminate: count("indeterminate"),
            paused: count("paused"),
            pending: count("configuration_pending"),
            not_checked: count("not_checked"),
            recent_events: self
                .operational_events(
                    subject,
                    &EventFilter {
                        limit: 8,
                        ..Default::default()
                    },
                )?
                .items,
            recent_failures: self
                .operational_events(
                    subject,
                    &EventFilter {
                        limit: 8,
                        severity: Some("critical".into()),
                        category: Some("integrity".into()),
                        kind: Some("violation".into()),
                        ..Default::default()
                    },
                )?
                .items,
            spreadsheets,
        };
        Ok(summary)
    }
}
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
