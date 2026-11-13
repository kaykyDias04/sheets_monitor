use super::Database;
use crate::{
    cloud_sync::{
        canonical_hash, decode, encode, error, Commit, Configuration, SyncPreferences,
        SyncRegistry, SyncResult, SyncStatus,
    },
    integrity::{fingerprint, BaselineVersion, StructuralSnapshot},
    management::Preferences,
};
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};

pub struct CloudCapture {
    pub payload: Configuration,
    pub base_commit: Option<String>,
    pub base_payload_hash: Option<String>,
    pub last_synced_at: Option<String>,
    pub device_id: String,
}
fn sql(_: rusqlite::Error) -> crate::integration::CommandError {
    error("cloud_storage")
}
fn account(c: &Connection, subject: &str) -> SyncResult<String> {
    c.query_row(
        "SELECT id FROM accounts WHERE provider_subject=?1 AND is_demo=0",
        [subject],
        |r| r.get(0),
    )
    .optional()
    .map_err(sql)?
    .ok_or_else(|| error("cloud_account"))
}
fn preferences(c: &Connection, aid: &str) -> SyncResult<Preferences> {
    let json: Option<String> = c
        .query_row(
            "SELECT preferences_json FROM management_preferences WHERE account_id=?1",
            [aid],
            |r| r.get(0),
        )
        .optional()
        .map_err(sql)?;
    json.map(|s| decode(&s))
        .unwrap_or_else(|| Ok(Preferences::default()))
}
fn payload(c: &Connection, subject: &str) -> SyncResult<Configuration> {
    let aid = account(c, subject)?;
    let p = preferences(c, &aid)?;
    let mut stmt=c.prepare("SELECT spreadsheet_id,name,paused,removed,scope_json,selected_sheet_ids_json,tabs_json FROM registered_spreadsheets WHERE account_id=?1 ORDER BY spreadsheet_id").map_err(sql)?;
    let rows = stmt
        .query_map([&aid], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, bool>(2)?,
                r.get::<_, bool>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
            ))
        })
        .map_err(sql)?;
    let mut registries = Vec::new();
    for row in rows {
        let (spreadsheet_id, name, paused, removed, scope, selected, tabs) = row.map_err(sql)?;
        registries.push(SyncRegistry {
            spreadsheet_id,
            name,
            paused,
            removed,
            scope: scope.map(|s| decode(&s)).transpose()?,
            selected_sheet_ids: decode(&selected)?,
            tabs: decode(&tabs)?,
        });
    }
    let mut stmt=c.prepare("SELECT b.id,COALESCE(b.sync_version,b.version),b.approved_at,b.approval_origin,COALESCE(s.active_baseline_id=b.id,0),n.snapshot_json,n.fingerprint FROM baseline_versions b JOIN integrity_scopes s ON s.id=b.scope_id AND s.account_id=b.account_id JOIN structural_snapshots n ON n.id=b.snapshot_id AND n.scope_id=b.scope_id AND n.account_id=b.account_id JOIN registered_spreadsheets r ON r.account_id=s.account_id AND r.spreadsheet_id=s.spreadsheet_id WHERE b.account_id=?1 AND b.approval_origin='human:native' ORDER BY s.spreadsheet_id,s.scope_hash,COALESCE(b.sync_version,b.version),b.id").map_err(sql)?;
    let rows = stmt
        .query_map([&aid], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
            ))
        })
        .map_err(sql)?;
    let mut baselines = Vec::new();
    for row in rows {
        let (id, version, approved_at, approval_origin, active, json, hash) = row.map_err(sql)?;
        let snapshot: StructuralSnapshot = decode(&json)?;
        if fingerprint(&snapshot) != hash {
            return Err(error("cloud_invalid_baseline"));
        }
        baselines.push(BaselineVersion {
            id,
            version,
            approved_at,
            approval_origin,
            active,
            snapshot,
        });
    }
    let result = Configuration {
        registries,
        preferences: SyncPreferences {
            interval_seconds: p.interval_seconds,
            notifications_enabled: p.notifications_enabled,
            history_days: p.history_days,
            theme: p.theme,
        },
        baselines,
    };
    result.validate(subject)?;
    Ok(result)
}
fn restored_resource(
    c: &Connection,
    aid: &str,
    kind: &str,
    external_id: &str,
) -> SyncResult<String> {
    let id = format!(
        "resource-{:x}",
        Sha256::digest(format!("{aid}:{kind}:{external_id}").as_bytes())
    );
    // A restored link is not proof that this device has queried or authorized the resource.
    c.execute("INSERT INTO integration_resources(id,account_id,kind,external_id,last_status,last_error_kind,last_queried_at) VALUES(?1,?2,?3,?4,'error','not_consulted','') ON CONFLICT(account_id,kind,external_id) DO NOTHING",params![id,aid,kind,external_id]).map_err(sql)?;
    c.query_row(
        "SELECT id FROM integration_resources WHERE account_id=?1 AND kind=?2 AND external_id=?3",
        params![aid, kind, external_id],
        |r| r.get(0),
    )
    .map_err(sql)
}
fn restore(c: &Connection, subject: &str, p: &Configuration) -> SyncResult<()> {
    p.validate(subject)?;
    let aid = account(c, subject)?;
    // Tombstone first, so replacing thirty active files never temporarily exceeds capacity.
    c.execute("UPDATE registered_spreadsheets SET removed=1,paused=1,revision=revision+1 WHERE account_id=?1",[&aid]).map_err(sql)?;
    for r in &p.registries {
        c.execute("INSERT INTO registered_spreadsheets(account_id,spreadsheet_id,name,paused,removed,scope_json,scope_hash,selected_sheet_ids_json,tabs_json,revision,authorization,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,1,'unknown',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')) ON CONFLICT(account_id,spreadsheet_id) DO UPDATE SET name=excluded.name,paused=excluded.paused,removed=excluded.removed,scope_json=excluded.scope_json,scope_hash=excluded.scope_hash,selected_sheet_ids_json=excluded.selected_sheet_ids_json,tabs_json=excluded.tabs_json,revision=registered_spreadsheets.revision+1,updated_at=excluded.updated_at",params![aid,r.spreadsheet_id,r.name,r.paused,r.removed,r.scope.as_ref().map(encode).transpose()?,r.scope.as_ref().map(|s|s.hash()),encode(&r.selected_sheet_ids)?,encode(&r.tabs)?]).map_err(sql)?;
        let scripts: std::collections::BTreeSet<_> = r
            .scope
            .iter()
            .chain(
                p.baselines
                    .iter()
                    .filter(|b| b.snapshot.spreadsheet_id == r.spreadsheet_id)
                    .map(|b| &b.snapshot.scope),
            )
            .flat_map(|scope| scope.scripts.iter().map(|script| script.script_id.as_str()))
            .collect();
        if !scripts.is_empty() {
            let sheet = restored_resource(c, &aid, "spreadsheet", &r.spreadsheet_id)?;
            for script in scripts {
                let script = restored_resource(c, &aid, "script", script)?;
                c.execute("INSERT INTO integration_associations(account_id,spreadsheet_resource_id,script_resource_id,created_at) VALUES(?1,?2,?3,strftime('%Y-%m-%dT%H:%M:%fZ','now')) ON CONFLICT DO NOTHING",params![aid,sheet,script]).map_err(sql)?;
            }
        }
    }
    let mut prefs = preferences(c, &aid)?;
    prefs.interval_seconds = p.preferences.interval_seconds;
    prefs.notifications_enabled = p.preferences.notifications_enabled;
    prefs.history_days = p.preferences.history_days;
    prefs.theme = p.preferences.theme.clone();
    c.execute("INSERT INTO management_preferences(account_id,preferences_json) VALUES(?1,?2) ON CONFLICT(account_id) DO UPDATE SET preferences_json=excluded.preferences_json",params![aid,encode(&prefs)?]).map_err(sql)?;
    c.execute("UPDATE app_settings SET interval_seconds=?1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id=1",[prefs.interval_seconds]).map_err(sql)?;
    // Preserve immutable history, but do not compare restored rules against observations from another configuration.
    c.execute(
        "UPDATE integrity_scopes SET active_baseline_id=NULL,previous_snapshot_id=NULL,latest_report_id=NULL,observation_revision=observation_revision+1 WHERE account_id=?1",
        [&aid],
    )
    .map_err(sql)?;
    c.execute("DELETE FROM integrity_observation_components WHERE scope_id IN (SELECT id FROM integrity_scopes WHERE account_id=?1)",[&aid]).map_err(sql)?;
    for b in &p.baselines {
        let s = &b.snapshot;
        let scope_id = format!(
            "scope-{}",
            fingerprint(&(&aid, &s.spreadsheet_id, &s.scope_hash))
        );
        c.execute("INSERT INTO integrity_scopes(id,account_id,spreadsheet_id,scope_hash,scope_json,created_at) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(account_id,spreadsheet_id,scope_hash) DO NOTHING",params![scope_id,aid,s.spreadsheet_id,s.scope_hash,encode(&s.scope)?,s.captured_at]).map_err(sql)?;
        let actual_scope:String=c.query_row("SELECT id FROM integrity_scopes WHERE account_id=?1 AND spreadsheet_id=?2 AND scope_hash=?3",params![aid,s.spreadsheet_id,s.scope_hash],|r|r.get(0)).map_err(sql)?;
        let existing: Option<(String, String, String)> = c
            .query_row(
                "SELECT account_id,scope_id,fingerprint FROM structural_snapshots WHERE id=?1",
                [&s.id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()
            .map_err(sql)?;
        if let Some((a, scope, hash)) = existing {
            if a != aid || scope != actual_scope || hash != fingerprint(s) {
                return Err(error("cloud_immutable_collision"));
            }
        } else {
            c.execute("INSERT INTO structural_snapshots(id,scope_id,account_id,captured_at,fingerprint,snapshot_json,complete) VALUES(?1,?2,?3,?4,?5,?6,1)",params![s.id,actual_scope,aid,s.captured_at,fingerprint(s),encode(s)?]).map_err(sql)?;
        }
        let existing:Option<(String,String,String,u32,String,String)>=c.query_row("SELECT account_id,scope_id,snapshot_id,COALESCE(sync_version,version),approved_at,approval_origin FROM baseline_versions WHERE id=?1",[&b.id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional().map_err(sql)?;
        if let Some(old) = existing {
            if old
                != (
                    aid.clone(),
                    actual_scope.clone(),
                    s.id.clone(),
                    b.version,
                    b.approved_at.clone(),
                    b.approval_origin.clone(),
                )
            {
                return Err(error("cloud_immutable_collision"));
            }
        } else {
            let ordinal: u64 = c.query_row("SELECT MAX(COALESCE(MAX(version),0)+1,?2) FROM baseline_versions WHERE scope_id=?1",params![actual_scope,b.version],|r|r.get(0)).map_err(sql)?;
            c.execute("INSERT INTO baseline_versions(id,scope_id,account_id,snapshot_id,version,approved_at,approval_origin,sync_version) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![b.id,actual_scope,aid,s.id,ordinal,b.approved_at,b.approval_origin,b.version]).map_err(sql)?;
        }
        if b.active {
            c.execute(
                "UPDATE integrity_scopes SET active_baseline_id=?1 WHERE id=?2 AND account_id=?3",
                params![b.id, actual_scope, aid],
            )
            .map_err(sql)?;
        }
    }
    // Cancel leases captured under previous rules; scheduler derives fresh revisions on its next pass.
    c.execute(
        "DELETE FROM monitoring_schedules WHERE account_id=?1",
        [&aid],
    )
    .map_err(sql)?;
    Ok(())
}
impl Database {
    pub fn cloud_capture(&mut self, subject: &str) -> SyncResult<CloudCapture> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let payload = payload(&tx, subject)?;
        tx.execute(
            "INSERT OR IGNORE INTO cloud_sync_device(id,device_id) VALUES(1,?1)",
            [uuid::Uuid::new_v4().to_string()],
        )
        .map_err(sql)?;
        let device_id = tx
            .query_row(
                "SELECT device_id FROM cloud_sync_device WHERE id=1",
                [],
                |r| r.get(0),
            )
            .map_err(sql)?;
        let state:Option<(Option<String>,Option<String>,Option<String>)>=tx.query_row("SELECT base_commit,base_payload_hash,last_synced_at FROM cloud_sync_state WHERE account_subject=?1",[subject],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(sql)?;
        let (base_commit, base_payload_hash, last_synced_at) = state.unwrap_or_default();
        tx.commit().map_err(sql)?;
        Ok(CloudCapture {
            payload,
            base_commit,
            base_payload_hash,
            last_synced_at,
            device_id,
        })
    }
    pub fn cloud_preferences_default(&self, subject: &str) -> SyncResult<bool> {
        let p = preferences(&self.connection, &account(&self.connection, subject)?)?;
        let d = Preferences::default();
        Ok(p.interval_seconds == d.interval_seconds
            && p.notifications_enabled == d.notifications_enabled
            && p.history_days == d.history_days
            && p.theme == d.theme)
    }
    pub fn cloud_cached(&self, subject: &str, id: &str) -> SyncResult<Option<Commit>> {
        let json:Option<String>=self.connection.query_row("SELECT commit_json FROM cloud_sync_commits WHERE account_subject=?1 AND commit_id=?2",params![subject,id],|r|r.get(0)).optional().map_err(sql)?;
        json.map(|s| decode(&s)).transpose()
    }
    pub fn cloud_cache(&self, subject: &str, id: &str, commit: &Commit) -> SyncResult<()> {
        if commit.account_subject != subject || commit.id()? != id {
            return Err(error("cloud_hash_mismatch"));
        }
        self.connection.execute("INSERT OR IGNORE INTO cloud_sync_commits(account_subject,commit_id,commit_json) VALUES(?1,?2,?3)",params![subject,id,encode(commit)?]).map_err(sql)?;
        Ok(())
    }
    pub fn cloud_finish(
        &mut self,
        subject: &str,
        expected_hash: &str,
        id: &str,
        incoming: Option<&Configuration>,
    ) -> SyncResult<SyncStatus> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        let current = payload(&tx, subject)?;
        if canonical_hash(&current)? != expected_hash {
            return Err(error("cloud_local_changed"));
        }
        if let Some(incoming) = incoming {
            let json = encode(&current)?;
            tx.execute("INSERT INTO cloud_sync_backups(id,account_subject,created_at,payload_hash,payload_json) VALUES(?1,?2,?3,?4,?5)",params![uuid::Uuid::new_v4().to_string(),subject,chrono::Utc::now().to_rfc3339(),expected_hash,json]).map_err(sql)?;
            let saved: String = tx
                .query_row(
                    "SELECT payload_json FROM cloud_sync_backups WHERE rowid=last_insert_rowid()",
                    [],
                    |r| r.get(0),
                )
                .map_err(sql)?;
            if canonical_hash(&decode::<Configuration>(&saved)?)? != expected_hash {
                return Err(error("cloud_backup_invalid"));
            }
            restore(&tx, subject, incoming)?;
        }
        let hash = canonical_hash(&payload(&tx, subject)?)?;
        let now = chrono::Utc::now().to_rfc3339();
        tx.execute("INSERT INTO cloud_sync_state(account_subject,base_commit,base_payload_hash,last_synced_at) VALUES(?1,?2,?3,?4) ON CONFLICT(account_subject) DO UPDATE SET base_commit=excluded.base_commit,base_payload_hash=excluded.base_payload_hash,last_synced_at=excluded.last_synced_at",params![subject,id,hash,now]).map_err(sql)?;
        tx.commit().map_err(sql)?;
        Ok(SyncStatus {
            state: "synced".into(),
            message: "Configuração sincronizada.".into(),
            conflicts: vec![],
            last_synced_at: Some(now),
            conflict_token: None,
        })
    }
    /// Explicit local recovery, guarded by the same payload CAS as a remote restore.
    pub fn cloud_restore_backup(
        &mut self,
        subject: &str,
        backup_id: &str,
        expected_hash: &str,
    ) -> SyncResult<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql)?;
        if canonical_hash(&payload(&tx, subject)?)? != expected_hash {
            return Err(error("cloud_local_changed"));
        }
        let (json,hash):(String,String)=tx.query_row("SELECT payload_json,payload_hash FROM cloud_sync_backups WHERE id=?1 AND account_subject=?2",params![backup_id,subject],|r|Ok((r.get(0)?,r.get(1)?))).map_err(sql)?;
        let old: Configuration = decode(&json)?;
        if canonical_hash(&old)? != hash {
            return Err(error("cloud_backup_invalid"));
        }
        let current = payload(&tx, subject)?;
        tx.execute("INSERT INTO cloud_sync_backups(id,account_subject,created_at,payload_hash,payload_json) VALUES(?1,?2,?3,?4,?5)",params![uuid::Uuid::new_v4().to_string(),subject,chrono::Utc::now().to_rfc3339(),expected_hash,encode(&current)?]).map_err(sql)?;
        restore(&tx, subject, &old)?;
        tx.execute(
            "DELETE FROM cloud_sync_state WHERE account_subject=?1",
            [subject],
        )
        .map_err(sql)?;
        tx.commit().map_err(sql)
    }
    pub fn cloud_save_status(&self, subject: &str, status: &SyncStatus) -> SyncResult<()> {
        self.connection.execute("INSERT INTO cloud_sync_status(account_subject,status_json) VALUES(?1,?2) ON CONFLICT(account_subject) DO UPDATE SET status_json=excluded.status_json",params![subject,encode(status)?]).map_err(sql)?;
        Ok(())
    }
    pub fn cloud_status(&self, subject: &str) -> SyncResult<SyncStatus> {
        let json: Option<String> = self
            .connection
            .query_row(
                "SELECT status_json FROM cloud_sync_status WHERE account_subject=?1",
                [subject],
                |r| r.get(0),
            )
            .optional()
            .map_err(sql)?;
        if let Some(json) = json {
            return decode(&json);
        }
        let last: Option<String> = self
            .connection
            .query_row(
                "SELECT last_synced_at FROM cloud_sync_state WHERE account_subject=?1",
                [subject],
                |r| r.get(0),
            )
            .optional()
            .map_err(sql)?
            .flatten();
        Ok(SyncStatus {
            state: "pending".into(),
            message: "Aguardando sincronização.".into(),
            conflicts: vec![],
            last_synced_at: last,
            conflict_token: None,
        })
    }
}
#[cfg(test)]
#[path = "../cloud_sync_tests.rs"]
mod tests;
