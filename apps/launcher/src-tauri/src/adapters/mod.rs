use std::path::PathBuf;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

pub fn scan_installed_app_roots() -> Vec<PathBuf> {
  let mut out: Vec<PathBuf> = Vec::new();
  #[cfg(target_os = "macos")]
  {
    out.extend(macos::scan_macos_app_roots());
  }
  #[cfg(target_os = "linux")]
  {
    out.extend(linux::scan_linux_app_roots());
  }
  #[cfg(target_os = "windows")]
  {
    out.extend(windows::scan_windows_app_roots());
  }
  out.sort();
  out.dedup();
  out
}

#[cfg(target_os = "macos")]
pub use macos::resolve_macos_name_and_icon;

#[cfg(target_os = "linux")]
pub use linux::resolve_icon_name;
