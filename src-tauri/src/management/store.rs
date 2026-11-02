use super::Database;
use crate::{
    google::{validate_google_id, SpreadsheetMetadata},
    integration::CommandError,
    integrity::runtime::BaselineSummary,
    integrity::{fingerprint, BaselineVersion, IntegrityEvent, MonitorScope},
    management::*,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::BTreeSet;
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
                "Selecione abas de grade disponíveis, respeitando o limite de 1024 abas e 20.000 células por escopo.",
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
        if !(60..=300).contains(&p.interval_seconds)
            || !(1..=3650).contains(&p.history_days)
            || !matches!(p.theme.as_str(), "system" | "light" | "dark")
            || p.start_with_windows
            || p.run_minimized
        {
            return Err(error("invalid_preferences","Revise as preferências. Inicialização automática e execução minimizada estarão disponíveis posteriormente."));
        }
        let aid = account(&self.connection, subject)?;
        self.connection.execute("INSERT INTO management_preferences(account_id,preferences_json) VALUES(?1,?2) ON CONFLICT(account_id) DO UPDATE SET preferences_json=excluded.preferences_json",params![aid,encode(p)?]).map_err(sql)?;
        Ok(p.clone())
    }
}
impl Database {
    pub fn operational_events(&self, subject: &str, f: &EventFilter) -> Result<Page<Activity>> {
        if f.limit == 0
            || f.limit > 100
            || f.offset > 1_000_000
            || f.query.len() > 256
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
        if let Some(id) = &f.spreadsheet_id {
            validate_google_id(id)?;
        }
        for value in [&f.kind, &f.severity, &f.status, &f.category]
            .into_iter()
            .flatten()
        {
            if value.len() > 128 {
                return Err(error("invalid_filter", "Filtro de atividade inválido."));
            }
        }
        let aid = account(&self.connection, subject)?;
        let predicate=" FROM integrity_events e JOIN integrity_reports p ON p.check_id=e.check_id AND p.account_id=e.account_id JOIN integrity_scopes s ON s.id=e.scope_id AND s.account_id=e.account_id JOIN registered_spreadsheets r ON r.account_id=e.account_id AND r.spreadsheet_id=s.spreadsheet_id WHERE e.account_id=?1 AND (?2 IS NULL OR s.spreadsheet_id=?2) AND (?3 IS NULL OR e.sheet_id=?3) AND (?4 IS NULL OR julianday(e.detected_at)>=julianday(?4)) AND (?5 IS NULL OR julianday(e.detected_at)<=julianday(?5)) AND (?6 IS NULL OR json_extract(e.event_json,'$.kind')=?6) AND (?7 IS NULL OR e.severity=?7) AND (?8 IS NULL OR p.status=?8) AND (?9 IS NULL OR json_extract(e.event_json,'$.category')=?9) AND (NOT ?10 OR json_extract(e.event_json,'$.category')='script' OR json_extract(e.event_json,'$.kind') LIKE '%script%' OR json_extract(e.event_json,'$.rule_id') LIKE 'script%') AND (?11='' OR instr(lower(r.name||' '||COALESCE(json_extract(e.event_json,'$.diagnosis'),'')||' '||COALESCE(json_extract(e.event_json,'$.rule_id'),'')),lower(?11))>0)";
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
            f.query
        ];
        let total = self
            .connection
            .query_row(&format!("SELECT count(*){predicate}"), values, |r| {
                r.get::<_, u64>(0)
            })
            .map_err(sql)?;
        let select="SELECT e.event_json,r.name,COALESCE((SELECT json_extract(j.value,'$.title') FROM structural_snapshots ss,json_each(ss.snapshot_json,'$.sheets') j WHERE ss.id=p.snapshot_id AND json_extract(j.value,'$.sheet_id')=e.sheet_id LIMIT 1),(SELECT json_extract(j.value,'$.title') FROM json_each(r.tabs_json) j WHERE json_extract(j.value,'$.sheet_id')=e.sheet_id LIMIT 1)),p.status,r.removed";
        let mut stmt=self.connection.prepare(&format!("{select}{predicate} ORDER BY e.detected_at DESC,e.event_id DESC LIMIT ?12 OFFSET ?13")).map_err(sql)?;
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
                    f.limit,
                    f.offset
                ],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, Option<String>>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, bool>(4)?,
                    ))
                },
            )
            .map_err(sql)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(sql)?;
        let items = rows
            .into_iter()
            .map(|(raw, spreadsheet_name, sheet_title, status, removed)| {
                Ok(Activity {
                    event: decode::<IntegrityEvent>(&raw)?,
                    spreadsheet_name,
                    sheet_title,
                    status,
                    removed,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Page {
            items,
            total,
            limit: f.limit,
            offset: f.offset,
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
