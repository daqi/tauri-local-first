use crate::adapters;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn load_app_descriptor(app_name: &str) -> Option<AppDescriptor> {
    for apps_dir in find_apps_dirs() {
        if let Ok(entries) = fs::read_dir(&apps_dir) {
            for ent in entries.flatten() {
                if !ent.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let Some(desc_file) = find_descriptor_file(&ent.path()) else {
                    continue;
                };
                if let Ok(txt) = fs::read_to_string(&desc_file) {
                    if let Ok(desc) = serde_json::from_str::<AppDescriptor>(&txt) {
                        if desc.id.eq_ignore_ascii_case(app_name) {
                            return Some(desc);
                        }
                    }
                }
            }
        }
    }
    None
}

fn build_deeplink_url(app_name: &str, args: Option<&str>) -> String {
    // 优先使用 tlfsuite.json 中声明
    if let Some(desc) = load_app_descriptor(app_name) {
        let app_id = desc.id;
        if let Some(scheme) = desc.scheme {
            if let Some(a) = args {
                let encoded = urlencoding::encode(a);
                return format!("{}://open?args={}", scheme, encoded);
            }
            return format!("{}://open", scheme);
        }
        // 无自定义 scheme 时回退统一入口
        if let Some(a) = args {
            let encoded = urlencoding::encode(a);
            return format!("tlfsuite://open?app={}&args={}", app_id, encoded);
        }
        return format!("tlfsuite://open?app={}", app_id);
    }
    if let Some(a) = args {
        let encoded = urlencoding::encode(a);
        return format!("tlfsuite://open?app={}&args={}", app_name, encoded);
    }
    format!("tlfsuite://open?app={}", app_name)
}

#[tauri::command]
pub fn open_with_args(app_name: String, args: Option<String>) -> Result<(), String> {
    let url = build_deeplink_url(&app_name, args.as_deref());

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ActionArgSpec {
    pub name: String,
    #[serde(rename = "type")]
    pub arg_type: Option<String>,
    #[serde(default)]
    pub required: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppAction {
    pub name: String,
    pub title: Option<String>,
    #[serde(default)]
    pub args: Vec<ActionArgSpec>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppDescriptor {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub scheme: Option<String>,
    #[serde(default)]
    pub actions: Vec<AppAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

fn find_apps_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    // 2) Workspace dev: from cwd walk up to find apps directory
    if let Ok(mut cur) = std::env::current_dir() {
        for _ in 0..4 {
            let candidate = cur.join("apps");
            if candidate.is_dir() {
                dirs.push(candidate);
                break;
            }
            if !cur.pop() {
                break;
            }
        }
    }

    // 3) Platform defaults via adapters
    dirs.extend(adapters::scan_installed_app_roots());

    // dedup
    dirs.sort();
    dirs.dedup();
    dirs
}

// 在一个候选目录下，尝试定位 tlfsuite.json，兼容常见打包位置
fn find_descriptor_file(entry_dir: &Path) -> Option<PathBuf> {
    if !entry_dir.is_dir() {
        return None;
    }
    let candidates = [
        entry_dir.join("tlfsuite.json"),
        entry_dir.join("Contents/Resources/tlfsuite.json"), // macOS .app bundle
        entry_dir.join("resources/tlfsuite.json"),          // 常见打包器
        entry_dir.join("share/tlfsuite/tlfsuite.json"),     // Linux 惯例
    ];
    for p in candidates {
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn mime_from_ext(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        _ => "application/octet-stream",
    }
}

fn file_to_data_url(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }
    let mime = mime_from_ext(path);
    if let Ok(bytes) = fs::read(path) {
        let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
        return Some(format!("data:{};base64,{}", mime, b64));
    }
    None
}

// 组装常见图标候选（按项目结构与打包器惯例）
fn default_icon_candidates(desc_dir: &Path, entry_dir: &Path) -> Vec<PathBuf> {
    let v = vec![
        desc_dir.join("icon.png"),
        desc_dir.join("icons/icon.png"),
        desc_dir.join("src-tauri/icons/icon.png"),
        desc_dir.join("public/icon.png"),
        desc_dir.join("assets/icon.png"),
        entry_dir.join("Contents/Resources/icon.png"),
    ];
    v
}

// 名称兜底：优先基于 id 做人类可读化；否则用目录名
fn fallback_display_name(desc_id: &str, entry_dir: &Path) -> String {
    if !desc_id.trim().is_empty() {
        let replaced = desc_id.replace(['_', '-', '.'], " ");
        let mut out = String::with_capacity(replaced.len());
        for (i, word) in replaced.split_whitespace().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                out.push(first.to_ascii_uppercase());
            }
            for c in chars {
                out.push(c.to_ascii_lowercase());
            }
        }
        if !out.is_empty() {
            return out;
        }
    }
    entry_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("App")
        .to_string()
}

// 解析 name/icon：优先 desc（tlfsuite.json），缺失时按平台补全，再回退通用候选
fn resolve_icon_and_name(
    desc_dir: &Path,
    entry_dir: &Path,
    icon_hint: &Option<String>,
) -> (Option<String>, Option<String>) {
    let mut name_override: Option<String> = None;
    let mut icon_data: Option<String> = None;

    // 1) 先用 desc 内的 icon
    if let Some(icon) = icon_hint {
        if icon.starts_with("data:") {
            icon_data = Some(icon.clone());
        } else {
            // Linux: 若为名称，按 XDG icon theme 解析
            #[cfg(target_os = "linux")]
            if !icon.contains('/') {
                if let Some(p) = crate::adapters::resolve_icon_name(icon) {
                    icon_data = file_to_data_url(&p);
                }
            }
            // 其他或解析失败：当作路径
            if icon_data.is_none() {
                #[cfg(target_os = "windows")]
                let icon_ref: &str = icon
                    .split(',')
                    .next()
                    .unwrap_or(icon.as_str())
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'');
                #[cfg(not(target_os = "windows"))]
                let icon_ref: &str = icon.as_str();
                let p = if Path::new(icon_ref).is_absolute() {
                    PathBuf::from(icon_ref)
                } else {
                    desc_dir.join(icon_ref)
                };
                icon_data = file_to_data_url(&p);
            }
        }
    }

    // 2) 平台补全（只在 desc 缺失时触发）
    #[cfg(target_os = "macos")]
    if icon_data.is_none() || name_override.is_none() {
        if entry_dir.extension().and_then(|s| s.to_str()) == Some("app") {
            let (nm, ic, _, _) = crate::adapters::resolve_macos_name_and_icon(entry_dir);
            if name_override.is_none() {
                name_override = nm;
            }
            if icon_data.is_none() {
                icon_data = ic;
            }
        }
    }

    // 3) 通用兜底
    if icon_data.is_none() {
        for p in default_icon_candidates(desc_dir, entry_dir) {
            if let Some(data_url) = file_to_data_url(&p) {
                icon_data = Some(data_url);
                break;
            }
        }
    }

    (icon_data, name_override)
}

#[tauri::command]
pub fn list_apps() -> Result<Vec<AppDescriptor>, String> {
    let roots = find_apps_dirs();
    if roots.is_empty() {
        return Err("apps directory not found".to_string());
    }
    let mut out: Vec<AppDescriptor> = Vec::new();
    for apps_dir in roots {
        if let Ok(entries) = fs::read_dir(&apps_dir) {
            for ent in entries.flatten() {
                if !ent.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    continue;
                }
                let Some(desc_file) = find_descriptor_file(&ent.path()) else {
                    continue;
                };
                let desc_dir = desc_file
                    .parent()
                    .unwrap_or(ent.path().as_path())
                    .to_path_buf();
                if let Ok(txt) = fs::read_to_string(&desc_file) {
                    if let Ok(mut desc) = serde_json::from_str::<AppDescriptor>(&txt) {
                        desc.path = Some(desc_dir.to_string_lossy().to_string());
                        let (icon_data, name_override) =
                            resolve_icon_and_name(&desc_dir, &ent.path(), &desc.icon);
                        if desc.name.is_none() {
                            desc.name = name_override
                                .or_else(|| Some(fallback_display_name(&desc.id, &ent.path())));
                        }
                        desc.icon = icon_data;
                        out.push(desc);
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}
