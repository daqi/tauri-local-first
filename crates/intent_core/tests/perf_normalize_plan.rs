//! Lightweight performance assertion tests for T033
//! Ensures normalize_signature + build_plan remain under soft thresholds in debug.

use intent_core::{build_plan, normalize_signature, ParsedIntent};
use serde_json::json;
use std::time::Instant;

fn sample_intents(n: usize) -> Vec<ParsedIntent> {
    (0..n)
        .map(|i| ParsedIntent {
            id: format!("i{i}"),
            action_name: if i % 2 == 0 {
                "switch".into()
            } else {
                "open".into()
            },
            target_app_id: Some(if i % 3 == 0 {
                "hosts".into()
            } else {
                "clipboard".into()
            }),
            params: json!({"idx": i}),
            confidence: 1.0,
            source_start: 0,
            source_end: 0,
            explicit: true,
        })
        .collect()
}

#[test]
fn perf_normalize_and_plan_debug_bounds() {
    let intents = sample_intents(10);
    // warm up
    for _ in 0..10 {
        let _ = normalize_signature(&intents);
    }
    let start = Instant::now();
    for _ in 0..100 {
        let _ = normalize_signature(&intents);
    }
    let dur_norm = start.elapsed();

    let start2 = Instant::now();
    for _ in 0..50 {
        let _ = build_plan(&intents, 4, "input");
    }
    let dur_plan = start2.elapsed();

    // Debug build thresholds (loose): < 15ms total for 100 norm ops, < 60ms for 50 plan ops
    assert!(
        dur_norm.as_millis() < 15,
        "normalize_signature too slow: {:?}",
        dur_norm
    );
    assert!(
        dur_plan.as_millis() < 60,
        "build_plan too slow: {:?}",
        dur_plan
    );
}
