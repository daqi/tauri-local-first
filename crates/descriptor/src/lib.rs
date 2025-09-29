//! TLFSuite Descriptor parsing & validation library.
//!
//! 提供：
//! - 结构类型 `AppDescriptor`, `Action`, `ActionArg`。
//! - 解析函数 `parse_descriptor_str` / `parse_descriptor_reader`。
//! - 结构化校验包含：必填字段、字段模式、语义化版本主版本支持、动作与参数合法性。
//! - 错误分类通过 `DescriptorError`。
//!
//! 设计：聚焦“概念正确性”，不做 IO 路径/监控；扫描与缓存应由上层负责。

use semver::Version;
use serde::Deserialize;
use std::io::Read;
use thiserror::Error;

/// 当前支持的 descriptor 主版本集合（可后续扩展）。
const SUPPORTED_MAJOR_VERSIONS: &[u64] = &[1];

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub scheme: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub actions: Vec<Action>,
    // 未来：categories, permissions, engines, tags ...
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Action {
    pub name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub args: Vec<ActionArg>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct ActionArg {
    pub name: String,
    #[serde(rename = "type")]
    pub arg_type: ArgType,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ArgType {
    String,
    Number,
    Boolean,
}

#[derive(Debug, Error)]
pub enum DescriptorError {
    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unsupported descriptor major version: {0}")]
    UnsupportedMajor(u64),
    #[error("invalid field: {field} - {msg}")]
    InvalidField { field: String, msg: String },
    #[error("duplicate action name: {0}")]
    DuplicateAction(String),
    #[error("duplicate argument name in action '{action}': {arg}")]
    DuplicateArg { action: String, arg: String },
}

pub type Result<T> = std::result::Result<T, DescriptorError>;

/// Parse descriptor from a &str and run validation.
pub fn parse_descriptor_str(s: &str) -> Result<AppDescriptor> {
    let raw: AppDescriptor = serde_json::from_str(s)?;
    validate(&raw)?;
    Ok(raw)
}

/// Parse descriptor from any reader implementing `Read` then validate.
pub fn parse_descriptor_reader<R: Read>(mut reader: R) -> Result<AppDescriptor> {
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    parse_descriptor_str(&buf)
}

/// Validate an already parsed descriptor.
pub fn validate(desc: &AppDescriptor) -> Result<()> {
    // id pattern
    if !regex_is("^[a-z0-9-_]{1,32}$", &desc.id) {
        return Err(DescriptorError::InvalidField {
            field: "id".into(),
            msg: "must match ^[a-z0-9-_]{1,32}$".into(),
        });
    }
    if desc.name.is_empty() {
        return Err(DescriptorError::InvalidField {
            field: "name".into(),
            msg: "cannot be empty".into(),
        });
    }
    if desc.description.is_empty() {
        return Err(DescriptorError::InvalidField {
            field: "description".into(),
            msg: "cannot be empty".into(),
        });
    }
    // version semver
    let ver = Version::parse(&desc.version).map_err(|_| DescriptorError::InvalidField {
        field: "version".into(),
        msg: "not valid semver".into(),
    })?;
    if !SUPPORTED_MAJOR_VERSIONS.contains(&ver.major) {
        return Err(DescriptorError::UnsupportedMajor(ver.major));
    }
    if let Some(scheme) = &desc.scheme {
        if !regex_is("^[a-z][a-z0-9+.-]*$", scheme) {
            return Err(DescriptorError::InvalidField {
                field: "scheme".into(),
                msg: "invalid scheme pattern".into(),
            });
        }
    }
    // actions unique & validate each
    let mut seen_actions = std::collections::HashSet::new();
    for action in &desc.actions {
        if !seen_actions.insert(&action.name) {
            return Err(DescriptorError::DuplicateAction(action.name.clone()));
        }
        validate_action(action)?;
    }
    Ok(())
}

fn validate_action(action: &Action) -> Result<()> {
    if action.name.is_empty() {
        return Err(DescriptorError::InvalidField {
            field: format!("action:{}:name", action.name),
            msg: "cannot be empty".into(),
        });
    }
    if !regex_is("^[a-z0-9-_]+$", &action.name) {
        return Err(DescriptorError::InvalidField {
            field: format!("action:{}:name", action.name),
            msg: "invalid pattern".into(),
        });
    }
    let mut seen_args = std::collections::HashSet::new();
    for arg in &action.args {
        if !seen_args.insert(&arg.name) {
            return Err(DescriptorError::DuplicateArg {
                action: action.name.clone(),
                arg: arg.name.clone(),
            });
        }
        if arg.name.is_empty() {
            return Err(DescriptorError::InvalidField {
                field: format!("action:{}:arg:name", action.name),
                msg: "cannot be empty".into(),
            });
        }
        if !regex_is("^[a-zA-Z0-9_]+$", &arg.name) {
            return Err(DescriptorError::InvalidField {
                field: format!("action:{}:arg:name", action.name),
                msg: "invalid pattern".into(),
            });
        }
        // ArgType enum ensures type validity
        let _ = &arg.arg_type;
    }
    Ok(())
}

fn regex_is(pattern: &str, text: &str) -> bool {
    // simple cached regex might be added later; using once_cell avoided for minimal deps now.
    let re = regex::Regex::new(pattern).expect("hardcoded pattern invalid");
    re.is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_minimal_ok() {
        let json = r#"{
            "id":"clipboard",
            "name":"Clipboard Manager",
            "description":"Clipboard history",
            "version":"1.0.0",
            "actions":[]
        }"#;
        let d = parse_descriptor_str(json).expect("should parse");
        assert_eq!(d.id, "clipboard");
    }

    #[test]
    fn reject_bad_id() {
        let json = r#"{
            "id":"Bad Id",
            "name":"X","description":"D","version":"1.0.0","actions":[]
        }"#;
        let err = parse_descriptor_str(json).unwrap_err();
        match err {
            DescriptorError::InvalidField { field, .. } => assert_eq!(field, "id"),
            _ => panic!("wrong error"),
        }
    }

    #[test]
    fn reject_unsupported_major() {
        let json = r#"{
            "id":"x","name":"X","description":"D","version":"2.0.0","actions":[]
        }"#;
        let err = parse_descriptor_str(json).unwrap_err();
        matches!(err, DescriptorError::UnsupportedMajor(2));
    }

    #[test]
    fn duplicate_action() {
        let json = r#"{
            "id":"x","name":"X","description":"D","version":"1.0.0",
            "actions":[{"name":"a"},{"name":"a"}]
        }"#;
        let err = parse_descriptor_str(json).unwrap_err();
        matches!(err, DescriptorError::DuplicateAction(_));
    }

    #[test]
    fn duplicate_arg() {
        let json = r#"{
            "id":"x","name":"X","description":"D","version":"1.0.0",
            "actions":[{"name":"a","args":[{"name":"p","type":"string"},{"name":"p","type":"number"}]}]
        }"#;
        let err = parse_descriptor_str(json).unwrap_err();
        matches!(err, DescriptorError::DuplicateArg { .. });
    }

    #[test]
    fn parse_reader_ok() {
        let json = r#"{"id":"x","name":"X","description":"D","version":"1.0.0","actions":[]}"#;
        let cur = Cursor::new(json);
        let d = parse_descriptor_reader(cur).expect("reader parse ok");
        assert_eq!(d.id, "x");
    }

    #[test]
    fn parse_reader_io_error() {
        struct Faulty;
        impl Read for Faulty { fn read(&mut self, _b: &mut [u8]) -> std::io::Result<usize> { Err(std::io::Error::new(std::io::ErrorKind::Other, "boom")) } }
        let err = parse_descriptor_reader(Faulty).unwrap_err();
        matches!(err, DescriptorError::Io(_));
    }
}
