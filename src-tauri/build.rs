fn main() {
    // tauri-build 仅在 `desktop` feature 启用时跑（CLI-only 编译时跳过）。
    if std::env::var("CARGO_FEATURE_DESKTOP").is_ok() {
        tauri_build::build();
    }
}
