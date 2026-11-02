//! Local IPC for the foundation sprint. No network, polling or Google auth.
use crate::database::{AppSettings, Database};
use serde_json::{json, Value};
use std::{fs, path::PathBuf};
use tauri::{Manager, State};

struct FoundationState {
    database_path: PathBuf,
}

impl FoundationState {
    fn database(&self) -> Result<Database, String> {
        Database::open(&self.database_path)
    }

    fn dashboard(&self) -> Result<Value, String> {
        let database = self.database()?;
        Ok(json!({
            "spreadsheets": database.list_demo_spreadsheets()?,
            "activities": database.demo_history()?,
            "settings": database.settings()?,
            "database_version": database.schema_version()?,
            "is_demo": true
        }))
    }
}

#[tauri::command]
fn foundation_get_dashboard(state: State<'_, FoundationState>) -> Result<Value, String> {
    state.dashboard()
}

#[tauri::command]
fn foundation_save_settings(
    state: State<'_, FoundationState>,
    settings: AppSettings,
) -> Result<(), String> {
    if settings.start_with_windows {
        return Err("Inicialização com Windows será implementada em uma sprint futura".into());
    }
    state.database()?.save_settings(&settings)
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            fs::create_dir_all(&directory)?;
            let state = FoundationState {
                database_path: directory.join("foundation-v1.db"),
            };
            let mut database = state.database().map_err(std::io::Error::other)?;
            database.seed_demo().map_err(std::io::Error::other)?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            foundation_get_dashboard,
            foundation_save_settings
        ])
        .run(tauri::generate_context!())
        .expect("Falha ao iniciar a fundação local do Sheets Monitor");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dashboard_serializes_persisted_examples_with_explicit_demo_marker() {
        let directory = tempfile::tempdir().unwrap();
        let state = FoundationState {
            database_path: directory.path().join("foundation-v1.db"),
        };
        state.database().unwrap().seed_demo().unwrap();
        let dashboard = state.dashboard().unwrap();
        assert_eq!(dashboard["is_demo"], true);
        assert_eq!(dashboard["database_version"], 1);
        let sheets = dashboard["spreadsheets"].as_array().unwrap();
        assert!(!sheets.is_empty());
        assert!(sheets.iter().all(|sheet| sheet["is_demo"] == true));
        assert_eq!(dashboard["settings"]["interval_seconds"], 60);
        assert_eq!(dashboard["settings"]["start_with_windows"], false);
        let forbidden = ["access_token", "refresh_token", "client_secret"];
        let encoded = dashboard.to_string();
        assert!(forbidden.iter().all(|name| !encoded.contains(name)));
    }
}
