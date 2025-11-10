// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod icon_utils;
mod window_utils;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::get_active_window,
            commands::get_app_icon
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
