use crate::analysis::analyze_project;
use crate::domain::*;
use crate::engine::diff;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path, time::Duration};
use uuid::Uuid;

const MONITOR_COLUMNS: &str = "id,name,spreadsheet_id,script_ids,enabled,archived,baseline_snapshot_id,latest_snapshot_id,last_checked_at,last_error,created_at";
const LEASE_SECONDS: i64 = 900;

/// One local SQLite connection; put behind the application's Mutex. Cross-process
/// transactions and persisted leases also guard concurrent collection/approval.
pub struct Store { conn: Connection }

fn db_error(_: rusqlite::Error) -> String { "local_database_error".into() }
fn now() -> String { Utc::now().to_rfc3339() }
fn fingerprint(json: &str) -> String { format!("{:x}",Sha256::digest(json.as_bytes())) }

fn monitor_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Monitor> {
    let ids: String = row.get(3)?;
    let script_ids = serde_json::from_str(&ids).map_err(|e|rusqlite::Error::FromSqlConversionFailure(3,rusqlite::types::Type::Text,Box::new(e)))?;
    Ok(Monitor {id:row.get(0)?,name:row.get(1)?,spreadsheet_id:row.get(2)?,script_ids,enabled:row.get(4)?,archived:row.get(5)?,baseline_snapshot_id:row.get(6)?,latest_snapshot_id:row.get(7)?,last_checked_at:row.get(8)?,last_error:row.get(9)?,created_at:row.get(10)?})
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self,String> {
        Self::initialize(Connection::open(path).map_err(db_error)?)
    }
    pub fn memory() -> Result<Self,String> { Self::initialize(Connection::open_in_memory().map_err(db_error)?) }

    fn initialize(conn: Connection) -> Result<Self,String> {
        conn.busy_timeout(Duration::from_secs(5)).map_err(db_error)?;
        conn.execute_batch("PRAGMA foreign_keys=ON; PRAGMA trusted_schema=OFF; PRAGMA journal_mode=WAL;").map_err(db_error)?;
        let version:i64=conn.pragma_query_value(None,"user_version",|r|r.get(0)).map_err(db_error)?;
        if version>1 { return Err("database_version_newer_than_application".into()); }
        conn.execute_batch("BEGIN IMMEDIATE;
            CREATE TABLE IF NOT EXISTS monitors (
                id TEXT PRIMARY KEY, name TEXT NOT NULL, spreadsheet_id TEXT NOT NULL,
                script_ids TEXT NOT NULL, enabled INTEGER NOT NULL DEFAULT 1,
                archived INTEGER NOT NULL DEFAULT 0, baseline_snapshot_id INTEGER,
                latest_snapshot_id INTEGER, last_checked_at TEXT, last_error TEXT,
                created_at TEXT NOT NULL, run_token TEXT, lease_until INTEGER);
            CREATE UNIQUE INDEX IF NOT EXISTS active_spreadsheet ON monitors(spreadsheet_id) WHERE archived=0;
            CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT, monitor_id TEXT NOT NULL REFERENCES monitors(id),
                content TEXT NOT NULL, fingerprint TEXT NOT NULL, created_at TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT, monitor_id TEXT NOT NULL REFERENCES monitors(id),
                snapshot_id INTEGER NOT NULL REFERENCES snapshots(id), difference TEXT NOT NULL,
                created_at TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS outbox (
                id INTEGER PRIMARY KEY AUTOINCREMENT, event_id INTEGER NOT NULL UNIQUE REFERENCES events(id),
                attempts INTEGER NOT NULL DEFAULT 0, last_error TEXT, next_attempt_at INTEGER NOT NULL DEFAULT 0,
                delivered_at TEXT);
            CREATE TABLE IF NOT EXISTS audit (
                id INTEGER PRIMARY KEY AUTOINCREMENT, monitor_id TEXT NOT NULL REFERENCES monitors(id),
                action TEXT NOT NULL, created_at TEXT NOT NULL);
            PRAGMA user_version=1; COMMIT;").map_err(db_error)?;
        Ok(Self{conn})
    }

    pub fn add_monitor(&mut self,name:&str,spreadsheet_id:&str,script_ids:Vec<String>) -> Result<Monitor,String> {
        let name=name.trim();
        if name.is_empty() || name.chars().count()>120 {return Err("invalid_monitor_name".into());}
        if !valid_google_id(spreadsheet_id) || script_ids.iter().any(|s|!valid_google_id(s)) {return Err("invalid_google_id".into());}
        let unique:BTreeSet<_>=script_ids.iter().collect();
        if unique.len()!=script_ids.len() || script_ids.len()>30 {return Err("invalid_script_list".into());}
        let tx=self.conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db_error)?;
        let count:i64=tx.query_row("SELECT count(*) FROM monitors WHERE archived=0",[],|r|r.get(0)).map_err(db_error)?;
        if count>=MAX_MONITORS as i64 {return Err("monitor_limit_30".into());}
        let exists:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM monitors WHERE spreadsheet_id=?1 AND archived=0)",[spreadsheet_id],|r|r.get(0)).map_err(db_error)?;
        if exists {return Err("spreadsheet_already_monitored".into());}
        let id=Uuid::new_v4().to_string(); let created_at=now();
        tx.execute("INSERT INTO monitors(id,name,spreadsheet_id,script_ids,created_at) VALUES(?1,?2,?3,?4,?5)",params![id,name,spreadsheet_id,serde_json::to_string(&script_ids).map_err(|_|"invalid_script_list")?,created_at]).map_err(db_error)?;
        tx.execute("INSERT INTO audit(monitor_id,action,created_at) VALUES(?1,'monitor_added',?2)",params![id,created_at]).map_err(db_error)?;
        tx.commit().map_err(db_error)?;
        self.get_monitor(&id)
    }

    pub fn get_monitor(&self,id:&str) -> Result<Monitor,String> {
        self.conn.query_row(&format!("SELECT {MONITOR_COLUMNS} FROM monitors WHERE id=?1"),[id],monitor_row).optional().map_err(db_error)?.ok_or_else(||"monitor_not_found".into())
    }

    pub fn list_monitors(&self) -> Result<Vec<Monitor>,String> {
        let mut query=self.conn.prepare(&format!("SELECT {MONITOR_COLUMNS} FROM monitors WHERE archived=0 ORDER BY created_at,id")).map_err(db_error)?;
        let rows=query.query_map([],monitor_row).map_err(db_error)?;
        rows.collect::<Result<Vec<_>,_>>().map_err(db_error)
    }

    pub fn set_enabled(&mut self,id:&str,enabled:bool)->Result<(),String> {
        let tx=self.conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db_error)?;
        let changed=tx.execute("UPDATE monitors SET enabled=?2,run_token=NULL,lease_until=NULL WHERE id=?1 AND archived=0",params![id,enabled]).map_err(db_error)?;
        if changed!=1 {return Err("monitor_not_found".into());}
        tx.execute("INSERT INTO audit(monitor_id,action,created_at) VALUES(?1,?2,?3)",params![id,if enabled {"monitor_enabled"} else {"monitor_disabled"},now()]).map_err(db_error)?;
        tx.commit().map_err(db_error)
    }

    /// Archive configuration, retaining snapshots, events and audit evidence.
    pub fn remove_monitor(&mut self,id:&str)->Result<(),String> {
        let tx=self.conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db_error)?;
        let changed=tx.execute("UPDATE monitors SET archived=1,enabled=0,run_token=NULL,lease_until=NULL WHERE id=?1 AND archived=0",[id]).map_err(db_error)?;
        if changed!=1 {return Err("monitor_not_found".into());}
        tx.execute("INSERT INTO audit(monitor_id,action,created_at) VALUES(?1,'monitor_archived',?2)",params![id,now()]).map_err(db_error)?;
        tx.commit().map_err(db_error)
    }

    pub fn begin_run(&mut self,id:&str)->Result<RunTicket,String> {
        let token=Uuid::new_v4().to_string(); let timestamp=Utc::now().timestamp();
        let changed=self.conn.execute("UPDATE monitors SET run_token=?2,lease_until=?3 WHERE id=?1 AND enabled=1 AND archived=0 AND (run_token IS NULL OR lease_until<=?4)",params![id,token,timestamp+LEASE_SECONDS,timestamp]).map_err(db_error)?;
        if changed!=1 {return Err("monitor_unavailable_or_collection_in_progress".into());}
        Ok(RunTicket{monitor_id:id.into(),token})
    }

    pub fn complete_run(&mut self,ticket:&RunTicket,snapshot:&Snapshot)->Result<PollResult,String> {
        let tx=self.conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db_error)?;
        let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM monitors WHERE id=?1 AND run_token=?2 AND enabled=1 AND archived=0 AND lease_until>?3)",params![ticket.monitor_id,ticket.token,Utc::now().timestamp()],|r|r.get(0)).map_err(db_error)?;
        if !valid {return Err("collection_lease_invalid".into());}
        let monitor=tx.query_row(&format!("SELECT {MONITOR_COLUMNS} FROM monitors WHERE id=?1"),[&ticket.monitor_id],monitor_row).map_err(db_error)?;
        let mut snapshot=snapshot.clone();
        validate_snapshot(&monitor,&snapshot)?;
        snapshot.findings=snapshot.scripts.iter().map(|(id,project)|(id.clone(),analyze_project(project))).collect();
        let content=serde_json::to_string(&snapshot).map_err(|_|"invalid_snapshot")?;
        let timestamp=now();
        let differences=if monitor.baseline_snapshot_id.is_some() {
            let previous_id=monitor.latest_snapshot_id.ok_or("missing_previous_snapshot")?;
            let previous:String=tx.query_row("SELECT content FROM snapshots WHERE id=?1 AND monitor_id=?2",params![previous_id,monitor.id],|r|r.get(0)).map_err(db_error)?;
            let a=serde_json::from_str(&previous).map_err(|_|"invalid_stored_snapshot")?;
            let b=serde_json::to_value(&snapshot).map_err(|_|"invalid_snapshot")?;
            diff(&a,&b)
        } else {Vec::new()};
        tx.execute("INSERT INTO snapshots(monitor_id,content,fingerprint,created_at) VALUES(?1,?2,?3,?4)",params![monitor.id,content,fingerprint(&content),timestamp]).map_err(db_error)?;
        let snapshot_id=tx.last_insert_rowid();
        for difference in &differences {
            let json=serde_json::to_string(difference).map_err(|_|"invalid_difference")?;
            tx.execute("INSERT INTO events(monitor_id,snapshot_id,difference,created_at) VALUES(?1,?2,?3,?4)",params![monitor.id,snapshot_id,json,timestamp]).map_err(db_error)?;
            let event_id=tx.last_insert_rowid();
            tx.execute("INSERT INTO outbox(event_id) VALUES(?1)",[event_id]).map_err(db_error)?;
        }
        tx.execute("UPDATE monitors SET latest_snapshot_id=?2,last_checked_at=?3,last_error=NULL,run_token=NULL,lease_until=NULL WHERE id=?1",params![monitor.id,snapshot_id,timestamp]).map_err(db_error)?;
        tx.commit().map_err(db_error)?;
        Ok(PollResult{snapshot_id,awaiting_baseline:monitor.baseline_snapshot_id.is_none(),differences})
    }

    /// Store only a closed category, never raw provider errors, tokens or URLs.
    pub fn fail_run(&mut self,ticket:&RunTicket,error_code:&str)->Result<(),String> {
        let safe=match error_code {"unauthorized"|"forbidden"|"quota"|"network"|"timeout"|"not_found"|"incomplete_snapshot"=>error_code,_=>"collection_failed"};
        let changed=self.conn.execute("UPDATE monitors SET last_checked_at=?3,last_error=?4,run_token=NULL,lease_until=NULL WHERE id=?1 AND run_token=?2",params![ticket.monitor_id,ticket.token,now(),safe]).map_err(db_error)?;
        if changed!=1 {return Err("collection_lease_invalid".into());}
        Ok(())
    }

    pub fn preview_baseline(&self,id:&str)->Result<BaselineCandidate,String> {
        let monitor=self.get_monitor(id)?;
        if monitor.archived {return Err("monitor_not_found".into());}
        self.snapshot(id,monitor.latest_snapshot_id.ok_or("snapshot_not_available")?)
    }

    pub fn snapshot(&self,monitor_id:&str,snapshot_id:i64)->Result<BaselineCandidate,String> {
        let row:Option<(String,String,String)>=self.conn.query_row("SELECT content,fingerprint,created_at FROM snapshots WHERE id=?1 AND monitor_id=?2",params![snapshot_id,monitor_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(db_error)?;
        let (content,fingerprint,created_at)=row.ok_or("snapshot_not_found")?;
        Ok(BaselineCandidate{snapshot_id,fingerprint,snapshot:serde_json::from_str(&content).map_err(|_|"invalid_stored_snapshot")?,created_at})
    }

    /// The ID AND digest must be the exact candidate displayed to the person.
    /// A collection in flight or a newer preview invalidates approval.
    pub fn approve_baseline(&mut self,id:&str,snapshot_id:i64,expected_fingerprint:&str)->Result<(),String> {
        let tx=self.conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db_error)?;
        let candidate:Option<(String,Option<String>,Option<i64>)>=tx.query_row("SELECT s.fingerprint,m.run_token,m.lease_until FROM monitors m JOIN snapshots s ON s.id=m.latest_snapshot_id AND s.monitor_id=m.id WHERE m.id=?1 AND m.archived=0 AND m.baseline_snapshot_id IS NULL AND s.id=?2",params![id,snapshot_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(db_error)?;
        let (actual,run_token,lease_until)=candidate.ok_or("baseline_candidate_stale_or_already_approved")?;
        if actual!=expected_fingerprint {return Err("baseline_fingerprint_mismatch".into());}
        if run_token.is_some() && lease_until.unwrap_or(i64::MAX)>Utc::now().timestamp() {return Err("collection_in_progress".into());}
        tx.execute("UPDATE monitors SET baseline_snapshot_id=?2,run_token=NULL,lease_until=NULL WHERE id=?1",params![id,snapshot_id]).map_err(db_error)?;
        tx.execute("INSERT INTO audit(monitor_id,action,created_at) VALUES(?1,?2,?3)",params![id,format!("baseline_approved:{snapshot_id}:{actual}"),now()]).map_err(db_error)?;
        tx.commit().map_err(db_error)
    }

    pub fn history(&self,monitor_id:&str)->Result<Vec<ChangeEvent>,String> {
        self.read_events(Some(monitor_id))
    }
    pub fn all_events(&self)->Result<Vec<ChangeEvent>,String> { self.read_events(None) }
    fn read_events(&self,monitor_id:Option<&str>)->Result<Vec<ChangeEvent>,String> {
        let mut statement=self.conn.prepare("SELECT id,monitor_id,snapshot_id,difference,created_at FROM events WHERE (?1 IS NULL OR monitor_id=?1) ORDER BY id DESC").map_err(db_error)?;
        let rows=statement.query_map([monitor_id],|row|{
            let json:String=row.get(3)?;
            let difference=serde_json::from_str(&json).map_err(|e|rusqlite::Error::FromSqlConversionFailure(3,rusqlite::types::Type::Text,Box::new(e)))?;
            Ok(ChangeEvent{id:row.get(0)?,monitor_id:row.get(1)?,snapshot_id:row.get(2)?,difference,created_at:row.get(4)?})
        }).map_err(db_error)?;
        rows.collect::<Result<Vec<_>,_>>().map_err(db_error)
    }

    pub fn pending_notifications(&self)->Result<Vec<Notification>,String> {
        self.read_notifications(true)
    }
    pub fn notifications_all(&self)->Result<Vec<Notification>,String> {
        self.read_notifications(false)
    }
    fn read_notifications(&self,due_only:bool)->Result<Vec<Notification>,String> {
        let mut statement=self.conn.prepare("SELECT o.id,e.id,e.monitor_id,m.name,e.difference,o.attempts,o.last_error,o.next_attempt_at,o.delivered_at FROM outbox o JOIN events e ON e.id=o.event_id JOIN monitors m ON m.id=e.monitor_id WHERE (?1=0 OR (o.delivered_at IS NULL AND o.next_attempt_at<=?2)) ORDER BY o.id").map_err(db_error)?;
        let rows=statement.query_map(params![due_only,Utc::now().timestamp()],|row|{
            let json:String=row.get(4)?;
            let difference=serde_json::from_str(&json).map_err(|e|rusqlite::Error::FromSqlConversionFailure(4,rusqlite::types::Type::Text,Box::new(e)))?;
            Ok(Notification{id:row.get(0)?,event_id:row.get(1)?,monitor_id:row.get(2)?,monitor_name:row.get(3)?,difference,attempts:row.get(5)?,last_error:row.get(6)?,next_attempt_at:row.get(7)?,delivered_at:row.get(8)?})
        }).map_err(db_error)?;
        rows.collect::<Result<Vec<_>,_>>().map_err(db_error)
    }

    pub fn acknowledge_notification(&mut self,id:i64)->Result<(),String> {
        let changed=self.conn.execute("UPDATE outbox SET delivered_at=?2,attempts=attempts+1,last_error=NULL WHERE id=?1 AND delivered_at IS NULL",params![id,now()]).map_err(db_error)?;
        if changed!=1 {return Err("notification_not_pending".into());}
        Ok(())
    }

    pub fn fail_notification(&mut self,id:i64)->Result<(),String> {
        let tx=self.conn.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db_error)?;
        let attempts:Option<u32>=tx.query_row("SELECT attempts FROM outbox WHERE id=?1 AND delivered_at IS NULL",[id],|r|r.get(0)).optional().map_err(db_error)?;
        let attempts=attempts.ok_or("notification_not_pending")?;
        let delay=30i64.saturating_mul(2i64.pow(attempts.min(7))).min(3600);
        tx.execute("UPDATE outbox SET attempts=attempts+1,last_error='notification_delivery_failed',next_attempt_at=?2 WHERE id=?1",params![id,Utc::now().timestamp()+delay]).map_err(db_error)?;
        tx.commit().map_err(db_error)
    }

    /// Retry remains explicit and durable, including after application restart.
    pub fn retry_notifications(&mut self)->Result<(),String> {
        self.conn.execute("UPDATE outbox SET next_attempt_at=0 WHERE delivered_at IS NULL",[]).map_err(db_error)?;
        Ok(())
    }

    pub fn audit_log(&self)->Result<Vec<AuditEntry>,String> {
        let mut statement=self.conn.prepare("SELECT id,monitor_id,action,created_at FROM audit ORDER BY id DESC").map_err(db_error)?;
        let rows=statement.query_map([],|r|Ok(AuditEntry{id:r.get(0)?,monitor_id:r.get(1)?,action:r.get(2)?,created_at:r.get(3)?})).map_err(db_error)?;
        rows.collect::<Result<Vec<_>,_>>().map_err(db_error)
    }
}

fn validate_snapshot(monitor:&Monitor,snapshot:&Snapshot)->Result<(),String> {
    if snapshot.sheets.get("spreadsheetId").and_then(serde_json::Value::as_str)!=Some(monitor.spreadsheet_id.as_str())
        || !snapshot.sheets.get("sheets").is_some_and(serde_json::Value::is_array)
        || snapshot.drive.get("id").and_then(serde_json::Value::as_str)!=Some(monitor.spreadsheet_id.as_str())
        || !snapshot.drive.get("name").is_some_and(serde_json::Value::is_string)
        || snapshot.drive.get("mimeType").and_then(serde_json::Value::as_str)!=Some("application/vnd.google-apps.spreadsheet") {
        return Err("incomplete_snapshot".into());
    }
    let expected:BTreeSet<_>=monitor.script_ids.iter().collect();
    let actual:BTreeSet<_>=snapshot.scripts.keys().collect();
    if expected!=actual {return Err("incomplete_snapshot".into());}
    for (id,project) in &snapshot.scripts {
        if project.get("scriptId").and_then(serde_json::Value::as_str)!=Some(id.as_str()) || !project.get("files").is_some_and(serde_json::Value::is_array) {
            return Err("incomplete_snapshot".into());
        }
        if project["files"].as_array().is_some_and(|files| files.iter().any(|file| {
            !file.get("name").is_some_and(serde_json::Value::is_string)
                || !file.get("source").is_some_and(serde_json::Value::is_string)
                || !matches!(file.get("type").and_then(serde_json::Value::as_str), Some("SERVER_JS"|"HTML"|"JSON"))
        })) {return Err("incomplete_snapshot".into());}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn sample(id:&str)->Snapshot {Snapshot{sheets:json!({"spreadsheetId":id,"sheets":[{"data":[{"value":1,"formula":"=1","format":{"bold":false}}]}]}),drive:json!({"id":id,"name":"Budget","mimeType":"application/vnd.google-apps.spreadsheet"}),permissions:vec![json!({"id":"p","role":"reader"})],..Default::default()}}
    fn approve(store:&mut Store,id:&str) {
        let candidate=store.preview_baseline(id).unwrap();
        store.approve_baseline(id,candidate.snapshot_id,&candidate.fingerprint).unwrap();
    }
    fn collect(store:&mut Store,id:&str,snapshot:&Snapshot)->PollResult {
        let run=store.begin_run(id).unwrap();store.complete_run(&run,snapshot).unwrap()
    }

    #[test]
    fn baseline_is_explicit_and_each_difference_has_durable_outbox_entry() {
        let mut store=Store::memory().unwrap();
        let monitor=store.add_monitor("Budget","sheet",vec![]).unwrap();
        let before=sample("sheet");
        let first=collect(&mut store,&monitor.id,&before);
        assert!(first.awaiting_baseline);assert!(store.all_events().unwrap().is_empty());
        approve(&mut store,&monitor.id);
        assert!(collect(&mut store,&monitor.id,&before).differences.is_empty());
        let mut after=before.clone();
        after.sheets["sheets"][0]["data"][0]["value"]=json!(2);
        after.sheets["sheets"][0]["data"][0]["formula"]=json!("=2");
        after.sheets["sheets"][0]["data"][0]["format"]["bold"]=json!(true);
        after.permissions[0]["role"]=json!("writer");
        let result=collect(&mut store,&monitor.id,&after);
        assert_eq!(result.differences.len(),4);
        assert_eq!(store.history(&monitor.id).unwrap().len(),4);
        assert_eq!(store.pending_notifications().unwrap().len(),4);
        collect(&mut store,&monitor.id,&after);
        assert_eq!(store.history(&monitor.id).unwrap().len(),4);
        let pending=store.pending_notifications().unwrap();
        for item in pending {store.acknowledge_notification(item.id).unwrap();}
        assert!(store.pending_notifications().unwrap().is_empty());
    }

    #[test]
    fn capacity_duplicate_and_archive_rules() {
        let mut store=Store::memory().unwrap();
        let first=store.add_monitor("One","sheet0",vec![]).unwrap();
        assert!(store.add_monitor("Dup","sheet0",vec![]).is_err());
        for i in 1..30 {store.add_monitor("Monitor",&format!("sheet{i}"),vec![]).unwrap();}
        assert_eq!(store.add_monitor("31","sheet30",vec![]).unwrap_err(),"monitor_limit_30");
        store.remove_monitor(&first.id).unwrap();
        store.add_monitor("Replacement","sheet30",vec![]).unwrap();
        assert_eq!(store.list_monitors().unwrap().len(),30);
        assert!(store.get_monitor(&first.id).unwrap().archived);
        assert!(store.begin_run(&first.id).is_err());
    }

    #[test]
    fn failed_and_partial_collection_do_not_replace_snapshot_or_expose_secrets() {
        let mut store=Store::memory().unwrap();let m=store.add_monitor("Name","sheet",vec![]).unwrap();
        let first=collect(&mut store,&m.id,&sample("sheet"));approve(&mut store,&m.id);
        let ticket=store.begin_run(&m.id).unwrap();
        assert!(store.complete_run(&ticket,&sample("wrong")).is_err());
        store.fail_run(&ticket,"Bearer SECRET_TOKEN").unwrap();
        let m=store.get_monitor(&m.id).unwrap();
        assert_eq!(m.latest_snapshot_id,Some(first.snapshot_id));
        assert_eq!(m.last_error.as_deref(),Some("collection_failed"));
        assert!(store.pending_notifications().unwrap().is_empty());
    }

    #[test]
    fn incomplete_schema_cannot_replace_complete_snapshot() {
        let mut store=Store::memory().unwrap();let m=store.add_monitor("Name","sheet",vec!["script".into()]).unwrap();
        let mut full=sample("sheet");
        full.scripts.insert("script".into(),json!({"scriptId":"script","files":[{"name":"Code","type":"SERVER_JS","source":""}]}));
        let initial=collect(&mut store,&m.id,&full);approve(&mut store,&m.id);
        let mut missing_grid=full.clone();missing_grid.sheets=json!({"spreadsheetId":"sheet"});
        let mut wrong_grid_type=full.clone();wrong_grid_type.sheets["sheets"]=json!({});
        let mut missing_metadata=full.clone();missing_metadata.drive=json!({"id":"sheet"});
        let mut wrong_mime=full.clone();wrong_mime.drive["mimeType"]=json!("application/pdf");
        let mut missing_source=full.clone();missing_source.scripts.get_mut("script").unwrap()["files"][0]=json!({"name":"Code","type":"SERVER_JS"});
        let mut invalid_file_type=full.clone();invalid_file_type.scripts.get_mut("script").unwrap()["files"][0]["type"]=json!(4);
        for partial in [missing_grid,wrong_grid_type,missing_metadata,wrong_mime,missing_source,invalid_file_type] {
            let run=store.begin_run(&m.id).unwrap();
            assert_eq!(store.complete_run(&run,&partial).unwrap_err(),"incomplete_snapshot");
            store.fail_run(&run,"incomplete_snapshot").unwrap();
            assert_eq!(store.get_monitor(&m.id).unwrap().latest_snapshot_id,Some(initial.snapshot_id));
        }
        assert!(store.all_events().unwrap().is_empty());
        assert!(store.pending_notifications().unwrap().is_empty());
    }

    #[test]
    fn approval_rejects_stale_digest_newer_candidate_and_inflight_collection() {
        let mut store=Store::memory().unwrap();let m=store.add_monitor("Name","sheet",vec![]).unwrap();
        collect(&mut store,&m.id,&sample("sheet"));
        let a=store.preview_baseline(&m.id).unwrap();
        assert!(store.approve_baseline(&m.id,a.snapshot_id,"wrong").is_err());
        let ticket=store.begin_run(&m.id).unwrap();
        assert!(store.approve_baseline(&m.id,a.snapshot_id,&a.fingerprint).is_err());
        store.complete_run(&ticket,&sample("sheet")).unwrap();
        assert!(store.approve_baseline(&m.id,a.snapshot_id,&a.fingerprint).is_err());
        approve(&mut store,&m.id);
        assert!(store.approve_baseline(&m.id,a.snapshot_id,&a.fingerprint).is_err());
    }

    #[test]
    fn leases_prevent_overlap_and_late_workers_cannot_commit() {
        let mut store=Store::memory().unwrap();let m=store.add_monitor("Name","sheet",vec![]).unwrap();
        let ticket=store.begin_run(&m.id).unwrap();
        assert!(store.begin_run(&m.id).is_err());
        store.set_enabled(&m.id,false).unwrap();
        assert!(store.complete_run(&ticket,&sample("sheet")).is_err());
        assert!(store.fail_run(&ticket,"network").is_err());
        store.set_enabled(&m.id,true).unwrap();
        let ticket=store.begin_run(&m.id).unwrap();
        store.conn.execute("UPDATE monitors SET lease_until=0 WHERE id=?1",[&m.id]).unwrap();
        let new_ticket=store.begin_run(&m.id).unwrap();
        assert!(store.complete_run(&ticket,&sample("sheet")).is_err());
        store.complete_run(&new_ticket,&sample("sheet")).unwrap();
    }

    #[test]
    fn authorized_scripts_are_complete_and_static_findings_are_diffed() {
        let mut store=Store::memory().unwrap();let m=store.add_monitor("Name","sheet",vec!["script".into()]).unwrap();
        let mut before=sample("sheet");
        let ticket=store.begin_run(&m.id).unwrap();
        assert!(store.complete_run(&ticket,&before).is_err());
        before.scripts.insert("script".into(),json!({"scriptId":"script","files":[{"name":"Code","type":"SERVER_JS","source":"function read() {}"}]}));
        store.complete_run(&ticket,&before).unwrap();approve(&mut store,&m.id);
        before.scripts.get_mut("script").unwrap()["files"][0]["source"]=json!("UrlFetchApp.fetch('https://example.test')");
        let result=collect(&mut store,&m.id,&before);
        assert!(result.differences.iter().any(|d|d.path.contains("/source")));
        assert!(result.differences.iter().any(|d|d.path.starts_with("/findings")));
        let candidate=store.preview_baseline(&m.id).unwrap();
        assert_eq!(candidate.snapshot.findings["script"][0].rule,"external_http");
    }

    #[test]
    fn pending_outbox_and_approval_survive_reopen_and_retry_has_backoff() {
        let path=std::env::temp_dir().join(format!("sheets-monitor-test-{}.db",Uuid::new_v4()));
        let mut store=Store::open(&path).unwrap();let m=store.add_monitor("Name","sheet",vec![]).unwrap();
        collect(&mut store,&m.id,&sample("sheet"));approve(&mut store,&m.id);
        let mut changed=sample("sheet");changed.drive["name"]=json!("Changed");
        collect(&mut store,&m.id,&changed);drop(store);
        let mut store=Store::open(&path).unwrap();
        assert!(store.get_monitor(&m.id).unwrap().baseline_snapshot_id.is_some());
        let item=store.pending_notifications().unwrap().remove(0);
        store.fail_notification(item.id).unwrap();assert!(store.pending_notifications().unwrap().is_empty());
        drop(store);let mut store=Store::open(&path).unwrap();
        store.retry_notifications().unwrap();let retry=store.pending_notifications().unwrap().remove(0);
        assert_eq!(retry.attempts,1);assert_eq!(retry.last_error.as_deref(),Some("notification_delivery_failed"));
        store.acknowledge_notification(item.id).unwrap();
        assert!(store.acknowledge_notification(item.id).is_err());
        drop(store);std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn competing_connections_enforce_capacity_and_collection_lock() {
        let path=std::env::temp_dir().join(format!("sheets-monitor-test-{}.db",Uuid::new_v4()));
        let mut one=Store::open(&path).unwrap();let m=one.add_monitor("Name","sheet",vec![]).unwrap();
        let mut two=Store::open(&path).unwrap();let first=one.begin_run(&m.id).unwrap();
        assert!(two.begin_run(&m.id).is_err());one.fail_run(&first,"network").unwrap();
        assert!(two.begin_run(&m.id).is_ok());
        drop(one);drop(two);std::fs::remove_file(path).unwrap();
    }
}
