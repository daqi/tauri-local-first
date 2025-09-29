// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod commands;
use tauri::{Builder, Manager};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // Focus main window when a second instance is attempted.
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::ping,
            commands::get_list,
            commands::set_list,
            commands::get_content_of_list,
            commands::get_system_hosts,
            commands::set_system_hosts,
            commands::get_hosts_content,
            commands::set_hosts_content,
            commands::close_main_window,
            commands::quit
        ])
        .setup(|app| {
            #[cfg(any(target_os = "linux", all(debug_assertions, windows)))]
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                app.deep_link().register_all()?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running launcher");
}
