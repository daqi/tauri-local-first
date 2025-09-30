use launcher_lib::commands::intent;

#[tokio::test]
async fn parse_intent_stub_not_implemented() {
    let err = intent::parse_intent(intent::IntentParseRequest { input: "hosts:switch(dev)".into(), explain: false })
        .await
        .expect_err("expected NOT_IMPLEMENTED error");
    assert_eq!(err.code, "NOT_IMPLEMENTED");
}

#[tokio::test]
async fn dry_run_stub_not_implemented() {
    let err = intent::dry_run(intent::PlanExecuteRequest { input: Some("hosts:switch(dev)".into()), plan_id: None, dry_run: true })
        .await
        .expect_err("expected NOT_IMPLEMENTED error");
    assert_eq!(err.code, "NOT_IMPLEMENTED");
}

#[tokio::test]
async fn execute_plan_stub_not_implemented() {
    let err = intent::execute_plan(intent::PlanExecuteRequest { input: Some("hosts:switch(dev)".into()), plan_id: None, dry_run: false })
        .await
        .expect_err("expected NOT_IMPLEMENTED error");
    assert_eq!(err.code, "NOT_IMPLEMENTED");
}

#[tokio::test]
async fn list_history_stub_not_implemented() {
    let err = intent::list_history(intent::ListHistoryRequest { limit: Some(10), after: None })
        .await
        .expect_err("expected NOT_IMPLEMENTED error");
    assert_eq!(err.code, "NOT_IMPLEMENTED");
}
