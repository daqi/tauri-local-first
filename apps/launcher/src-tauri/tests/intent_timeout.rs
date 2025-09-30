use launcher_lib::commands::intent;

// This test crafts a command including a "hang" action to force timeout.
// Assuming parser syntax supports: hosts:hang() producing an intent with action_name="hang".
#[tokio::test]
async fn execute_plan_timeout() {
    // input with one hanging intent and one fast intent
    let input = "hosts:hang() hosts:switch(dev)"; // order: hang then switch
    let resp = intent::execute_plan(intent::PlanExecuteRequest {
        input: Some(input.into()),
        plan_id: None,
        dry_run: false,
        timeout_ms: Some(150),
    })
    .await
    .expect("expected response");

    let overall = resp.get("overallStatus").and_then(|v| v.as_str()).unwrap();
    assert!(
        overall == "partial" || overall == "failed",
        "overallStatus should reflect timeout impact"
    );
    let actions = resp.get("actions").and_then(|v| v.as_array()).unwrap();
    assert!(
        actions
            .iter()
            .any(|a| a.get("status").and_then(|v| v.as_str()) == Some("timeout")),
        "one action should timeout"
    );
}
