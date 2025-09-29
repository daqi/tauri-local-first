//! Descriptor Registry
//!
//! 职责：
//! - 从给定目录列表扫描可能的 `tlfsuite.json` 描述文件
//! - 使用 `descriptor` crate 解析与校验
//! - 处理重复 appId（忽略后出现的）
//! - 收集扫描问题（issues）用于上层日志
//! - 返回内存索引：按 appId 与 actions 查询
//!
//! 非职责：
//! - 实时文件监控（后续 watch 模块扩展）
//! - IPC 调用或执行逻辑
//!
//! 路径策略（按优先级顺序尝试）：
//! 1. <dir>/tlfsuite.json
//! 2. <dir>/Contents/Resources/tlfsuite.json (macOS bundle)
//! 3. <dir>/resources/tlfsuite.json
//! 4. <dir>/share/tlfsuite/tlfsuite.json

use descriptor::{parse_descriptor_reader, AppDescriptor, DescriptorError};
use serde::Serialize;
use std::{collections::HashMap, fs::File, path::{Path, PathBuf}};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Registry {
    pub apps: HashMap<String, AppRecord>,
    pub issues: Vec<ScanIssue>,
}

impl Registry {
    /// Get app record by id.
    pub fn get(&self, id: &str) -> Option<&AppRecord> { self.apps.get(id) }
}

#[cfg(test)]
impl Registry {
    pub fn from_map_for_test(apps: std::collections::BTreeMap<String, AppRecord>) -> Self {
        let apps = apps.into_iter().collect();
        Self { apps, issues: vec![] }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppRecord {
    pub descriptor: AppDescriptor,
    pub origin_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScanIssue {
    pub code: IssueCode,
    pub path: PathBuf,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IssueCode {
    ParseError,
    IoError,
    UnsupportedDescriptorVersion,
    InvalidField,
    DuplicateAppId,
    DuplicateAction,
    DuplicateArg,
    NotFound, // no descriptor in dir
}

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("no directories provided")]
    EmptyInput,
}

/// Scan a list of application root directories building a registry.
pub fn scan_directories<I, P>(dirs: I) -> Result<Registry, ScanError>
where
    I: IntoIterator<Item = P>,
    P: Into<PathBuf>,
{
    let mut apps = HashMap::new();
    let mut issues = Vec::new();
    let mut any_dir = false;

    for dir in dirs {
        any_dir = true;
        let dir: PathBuf = dir.into();
        let (maybe_path, mut local_issues) = locate_descriptor(&dir);
        issues.append(&mut local_issues);
        let path = match maybe_path { Some(p) => p, None => continue };
        match load_descriptor(&path) {
            Ok(desc) => {
                let id = desc.id.clone();
                if apps.contains_key(&id) {
                    issues.push(ScanIssue { code: IssueCode::DuplicateAppId, path: path.clone(), detail: format!("duplicate app id: {id}") });
                } else {
                    apps.insert(id, AppRecord { descriptor: desc, origin_path: path });
                }
            }
            Err((code, detail)) => {
                issues.push(ScanIssue { code, path: path.clone(), detail });
            }
        }
    }

    if !any_dir { return Err(ScanError::EmptyInput); }

    Ok(Registry { apps, issues })
}

fn locate_descriptor(dir: &Path) -> (Option<PathBuf>, Vec<ScanIssue>) {
    let mut issues = Vec::new();
    let candidates = [
        dir.join("tlfsuite.json"),
        dir.join("Contents/Resources/tlfsuite.json"),
        dir.join("resources/tlfsuite.json"),
        dir.join("share/tlfsuite/tlfsuite.json"),
    ];
    for c in candidates.iter() {
        if c.exists() { return (Some(c.clone()), issues) }
    }
    issues.push(ScanIssue { code: IssueCode::NotFound, path: dir.to_path_buf(), detail: "no descriptor found".into() });
    (None, issues)
}

fn load_descriptor(path: &Path) -> Result<AppDescriptor, (IssueCode, String)> {
    let file = File::open(path).map_err(|e| (IssueCode::IoError, e.to_string()))?;
    match parse_descriptor_reader(file) {
        Ok(desc) => Ok(desc),
        Err(e) => Err(map_descriptor_error(e)),
    }
}

fn map_descriptor_error(e: DescriptorError) -> (IssueCode, String) {
    use DescriptorError::*;
    match e {
        Json(err) => (IssueCode::ParseError, err.to_string()),
        Io(err) => (IssueCode::IoError, err.to_string()),
        UnsupportedMajor(m) => (IssueCode::UnsupportedDescriptorVersion, format!("unsupported major version {m}")),
        InvalidField { field, msg } => (IssueCode::InvalidField, format!("{field}: {msg}")),
        DuplicateAction(name) => (IssueCode::DuplicateAction, format!("{name}")),
        DuplicateArg { action, arg } => (IssueCode::DuplicateArg, format!("{action}:{arg}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io::Write};
    use tempfile::tempdir;

    fn write_descriptor(dir: &Path, rel: &str, content: &str) -> PathBuf {
        let path = dir.join(rel);
        if let Some(p) = path.parent() { fs::create_dir_all(p).unwrap(); }
        let mut f = File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    fn minimal(id: &str) -> String {
        format!("{{\n  \"id\":\"{id}\",\n  \"name\":\"N\",\n  \"description\":\"D\",\n  \"version\":\"1.0.0\",\n  \"actions\":[]\n}}")
    }

    #[test]
    fn scan_success_single() {
        let tmp = tempdir().unwrap();
        write_descriptor(tmp.path(), "tlfsuite.json", &minimal("a"));
        let reg = scan_directories([tmp.path()]).unwrap();
        assert_eq!(reg.apps.len(), 1);
        assert!(reg.issues.iter().any(|i| matches!(i.code, IssueCode::NotFound)) == false);
    }

    #[test]
    fn scan_duplicate() {
        let t1 = tempdir().unwrap();
        let t2 = tempdir().unwrap();
        write_descriptor(t1.path(), "tlfsuite.json", &minimal("dup"));
        write_descriptor(t2.path(), "tlfsuite.json", &minimal("dup"));
        let reg = scan_directories([t1.path(), t2.path()]).unwrap();
        assert_eq!(reg.apps.len(), 1);
        assert!(reg.issues.iter().any(|i| matches!(i.code, IssueCode::DuplicateAppId)));
    }

    #[test]
    fn scan_parse_error() {
        let tmp = tempdir().unwrap();
        write_descriptor(tmp.path(), "tlfsuite.json", "{ broken");
        let reg = scan_directories([tmp.path()]).unwrap();
        assert_eq!(reg.apps.len(), 0);
        assert!(reg.issues.iter().any(|i| matches!(i.code, IssueCode::ParseError)));
    }

    #[test]
    fn scan_unsupported_version() {
        let tmp = tempdir().unwrap();
        write_descriptor(tmp.path(), "tlfsuite.json", &minimal("x").replace("1.0.0", "2.0.0"));
        let reg = scan_directories([tmp.path()]).unwrap();
        assert_eq!(reg.apps.len(), 0);
        assert!(reg.issues.iter().any(|i| matches!(i.code, IssueCode::UnsupportedDescriptorVersion)));
    }

    #[test]
    fn scan_not_found() {
        let t1 = tempdir().unwrap();
        let reg = scan_directories([t1.path()]).unwrap();
        assert_eq!(reg.apps.len(), 0);
        assert!(reg.issues.iter().any(|i| matches!(i.code, IssueCode::NotFound)));
    }
}
