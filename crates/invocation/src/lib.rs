//! Invocation core types & validation logic
//!
//! 目标：统一跨应用调用的数据结构与参数校验；不包含具体传输（IPC/Deep Link）。
//!
//! 提供：
//! - `InvocationRequest` / `InvocationResult`
//! - `InvocationStatus` / `InvocationError` / `ArgError`
//! - `validate_args(action, incoming)` 参数校验（缺参/类型错误聚合）
//! - 解析基础类型：string / number(f64) / boolean(true,false,1,0,on,off,yes,no)
//! - 耗时辅助：`time_exec(|| { ... })`

use descriptor::{Action, ActionArg, ArgType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvocationRequest {
    pub app_id: String,
    pub action: String,
    pub args: HashMap<String, String>,
    #[serde(default)]
    pub ts_ms: u128,
}

impl InvocationRequest {
    pub fn new(app_id: impl Into<String>, action: impl Into<String>, args: HashMap<String, String>) -> Self {
        Self { app_id: app_id.into(), action: action.into(), args, ts_ms: crate::now_ms() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum InvocationStatus { Success, Error, Processing }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvocationResult {
    pub status: InvocationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<InvocationError>,
    pub meta: InvocationMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvocationMeta {
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvocationError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub arg_errors: Vec<ArgError>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArgError {
    pub name: String,
    pub kind: ArgErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArgErrorKind { Missing, TypeMismatch }

#[derive(Debug, Error)]
pub enum ValidationFailure {
    #[error("missing or invalid arguments")] 
    Invalid(Vec<ArgError>),
}

/// Validate args according to Action definition.
pub fn validate_args(action: &Action, incoming: &HashMap<String, String>) -> Result<(), ValidationFailure> {
    let mut errors = Vec::new();
    // index expected args by name for quick look
    for def in &action.args {
        match incoming.get(&def.name) {
            None if def.required => errors.push(ArgError { name: def.name.clone(), kind: ArgErrorKind::Missing, message: "required".into() }),
            Some(v) => {
                // Policy: only type-validate if required OR value parses successfully; optional + invalid -> ignore silently
                if def.required {
                    if let Err(msg) = check_type(&def.arg_type, v) {
                        errors.push(ArgError { name: def.name.clone(), kind: ArgErrorKind::TypeMismatch, message: msg });
                    }
                } else {
                    // Optional: attempt parse but do not record error if fails
                    let _ = check_type(&def.arg_type, v);
                }
            }
            _ => {}
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(ValidationFailure::Invalid(errors)) }
}

fn check_type(t: &ArgType, raw: &str) -> Result<(), String> {
    match t {
        ArgType::String => Ok(()),
        ArgType::Number => raw.parse::<f64>().map(|_| ()).map_err(|_| "not a valid number".into()),
        ArgType::Boolean => parse_bool(raw).map(|_| ()).map_err(|_| "not a valid boolean".into()),
    }
}

fn parse_bool(v: &str) -> Result<bool, ()> {
    match v.to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => Err(()),
    }
}

/// Helper to measure execution duration in ms.
pub fn time_exec<F, R>(f: F) -> (R, u128)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let dur = start.elapsed().as_millis();
    (result, dur)
}

/// Utility for timestamp ms.
pub fn now_ms() -> u128 { 
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::from_millis(0)).as_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use descriptor::{ActionArg};

    fn action(args: Vec<(&str, ArgType, bool)>) -> Action {
        Action { 
            name: "test".into(),
            title: None,
            args: args.into_iter().map(|(n,t,r)| ActionArg { name: n.to_string(), arg_type: t, required: r }).collect()
        }
    }

    #[test]
    fn validate_ok_all_types() {
        let act = action(vec![
            ("s", ArgType::String, true),
            ("n", ArgType::Number, true),
            ("b", ArgType::Boolean, true)
        ]);
        let mut incoming = HashMap::new();
        incoming.insert("s".into(), "hello".into());
        incoming.insert("n".into(), "3.14".into());
        incoming.insert("b".into(), "yes".into());
        assert!(validate_args(&act, &incoming).is_ok());
    }

    #[test]
    fn validate_missing_and_type_errors() {
        let act = action(vec![
            ("s", ArgType::String, true),
            ("n", ArgType::Number, true),
            ("b", ArgType::Boolean, false)
        ]);
        let mut incoming = HashMap::new();
        incoming.insert("s".into(), "hello".into());
        incoming.insert("n".into(), "abc".into()); // invalid number
        incoming.insert("b".into(), "wat".into()); // invalid boolean (not required though)
        let err = validate_args(&act, &incoming).unwrap_err();
        match err { ValidationFailure::Invalid(list) => {
            assert_eq!(list.len(), 1); // only number type mismatch (optional invalid boolean ignored)
            assert_eq!(list[0].name, "n");
        }}
    }

    #[test]
    fn boolean_variants() {
        let act = action(vec![("b", ArgType::Boolean, true)]);
        for (raw, expect) in [("true",true),("1",true),("on",true),("yes",true),("false",false),("0",false),("off",false),("no",false)] {
            let mut m = HashMap::new(); m.insert("b".into(), raw.into());
            assert!(validate_args(&act, &m).is_ok(), "{raw} should parse");
            assert_eq!(parse_bool(raw).unwrap(), expect);
        }
    }

    #[test]
    fn time_exec_measures() {
        let (_res, d) = time_exec(|| { let mut s=0; for i in 0..1000 { s+=i; } s });
        assert!(d >= 0); // basic sanity
    }
}
