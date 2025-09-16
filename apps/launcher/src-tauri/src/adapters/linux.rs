use std::fs;
use std::path::{Path, PathBuf};

pub fn scan_linux_app_roots() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(home) = dirs_next::home_dir() {
        // Linux 用户级 desktop entries
        let user_apps = home.join(".local/share/applications");
        if user_apps.is_dir() {
            dirs.extend(scan_desktop_entries(&user_apps));
        }
    }
    let sys_apps = PathBuf::from("/usr/share/applications");
    if sys_apps.is_dir() {
        dirs.extend(scan_desktop_entries(&sys_apps));
    }
    dirs.retain(|p| p.is_dir());
    dirs
}

fn scan_desktop_entries(dir: &Path) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for ent in entries.flatten() {
            let p = ent.path();
            if p.extension().and_then(|s| s.to_str()) != Some("desktop") {
                continue;
            }
            if let Ok(txt) = fs::read_to_string(&p) {
                if let Some(exec_line) = txt.lines().find(|l| l.trim_start().starts_with("Exec=")) {
                    let cmd = exec_line.trim_start_matches("Exec=").trim();
                    if let Some(exec_path) = parse_exec_path(cmd) {
                        // 候选根：可执行文件所在目录的上级目录
                        if let Some(parent) = exec_path.parent() {
                            roots.push(parent.to_path_buf());
                            if let Some(parent2) = parent.parent() {
                                roots.push(parent2.to_path_buf());
                            }
                        }
                    }
                }
            }
        }
    }
    roots
}

fn parse_exec_path(cmd: &str) -> Option<PathBuf> {
    // 去掉参数/占位符，如 %U, %F 等
    let first = cmd.split_whitespace().next()?;
    let cleaned = first
        .replace("%U", "")
        .replace("%u", "")
        .replace("%F", "")
        .replace("%f", "");
    let path = Path::new(cleaned.trim());
    if path.is_absolute() && path.exists() {
        return fs::canonicalize(path).ok();
    }
    // 在 PATH 中查找
    if let Ok(path_env) = std::env::var("PATH") {
        for p in path_env.split(':') {
            let cand = Path::new(p).join(cleaned.trim());
            if cand.exists() {
                return fs::canonicalize(cand).ok();
            }
        }
    }
    None
}

// 解析 XDG 图标主题中的图标名称为具体文件路径
// 参考：https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html
pub fn resolve_icon_name(name: &str) -> Option<PathBuf> {
    // 如果已经是路径（包含分隔符或为绝对路径），不处理
    if name.contains('/') || Path::new(name).is_absolute() {
        let p = PathBuf::from(name);
        return p.exists().then_some(p);
    }

    // 若已经带扩展名，优先按该扩展查找
    let provided_ext = Path::new(name)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());

    let data_dirs = xdg_data_dirs();
    let themes = icon_theme_candidates();
    // 常见尺寸与上下文
    let sizes = [
        "512x512", "256x256", "192x192", "128x128", "96x96", "64x64", "48x48", "32x32", "24x24",
        "22x22", "16x16",
    ];
    let contexts = [
        "apps",
        "actions",
        "places",
        "mimetypes",
        "categories",
        "devices",
        "emblems",
        "status",
        "panel",
    ];
    let exts: Vec<&str> = match provided_ext.as_deref() {
        Some(ext) => vec![ext],
        None => vec!["png", "svg", "xpm"],
    };

    // 1) 在 icons/<theme> 下按优先顺序查找
    for dir in &data_dirs {
        for theme in &themes {
            let base = dir.join("icons").join(theme);
            // a) 固定尺寸下的上下文优先
            for size in &sizes {
                for ctx in &contexts {
                    for ext in &exts {
                        let cand = base.join(size).join(ctx).join(format!("{}.{ext}", name));
                        if cand.exists() {
                            return Some(cand);
                        }
                    }
                }
                // b) 固定尺寸但无上下文
                for ext in &exts {
                    let cand = base.join(size).join(format!("{}.{ext}", name));
                    if cand.exists() {
                        return Some(cand);
                    }
                }
            }
            // c) 可缩放
            for ctx in &contexts {
                let cand = base
                    .join("scalable")
                    .join(ctx)
                    .join(format!("{}.svg", name));
                if cand.exists() {
                    return Some(cand);
                }
            }
            let cand = base.join("scalable").join(format!("{}.svg", name));
            if cand.exists() {
                return Some(cand);
            }
        }
    }

    // 2) pixmaps 作为兜底
    for dir in &data_dirs {
        let pixmaps = dir.join("pixmaps");
        for ext in &exts {
            let cand = pixmaps.join(format!("{}.{ext}", name));
            if cand.exists() {
                return Some(cand);
            }
        }
    }
    for ext in &exts {
        let cand = PathBuf::from("/usr/share/pixmaps").join(format!("{}.{ext}", name));
        if cand.exists() {
            return Some(cand);
        }
    }

    // 3) 若名称里含有 '-'，尝试逐级截断（如 mime 类型风格 text-x-script → text-x）
    if name.contains('-') {
        let mut parts: Vec<&str> = name.split('-').collect();
        while parts.len() > 1 {
            parts.pop();
            let parent = parts.join("-");
            if let Some(p) = resolve_icon_name(&parent) {
                return Some(p);
            }
        }
    }

    None
}

fn xdg_data_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    // $XDG_DATA_HOME 或 ~/.local/share
    if let Ok(xdg_home) = std::env::var("XDG_DATA_HOME") {
        let p = PathBuf::from(xdg_home);
        if p.is_dir() {
            dirs.push(p);
        }
    } else if let Some(home) = dirs_next::home_dir() {
        let p = home.join(".local/share");
        if p.is_dir() {
            dirs.push(p);
        }
    }

    // $XDG_DATA_DIRS 或默认 /usr/local/share:/usr/share
    if let Ok(xdg_dirs) = std::env::var("XDG_DATA_DIRS") {
        for d in xdg_dirs.split(':') {
            let p = PathBuf::from(d);
            if p.is_dir() {
                dirs.push(p);
            }
        }
    } else {
        let defaults = ["/usr/local/share", "/usr/share"];
        for d in defaults {
            let p = PathBuf::from(d);
            if p.is_dir() {
                dirs.push(p);
            }
        }
    }

    // 去重
    dirs.sort();
    dirs.dedup();
    dirs
}

fn icon_theme_candidates() -> Vec<String> {
    let mut themes: Vec<String> = Vec::new();
    if let Ok(theme) = std::env::var("XDG_ICON_THEME") {
        if !theme.trim().is_empty() {
            themes.push(theme);
        }
    }
    // 常见回退主题
    for t in ["hicolor", "Adwaita", "Papirus"].iter() {
        themes.push((*t).to_string());
    }
    themes.sort();
    themes.dedup();
    themes
}
