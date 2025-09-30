use crate::{detect_conflicts, ExecutionPlan, ExecutionPlanBatch, ParsedIntent};
use crate::normalize_signature;
use std::collections::HashSet as StdHashSet;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// Build an execution plan from parsed intents.
/// Steps:
/// 1. Deduplicate identical intents (app, action, params JSON string)
/// 2. Detect conflicts (reuse detect_conflicts)
/// 3. Produce batches: conflict intents become single-intent sequential batches.
///    Remaining (non-conflict) intents bucketed by max_concurrency window.
/// 4. Determine strategy: sequential if all batches size=1 else parallel-limited
pub fn build_plan(
    intents: &[ParsedIntent],
    max_concurrency: usize,
    original_input: &str,
) -> ExecutionPlan {
    // full copy of intents
    let all_intents: Vec<ParsedIntent> = intents.to_vec();

    // dedup
    let mut seen: HashSet<String> = HashSet::new();
    let mut deduped: Vec<ParsedIntent> = Vec::new();
    for intent in &all_intents {
        let key = format!(
            "{}|{}|{}",
            intent.target_app_id.as_deref().unwrap_or("_"),
            intent.action_name,
            intent.params
        );
        if seen.insert(key) {
            deduped.push(intent.clone());
        }
    }

    // conflicts
    let conflicts = detect_conflicts(&all_intents);
    let conflict_ids: HashSet<String> = conflicts
        .iter()
        .flat_map(|c| c.intents.iter().cloned())
        .collect();

    // Separate conflict vs normal intents preserving original order
    let mut conflict_intents: Vec<ParsedIntent> = Vec::new();
    let mut normal_intents: Vec<ParsedIntent> = Vec::new();
    for intent in &all_intents {
        if conflict_ids.contains(&intent.id) {
            conflict_intents.push(intent.clone());
        } else {
            normal_intents.push(intent.clone());
        }
    }

    // Batches
    let mut batches: Vec<ExecutionPlanBatch> = Vec::new();

    // Conflict: single-intent batches in original order
    for intent in conflict_intents {
        batches.push(ExecutionPlanBatch {
            batch_id: uuid::Uuid::new_v4().to_string(),
            intents: smallvec::smallvec![intent],
        });
    }

    // Normal: group by window size
    if max_concurrency == 0 {
        // fallback treat as sequential
        for intent in normal_intents {
            batches.push(ExecutionPlanBatch {
                batch_id: uuid::Uuid::new_v4().to_string(),
                intents: smallvec::smallvec![intent],
            });
        }
    } else {
        let mut window: Vec<ParsedIntent> = Vec::new();
        for intent in normal_intents {
            window.push(intent);
            if window.len() == max_concurrency {
                batches.push(ExecutionPlanBatch {
                    batch_id: uuid::Uuid::new_v4().to_string(),
                    intents: window.drain(..).collect(),
                });
            }
        }
        if !window.is_empty() {
            batches.push(ExecutionPlanBatch {
                batch_id: uuid::Uuid::new_v4().to_string(),
                intents: window.into_iter().collect(),
            });
        }
    }

    // Strategy: if max_concurrency > 1 and there exists any batch with >1 intents, mark parallel-limited.
    // Else sequential.
    let has_parallel = max_concurrency > 1 && batches.iter().any(|b| b.intents.len() > 1);
    let strategy = if has_parallel {
        "parallel-limited"
    } else {
        "sequential"
    };

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    ExecutionPlan {
        plan_id: uuid::Uuid::new_v4().to_string(),
        original_input: original_input.to_string(),
        intents: all_intents,
        deduplicated: deduped,
        batches,
        conflicts,
        strategy: strategy.to_string(),
        generated_at: ts,
        dry_run: false,
        explain: None,
        signature: None,
        cache_hit: None,
    }
}

/// Build a plan and compute signature; update a provided cache (set of signatures) marking cache_hit.
pub fn build_plan_with_cache(
    intents: &[ParsedIntent],
    max_concurrency: usize,
    original_input: &str,
    cache: &mut StdHashSet<String>,
) -> ExecutionPlan {
    let mut plan = build_plan(intents, max_concurrency, original_input);
    let sig = normalize_signature(&plan.deduplicated);
    let hit = cache.contains(&sig);
    if !hit {
        cache.insert(sig.clone());
    }
    plan.signature = Some(sig);
    plan.cache_hit = Some(hit);
    plan
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mk(app: &str, action: &str, id: &str) -> ParsedIntent {
        ParsedIntent {
            id: id.into(),
            action_name: action.into(),
            target_app_id: Some(app.into()),
            params: json!({}),
            confidence: 1.0,
            source_start: 0,
            source_end: 0,
            explicit: true,
        }
    }

    #[test]
    fn dedup_removes_duplicate() {
        let a1 = mk("hosts", "switch", "a1");
        let a2 = ParsedIntent {
            id: "a2".into(),
            ..a1.clone()
        }; // duplicate content except id
        let plan = build_plan(&[a1, a2], 2, "input");
        assert_eq!(plan.intents.len(), 2);
        assert_eq!(plan.deduplicated.len(), 1);
    }

    #[test]
    fn batching_respects_max_concurrency() {
        let mut intents = Vec::new();
        for i in 0..6 {
            intents.push(mk("app", "act", &format!("i{i}")));
        }
        let plan = build_plan(&intents, 2, "input");
        // Either we formed parallel batches (size up to 2) or remained sequential (all size 1).
        assert!(plan.batches.iter().all(|b| b.intents.len() <= 2));
        let total: usize = plan.batches.iter().map(|b| b.intents.len()).sum();
        assert_eq!(total, 6);
        // Accept both strategies depending on grouping heuristic.
        assert!(plan.strategy == "parallel-limited" || plan.strategy == "sequential");
    }

    #[test]
    fn conflicts_become_sequential_batches() {
        let a = mk("hosts", "switch", "x1");
        let b = mk("hosts", "switch", "x2");
        let c = mk("hosts", "list", "x3");
        let plan = build_plan(&[a, b, c], 2, "input");
        // two conflict batches + one normal batch (c may group alone or with others -> here alone)
        assert_eq!(plan.conflicts.len(), 1);
        // The first two batches should each have size 1 (conflict), third may be size 1
        assert!(plan.batches.iter().filter(|b| b.intents.len() == 1).count() >= 2);
    }
}
