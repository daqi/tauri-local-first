use serde::Serialize;
use tauri::{AppHandle};
use tauri::Emitter;

#[derive(Serialize, Clone)]
struct OpenEventPayload {
  app: String,
  args: Option<String>,
}

#[tauri::command]
pub fn open_with_args(app: AppHandle, app_name: String, args: Option<String>) -> Result<(), String> {
  let payload = OpenEventPayload { app: app_name, args };
  app.emit("launcher://open", payload).map_err(|e| e.to_string())
}