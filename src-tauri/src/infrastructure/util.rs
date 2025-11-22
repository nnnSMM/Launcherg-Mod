use std::path::PathBuf;
use std::sync::Arc;
use std::{fs, path::Path};

use tauri::AppHandle;
use tauri::Manager;

const ROOT_DIR: &str = "launcherg";

fn get_abs_dir(root: Option<PathBuf>) -> String {
    match root {
        Some(root) => {
            let path = &root.join(Path::new(ROOT_DIR));
            fs::create_dir_all(path).unwrap();
            return path.to_string_lossy().to_string();
        }
        None => {
            fs::create_dir_all(ROOT_DIR).unwrap();
            let path_str = fs::canonicalize(ROOT_DIR)
                .unwrap()
                .to_string_lossy()
                .to_string();
            // Remove \\?\ prefix on Windows if present
            #[cfg(target_os = "windows")]
            let path_str = if path_str.starts_with(r"\\?\") {
                path_str[4..].to_string()
            } else {
                path_str
            };
            return path_str;
        }
    }
}

pub fn get_save_root_abs_dir(handle: &Arc<AppHandle>) -> String {
    let root = handle.path().app_config_dir().ok();
    get_abs_dir(root)
}

pub fn get_save_root_abs_dir_with_ptr_handle(handle: &AppHandle) -> String {
    let root = handle.path().app_config_dir().ok();
    get_abs_dir(root)
}
