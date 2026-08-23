// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod hardware;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(feature = "wdio")]
    // Embedded WebDriver server for E2E test builds only.
    let builder = builder.plugin(tauri_plugin_wdio_webdriver::init());

    builder
        .invoke_handler(tauri::generate_handler![hardware::scan_hardware_basic])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
