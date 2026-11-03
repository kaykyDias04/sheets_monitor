fn main() {
    // Tauri embeds the application's manifest in its product binary, while this
    // optional example needs its own Common Controls 6 activation context.
    if std::env::var_os("CARGO_FEATURE_NATIVE_PROBE").is_some()
        && std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        let manifest = std::path::PathBuf::from(
            std::env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest directory"),
        )
        .join("native-probe.manifest");
        println!("cargo:rerun-if-changed=native-probe.manifest");
        println!("cargo:rustc-link-arg-examples=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-examples=/MANIFESTINPUT:{}",
            manifest.display()
        );
    }
    println!("cargo:rerun-if-env-changed=SHEETS_MONITOR_OAUTH_CLIENT_ID");
    println!("cargo:rerun-if-env-changed=SHEETS_MONITOR_OAUTH_DESKTOP_SECRET");
    if std::env::var_os("CARGO_FEATURE_DESKTOP").is_some() {
        println!("cargo:rerun-if-changed=../dist");
        tauri_build::build()
    }
}
