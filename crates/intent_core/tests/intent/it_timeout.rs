use intent_core::{build_plan, execute, ExecOptions, ParsedIntent};
use serde_json::json;

fn mk(id: &str, action: &str) -> ParsedIntent {
    ParsedIntent {
        id: id.into(),
        action_name: action.into(),
        target_app_id: Some("app".into()),
        params: json!({}),
        confidence: 1.0,
        source_start: 0,
        source_end: 0,
        explicit: true,
    }
}

#[tokio::test]
async fn timeout_partial_status() {
    let intents = vec![
        mk("i1", "act"),
        mk("i2", "act"),
        mk("i3", "hang"), // will timeout
    ];
    let plan = build_plan(&intents, 2, "input");
    let outcome = execute(&plan, &ExecOptions { timeout_ms: 100, simulate: false }).await;
    assert!(outcome.results.iter().any(|r| r.status == "timeout"));
    assert!(outcome.results.iter().any(|r| r.status == "success"));
    assert_eq!(outcome.overall_status, "partial");
}
