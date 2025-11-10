// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod icon_utils;
mod window_utils;

use tauri_plugin_log::{Target, TargetKind};

fn main() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("chronosync.log".into()),
                    }),
                ])
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::get_active_window,
            commands::get_app_icon
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
