use intent_core::{build_plan_with_cache, ParsedIntent};
use serde_json::json;
use std::collections::HashSet;

fn mk(id: &str, action: &str) -> ParsedIntent {
    ParsedIntent {
        id: id.into(),
        action_name: action.into(),
        target_app_id: Some("hosts".into()),
        params: json!({"group":"dev"}),
        confidence: 1.0,
        source_start: 0,
        source_end: 0,
        explicit: true,
    }
}

#[test]
fn history_cache_reuse_signature() {
    let intents1 = vec![mk("i1", "switch"), mk("i2", "switch")]; // duplicate content except ids
    let intents2 = vec![mk("i3", "switch"), mk("i4", "switch")];
    let mut cache: HashSet<String> = HashSet::new();

    let plan1 = build_plan_with_cache(&intents1, 2, "hosts:switch(dev)", &mut cache);
    assert_eq!(plan1.cache_hit, Some(false));
    assert!(plan1.signature.is_some());
    let sig = plan1.signature.clone().unwrap();

    let plan2 = build_plan_with_cache(&intents2, 2, "hosts:switch(dev)", &mut cache);
    assert_eq!(plan2.cache_hit, Some(true));
    assert_eq!(plan2.signature.as_ref().unwrap(), &sig);
    // Dedup should reduce to one intent in both plans
    assert_eq!(plan1.deduplicated.len(), 1);
    assert_eq!(plan2.deduplicated.len(), 1);
}
