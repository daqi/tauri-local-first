use launcher_lib::commands::intent;

#[tokio::test]
async fn history_empty_then_single() {
    intent::_test_reset_state();
    // empty
    let empty = intent::list_history(intent::ListHistoryRequest { limit: Some(10), after: None }).await.expect("list ok");
    assert_eq!(empty.get("items").and_then(|v| v.as_array()).unwrap().len(), 0);

    // execute one plan
    let exec = intent::execute_plan(intent::PlanExecuteRequest { input: Some("hosts:switch(dev)".into()), plan_id: None, dry_run: false, timeout_ms: None }).await.expect("exec ok");
    assert_eq!(exec.get("overallStatus").and_then(|v| v.as_str()), Some("success"));

    let listed = intent::list_history(intent::ListHistoryRequest { limit: Some(10), after: None }).await.expect("list ok");
    let items = listed.get("items").and_then(|v| v.as_array()).unwrap();
    assert!(!items.is_empty());
    let last = items.last().unwrap();
    assert!(last.get("seq").is_some());
    assert_eq!(last.get("overallStatus").and_then(|v| v.as_str()), Some("success"));
}

#[tokio::test]
async fn history_pagination() {
    intent::_test_reset_state();
    // create multiple entries
    for _ in 0..5 {
    intent::execute_plan(intent::PlanExecuteRequest { input: Some("hosts:switch(dev)".into()), plan_id: None, dry_run: false, timeout_ms: None }).await.expect("exec ok");
    }
    // fetch first 2
    let first = intent::list_history(intent::ListHistoryRequest { limit: Some(2), after: None }).await.expect("list ok");
    let items1 = first.get("items").and_then(|v| v.as_array()).unwrap();
    assert_eq!(items1.len(), 2);
    let next_after = first.get("nextAfter").and_then(|v| v.as_u64()).expect("nextAfter");

    // fetch next page
    let second = intent::list_history(intent::ListHistoryRequest { limit: Some(2), after: Some(next_after) }).await.expect("list ok");
    let items2 = second.get("items").and_then(|v| v.as_array()).unwrap();
    assert!(items2.len() <= 2 && items2.len() >= 1);
    if let Some(next_after2) = second.get("nextAfter").and_then(|v| v.as_u64()) {
        // there is a third page
        let third = intent::list_history(intent::ListHistoryRequest { limit: Some(2), after: Some(next_after2) }).await.expect("list ok");
        let items3 = third.get("items").and_then(|v| v.as_array()).unwrap();
        assert!(items3.len() <= 2);
        assert!(third.get("nextAfter").unwrap().is_null());
    }
}
