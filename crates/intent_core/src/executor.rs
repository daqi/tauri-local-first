use crate::{ActionResult, ExecutionPlan, ParsedIntent};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;

#[derive(Debug, Clone)]
pub struct ExecOptions {
    pub timeout_ms: u64,
    pub simulate: bool,
}

impl Default for ExecOptions {
    fn default() -> Self {
        Self {
            timeout_ms: 5_000,
            simulate: false,
        }
    }
}

#[derive(Debug)]
pub struct ExecutionOutcome {
    pub results: Vec<ActionResult>,
    pub overall_status: String,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

async fn mock_invoke(intent: &ParsedIntent) -> Result<(), &'static str> {
    if intent.action_name == "hang" {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok(())
    } else if intent.action_name == "fail" {
        Err("fail-simulated")
    } else {
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(())
    }
}

pub async fn execute(plan: &ExecutionPlan, opts: &ExecOptions) -> ExecutionOutcome {
    let mut results = Vec::new();
    for batch in &plan.batches {
        // sequential batches; within batch run concurrently
        let mut handles = Vec::new();
        for intent in &batch.intents {
            let intent_clone = intent.clone();
            let timeout_dur = Duration::from_millis(opts.timeout_ms);
            let simulate = opts.simulate;
            handles.push(tokio::spawn(async move {
                let started = now_ms();
                if simulate {
                    return ActionResult {
                        intent_id: intent_clone.id,
                        status: "simulated".into(),
                        reason: None,
                        retry_hint: None,
                        // Placeholder predicted effects (future: static/dynamic inference)
                        predicted_effects: Some(vec![format!("{}:{}", intent_clone.target_app_id.clone().unwrap_or_default(), intent_clone.action_name)]),
                        duration_ms: Some(0),
                        started_at: started,
                        finished_at: Some(started),
                    };
                }
                match timeout(timeout_dur, mock_invoke(&intent_clone)).await {
                    Err(_) => ActionResult {
                        intent_id: intent_clone.id,
                        status: "timeout".into(),
                        reason: Some("timeout_ms exceeded".into()),
                        retry_hint: Some("increase-timeout".into()),
                        predicted_effects: None,
                        duration_ms: None,
                        started_at: started,
                        finished_at: None,
                    },
                    Ok(Err(e)) => {
                        let finished = now_ms();
                        ActionResult {
                            intent_id: intent_clone.id,
                            status: "failed".into(),
                            reason: Some(e.to_string()),
                            retry_hint: Some("retry-later".into()),
                            predicted_effects: None,
                            duration_ms: Some((finished - started) as u32),
                            started_at: started,
                            finished_at: Some(finished),
                        }
                    }
                    Ok(Ok(_)) => {
                        let finished = now_ms();
                        ActionResult {
                            intent_id: intent_clone.id,
                            status: "success".into(),
                            reason: None,
                            retry_hint: None,
                            predicted_effects: None,
                            duration_ms: Some((finished - started) as u32),
                            started_at: started,
                            finished_at: Some(finished),
                        }
                    }
                }
            }));
        }
        for h in handles {
            if let Ok(r) = h.await {
                results.push(r);
            }
        }
    }

    // derive overall status
    let mut any_success = false;
    let mut any_fail_like = false;
    let mut any_simulated = false;
    for r in &results {
        match r.status.as_str() {
            "success" => any_success = true,
            "failed" | "timeout" => any_fail_like = true,
            "simulated" => any_simulated = true,
            _ => {}
        }
    }
    let overall = if any_fail_like && any_success {
        "partial"
    } else if any_fail_like && !any_success && !any_simulated {
        "failed"
    } else {
        "success"
    };

    ExecutionOutcome {
        results,
        overall_status: overall.into(),
    }
}

/// Convenience helper for dry run simulation (T012)
pub async fn simulate_plan(plan: &ExecutionPlan) -> ExecutionOutcome {
    execute(plan, &ExecOptions { timeout_ms: 0, simulate: true }).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExecutionPlan, ExecutionPlanBatch};

    fn mk_intent(id: &str, action: &str) -> ParsedIntent {
        ParsedIntent {
            id: id.into(),
            action_name: action.into(),
            target_app_id: Some("test".into()),
            params: serde_json::json!({}),
            confidence: 1.0,
            source_start: 0,
            source_end: 0,
            explicit: true,
        }
    }

    fn simple_plan(intents: Vec<ParsedIntent>) -> ExecutionPlan {
        ExecutionPlan {
            plan_id: "p".into(),
            original_input: "input".into(),
            intents: intents.clone(),
            deduplicated: intents.clone(),
            batches: intents
                .into_iter()
                .map(|i| ExecutionPlanBatch {
                    batch_id: uuid::Uuid::new_v4().to_string(),
                    intents: smallvec::smallvec![i],
                })
                .collect(),
            conflicts: vec![],
            strategy: "sequential".into(),
            generated_at: 0,
            dry_run: false,
            explain: None,
        }
    }

    #[tokio::test]
    async fn timeout_single_action() {
        let plan = simple_plan(vec![mk_intent("i1", "hang")]);
        let outcome = execute(
            &plan,
            &ExecOptions {
                timeout_ms: 100,
                simulate: false,
            },
        )
        .await;
        assert_eq!(outcome.results[0].status, "timeout");
        assert_eq!(outcome.overall_status, "failed");
    }

    #[tokio::test]
    async fn mix_fast_and_hang() {
        let plan = simple_plan(vec![mk_intent("i1", "act"), mk_intent("i2", "hang")]);
        let outcome = execute(
            &plan,
            &ExecOptions {
                timeout_ms: 100,
                simulate: false,
            },
        )
        .await;
        assert!(outcome.results.iter().any(|r| r.status == "success"));
        assert!(outcome.results.iter().any(|r| r.status == "timeout"));
        assert_eq!(outcome.overall_status, "partial");
    }

    #[tokio::test]
    async fn simulate_mode_all_simulated() {
        let plan = simple_plan(vec![mk_intent("i1", "act"), mk_intent("i2", "fail")]);
        let outcome = execute(
            &plan,
            &ExecOptions {
                timeout_ms: 10,
                simulate: true,
            },
        )
        .await;
        assert!(outcome.results.iter().all(|r| r.status == "simulated"));
        assert_eq!(outcome.overall_status, "success");
    }

    #[tokio::test]
    async fn empty_plan() {
        let plan = simple_plan(vec![]);
        let outcome = execute(&plan, &ExecOptions::default()).await;
        assert!(outcome.results.is_empty());
        assert_eq!(outcome.overall_status, "success");
    }

    #[tokio::test]
    async fn zero_timeout_all_timeout() {
        let plan = simple_plan(vec![mk_intent("i1", "act"), mk_intent("i2", "hang")]);
        let outcome = execute(
            &plan,
            &ExecOptions {
                timeout_ms: 0,
                simulate: false,
            },
        )
        .await;
        assert!(outcome.results.iter().all(|r| r.status == "timeout"));
        assert_eq!(outcome.overall_status, "failed");
    }

    #[tokio::test]
    async fn dry_run_parity_structure() {
        // Build a simple plan with two fast actions
        let plan = simple_plan(vec![mk_intent("i1", "act"), mk_intent("i2", "act")]);
        let simulated = execute(&plan, &ExecOptions { timeout_ms: 10, simulate: true }).await;
        let executed = execute(&plan, &ExecOptions { timeout_ms: 200, simulate: false }).await;
        assert_eq!(simulated.results.len(), executed.results.len());
        for (s, e) in simulated.results.iter().zip(executed.results.iter()) {
            assert_eq!(s.intent_id, e.intent_id);
            assert_eq!(s.status, "simulated");
            assert!(e.status == "success" || e.status == "failed" || e.status == "timeout");
            assert!(s.predicted_effects.is_some());
        }
        assert_eq!(simulated.overall_status, "success");
    }
}
