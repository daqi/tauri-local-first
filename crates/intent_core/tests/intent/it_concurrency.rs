use intent_core::{build_plan};
use serde_json::json;

fn mk_intent(id: &str) -> intent_core::ParsedIntent {
    intent_core::ParsedIntent {
        id: id.into(),
        action_name: "op".into(),
        target_app_id: Some("app".into()),
        params: json!({}),
        confidence: 1.0,
        source_start: 0,
        source_end: 0,
        explicit: true,
    }
}

#[test]
fn concurrency_batch_limit() {
    let intents: Vec<_> = (0..6).map(|i| mk_intent(&format!("i{i}"))).collect();
    let plan = build_plan(&intents, 2, "input");
    // Validate each batch size <= 2 and total intents accounted
    assert!(plan.batches.iter().all(|b| b.intents.len() <= 2));
    let total: usize = plan.batches.iter().map(|b| b.intents.len()).sum();
    assert_eq!(total, 6);
    // strategy may be sequential or parallel-limited depending on heuristic
    assert!(plan.strategy == "parallel-limited" || plan.strategy == "sequential");
}
