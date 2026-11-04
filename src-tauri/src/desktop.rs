//! Native lifecycle and opt-in, current-user Windows startup registration.
use serde::Serialize;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

#[derive(Clone, Copy, Debug)]
pub enum NativeAction {
    Open,
    CheckNow,
    Settings,
    Pause,
    Resume,
    Exit,
}
pub struct DesktopCallbacks {
    pub action: Arc<dyn Fn(NativeAction) + Send + Sync>,
    /// Must cancel and perform bounded service cleanup; called off the UI thread.
    pub shutdown: Arc<dyn Fn() + Send + Sync>,
}
#[derive(Clone, Debug, Serialize)]
pub struct StartupStatus {
    pub registered: bool,
    pub managed: bool,
    pub os_execution: String,
    pub error_code: Option<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct DesktopStatus {
    pub tray_available: bool,
    pub notification_capability: String,
    pub startup: StartupStatus,
    pub exiting: bool,
}
struct Inner {
    app: AppHandle,
    callbacks: DesktopCallbacks,
    tray_available: AtomicBool,
    exiting: AtomicBool,
    exit_ready: AtomicBool,
    state_item: Mutex<Option<MenuItem<tauri::Wry>>>,
    pending_installer: Mutex<Option<std::path::PathBuf>>,
}
#[derive(Clone)]
pub struct DesktopShell(Arc<Inner>);

/// Register this builder before setup so a second process never starts services.
pub fn builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default().plugin(tauri_plugin_single_instance::init(|app, _, _| {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    }))
}
pub fn install(app: &AppHandle, callbacks: DesktopCallbacks) -> DesktopShell {
    let shell = DesktopShell(Arc::new(Inner {
        app: app.clone(),
        callbacks,
        tray_available: AtomicBool::new(false),
        exiting: AtomicBool::new(false),
        exit_ready: AtomicBool::new(false),
        state_item: Mutex::new(None),
        pending_installer: Mutex::new(None),
    }));
    shell
        .0
        .tray_available
        .store(install_tray(&shell).is_ok(), Ordering::SeqCst);
    if let Some(window) = app.get_webview_window("main") {
        let close = shell.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if close.0.exit_ready.load(Ordering::SeqCst) {
                    return;
                }
                api.prevent_close();
                if !close.0.exiting.load(Ordering::SeqCst)
                    && close.0.tray_available.load(Ordering::SeqCst)
                    && close
                        .0
                        .app
                        .get_webview_window("main")
                        .is_some_and(|window| window.hide().is_ok())
                {
                    return;
                }
                // Without a usable tray, keep the window reachable. Explicit
                // Sair remains the only voluntary shutdown action.
                if !close.0.exiting.load(Ordering::SeqCst) {
                    close.show_main();
                }
            }
        });
    }
    shell
}
fn install_tray(shell: &DesktopShell) -> tauri::Result<()> {
    let app = &shell.0.app;
    let open = MenuItem::with_id(app, "open", "Abrir Sheets Monitor", true, None::<&str>)?;
    let state = MenuItem::with_id(
        app,
        "state",
        "Monitoramento: aguardando estado",
        true,
        None::<&str>,
    )?;
    let check = MenuItem::with_id(app, "check", "Verificar agora", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pausar monitoramento", true, None::<&str>)?;
    let resume = MenuItem::with_id(app, "resume", "Retomar monitoramento", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Preferências", true, None::<&str>)?;
    let exit = MenuItem::with_id(app, "exit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[&open, &state, &check, &pause, &resume, &settings, &exit],
    )?;
    let menu_shell = shell.clone();
    let click_shell = shell.clone();
    let mut tray = TrayIconBuilder::with_id("sheets-monitor-tray")
        .tooltip("Sheets Monitor — monitoramento em segundo plano")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |_, event| {
            let action = match event.id.as_ref() {
                "open" | "state" => NativeAction::Open,
                "check" => NativeAction::CheckNow,
                "pause" => NativeAction::Pause,
                "resume" => NativeAction::Resume,
                "settings" => NativeAction::Settings,
                "exit" => {
                    menu_shell.request_exit();
                    return;
                }
                _ => return,
            };
            if matches!(action, NativeAction::Open | NativeAction::Settings) {
                menu_shell.show_main();
            }
            if !menu_shell.0.exiting.load(Ordering::SeqCst) {
                (menu_shell.0.callbacks.action)(action);
            }
        })
        .on_tray_icon_event(move |_, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } | TrayIconEvent::DoubleClick {
                    button: MouseButton::Left,
                    ..
                }
            ) {
                click_shell.show_main();
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    if let Ok(mut item) = shell.0.state_item.lock() {
        *item = Some(state);
    }
    Ok(())
}
impl DesktopShell {
    pub fn status(&self) -> DesktopStatus {
        DesktopStatus {
            tray_available: self.0.tray_available.load(Ordering::SeqCst),
            notification_capability: if cfg!(windows) {
                "desktop_unverified"
            } else {
                "unsupported"
            }
            .into(),
            startup: startup_status(),
            exiting: self.0.exiting.load(Ordering::SeqCst),
        }
    }
    pub fn set_startup(&self, enabled: bool) -> Result<DesktopStatus, String> {
        set_startup(enabled)?;
        let result = self.status();
        use tauri::Emitter;
        let _ = self.0.app.emit("desktop-state-changed", &result);
        Ok(result)
    }
    pub fn show_main(&self) {
        if let Some(window) = self.0.app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    }
    pub fn set_monitoring_state(&self, state: &str) {
        let label = match state {
            "running" => "Em execução",
            "paused" => "Pausado",
            "fault" => "Falha operacional",
            _ => "Indeterminado",
        };
        if let Ok(item) = self.0.state_item.lock() {
            if let Some(item) = item.as_ref() {
                let _ = item.set_text(format!("Monitoramento: {label}"));
            }
        }
        if let Some(tray) = self.0.app.tray_by_id("sheets-monitor-tray") {
            let _ = tray.set_tooltip(Some(format!("Sheets Monitor — {label}")));
        }
    }
    pub fn hide_for_startup(&self) {
        if self.0.tray_available.load(Ordering::SeqCst) {
            if let Some(window) = self.0.app.get_webview_window("main") {
                let _ = window.hide();
            }
        }
    }
    /// Invoke from App::run ExitRequested. Exit stays blocked until cleanup completes.
    pub fn handle_exit_requested(&self, api: &tauri::ExitRequestApi) {
        if !self.0.exit_ready.load(Ordering::SeqCst) {
            api.prevent_exit();
            self.request_exit();
        }
    }
    pub fn request_exit_for_update(
        &self,
        installer_path: std::path::PathBuf,
    ) -> Result<(), String> {
        if !installer_path.is_file() {
            return Err("staged_installer_missing".into());
        }
        if let Ok(mut guard) = self.0.pending_installer.lock() {
            *guard = Some(installer_path);
        }
        self.request_exit();
        Ok(())
    }
    pub fn request_exit(&self) {
        if self.0.exiting.swap(true, Ordering::SeqCst) {
            return;
        }
        let shell = self.clone();
        let fallback = self.clone();
        let started = std::thread::Builder::new()
            .name("desktop-shutdown".into())
            .spawn(move || {
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    (shell.0.callbacks.shutdown)()
                }));
                shell.0.app.remove_tray_by_id("sheets-monitor-tray");
                shell.0.tray_available.store(false, Ordering::SeqCst);
                if let Ok(mut pending) = shell.0.pending_installer.lock() {
                    if let Some(installer) = pending.take() {
                        let _ = std::process::Command::new(&installer)
                            .args(["/P", "/UPDATE", "/R"])
                            .spawn();
                    }
                }
                shell.0.exit_ready.store(true, Ordering::SeqCst);
                shell.0.app.exit(0);
            });
        if started.is_err() {
            // Keep the visible application usable rather than exit without cleanup.
            fallback.0.exiting.store(false, Ordering::SeqCst);
            fallback.show_main();
        }
    }
}

trait StartupStore {
    fn read(&self) -> Result<Option<String>, String>;
    fn write(&self, command: &str) -> Result<(), String>;
    fn remove(&self) -> Result<(), String>;
}
fn startup_command(executable: &Path) -> Result<String, String> {
    let path = executable.to_str().ok_or("startup_path_unsupported")?;
    if !executable.is_absolute() || path.chars().any(|c| c == '"' || c.is_control()) {
        return Err("startup_path_unsupported".into());
    }
    let command = format!("\"{path}\" --from-autostart");
    if command.encode_utf16().count() > 260 {
        return Err("startup_path_too_long".into());
    }
    Ok(command)
}
fn observe(store: &dyn StartupStore, expected: &str) -> StartupStatus {
    match store.read() {
        Ok(value) => {
            let managed = value.as_deref() == Some(expected);
            StartupStatus {
                registered: value.is_some(),
                managed,
                os_execution: "unknown".into(),
                error_code: (value.is_some() && !managed).then(|| "startup_entry_conflict".into()),
            }
        }
        Err(code) => StartupStatus {
            registered: false,
            managed: false,
            os_execution: "unknown".into(),
            error_code: Some(code),
        },
    }
}
fn change(store: &dyn StartupStore, expected: &str, enabled: bool) -> Result<(), String> {
    if store.read()?.is_some_and(|value| value != expected) {
        return Err("startup_entry_conflict".into());
    }
    if enabled {
        store.write(expected)?;
    } else {
        store.remove()?;
    }
    let after = observe(store, expected);
    if after.error_code.is_some() || after.registered != enabled || (enabled && !after.managed) {
        return Err("startup_verification_failed".into());
    }
    Ok(())
}
#[cfg(windows)]
struct CurrentUserStartup;
#[cfg(windows)]
impl StartupStore for CurrentUserStartup {
    fn read(&self) -> Result<Option<String>, String> {
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};
        let key = match RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
        {
            Ok(key) => key,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err("startup_read_failed".into()),
        };
        match key.get_value("SheetsMonitorDesktop") {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err("startup_read_failed".into()),
        }
    }
    fn write(&self, command: &str) -> Result<(), String> {
        use winreg::{enums::HKEY_CURRENT_USER, RegKey};
        let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .map_err(|_| "startup_write_failed")?;
        key.set_value("SheetsMonitorDesktop", &command)
            .map_err(|_| "startup_write_failed".into())
    }
    fn remove(&self) -> Result<(), String> {
        use winreg::{
            enums::{HKEY_CURRENT_USER, KEY_SET_VALUE},
            RegKey,
        };
        let key = match RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
            KEY_SET_VALUE,
        ) {
            Ok(key) => key,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(_) => return Err("startup_write_failed".into()),
        };
        match key.delete_value("SheetsMonitorDesktop") {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(_) => Err("startup_write_failed".into()),
        }
    }
}
fn startup_status() -> StartupStatus {
    #[cfg(windows)]
    if let Ok(command) = std::env::current_exe()
        .map_err(|_| "startup_path_unsupported".to_string())
        .and_then(|path| startup_command(&path))
    {
        return observe(&CurrentUserStartup, &command);
    }
    StartupStatus {
        registered: false,
        managed: false,
        os_execution: "unknown".into(),
        error_code: Some(
            if cfg!(windows) {
                "startup_path_unsupported"
            } else {
                "unsupported"
            }
            .into(),
        ),
    }
}
fn set_startup(enabled: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        let path = std::env::current_exe().map_err(|_| "startup_path_unsupported")?;
        change(&CurrentUserStartup, &startup_command(&path)?, enabled)
    }
    #[cfg(not(windows))]
    {
        let _ = enabled;
        Err("unsupported".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    #[derive(Default)]
    struct Memory {
        value: Mutex<Option<String>>,
        writes: std::sync::atomic::AtomicUsize,
    }
    impl StartupStore for Memory {
        fn read(&self) -> Result<Option<String>, String> {
            Ok(self.value.lock().unwrap().clone())
        }
        fn write(&self, command: &str) -> Result<(), String> {
            *self.value.lock().unwrap() = Some(command.into());
            self.writes.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
        fn remove(&self) -> Result<(), String> {
            *self.value.lock().unwrap() = None;
            self.writes.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
    #[test]
    fn startup_observation_never_writes_and_opt_in_verifies_actual_registration() {
        let memory = Memory::default();
        let command = "\"C:\\Program Files\\Sheets.exe\" --from-autostart";
        assert!(!observe(&memory, command).registered);
        assert_eq!(memory.writes.load(Ordering::SeqCst), 0);
        change(&memory, command, true).unwrap();
        assert!(observe(&memory, command).managed);
        assert_eq!(observe(&memory, command).os_execution, "unknown");
        change(&memory, command, false).unwrap();
        assert!(!observe(&memory, command).registered);
    }
    #[test]
    fn different_installation_is_preserved_for_enable_and_disable() {
        let memory = Memory {
            value: Mutex::new(Some("other installation".into())),
            ..Default::default()
        };
        for enabled in [true, false] {
            assert_eq!(
                change(&memory, "expected", enabled),
                Err("startup_entry_conflict".into())
            );
        }
        assert_eq!(memory.writes.load(Ordering::SeqCst), 0);
        let state = observe(&memory, "expected");
        assert!(state.registered);
        assert!(!state.managed);
    }
    #[test]
    fn startup_command_quotes_owned_path_rejects_relative_and_long_input() {
        assert!(startup_command(Path::new("relative.exe")).is_err());
        #[cfg(windows)]
        {
            assert_eq!(
                startup_command(Path::new("C:\\Program Files\\Sheets.exe")).unwrap(),
                "\"C:\\Program Files\\Sheets.exe\" --from-autostart"
            );
            assert!(startup_command(Path::new(&format!("C:\\{}.exe", "x".repeat(260)))).is_err());
        }
    }
    #[test]
    fn startup_errors_never_report_a_successful_registration() {
        struct Failure;
        impl StartupStore for Failure {
            fn read(&self) -> Result<Option<String>, String> {
                Err("startup_read_failed".into())
            }
            fn write(&self, _: &str) -> Result<(), String> {
                panic!("read failure cannot authorize write")
            }
            fn remove(&self) -> Result<(), String> {
                panic!("read failure cannot authorize remove")
            }
        }
        let status = observe(&Failure, "expected");
        assert!(!status.registered);
        assert_eq!(status.error_code.as_deref(), Some("startup_read_failed"));
        assert_eq!(
            change(&Failure, "expected", true),
            Err("startup_read_failed".into())
        );
        struct IgnoredWrite;
        impl StartupStore for IgnoredWrite {
            fn read(&self) -> Result<Option<String>, String> {
                Ok(None)
            }
            fn write(&self, _: &str) -> Result<(), String> {
                Ok(())
            }
            fn remove(&self) -> Result<(), String> {
                Ok(())
            }
        }
        assert_eq!(
            change(&IgnoredWrite, "expected", true),
            Err("startup_verification_failed".into())
        );
    }
}
