use crate::{DescriptorLoadIssue};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionDescriptor {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub requires_elevation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApplicationDescriptor {
    pub app_id: String,
    #[serde(default)]
    pub actions: Vec<ActionDescriptor>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScanResult {
    pub descriptors: Vec<ApplicationDescriptor>,
    pub issues: Vec<DescriptorLoadIssue>,
}

pub fn scan<P: AsRef<Path>>(roots: &[P]) -> ScanResult {
    let mut descriptors = Vec::new();
    let mut issues = Vec::new();
    for root in roots {
        let root_path = root.as_ref();
        if !root_path.exists() { continue; }
        // walk only one level for simplicity
        let entries = match fs::read_dir(root_path) {
            Ok(e) => e,
            Err(e) => {
                issues.push(DescriptorLoadIssue { app_id: None, level: "PARSE_ERROR".into(), message: format!("read_dir error: {e}"), path: root_path.display().to_string() });
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let file = path.join("tlfsuite.json");
                if file.exists() {
                    match load_descriptor(&file) {
                        Ok((desc, mut desc_issues)) => {
                            if let Some(d) = desc { descriptors.push(d); }
                            issues.append(&mut desc_issues);
                        }
                        Err(e) => issues.push(DescriptorLoadIssue { app_id: None, level: "PARSE_ERROR".into(), message: e.to_string(), path: file.display().to_string() }),
                    }
                }
            }
        }
    }
    ScanResult { descriptors, issues }
}

fn load_descriptor(path: &Path) -> Result<(Option<ApplicationDescriptor>, Vec<DescriptorLoadIssue>), anyhow::Error> {
    let text = fs::read_to_string(path)?;
    let mut issues = Vec::new();
    let value: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            return Ok((None, vec![DescriptorLoadIssue { app_id: None, level: "PARSE_ERROR".into(), message: e.to_string(), path: path.display().to_string() }]));
        }
    };
    // schema validation minimal
    let app_id = match value.get("app_id").and_then(|v| v.as_str()) {
        Some(id) if !id.is_empty() => id.to_string(),
        _ => {
            issues.push(DescriptorLoadIssue { app_id: None, level: "SCHEMA_ERROR".into(), message: "missing app_id".into(), path: path.display().to_string() });
            return Ok((None, issues));
        }
    };
    let actions_val = value.get("actions").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let mut actions = Vec::new();
    let mut names = std::collections::HashSet::new();
    for a in actions_val {
        let name = match a.get("name").and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => {
                issues.push(DescriptorLoadIssue { app_id: Some(app_id.clone()), level: "SCHEMA_ERROR".into(), message: "action missing name".into(), path: path.display().to_string() });
                continue;
            }
        };
        if !names.insert(name.clone()) {
            issues.push(DescriptorLoadIssue { app_id: Some(app_id.clone()), level: "SEMANTIC_ERROR".into(), message: format!("duplicate action name: {name}"), path: path.display().to_string() });
        }
        let requires_elevation = a.get("requires_elevation").and_then(|v| v.as_bool()).unwrap_or(false);
        actions.push(ActionDescriptor { name, description: None, requires_elevation });
    }
    let desc = ApplicationDescriptor { app_id: app_id.clone(), actions };
    Ok((Some(desc), issues))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    fn write_file(p: &Path, content: &str) {
        fs::write(p, content).unwrap();
    }

    #[test]
    fn parse_error() {
        let dir = tempdir().unwrap();
        let app_dir = dir.path().join("app1");
        fs::create_dir(&app_dir).unwrap();
        let file = app_dir.join("tlfsuite.json");
        write_file(&file, "{invalid-json}");
        let res = scan(&[dir.path()]);
        assert!(res.descriptors.is_empty());
        assert!(res.issues.iter().any(|i| i.level == "PARSE_ERROR"));
    }

    #[test]
    fn schema_error_missing_app_id() {
        let dir = tempdir().unwrap();
        let app_dir = dir.path().join("app2");
        fs::create_dir(&app_dir).unwrap();
        let file = app_dir.join("tlfsuite.json");
        write_file(&file, r#"{"actions": []}"#);
        let res = scan(&[dir.path()]);
        assert!(res.descriptors.is_empty());
        assert!(res.issues.iter().any(|i| i.level == "SCHEMA_ERROR"));
    }

    #[test]
    fn semantic_duplicate_action() {
        let dir = tempdir().unwrap();
        let app_dir = dir.path().join("app3");
        fs::create_dir(&app_dir).unwrap();
        let file = app_dir.join("tlfsuite.json");
        write_file(&file, r#"{"app_id":"x","actions":[{"name":"a"},{"name":"a"}]}"#);
        let res = scan(&[dir.path()]);
        assert_eq!(res.descriptors.len(), 1);
        assert!(res.issues.iter().any(|i| i.level == "SEMANTIC_ERROR"));
    }

    #[test]
    fn ok_descriptor() {
        let dir = tempdir().unwrap();
        let app_dir = dir.path().join("app4");
        fs::create_dir(&app_dir).unwrap();
        let file = app_dir.join("tlfsuite.json");
        write_file(&file, r#"{"app_id":"ok","actions":[{"name":"run","requires_elevation":true}]}"#);
        let res = scan(&[dir.path()]);
        assert_eq!(res.descriptors.len(), 1);
        assert!(res.issues.is_empty());
        assert_eq!(res.descriptors[0].app_id, "ok");
        assert_eq!(res.descriptors[0].actions[0].requires_elevation, true);
    }
}
