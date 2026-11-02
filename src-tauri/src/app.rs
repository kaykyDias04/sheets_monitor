use crate::{domain::Snapshot, fixtures, google::GoogleApi, oauth::{self, OAuthConfig}, store::Store};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::{Path, PathBuf}, sync::{Arc, Mutex}, thread, time::{Duration, Instant}};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_notification::NotificationExt;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all="snake_case")]
enum Mode { Fixture, Google }

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AppSettings { mode: Mode, interval_seconds: u64 }
impl Default for AppSettings { fn default()->Self {Self{mode:Mode::Fixture,interval_seconds:300}} }
impl AppSettings {
    fn validate(&self)->Result<(),String> {
        if !(60..=86400).contains(&self.interval_seconds) {return Err("O intervalo deve estar entre 60 e 86400 segundos".into());}
        Ok(())
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct Config {
    #[serde(default)] settings: AppSettings,
    #[serde(default)] client_id: String,
    #[serde(default)] account_partition: Option<String>,
}
impl Config {
    fn oauth(&self)->OAuthConfig {
        OAuthConfig {client_id:self.client_id.clone(),client_secret:std::env::var("SHEETS_MONITOR_CLIENT_SECRET").ok().filter(|s| !s.is_empty())}
    }
    fn database_path(&self,dir:&Path)->Result<PathBuf,String> {
        match self.settings.mode {
            Mode::Fixture=>Ok(dir.join("fixture.db")),
            Mode::Google=> {
                let hash=self.account_partition.as_ref().filter(|h|h.len()==64 && h.bytes().all(|b|b.is_ascii_hexdigit()))
                    .ok_or("Conecte uma conta antes de usar o modo Google")?;
                Ok(dir.join(format!("google-{hash}.db")))
            }
        }
    }
}

struct AppState {
    directory:PathBuf,
    config:Mutex<Config>,
    run_gate:Mutex<()>,
}
type SharedState=Arc<AppState>;

impl AppState {
    fn config(&self)->Result<Config,String> {self.config.lock().map(|c|c.clone()).map_err(|_|"Configuração indisponível".into())}
    fn save_config(&self,config:Config)->Result<(),String> {
        let mut current=self.config.lock().map_err(|_|"Configuração indisponível")?;
        persist_config(&self.directory,&config)?;
        *current=config;
        Ok(())
    }
    fn store(&self,config:&Config)->Result<Store,String> { Store::open(config.database_path(&self.directory)?) }
    fn log(&self,category:&str) {
        // Closed categories only, never provider errors, IDs, source, paths or tokens.
        let category=match category {"collection_ok"|"collection_failed"|"notification_failed"|"scheduler_failed"|"connected"|"disconnected"=>category,_=>"operation_failed"};
        let path=self.directory.join("operations.log");
        if fs::metadata(&path).is_ok_and(|m|m.len()>5*1024*1024) {
            let _=fs::remove_file(self.directory.join("operations.previous.log"));
            let _=fs::rename(&path,self.directory.join("operations.previous.log"));
        }
        if let Ok(mut file)=fs::OpenOptions::new().create(true).append(true).open(path) {
            let _=writeln!(file,"{} {}",Utc::now().to_rfc3339(),category);
        }
    }
}

fn persist_config(directory:&Path,config:&Config)->Result<(),String> {
    let path=directory.join("config.json");let temporary=directory.join("config.pending.json");let backup=directory.join("config.backup.json");
    let bytes=serde_json::to_vec_pretty(config).map_err(|_|"Configuração inválida")?;
    fs::write(&temporary,bytes).map_err(|_|"Falha ao salvar configuração local")?;
    let _=fs::remove_file(&backup);
    let had_previous=path.exists();
    if had_previous {fs::rename(&path,&backup).map_err(|_|"Falha ao preservar configuração anterior")?;}
    if fs::rename(&temporary,&path).is_err() {
        if had_previous {let _=fs::rename(&backup,&path);}
        return Err("Falha ao concluir configuração local".into());
    }
    let _=fs::remove_file(backup);Ok(())
}

fn read_config(directory:&Path)->Result<Config,String> {
    let path=directory.join("config.json");let backup=directory.join("config.backup.json");
    let source=if path.exists(){path}else if backup.exists(){backup}else{return Ok(Config::default())};
    let bytes=fs::read(source).map_err(|_|"Falha ao ler configuração")?;
    let config:Config=serde_json::from_slice(&bytes).map_err(|_|"Configuração local inválida; preserve o arquivo para diagnóstico")?;
    config.settings.validate()?;Ok(config)
}

fn partition(client_id:&str,identity:&str)->String {
    format!("{:x}",Sha256::digest(format!("{client_id}:{identity}").as_bytes()))
}

fn google_api(config:&Config)->Result<GoogleApi,String> {
    let token=oauth::access_token(&config.oauth())?;
    let api=GoogleApi::new(token)?;
    let identity=api.account_identity()?;
    if config.account_partition.as_deref()!=Some(partition(&config.client_id,&identity).as_str()) {
        return Err("A identidade Google mudou; conecte novamente para isolar o histórico da conta".into());
    }
    Ok(api)
}

fn collect_locked(state:&AppState,config:&Config,id:Option<&str>)->Result<(),String> {
    let mut store=state.store(config)?;
    let monitors=match id {Some(id)=>vec![store.get_monitor(id)?],None=>store.list_monitors()?};
    let mut failures=0;
    // One authenticated client/identity check per batch, not one token refresh per sheet.
    let api=match config.settings.mode {Mode::Fixture=>None,Mode::Google=>Some(google_api(config))};
    for monitor in monitors {
        if monitor.archived || !monitor.enabled {continue;}
        let ticket=match store.begin_run(&monitor.id) {Ok(ticket)=>ticket,Err(_)=>{failures+=1;continue;}};
        let snapshot:Result<Snapshot,String>=match &api {
            None=>Ok(fixtures::collect(&monitor,monitor.latest_snapshot_id.unwrap_or(0) as u64 + 1)),
            Some(Ok(api))=>api.fetch_snapshot(&monitor.spreadsheet_id,&monitor.script_ids)
                .and_then(|value|serde_json::from_value(value).map_err(|_|"Snapshot Google inválido".into())),
            Some(Err(error))=>Err(error.clone()),
        };
        match snapshot.and_then(|snapshot|store.complete_run(&ticket,&snapshot)) {
            Ok(_)=>state.log("collection_ok"),
            Err(error)=>{
                failures+=1;
                let code=if error.contains("401") || error.contains("Conecte") || error.contains("reconect") {"unauthorized"}
                    else if error.contains("429") || error.contains("quota") {"quota"}
                    else if error.contains("403") {"forbidden"} else if error.contains("404") {"not_found"}
                    else if error.contains("rede") {"network"} else {"collection_failed"};
                let _=store.fail_run(&ticket,code);state.log("collection_failed");
            }
        }
    }
    if failures>0 {Err(format!("{failures} coleta(s) falharam. Consulte o estado das planilhas; capturas anteriores foram preservadas."))}else{Ok(())}
}

fn flush_notifications_locked(app:&AppHandle,state:&AppState,config:&Config)->Result<(),String> {
    let mut store=state.store(config)?;
    for notice in store.pending_notifications()? {
        let title=if config.settings.mode==Mode::Fixture {"Sheets Monitor · Fixture local"}else{"Sheets Monitor"};
        let body=format!("Diferença #{} detectada. Abra o histórico no aplicativo para revisar.",notice.event_id);
        match app.notification().builder().title(title).body(body).show() {
            Ok(())=>store.acknowledge_notification(notice.id)?,
            Err(_)=>{store.fail_notification(notice.id)?;state.log("notification_failed");}
        }
    }
    Ok(())
}

#[tauri::command]
fn get_dashboard(state:State<'_,SharedState>)->Result<Value,String> {
    let config=state.config()?;let store=state.store(&config)?;
    let monitors=store.list_monitors()?;let events=store.all_events()?;
    let sheets:Vec<_>=monitors.iter().map(|m| {
        let changed=events.iter().any(|e|e.monitor_id==m.id && Some(e.snapshot_id)==m.latest_snapshot_id);
        let status=if m.last_error.is_some(){"error"}else if m.baseline_snapshot_id.is_none(){"pending"}else if changed{"changed"}else{"healthy"};
        json!({"id":m.id,"spreadsheet_id":m.spreadsheet_id,"name":m.name,"script_ids":m.script_ids,"status":status,"baseline_approved":m.baseline_snapshot_id.is_some(),"last_checked":m.last_checked_at,"last_error":m.last_error})
    }).collect();
    let events:Vec<_>=events.iter().map(|e| {
        let name=store.get_monitor(&e.monitor_id).map(|m|m.name).unwrap_or_else(|_|"Planilha arquivada".into());
        json!({"id":e.id.to_string(),"sheet_id":e.monitor_id,"sheet_name":name,"detected_at":e.created_at,"path":e.difference.path,"kind":e.difference.kind,"before":e.difference.before,"after":e.difference.after})
    }).collect();
    let notifications:Vec<_>=store.notifications_all()?.into_iter().map(|n|json!({"id":n.id.to_string(),"event_id":n.event_id.to_string(),"status":if n.delivered_at.is_some(){"sent"}else if n.last_error.is_some(){"failed"}else{"pending"},"attempts":n.attempts})).collect();
    let connected=!config.client_id.is_empty() && oauth::is_connected(&config.oauth()).unwrap_or(false);
    Ok(json!({"sheets":sheets,"events":events,"notifications":notifications,"settings":config.settings,"connected":connected,"client_id":config.client_id}))
}

#[derive(Deserialize)]
struct AddSheet { id:String,name:String,script_ids:Vec<String> }
#[tauri::command]
fn add_sheet(state:State<'_,SharedState>,input:AddSheet)->Result<(),String> {
    let _gate=state.run_gate.try_lock().map_err(|_|"Uma operação está em andamento")?;
    let config=state.config()?;state.store(&config)?.add_monitor(&input.name,&input.id,input.script_ids)?;Ok(())
}
#[tauri::command]
fn remove_sheet(state:State<'_,SharedState>,id:String)->Result<(),String> {
    let _gate=state.run_gate.try_lock().map_err(|_|"Uma operação está em andamento")?;
    let config=state.config()?;state.store(&config)?.remove_monitor(&id)
}
#[tauri::command]
async fn run_check(app:AppHandle,state:State<'_,SharedState>,id:Option<String>)->Result<(),String> {
    let state=state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate=state.run_gate.try_lock().map_err(|_|"Uma coleta está em andamento")?;
        let config=state.config()?;let result=collect_locked(&state,&config,id.as_deref());
        let notification_result=flush_notifications_locked(&app,&state,&config);
        result.and(notification_result)
    }).await.map_err(|_|"A tarefa de coleta foi interrompida")?
}
#[tauri::command]
async fn preview_baseline(state:State<'_,SharedState>,id:String)->Result<Value,String> {
    let state=state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate=state.run_gate.try_lock().map_err(|_|"Uma coleta está em andamento")?;
        let config=state.config()?;
        if state.store(&config)?.get_monitor(&id)?.latest_snapshot_id.is_none() {collect_locked(&state,&config,Some(&id))?;}
        let candidate=state.store(&config)?.preview_baseline(&id)?;
        Ok(json!({"snapshot":candidate.snapshot,"fingerprint":format!("{}:{}",candidate.snapshot_id,candidate.fingerprint)}))
    }).await.map_err(|_|"A prévia foi interrompida")?
}
#[tauri::command]
fn approve_baseline(state:State<'_,SharedState>,id:String,fingerprint:String)->Result<(),String> {
    let _gate=state.run_gate.try_lock().map_err(|_|"Uma coleta está em andamento")?;
    let (snapshot,digest)=fingerprint.split_once(':').ok_or("Prévia inválida; carregue novamente")?;
    let snapshot=snapshot.parse::<i64>().map_err(|_|"Prévia inválida")?;
    let config=state.config()?;state.store(&config)?.approve_baseline(&id,snapshot,digest)
}
#[tauri::command]
fn update_settings(state:State<'_,SharedState>,settings:AppSettings)->Result<(),String> {
    settings.validate()?;
    let _gate=state.run_gate.try_lock().map_err(|_|"Uma operação está em andamento")?;
    let mut config=state.config()?;config.settings=settings;
    let _=config.database_path(&state.directory)?;
    state.save_config(config)
}
#[tauri::command]
async fn connect_google(state:State<'_,SharedState>,client_id:String)->Result<(),String> {
    let state=state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate=state.run_gate.try_lock().map_err(|_|"Uma operação está em andamento")?;
        let mut config=state.config()?;config.client_id=client_id.trim().to_owned();
        let oauth_config=config.oauth();oauth::connect(&oauth_config)?;
        let api=GoogleApi::new(oauth::access_token(&oauth_config)?)?;
        config.account_partition=Some(partition(&config.client_id,&api.account_identity()?));
        state.save_config(config)?;state.log("connected");Ok(())
    }).await.map_err(|_|"A conexão foi interrompida")?
}
#[tauri::command]
async fn disconnect_google(state:State<'_,SharedState>)->Result<(),String> {
    let state=state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate=state.run_gate.try_lock().map_err(|_|"Uma operação está em andamento")?;
        let config=state.config()?;
        if !config.client_id.is_empty(){oauth::disconnect(&config.oauth())?;}
        state.log("disconnected");Ok(())
    }).await.map_err(|_|"A desconexão foi interrompida")?
}
#[tauri::command]
async fn retry_notifications(app:AppHandle,state:State<'_,SharedState>)->Result<(),String> {
    let state=state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _gate=state.run_gate.try_lock().map_err(|_|"Uma operação está em andamento")?;
        let config=state.config()?;state.store(&config)?.retry_notifications()?;
        flush_notifications_locked(&app,&state,&config)
    }).await.map_err(|_|"O reenvio foi interrompido")?
}
#[tauri::command]
fn export_report(state:State<'_,SharedState>)->Result<String,String> {
    let _gate=state.run_gate.try_lock().map_err(|_|"Uma operação está em andamento; tente exportar novamente")?;
    let config=state.config()?;let store=state.store(&config)?;
    let monitors=store.list_monitors()?;
    let snapshots:Vec<_>=monitors.iter().filter_map(|m|m.latest_snapshot_id.map(|id|(m,id)))
        .map(|(m,id)|store.snapshot(&m.id,id).map(|s|json!({"monitor_id":m.id,"candidate":s}))).collect::<Result<_,_>>()?;
    let baselines:Vec<_>=monitors.iter().filter_map(|m|m.baseline_snapshot_id.map(|id|(m,id)))
        .map(|(m,id)|store.snapshot(&m.id,id).map(|s|json!({"monitor_id":m.id,"baseline":s}))).collect::<Result<_,_>>()?;
    let report=json!({"format_version":1,"exported_at":Utc::now().to_rfc3339(),"mode":config.settings.mode,"monitors":monitors,"latest_snapshots":snapshots,"baselines":baselines,"events":store.all_events()?,"notifications":store.notifications_all()?,"audit":store.audit_log()?,"limitations":["Fixtures são sintéticas e não comprovam integração Google real.","Polling compara capturas; edições intermediárias revertidas podem não ser observadas.","A análise estática não executa scripts nem garante ausência de comportamento perigoso.","Notificação enviada significa aceitação pela API do sistema, não confirmação de leitura."]});
    let directory=state.directory.join("reports");fs::create_dir_all(&directory).map_err(|_|"Falha ao criar pasta de relatórios")?;
    let path=directory.join(format!("sheets-monitor-{}-{}.json",Utc::now().format("%Y%m%dT%H%M%SZ"),uuid::Uuid::new_v4()));
    let bytes=serde_json::to_vec_pretty(&report).map_err(|_|"Falha ao gerar relatório")?;
    fs::write(&path,bytes).map_err(|_|"Falha ao salvar relatório")?;
    Ok(path.to_string_lossy().into_owned())
}

fn start_scheduler(app:AppHandle,state:SharedState) {
    thread::spawn(move || {
        let mut last_poll=Instant::now();let mut last_outbox=Instant::now();
        loop {
            thread::sleep(Duration::from_secs(1));
            let Ok(config)=state.config() else {continue};
            let poll_due=last_poll.elapsed()>=Duration::from_secs(config.settings.interval_seconds);
            let notices_due=last_outbox.elapsed()>=Duration::from_secs(30);
            if !poll_due && !notices_due {continue;}
            let Ok(_gate)=state.run_gate.try_lock() else {continue};
            let Ok(config)=state.config() else {continue};
            if poll_due {
                last_poll=Instant::now();
                // Pending baselines require the explicit preview/review workflow.
                if let Ok(store)=state.store(&config) {
                    if let Ok(monitors)=store.list_monitors() {
                        for monitor in monitors.into_iter().filter(|m|m.enabled && m.baseline_snapshot_id.is_some()) {
                            if collect_locked(&state,&config,Some(&monitor.id)).is_err(){state.log("scheduler_failed");}
                        }
                    }
                }
            }
            if notices_due || poll_due {
                last_outbox=Instant::now();
                if flush_notifications_locked(&app,&state,&config).is_err(){state.log("notification_failed");}
            }
        }
    });
}

pub fn run() {
    tauri::Builder::default().plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let directory=app.path().app_data_dir()?;fs::create_dir_all(&directory)?;
            let config=read_config(&directory).map_err(std::io::Error::other)?;
            let state=Arc::new(AppState{directory,config:Mutex::new(config),run_gate:Mutex::new(())});
            app.manage(state.clone());start_scheduler(app.handle().clone(),state);Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_dashboard,add_sheet,remove_sheet,run_check,preview_baseline,approve_baseline,update_settings,connect_google,disconnect_google,retry_notifications,export_report])
        .run(tauri::generate_context!()).expect("Falha ao iniciar Sheets Monitor");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn data_sources_and_google_accounts_have_distinct_databases() {
        let root=Path::new("local-data");let fixture=Config::default();
        let mut a=fixture.clone();a.settings.mode=Mode::Google;a.client_id="clientA".into();a.account_partition=Some(partition("clientA","userA"));
        let mut b=a.clone();b.account_partition=Some(partition("clientA","userB"));
        assert_ne!(fixture.database_path(root).unwrap(),a.database_path(root).unwrap());
        assert_ne!(a.database_path(root).unwrap(),b.database_path(root).unwrap());
        assert_ne!(partition("clientA","userA"),partition("clientB","userA"));
    }
    #[test]
    fn config_roundtrip_has_no_secret_and_interval_is_bounded() {
        let temp=tempfile::tempdir().unwrap();let mut config=Config::default();config.client_id="fixture.apps.googleusercontent.com".into();
        persist_config(temp.path(),&config).unwrap();
        assert_eq!(read_config(temp.path()).unwrap().client_id,config.client_id);
        assert!(!fs::read_to_string(temp.path().join("config.json")).unwrap().contains("secret"));
        for seconds in [0,59,86401,u64::MAX] {assert!(AppSettings{mode:Mode::Fixture,interval_seconds:seconds}.validate().is_err());}
    }
}
