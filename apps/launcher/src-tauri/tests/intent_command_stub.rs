use launcher_lib::commands::intent;

#[tokio::test]
async fn parse_intent_success() {
    let resp = intent::parse_intent(intent::IntentParseRequest {
        input: "hosts:switch(dev)".into(),
        explain: true,
    })
    .await
    .expect("expected success response");
    assert!(resp.get("planId").is_some());
    assert!(resp.get("cacheHit").and_then(|v| v.as_bool()).is_some(), "cacheHit bool present");
    assert!(resp.get("signature").and_then(|v| v.as_str()).is_some());
    assert!(resp.get("batches").is_some());
    assert!(
        resp.get("explain").is_some(),
        "explain should be present when explain=true"
    );
}

#[tokio::test]
async fn dry_run_success_with_input() {
    let resp = intent::dry_run(intent::PlanExecuteRequest {
        input: Some("hosts:switch(dev)".into()),
        plan_id: None,
        dry_run: true,
    })
    .await
    .expect("expected success response");

    assert!(resp.get("planId").is_some(), "planId missing");
    assert_eq!(resp.get("overallStatus").and_then(|v| v.as_str()), Some("success"));
    assert!(resp.get("cacheHit").and_then(|v| v.as_bool()).is_some(), "cacheHit bool present");
    let actions = resp.get("actions").and_then(|v| v.as_array()).expect("actions array");
    assert!(!actions.is_empty(), "actions should not be empty");
    let first = &actions[0];
    assert_eq!(first.get("status").and_then(|v| v.as_str()), Some("simulated"));
}

#[tokio::test]
async fn dry_run_success_with_plan_id() {
    // first build plan via parse_intent to ensure cache insertion
    let parse_resp = intent::parse_intent(intent::IntentParseRequest { input: "hosts:switch(dev)".into(), explain: false }).await.expect("parse success");
    let pid = parse_resp.get("planId").and_then(|v| v.as_str()).expect("planId");

    let resp = intent::dry_run(intent::PlanExecuteRequest { input: None, plan_id: Some(pid.into()), dry_run: true })
        .await
        .expect("expected success response");
    assert_eq!(resp.get("planId").and_then(|v| v.as_str()), Some(pid));
    assert_eq!(resp.get("overallStatus").and_then(|v| v.as_str()), Some("success"));
    let actions = resp.get("actions").and_then(|v| v.as_array()).expect("actions array");
    assert!(!actions.is_empty(), "actions should not be empty");
}

#[tokio::test]
async fn execute_plan_stub_not_implemented() {
    let err = intent::execute_plan(intent::PlanExecuteRequest {
        input: Some("hosts:switch(dev)".into()),
        plan_id: None,
        dry_run: false,
    })
    .await
    .expect_err("expected NOT_IMPLEMENTED error");
    assert_eq!(err.code, "NOT_IMPLEMENTED");
}

#[tokio::test]
async fn list_history_stub_not_implemented() {
    let err = intent::list_history(intent::ListHistoryRequest {
        limit: Some(10),
        after: None,
    })
    .await
    .expect_err("expected NOT_IMPLEMENTED error");
    assert_eq!(err.code, "NOT_IMPLEMENTED");
}
