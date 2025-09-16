#[cfg(target_os = "macos")]
use base64::Engine;
#[cfg(target_os = "macos")]
use plist::{Dictionary, Value as PlistValue};
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::process::Command;
#[cfg(target_os = "macos")]
use std::time::{SystemTime, UNIX_EPOCH};

pub fn scan_macos_app_roots() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    dirs.push(PathBuf::from("/Applications"));
    dirs.retain(|p| p.is_dir());
    dirs
}

#[cfg(target_os = "macos")]
fn find_icns_path(app_bundle: &Path, dict: Dictionary) -> Option<PathBuf> {
    let res = app_bundle.join("Contents/Resources");
    if let Some(PlistValue::String(icon_base)) = dict.get("CFBundleIconFile") {
        let mut name = icon_base.clone();
        if !name.ends_with(".icns") {
            name.push_str(".icns");
        }
        let p = res.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    let fallback = res.join("icon.icns");
    if fallback.exists() {
        return Some(fallback);
    }
    None
}

#[cfg(target_os = "macos")]
fn icns_to_data_url_sips(icns_path: &Path) -> Option<String> {
    let out = icns_to_png_file_sips(icns_path)?;
    let bytes = std::fs::read(&out).ok()?;
    // 读取后清理临时文件，避免堆积
    let _ = std::fs::remove_file(&out);
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Some(format!("data:image/png;base64,{}", b64))
}

#[cfg(target_os = "macos")]
fn icns_to_png_file_sips(icns_path: &Path) -> Option<PathBuf> {
    if !icns_path.exists() {
        return None;
    }
    let tmp_dir = std::env::temp_dir().join("tlfsuite_icon_cache");
    let _ = std::fs::create_dir_all(&tmp_dir);
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis();
    let stem = icns_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("icon");
    let out = tmp_dir.join(format!("{}_{}.png", stem, ts));
    let status = Command::new("sips")
        .arg("-s")
        .arg("format")
        .arg("png")
        .arg(icns_path)
        .arg("--out")
        .arg(&out)
        .status()
        .ok()?;
    if !status.success() || !out.exists() {
        return None;
    }
    Some(out)
}

#[cfg(target_os = "macos")]
pub fn resolve_macos_name_and_icon(
    app_bundle: &Path,
) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    let info = app_bundle.join("Contents/Info.plist");
    let mut icon: Option<String> = None;
    if info.exists() {
        if let Ok(file) = fs::File::open(&info) {
            if let Ok(PlistValue::Dictionary(dict)) = PlistValue::from_reader(file) {
                // Use a clone for find_icns_path so we can still read values from dict afterwards
                if let Some(icns) = find_icns_path(app_bundle, dict.clone()) {
                    icon = icns_to_data_url_sips(&icns);
                }
                let get_str = |k: &str| -> Option<String> {
                  dict.get(k).and_then(|v| {
                    if let PlistValue::String(s) = v {
                      Some(s.clone())
                    } else {
                      None
                    }
                  })
                };
                let id = get_str("CFBundleIdentifier");
                let name = get_str("CFBundleName");
                let exec = get_str("CFBundleExecutable");
                return (name, icon, id, exec);
            }
        }
    }
    (None, icon, None, None)
}
