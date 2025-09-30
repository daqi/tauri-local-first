use intent_core::{
    build_plan_with_cache, compute_concurrency, execute, simulate_plan, ExecOptions, IntentParser,
    ParseOptions, RuleBasedParser,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, atomic::{AtomicU64, Ordering}};
use std::time::{SystemTime, UNIX_EPOCH};

static PARSER: Lazy<RuleBasedParser> = Lazy::new(|| RuleBasedParser::new());
static PLAN_CACHE: Lazy<Mutex<HashMap<String, intent_core::ExecutionPlan>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static SIGNATURE_CACHE: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));
// History storage (simple in-memory ring buffer)
const MAX_HISTORY: usize = 200;
#[derive(Debug, Clone, Serialize)]
pub struct HistoryEntry {
    pub seq: u64,
    pub plan_id: String,
    pub input: String,
    pub overall_status: String,
    pub actions: Vec<HistoryAction>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct HistoryAction {
    pub intent_id: String,
    pub status: String,
}

static HISTORY: Lazy<Mutex<Vec<HistoryEntry>>> = Lazy::new(|| Mutex::new(Vec::new()));
static HISTORY_SEQ: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

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
    PLAN_CACHE
        .lock()
        .map_err(|_| ErrorPayload {
            code: "LOCK_POISON".into(),
            message: "plan cache lock".into(),
        })?
        .insert(plan_id.clone(), plan.clone());
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
    // Validate mutual exclusivity of input vs plan_id
    if req.input.is_some() && req.plan_id.is_some() {
        return Err(ErrorPayload {
            code: "INVALID_INPUT".into(),
            message: "provide either input or planId".into(),
        });
    }
    // Acquire plan
    let plan = if let Some(pid) = &req.plan_id {
        let guard = PLAN_CACHE.lock().map_err(|_| ErrorPayload {
            code: "LOCK_POISON".into(),
            message: "plan cache lock".into(),
        })?;
        let p = guard.get(pid).ok_or_else(|| ErrorPayload {
            code: "PLAN_NOT_FOUND".into(),
            message: "planId not found".into(),
        })?;
        p.clone()
    } else if let Some(input) = &req.input {
        if input.trim().is_empty() {
            return Err(ErrorPayload {
                code: "INVALID_INPUT".into(),
                message: "input is empty".into(),
            });
        }
        let parse_res = PARSER.parse(
            input,
            &ParseOptions {
                enable_explain: false,
            },
        );
        let logical = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let max_c = compute_concurrency(logical);
        let mut sig_cache = SIGNATURE_CACHE.lock().map_err(|_| ErrorPayload {
            code: "LOCK_POISON".into(),
            message: "signature cache lock".into(),
        })?;
        let plan = build_plan_with_cache(&parse_res.intents, max_c, input, &mut *sig_cache);
        PLAN_CACHE
            .lock()
            .map_err(|_| ErrorPayload {
                code: "LOCK_POISON".into(),
                message: "plan cache lock".into(),
            })?
            .insert(plan.plan_id.clone(), plan.clone());
        plan
    } else {
        return Err(ErrorPayload {
            code: "INVALID_INPUT".into(),
            message: "missing input or planId".into(),
        });
    };

    let outcome = simulate_plan(&plan).await;
    let actions: Vec<serde_json::Value> = outcome
        .results
        .iter()
        .map(|r| {
            serde_json::json!({
                "intentId": r.intent_id,
                "status": r.status,
                "predictedEffects": r.predicted_effects,
            })
        })
        .collect();
    let resp = serde_json::json!({
        "planId": plan.plan_id,
        "overallStatus": outcome.overall_status,
        "actions": actions,
        "batches": plan.batches.len(),
        "conflicts": plan.conflicts.len(),
        "cacheHit": plan.cache_hit.unwrap_or(false),
    });
    Ok(resp)
}

#[tauri::command]
pub async fn execute_plan(req: PlanExecuteRequest) -> CommandResult<serde_json::Value> {
    if req.input.is_some() && req.plan_id.is_some() {
        return Err(ErrorPayload {
            code: "INVALID_INPUT".into(),
            message: "provide either input or planId".into(),
        });
    }
    let plan = if let Some(pid) = &req.plan_id {
        let guard = PLAN_CACHE.lock().map_err(|_| ErrorPayload {
            code: "LOCK_POISON".into(),
            message: "plan cache lock".into(),
        })?;
        guard.get(pid).cloned().ok_or_else(|| ErrorPayload {
            code: "PLAN_NOT_FOUND".into(),
            message: "planId not found".into(),
        })?
    } else if let Some(input) = &req.input {
        if input.trim().is_empty() {
            return Err(ErrorPayload {
                code: "INVALID_INPUT".into(),
                message: "input is empty".into(),
            });
        }
        let parse_res = PARSER.parse(
            input,
            &ParseOptions {
                enable_explain: false,
            },
        );
        let logical = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let max_c = compute_concurrency(logical);
        let mut sig_cache = SIGNATURE_CACHE.lock().map_err(|_| ErrorPayload {
            code: "LOCK_POISON".into(),
            message: "signature cache lock".into(),
        })?;
        let plan = build_plan_with_cache(&parse_res.intents, max_c, input, &mut *sig_cache);
        PLAN_CACHE
            .lock()
            .map_err(|_| ErrorPayload {
                code: "LOCK_POISON".into(),
                message: "plan cache lock".into(),
            })?
            .insert(plan.plan_id.clone(), plan.clone());
        plan
    } else {
        return Err(ErrorPayload {
            code: "INVALID_INPUT".into(),
            message: "missing input or planId".into(),
        });
    };

    let outcome = execute(
        &plan,
        &ExecOptions {
            timeout_ms: 2_000,
            simulate: false,
        },
    )
    .await;
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
    // record history
    let actions_for_history: Vec<HistoryAction> = outcome.results.iter().map(|r| HistoryAction { intent_id: r.intent_id.clone(), status: r.status.clone() }).collect();
    let seq = HISTORY_SEQ.fetch_add(1, Ordering::SeqCst) + 1; // start seq at 1
    let entry = HistoryEntry {
        seq,
        plan_id: plan.plan_id.clone(),
        input: plan.original_input.clone(),
        overall_status: outcome.overall_status.clone(),
        actions: actions_for_history,
        created_at: now_ms(),
    };
    if let Ok(mut hist) = HISTORY.lock() {
        hist.push(entry);
        if hist.len() > MAX_HISTORY { let overflow = hist.len() - MAX_HISTORY; hist.drain(0..overflow); }
    }

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
    let after = req.after.unwrap_or(0);
    let hist = HISTORY.lock().map_err(|_| ErrorPayload { code: "LOCK_POISON".into(), message: "history lock".into() })?;
    let mut filtered: Vec<&HistoryEntry> = hist.iter().filter(|e| e.seq > after).collect();
    // already ordered by insertion seq
    let more = filtered.len() > limit;
    if filtered.len() > limit { filtered.truncate(limit); }
    let next_after = filtered.last().map(|e| e.seq).unwrap_or(0);
    let items_json: Vec<serde_json::Value> = filtered.into_iter().map(|e| serde_json::json!({
        "seq": e.seq,
        "planId": e.plan_id,
        "input": e.input,
        "overallStatus": e.overall_status,
        "actions": e.actions.iter().map(|a| serde_json::json!({"intentId": a.intent_id, "status": a.status})).collect::<Vec<_>>(),
        "createdAt": e.created_at,
    })).collect();
    Ok(serde_json::json!({
        "items": items_json,
        "nextAfter": if more { serde_json::Value::from(next_after) } else { serde_json::Value::Null },
    }))
}

pub fn _test_reset_state() {
    if let Ok(mut c) = PLAN_CACHE.lock() { c.clear(); }
    if let Ok(mut s) = SIGNATURE_CACHE.lock() { s.clear(); }
    if let Ok(mut h) = HISTORY.lock() { h.clear(); }
    HISTORY_SEQ.store(0, Ordering::SeqCst);
}
