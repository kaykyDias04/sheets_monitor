//! Explicit isolated native QA binary; never part of the product entry point.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
fn main() {
    if let Err(error) = sheets_monitor_lib::native_probe::run() {
        eprintln!("native_probe_failed: {error}");
        std::process::exit(1);
    }
}
