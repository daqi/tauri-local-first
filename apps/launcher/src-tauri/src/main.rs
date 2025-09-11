#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{AppHandle, Builder, Manager};

#[tauri::command]
fn open_with_args(app: AppHandle, app_name: String, args: Option<String>) -> Result<(), String> {
  // 简单示例：通过事件通知其他窗口/插件（后续也可扩展为启动外部 EXE 并携带参数）
  app
    .emit_all("launcher://open", (app_name, args))
    .map_err(|e| e.to_string())
}

fn main() {
  Builder::default()
    .plugin(tauri_plugin_shell::init())
    .invoke_handler(tauri::generate_handler![open_with_args])
    .setup(|_app| {
      // TODO: 注册自定义协议/深链或解析命令行参数进行路由分发
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running launcher");
}
