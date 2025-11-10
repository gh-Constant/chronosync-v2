pub mod commands;
pub mod icon_utils;
pub mod window_utils;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::get_active_window,
            commands::get_app_icon
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
