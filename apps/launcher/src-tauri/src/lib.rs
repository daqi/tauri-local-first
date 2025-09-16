pub mod commands;
pub mod adapters;

use tauri::{ Builder, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  Builder::default()
    // Single instance must be first
    .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
      if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_focus();
      }
    }))
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_deep_link::init())
    .invoke_handler(tauri::generate_handler![
      commands::open_with_args,
      commands::list_apps
    ]).setup(|app| {
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
