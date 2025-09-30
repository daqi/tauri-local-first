use intent_core::{
    build_plan_with_cache, compute_concurrency, execute, simulate_plan, ExecOptions, IntentParser,
    ParseOptions, RuleBasedParser, HistoryStore, InMemoryHistoryStore, CommandHistoryRecord,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use intent_core::now_ms;

static PARSER: Lazy<RuleBasedParser> = Lazy::new(|| RuleBasedParser::new());
struct CachedPlan { plan: intent_core::ExecutionPlan, inserted_at: u64 }
static PLAN_CACHE: Lazy<Mutex<HashMap<String, CachedPlan>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static SIGNATURE_CACHE: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));
static HISTORY_STORE: Lazy<Mutex<Box<dyn HistoryStore>>> = Lazy::new(|| {
    Mutex::new(Box::new(InMemoryHistoryStore::default()) as Box<dyn HistoryStore>)
});


#[derive(Debug, Serialize, Deserialize)]
pub struct IntentParseRequest {
    pub input: String,
    #[serde(default)]
    pub explain: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlanExecuteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<String>,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListHistoryRequest {
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub after: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
}

type CommandResult<T> = Result<T, ErrorPayload>;

const PLAN_TTL_MS: u64 = 2 * 60 * 1000; // 2 minutes

fn purge_expired_plans(map: &mut HashMap<String, CachedPlan>, now: u64) {
    let expired: Vec<String> = map
        .iter()
        .filter(|(_, v)| now.saturating_sub(v.inserted_at) > PLAN_TTL_MS)
        .map(|(k, _)| k.clone())
        .collect();
    for k in expired { map.remove(&k); }
}

fn acquire_plan_from_request(req_input: &Option<String>, req_plan_id: &Option<String>) -> CommandResult<intent_core::ExecutionPlan> {
    if req_input.is_some() && req_plan_id.is_some() {
        return Err(ErrorPayload { code: "INVALID_INPUT".into(), message: "provide either input or planId".into() });
    }
    if let Some(pid) = req_plan_id {
        let mut guard = PLAN_CACHE.lock().map_err(|_| ErrorPayload { code: "LOCK_POISON".into(), message: "plan cache lock".into() })?;
        purge_expired_plans(&mut *guard, now_ms());
        let p = guard.get(pid).ok_or_else(|| ErrorPayload { code: "PLAN_NOT_FOUND".into(), message: "planId not found".into() })?;
        Ok(p.plan.clone())
    } else if let Some(input) = req_input {
        if input.trim().is_empty() { return Err(ErrorPayload { code: "INVALID_INPUT".into(), message: "input is empty".into() }); }
        let parse_res = PARSER.parse(input, &ParseOptions { enable_explain: false });
        let logical = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        let max_c = compute_concurrency(logical);
        let mut sig_cache = SIGNATURE_CACHE.lock().map_err(|_| ErrorPayload { code: "LOCK_POISON".into(), message: "signature cache lock".into() })?;
        let plan = build_plan_with_cache(&parse_res.intents, max_c, input, &mut *sig_cache);
        let mut guard = PLAN_CACHE.lock().map_err(|_| ErrorPayload { code: "LOCK_POISON".into(), message: "plan cache lock".into() })?;
        purge_expired_plans(&mut *guard, now_ms());
        guard.insert(plan.plan_id.clone(), CachedPlan { plan: plan.clone(), inserted_at: now_ms() });
        Ok(plan)
    } else {
        Err(ErrorPayload { code: "INVALID_INPUT".into(), message: "missing input or planId".into() })
    }
}

#[tauri::command]
pub async fn parse_intent(req: IntentParseRequest) -> CommandResult<serde_json::Value> {
    if req.input.trim().is_empty() {
        return Err(ErrorPayload {
            code: "INVALID_INPUT".into(),
            message: "input is empty".into(),
        });
    }
    // parse
    let parse_res = PARSER.parse(
        &req.input,
        &ParseOptions {
            enable_explain: req.explain,
        },
    );
    // concurrency heuristic (logical cores /2 capped handled by compute)
    let logical = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let max_c = compute_concurrency(logical);
    // build plan with cache
    let mut sig_cache = SIGNATURE_CACHE.lock().map_err(|_| ErrorPayload {
        code: "LOCK_POISON".into(),
        message: "signature cache lock".into(),
    })?;
    let mut plan = build_plan_with_cache(&parse_res.intents, max_c, &req.input, &mut *sig_cache);
    // attach explain if present
    if req.explain {
        plan.explain = parse_res.explain;
    }
    let plan_id = plan.plan_id.clone();
    let mut cache = PLAN_CACHE.lock().map_err(|_| ErrorPayload { code: "LOCK_POISON".into(), message: "plan cache lock".into() })?;
    purge_expired_plans(&mut *cache, now_ms());
    cache.insert(plan_id.clone(), CachedPlan { plan: plan.clone(), inserted_at: now_ms() });
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
pub async fn dry_run(req: PlanExecuteRequest) -> CommandResult<serde_json::Value> {
    let plan = acquire_plan_from_request(&req.input, &req.plan_id)?;
    let outcome = simulate_plan(&plan).await;
    // persist history record
    let mut intents_summary: Vec<String> = plan
        .deduplicated
        .iter()
        .map(|i| i.action_name.clone())
        .collect();
    if intents_summary.is_empty() {
        intents_summary = plan.intents.iter().map(|i| i.action_name.clone()).collect();
    }
    let record = CommandHistoryRecord {
        signature: plan.signature.clone().unwrap_or_else(|| plan.plan_id.clone()),
        input: plan.original_input.clone(),
        intents_summary: intents_summary,
        overall_status: outcome.overall_status.clone(),
        created_at: now_ms(),
        plan_size: plan.deduplicated.len() as u32,
        explain_used: plan.explain.is_some(),
    };
    if let Ok(mut store) = HISTORY_STORE.lock() { store.save(record); }

    let actions: Vec<serde_json::Value> = outcome.results.iter().map(|r| serde_json::json!({
        "intentId": r.intent_id,
        "status": r.status,
        "predictedEffects": r.predicted_effects,
    })).collect();
    Ok(serde_json::json!({
        "planId": plan.plan_id,
        "overallStatus": outcome.overall_status,
        "actions": actions,
        "batches": plan.batches.len(),
        "conflicts": plan.conflicts.len(),
        "cacheHit": plan.cache_hit.unwrap_or(false),
    }))
}

#[tauri::command]
pub async fn execute_plan(req: PlanExecuteRequest) -> CommandResult<serde_json::Value> {
    let plan = acquire_plan_from_request(&req.input, &req.plan_id)?;
    // timeout validation
    let timeout_ms = req.timeout_ms.unwrap_or(2_000);
    if timeout_ms < 100 || timeout_ms > 30_000 { return Err(ErrorPayload { code: "INVALID_INPUT".into(), message: "timeoutMs out of range (100-30000)".into() }); }

    let outcome = execute(&plan, &ExecOptions { timeout_ms, simulate: false }).await;
    let actions: Vec<serde_json::Value> = outcome
        .results
        .iter()
        .map(|r| {
            serde_json::json!({
                "intentId": r.intent_id,
                "status": r.status,
                "reason": r.reason,
                "retryHint": r.retry_hint,
                "durationMs": r.duration_ms,
            })
        })
        .collect();
    // record history via store
    let mut intents_summary: Vec<String> = plan
        .deduplicated
        .iter()
        .map(|i| i.action_name.clone())
        .collect();
    if intents_summary.is_empty() {
        intents_summary = plan.intents.iter().map(|i| i.action_name.clone()).collect();
    }
    let record = CommandHistoryRecord {
        signature: plan.signature.clone().unwrap_or_else(|| plan.plan_id.clone()),
        input: plan.original_input.clone(),
        intents_summary,
        overall_status: outcome.overall_status.clone(),
        created_at: now_ms(),
        plan_size: plan.deduplicated.len() as u32,
        explain_used: plan.explain.is_some(),
    };
    if let Ok(mut store) = HISTORY_STORE.lock() { store.save(record); }

    Ok(serde_json::json!({
        "planId": plan.plan_id,
        "overallStatus": outcome.overall_status,
        "actions": actions,
        "batches": plan.batches.len(),
        "conflicts": plan.conflicts.len(),
        "cacheHit": plan.cache_hit.unwrap_or(false),
    }))
}

#[tauri::command]
pub async fn list_history(req: ListHistoryRequest) -> CommandResult<serde_json::Value> {
    let limit = req.limit.unwrap_or(20).min(100) as usize; // cap
    let after_opt = if let Some(a) = req.after { if a > 0 { Some(a) } else { None } } else { None };
    let store_guard = HISTORY_STORE.lock().map_err(|_| ErrorPayload { code: "LOCK_POISON".into(), message: "history lock".into() })?;
    let items = store_guard.list(limit, after_opt);
    let next_after = items.last().map(|r| r.created_at);
    let json_items: Vec<serde_json::Value> = items.into_iter().map(|r| serde_json::json!({
        "signature": r.signature,
        "input": r.input,
        "overallStatus": r.overall_status,
        "planSize": r.plan_size,
        "explainUsed": r.explain_used,
        "createdAt": r.created_at,
        "intents": r.intents_summary,
    })).collect();
    Ok(serde_json::json!({
        "items": json_items,
        "nextAfter": next_after,
    }))
}

pub fn _test_reset_state() {
    if let Ok(mut c) = PLAN_CACHE.lock() { c.clear(); }
    if let Ok(mut s) = SIGNATURE_CACHE.lock() { s.clear(); }
    if let Ok(mut h) = HISTORY_STORE.lock() { let cutoff = 0; h.purge_older_than(cutoff); }
}
