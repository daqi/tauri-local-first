use std::path::PathBuf;
#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

pub fn scan_windows_app_roots() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    // 注册表 Uninstall 项
    #[cfg(windows)]
    {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let uninstall_paths = [
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ];
        for root in [&hklm, &hkcu] {
            for sub in uninstall_paths {
                if let Ok(key) = root.open_subkey_with_flags(sub, KEY_READ) {
                    for item in key.enum_keys().flatten() {
                        if let Ok(app_key) = key.open_subkey_with_flags(&item, KEY_READ) {
                            let install_loc: Result<String, _> =
                                app_key.get_value("InstallLocation");
                            if let Ok(path) = install_loc {
                                let p = PathBuf::from(path);
                                if p.is_dir() {
                                    dirs.push(p);
                                }
                            }
                            let display_icon: Result<String, _> = app_key.get_value("DisplayIcon");
                            if let Ok(icon) = display_icon {
                                let p = PathBuf::from(icon);
                                if let Some(dir) = p.parent() {
                                    if dir.is_dir() {
                                        dirs.push(dir.to_path_buf());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    dirs.retain(|p| p.is_dir());
    dirs
}
