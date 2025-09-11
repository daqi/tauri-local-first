// Implement core SwitchHosts commands for Tauri
// NOTE: Returned JSON shapes aim to match SwitchHosts TypeScript interfaces in
// SwitchHosts/src/common/data.d.ts (IHostsListObject, IHostsContentObject, ITrashcanObject, etc.)
// We keep storage as serde_json::Value for flexibility but preserve fields like
// `id`, `title`, `on`, `type`, `children`, `content`, `add_time_ms` to maintain compatibility.
use serde_json::json;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn data_dir() -> PathBuf {
    if let Ok(dir) = env::var("SWEETHOSTS_DATA_DIR") {
        return PathBuf::from(dir);
    }

    if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home).join(".sweethosts");
        }
    }

    // fallback to current dir
    PathBuf::from(".").join("sweethosts")
}

fn ensure_data_dir() -> std::io::Result<()> {
    let d = data_dir();
    if !d.exists() {
        fs::create_dir_all(&d)?;
    }
    Ok(())
}

fn read_json_array(p: PathBuf) -> Vec<Value> {
    if !p.exists() {
        return vec![];
    }
    match fs::read_to_string(p) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => vec![],
    }
}

fn write_json_array(p: PathBuf, v: &Vec<Value>) -> bool {
    match fs::write(p, serde_json::to_string(v).unwrap_or_default()) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tauri::command]
pub fn ping() -> String {
    "pong".to_string()
}

#[tauri::command]
pub fn get_list() -> Vec<Value> {
    if let Err(_) = ensure_data_dir() {
        return vec![];
    }

    let mut p = data_dir();
    p.push("list.json");
    read_json_array(p)
}

#[tauri::command]
pub fn set_list(v: Vec<Value>) -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut p = data_dir();
    p.push("list.json");
    write_json_array(p, &v)
}

#[tauri::command]
pub fn get_content_of_list() -> String {
    // read list.json and collect ids where on == true
    let list = get_list();
    let mut ids: Vec<String> = Vec::new();

    fn collect(items: &Vec<Value>, out: &mut Vec<String>) {
        for item in items {
            if let Some(on) = item.get("on").and_then(|v| v.as_bool()) {
                if on {
                    if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                        out.push(id.to_string());
                    }
                }
            }
            if let Some(children) = item.get("children").and_then(|v| v.as_array()) {
                collect(children, out);
            }
        }
    }

    collect(&list, &mut ids);

    let mut contents: Vec<String> = Vec::new();
    for id in ids {
        let mut p = data_dir();
        p.push(format!("hosts_content_{}.txt", id));
        if let Ok(s) = fs::read_to_string(p) {
            contents.push(s);
        }
    }

    let content = contents.join("\n\n");
    content
}

pub fn get_path_of_system_hosts() -> String {
    if cfg!(target_os = "windows") {
        std::env::var("windir")
            .map(|w| format!("{}\\system32\\drivers\\etc\\hosts", w))
            .unwrap_or_else(|_| "C:\\Windows\\system32\\drivers\\etc\\hosts".to_string())
    } else {
        "/etc/hosts".to_string()
    }
}

#[tauri::command]
pub fn get_system_hosts() -> String {
    let p = get_path_of_system_hosts();
    fs::read_to_string(p).unwrap_or_default()
}

#[tauri::command]
pub fn get_hosts_content(id: String) -> String {
    if let Err(_) = ensure_data_dir() {
        return String::new();
    }
    let mut p = data_dir();
    p.push(format!("hosts_content_{}.txt", id));
    match fs::read_to_string(p) {
        Ok(s) => s,
        Err(_) => String::new(),
    }
}

#[tauri::command]
pub fn set_hosts_content(id: String, content: String) -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut p = data_dir();
    p.push(format!("hosts_content_{}.txt", id));
    fs::write(p, content).is_ok()
}

#[tauri::command]
pub fn set_system_hosts(content: String, opts: Option<String>) -> Value {
    let sys_path = get_path_of_system_hosts();

    // read old content
    let old_content = fs::read_to_string(&sys_path).unwrap_or_default();

    // respect safe mode
    if std::env::var("SWEETHOSTS_SAFE_MODE").unwrap_or_default() == "1" {
        // write to temp file instead
        let mut tmp = env::temp_dir();
        tmp.push(format!(
            "sweethosts_safe_{}.hosts",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        ));
        let _ = fs::write(&tmp, &content);
        return json!({ "success": true, "old_content": old_content, "new_content": content, "safe_path": tmp.to_string_lossy() });
    }

    // try direct write first
    match fs::write(&sys_path, &content) {
        Ok(_) => {
            // success
            let res =
                json!({ "success": true, "old_content": old_content, "new_content": content });
            return res;
        }
        Err(_) => {
            // try sudo fallback on unix
            if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
                if let Some(pw) = opts {
                    // write tmp file
                    let mut tmp = env::temp_dir();
                    let rand_part = format!(
                        "{}",
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_nanos())
                            .unwrap_or(0)
                    );
                    tmp.push(format!("swh_{}.txt", rand_part));
                    let _ = fs::write(&tmp, &content);

                    let cmd = format!(
                        "echo '{}' | sudo -S sh -c 'cat \"{}\" > \"{}\" && chmod 644 \"{}\"'",
                        pw.replace("'", "'\\''"),
                        tmp.to_string_lossy(),
                        sys_path,
                        sys_path
                    );

                    let output = Command::new("sh").arg("-c").arg(cmd).output();
                    // cleanup tmp
                    let _ = fs::remove_file(&tmp);

                    if let Ok(o) = output {
                        if o.status.success() {
                            let res = json!({ "success": true, "old_content": old_content, "new_content": content });
                            return res;
                        } else {
                            let msg = String::from_utf8_lossy(&o.stderr).to_string();
                            return json!({ "success": false, "code": "no_access", "message": msg });
                        }
                    }
                }
            }
        }
    }

    json!({ "success": false, "code": "no_access" })
}

#[tauri::command]
pub fn close_main_window() -> bool {
    // Window control should be handled via tauri::Window in app code; noop
    true
}

#[tauri::command]
pub fn quit() -> bool {
    // Tauri app exit should be invoked from main; noop
    true
}
