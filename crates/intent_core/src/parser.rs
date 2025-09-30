use crate::{ParsedIntent, ExplainPayload, MatchedRule};
use regex::Regex;
use serde_json::json;
use std::collections::HashMap;

/// Parsing options (future extension)
#[derive(Debug, Default, Clone)]
pub struct ParseOptions {
    pub enable_explain: bool,
}

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub intents: Vec<ParsedIntent>,
    pub explain: Option<ExplainPayload>,
}

pub trait IntentParser: Send + Sync {
    fn parse(&self, input: &str, opts: &ParseOptions) -> ParseResult;
}

/// A very small rule-based parser:
/// - Explicit syntax: app:action(args?) e.g. hosts:switch(dev)
/// - Keyword mapping: if input contains certain keyword map to an action
pub struct RuleBasedParser {
    keyword_map: HashMap<&'static str, (&'static str, &'static str)>, // keyword -> (app, action)
    explicit_re: Regex,
}

impl RuleBasedParser {
    pub fn new() -> Self {
        let mut keyword_map = HashMap::new();
        keyword_map.insert("hosts", ("hosts", "switch"));
        keyword_map.insert("剪贴板", ("clipboard", "openHistory"));
        Self {
            keyword_map,
            explicit_re: Regex::new(
                r"(?P<app>[a-zA-Z0-9_]+):(?P<action>[a-zA-Z0-9_]+)\((?P<args>[^)]*)\)",
            )
            .unwrap(),
        }
    }
}

impl IntentParser for RuleBasedParser {
    fn parse(&self, input: &str, opts: &ParseOptions) -> ParseResult {
        let mut intents = Vec::new();
        let mut explain_tokens: Vec<String> = Vec::new();
        let mut matched_rules: Vec<MatchedRule> = Vec::new();
        let mut covered_apps: Vec<String> = Vec::new();

        // explicit matches first
        for caps in self.explicit_re.captures_iter(input) {
            let app = caps.name("app").unwrap().as_str();
            let action = caps.name("action").unwrap().as_str();
            let args = caps.name("args").map(|m| m.as_str()).unwrap_or("");
            let params = if args.is_empty() {
                json!({})
            } else {
                json!({ "arg": args })
            };
            intents.push(ParsedIntent {
                id: uuid::Uuid::new_v4().to_string(),
                action_name: action.to_string(),
                target_app_id: Some(app.to_string()),
                params,
                confidence: 1.0,
                source_start: 0,
                source_end: input.len() as u32,
                explicit: true,
            });
            if opts.enable_explain {
                explain_tokens.push(format!("explicit:{}:{}", app, action));
                matched_rules.push(MatchedRule { rule_id: format!("explicit:{}:{}", app, action), weight: 1.0, intent_id: intents.last().map(|i| i.id.clone()) });
            }
            covered_apps.push(app.to_string());
        }

        // keyword scanning (very naive)
        for (kw, (app, action)) in &self.keyword_map {
            if input.contains(kw) && !covered_apps.iter().any(|a| a == app) {
                intents.push(ParsedIntent {
                    id: uuid::Uuid::new_v4().to_string(),
                    action_name: (*action).to_string(),
                    target_app_id: Some((*app).to_string()),
                    params: json!({}),
                    confidence: 0.75,
                    source_start: 0,
                    source_end: input.len() as u32,
                    explicit: false,
                });
                if opts.enable_explain {
                    explain_tokens.push(format!("kw:{}->{}:{}", kw, app, action));
                    matched_rules.push(MatchedRule { rule_id: format!("kw:{}", kw), weight: 0.75, intent_id: intents.last().map(|i| i.id.clone()) });
                }
            }
        }
        let explain = if opts.enable_explain {
            Some(ExplainPayload { tokens: explain_tokens, matched_rules })
        } else { None };
        ParseResult { intents, explain }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_parsing() {
        let p = RuleBasedParser::new();
        let r = p.parse("hosts:switch(dev)", &ParseOptions::default());
        assert_eq!(r.intents.len(), 1);
        assert!(r.intents[0].explicit);
        assert_eq!(r.intents[0].action_name, "switch");
        assert_eq!(r.intents[0].target_app_id.as_deref(), Some("hosts"));
        assert!(r.explain.is_none());
    }

    #[test]
    fn mixed_keywords_and_explicit() {
        let p = RuleBasedParser::new();
        let input = "启用开发 hosts:switch(dev) 并查看剪贴板";
        let r = p.parse(input, &ParseOptions::default());
        // one explicit + 2 keyword (hosts, 剪贴板)
        assert!(r.intents.len() >= 2); // depending on keywords
        assert!(r.intents.iter().any(|i| i.explicit));
        assert!(r.explain.is_none());
    }

    #[test]
    fn explain_mode_enabled() {
        let p = RuleBasedParser::new();
        let input = "查看剪贴板 hosts:switch(dev)";
        let r = p.parse(input, &ParseOptions { enable_explain: true });
        assert!(r.explain.is_some());
        let e = r.explain.unwrap();
        assert!(!e.tokens.is_empty());
        assert!(!e.matched_rules.is_empty());
        // Each matched rule should have a rule_id
        assert!(e.matched_rules.iter().all(|mr| !mr.rule_id.is_empty()));
    }
}
