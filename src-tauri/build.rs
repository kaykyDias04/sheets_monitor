fn main() {
    if std::env::var_os("CARGO_FEATURE_DESKTOP").is_some() {
        println!("cargo:rerun-if-changed=../dist");
        tauri_build::build()
    }
}
