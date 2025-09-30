use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

pub type IntentId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ParsedIntent {
    pub id: IntentId,
    pub action_name: String,
    pub target_app_id: Option<String>,
    #[serde(default)]
    pub params: serde_json::Value,
    pub confidence: f32,
    pub source_start: u32,
    pub source_end: u32,
    pub explicit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConflictDetection {
    pub conflict_key: String,
    pub intents: Vec<IntentId>,
    pub reason: String,
    pub resolution: String, // force-order | user-select | drop-conflicting
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionPlanBatch {
    pub batch_id: String,
    pub intents: SmallVec<[ParsedIntent; 4]>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionPlan {
    pub plan_id: String,
    pub original_input: String,
    pub intents: Vec<ParsedIntent>,
    pub deduplicated: Vec<ParsedIntent>,
    pub batches: Vec<ExecutionPlanBatch>,
    pub conflicts: Vec<ConflictDetection>,
    pub strategy: String,
    pub generated_at: u64,
    pub dry_run: bool,
    pub explain: Option<ExplainPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplainPayload {
    pub tokens: Vec<String>,
    pub matched_rules: Vec<MatchedRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchedRule {
    pub rule_id: String,
    pub weight: f32,
    pub intent_id: Option<IntentId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionResult {
    pub intent_id: IntentId,
    pub status: String, // success | failed | skipped | timeout | simulated
    pub reason: Option<String>,
    pub retry_hint: Option<String>,
    pub predicted_effects: Option<Vec<String>>,
    pub duration_ms: Option<u32>,
    pub started_at: u64,
    pub finished_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandHistoryRecord {
    pub signature: String,
    pub input: String,
    pub intents_summary: Vec<String>,
    pub overall_status: String,
    pub created_at: u64,
    pub plan_size: u32,
    pub explain_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DescriptorLoadIssue {
    pub app_id: Option<String>,
    pub level: String, // PARSE_ERROR | SCHEMA_ERROR | SEMANTIC_ERROR
    pub message: String,
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn roundtrip_parsed_intent() {
        let intent = ParsedIntent {
            id: "i1".into(),
            action_name: "switch".into(),
            target_app_id: Some("hosts".into()),
            params: json!({"group":"dev"}),
            confidence: 0.92,
            source_start: 0,
            source_end: 5,
            explicit: true,
        };
        let s = serde_json::to_string(&intent).unwrap();
        let back: ParsedIntent = serde_json::from_str(&s).unwrap();
        assert_eq!(intent, back);
    }

    #[test]
    fn roundtrip_execution_plan() {
        let intent = ParsedIntent {
            id: "i1".into(),
            action_name: "switch".into(),
            target_app_id: Some("hosts".into()),
            params: Value::Null,
            confidence: 0.8,
            source_start: 0,
            source_end: 3,
            explicit: false,
        };
        let plan = ExecutionPlan {
            plan_id: "p1".into(),
            original_input: "switch hosts".into(),
            intents: vec![intent.clone()],
            deduplicated: vec![intent.clone()],
            batches: vec![],
            conflicts: vec![],
            strategy: "sequential".into(),
            generated_at: 0,
            dry_run: true,
            explain: None,
        };
        let s = serde_json::to_string(&plan).unwrap();
        let back: ExecutionPlan = serde_json::from_str(&s).unwrap();
        assert_eq!(plan, back);
    }
}
