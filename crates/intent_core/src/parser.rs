use crate::ParsedIntent;
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
    pub explain_tokens: Vec<String>,
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
    fn parse(&self, input: &str, _opts: &ParseOptions) -> ParseResult {
        let mut intents = Vec::new();
        let mut explain_tokens = Vec::new();
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
            explain_tokens.push(format!("explicit:{}:{}", app, action));
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
                explain_tokens.push(format!("kw:{}->{}:{}", kw, app, action));
            }
        }

        ParseResult {
            intents,
            explain_tokens,
        }
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
    }

    #[test]
    fn mixed_keywords_and_explicit() {
        let p = RuleBasedParser::new();
        let input = "启用开发 hosts:switch(dev) 并查看剪贴板";
        let r = p.parse(input, &ParseOptions::default());
        // one explicit + 2 keyword (hosts, 剪贴板)
        assert!(r.intents.len() >= 2); // depending on keywords
        assert!(r.intents.iter().any(|i| i.explicit));
    }
}
