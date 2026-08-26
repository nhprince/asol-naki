//! Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

pub mod battery;
pub mod display;
pub mod hardware;
pub mod integrity;
pub mod models_db;
pub mod scoring;
pub mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(feature = "wdio")]
    // Embedded WebDriver server for E2E test builds only.
    let builder = builder.plugin(tauri_plugin_wdio_webdriver::init());

    builder
        .invoke_handler(tauri::generate_handler![
            hardware::scan_hardware_basic,
            hardware::scan_hardware_full,
            battery::scan_battery,
            storage::scan_storage,
            display::scan_display,
            integrity::run_integrity_checks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
