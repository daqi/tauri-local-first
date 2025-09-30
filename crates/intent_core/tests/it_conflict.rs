use intent_core::{build_plan, ParsedIntent};
use serde_json::json;

fn mk(id: &str, app: &str, action: &str) -> ParsedIntent {
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
fn conflict_detection_integration() {
    let i1 = mk("i1", "hosts", "switch");
    let i2 = mk("i2", "hosts", "switch");
    let i3 = mk("i3", "hosts", "list");
    let intents = vec![i1.clone(), i2.clone(), i3.clone()];

    let plan = build_plan(&intents, 3, "hosts:switch(dev) hosts:switch(prod) hosts:list()");

    assert_eq!(plan.conflicts.len(), 1);
    let conflict = &plan.conflicts[0];
    assert_eq!(conflict.intents.len(), 2);
    assert!(conflict.intents.contains(&i1.id) && conflict.intents.contains(&i2.id));
    assert_eq!(conflict.resolution, "force-order");

    assert!(plan.batches.len() >= 2);
    assert_eq!(plan.batches[0].intents.len(), 1);
    assert_eq!(plan.batches[1].intents.len(), 1);
    let first_id = &plan.batches[0].intents[0].id;
    let second_id = &plan.batches[1].intents[0].id;
    assert_ne!(first_id, second_id);
    assert!(first_id == &i1.id || first_id == &i2.id);
    assert!(second_id == &i1.id || second_id == &i2.id);

    assert_eq!(plan.strategy, "sequential");

    let later_contains_i3 = plan
        .batches
        .iter()
        .skip(2)
        .any(|b| b.intents.iter().any(|i| i.id == i3.id));
    assert!(later_contains_i3, "non-conflict intent should appear after conflict batches");
}
