use intent_core::{build_plan, scan, simulate_plan, ParseOptions, RuleBasedParser};

#[test]
fn dry_run_pipeline_success() {
    // Prepare a fake descriptor directory in temp
    let dir = tempfile::tempdir().unwrap();
    let app_dir = dir.path().join("hostsManager");
    std::fs::create_dir(&app_dir).unwrap();
    std::fs::write(
        app_dir.join("tlfsuite.json"),
        r#"{"app_id":"hosts","actions":[{"name":"switch"}]}"#,
    )
    .unwrap();

    // Scan descriptors (not yet directly used in parser, but ensures scan integration works)
    let scan_res = scan(&[dir.path()]);
    assert_eq!(scan_res.descriptors.len(), 1);

    // Parse input
    let parser = RuleBasedParser::new();
    let input = "hosts:switch(dev)";
    let parse_res = parser.parse(
        input,
        &ParseOptions {
            enable_explain: false,
        },
    );
    assert!(!parse_res.intents.is_empty());

    // Build plan
    let plan = build_plan(&parse_res.intents, 2, input);
    assert!(!plan.batches.is_empty());

    // Dry run simulate
    let outcome = futures::executor::block_on(async { simulate_plan(&plan).await });
    assert!(outcome.results.iter().all(|r| r.status == "simulated"));
    assert_eq!(outcome.overall_status, "success");
}
