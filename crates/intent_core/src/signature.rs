use crate::ParsedIntent;
use blake3::Hasher;

pub fn normalize_signature(intents: &[ParsedIntent]) -> String {
    // Normalize by sorting on (action_name, target_app_id, params canonical string)
    let mut parts: Vec<String> = intents
        .iter()
        .map(|i| {
            let mut base = format!(
                "{}|{}|{}",
                i.action_name,
                i.target_app_id.as_deref().unwrap_or("-"),
                canonical_params(&i.params)
            );
            // explicit flag & confidence not part of semantics
            if i.explicit { base.push_str("|E"); }
            base
        })
        .collect();
    parts.sort();
    let mut hasher = Hasher::new();
    for p in parts {
        hasher.update(p.as_bytes());
        hasher.update(&[0]);
    }
    hasher.finalize().to_hex().to_string()
}

fn canonical_params(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Object(map) => {
            let mut items: Vec<(String, String)> = map
                .iter()
                .map(|(k, val)| (k.clone(), canonical_params(val)))
                .collect();
            items.sort_by(|a, b| a.0.cmp(&b.0));
            let inner: Vec<String> = items
                .into_iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            inner.join(&";".to_string())
        }
        serde_json::Value::Array(arr) => {
            let mapped: Vec<String> = arr.iter().map(canonical_params).collect();
            format!("[{}]", mapped.join(","))
        }
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Null => "null".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::ParsedIntent;

    fn intent(action: &str, params: serde_json::Value) -> ParsedIntent {
        ParsedIntent {
            id: uuid::Uuid::new_v4().to_string(),
            action_name: action.to_string(),
            target_app_id: Some("app".into()),
            params,
            confidence: 1.0,
            source_start: 0,
            source_end: 1,
            explicit: true,
        }
    }

    #[test]
    fn order_invariance() {
        let a = intent("switch", json!({"group":"dev","mode":"fast"}));
        let b = intent("open", json!({"view":"history"}));
        let sig1 = normalize_signature(&[a.clone(), b.clone()]);
        let sig2 = normalize_signature(&[b, a]);
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn param_object_order_invariance() {
        let a1 = intent("switch", json!({"a":1,"b":2}));
        let a2 = intent("switch", json!({"b":2,"a":1}));
        let s1 = normalize_signature(&[a1]);
        let s2 = normalize_signature(&[a2]);
        assert_eq!(s1, s2);
    }
}
