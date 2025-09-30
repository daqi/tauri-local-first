use serde::{Serialize, Deserialize};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::collections::{HashMap, HashSet};
use intent_core::{RuleBasedParser, ParseOptions, build_plan_with_cache, compute_concurrency};

static PARSER: Lazy<RuleBasedParser> = Lazy::new(|| RuleBasedParser::new());
static PLAN_CACHE: Lazy<Mutex<HashMap<String, intent_core::ExecutionPlan>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static SIGNATURE_CACHE: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

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
pub async fn parse_intent(req: IntentParseRequest) -> CommandResult<serde_json::Value> {
    if req.input.trim().is_empty() {
        return Err(ErrorPayload { code: "INVALID_INPUT".into(), message: "input is empty".into() });
    }
    // parse
    let parse_res = PARSER.parse(&req.input, &ParseOptions { enable_explain: req.explain });
    // concurrency heuristic (logical cores /2 capped handled by compute)
    let logical = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let max_c = compute_concurrency(logical);
    // build plan with cache
    let mut sig_cache = SIGNATURE_CACHE.lock().map_err(|_| ErrorPayload { code: "LOCK_POISON".into(), message: "signature cache lock".into() })?;
    let mut plan = build_plan_with_cache(&parse_res.intents, max_c, &req.input, &mut *sig_cache);
    // attach explain if present
    if req.explain { plan.explain = parse_res.explain; }
    let plan_id = plan.plan_id.clone();
    PLAN_CACHE.lock().map_err(|_| ErrorPayload { code: "LOCK_POISON".into(), message: "plan cache lock".into() })?.insert(plan_id.clone(), plan.clone());
    let resp = serde_json::json!({
        "planId": plan_id,
        "strategy": plan.strategy,
        "batches": plan.batches.len(),
        "conflicts": plan.conflicts.len(),
        "cacheHit": plan.cache_hit.unwrap_or(false),
        "signature": plan.signature,
        "explain": plan.explain,
    });
    Ok(resp)
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
