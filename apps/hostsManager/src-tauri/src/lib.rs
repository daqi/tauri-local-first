// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod commands;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
