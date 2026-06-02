use std::path::PathBuf;
use std::{fs, path::Path};

use tauri::AppHandle;
use tauri::Manager;

const ROOT_DIR: &str = "launcherg";

fn get_abs_dir(root: Option<PathBuf>) -> String {
    match root {
        Some(root) => {
            let path = root.join(Path::new(ROOT_DIR));
            if let Err(e) = fs::create_dir_all(&path) {
                eprintln!(
                    "Failed to create app data directory {}: {}",
                    path.display(),
                    e
                );
            }
            path.to_string_lossy().to_string()
        }
        None => {
            if let Err(e) = fs::create_dir_all(ROOT_DIR) {
                eprintln!(
                    "Failed to create fallback data directory {}: {}",
                    ROOT_DIR, e
                );
            }
            let path_str = match fs::canonicalize(ROOT_DIR) {
                Ok(path) => path.to_string_lossy().to_string(),
                Err(e) => {
                    eprintln!(
                        "Failed to canonicalize fallback data directory {}: {}",
                        ROOT_DIR, e
                    );
                    ROOT_DIR.to_string()
                }
            };
            // Remove \\?\ prefix on Windows if present
            #[cfg(target_os = "windows")]
            let path_str = if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
                stripped.to_string()
            } else {
                path_str
            };
            path_str
        }
    }
}

pub fn get_save_root_abs_dir(handle: &AppHandle) -> String {
    let root = handle.path().app_config_dir().ok();
    get_abs_dir(root)
}
