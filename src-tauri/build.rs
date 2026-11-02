fn main() {
    println!("cargo:rerun-if-env-changed=SHEETS_MONITOR_OAUTH_CLIENT_ID");
    println!("cargo:rerun-if-env-changed=SHEETS_MONITOR_OAUTH_DESKTOP_SECRET");
    if std::env::var_os("CARGO_FEATURE_DESKTOP").is_some() {
        println!("cargo:rerun-if-changed=../dist");
        tauri_build::build()
    }
}
