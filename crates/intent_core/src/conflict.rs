use crate::{ConflictDetection, ParsedIntent};
use std::collections::HashMap;

/// Detect conflicts among intents.
/// Current heuristic: intents with same (target_app_id, action_name) considered mutually exclusive
/// and grouped into a single ConflictDetection when count > 1.
/// Future: incorporate descriptor-declared conflict keys.
pub fn detect_conflicts(intents: &[ParsedIntent]) -> Vec<ConflictDetection> {
    let mut groups: HashMap<(Option<&str>, &str), Vec<&ParsedIntent>> = HashMap::new();
    for intent in intents {
        let key = (intent.target_app_id.as_deref(), intent.action_name.as_str());
        groups.entry(key).or_default().push(intent);
    }
    let mut conflicts = Vec::new();
    for ((app_opt, action), list) in groups.into_iter() {
        if list.len() > 1 {
            let conflict_key = format!("{}::{}", app_opt.unwrap_or("_"), action);
            conflicts.push(ConflictDetection {
                conflict_key,
                intents: list.iter().map(|i| i.id.clone()).collect(),
                reason: "same-app-action-mutual-exclusion".to_string(),
                resolution: "force-order".to_string(),
            });
        }
    }
    conflicts
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_intent(id: &str, app: &str, action: &str) -> ParsedIntent {
        ParsedIntent {
            id: id.to_string(),
            action_name: action.to_string(),
            target_app_id: Some(app.to_string()),
            params: json!({}),
            confidence: 1.0,
            source_start: 0,
            source_end: 0,
            explicit: true,
        }
    }

    #[test]
    fn detects_conflict_same_app_action() {
        let a = make_intent("i1", "hosts", "switch");
        let b = make_intent("i2", "hosts", "switch");
        let conflicts = detect_conflicts(&[a.clone(), b.clone()]);
        assert_eq!(conflicts.len(), 1);
        let c = &conflicts[0];
        assert_eq!(c.intents.len(), 2);
        assert_eq!(c.resolution, "force-order");
        assert!(c.intents.contains(&a.id) && c.intents.contains(&b.id));
    }

    #[test]
    fn no_conflict_different_action() {
        let a = make_intent("i1", "hosts", "switch");
        let b = make_intent("i2", "hosts", "list");
        let conflicts = detect_conflicts(&[a, b]);
        assert!(conflicts.is_empty());
    }
}
