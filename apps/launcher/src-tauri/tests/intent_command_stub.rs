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
    assert_eq!(resp.get("cacheHit").and_then(|v| v.as_bool()), Some(false));
    assert!(resp.get("signature").and_then(|v| v.as_str()).is_some());
    assert!(resp.get("batches").is_some());
    assert!(resp.get("explain").is_some(), "explain should be present when explain=true");
}

#[tokio::test]
async fn dry_run_stub_not_implemented() {
    let err = intent::dry_run(intent::PlanExecuteRequest {
        input: Some("hosts:switch(dev)".into()),
        plan_id: None,
        dry_run: true,
    })
    .await
    .expect_err("expected NOT_IMPLEMENTED error");
    assert_eq!(err.code, "NOT_IMPLEMENTED");
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
