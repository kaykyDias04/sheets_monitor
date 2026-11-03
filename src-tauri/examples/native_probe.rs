//! Explicit isolated native QA binary; never part of the product entry point.
fn main() {
    if let Err(error) = sheets_monitor_lib::native_probe::run() {
        eprintln!("native_probe_failed: {error}");
        std::process::exit(1);
    }
}
