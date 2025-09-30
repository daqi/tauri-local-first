use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IntentParseRequest {
    pub input: String,
    #[serde(default)]
    pub explain: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanExecuteRequest {
    #[serde(skip_serializing_if = "Option::is_none")] pub input: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub plan_id: Option<String>,
    #[serde(default)] pub dry_run: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListHistoryRequest {
    #[serde(default)] pub limit: Option<u32>,
    #[serde(default)] pub after: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
}

type CommandResult<T> = Result<T, ErrorPayload>;

#[tauri::command]
pub async fn parse_intent(_req: IntentParseRequest) -> CommandResult<()> {
    Err(ErrorPayload { code: "NOT_IMPLEMENTED".into(), message: "parse_intent not implemented".into() })
}

#[tauri::command]
pub async fn dry_run(_req: PlanExecuteRequest) -> CommandResult<()> {
    Err(ErrorPayload { code: "NOT_IMPLEMENTED".into(), message: "dry_run not implemented".into() })
}

#[tauri::command]
pub async fn execute_plan(_req: PlanExecuteRequest) -> CommandResult<()> {
    Err(ErrorPayload { code: "NOT_IMPLEMENTED".into(), message: "execute_plan not implemented".into() })
}

#[tauri::command]
pub async fn list_history(_req: ListHistoryRequest) -> CommandResult<()> {
    Err(ErrorPayload { code: "NOT_IMPLEMENTED".into(), message: "list_history not implemented".into() })
}
