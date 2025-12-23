//! ゲーム実行とプロセス監視モジュール
//!
//! このモジュールはゲームプロセスの起動、監視、プレイ時間の追跡を担当します。
//! テスト可能な小さなユニットに分割されています。

use std::sync::Arc;

use chrono::Local;
use sysinfo::{ProcessExt, System, SystemExt};
use tauri::{AppHandle, Emitter};
use tokio::time::{interval, Duration, Instant};

use crate::{
    domain::{
        collection::CollectionElement, file::get_exe_path_from_lnk,
        repository::collection::CollectionRepository, Id,
    },
    infrastructure::repositoryimpl::repository::RepositoriesExt,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use super::pause_manager::PauseManager;
use super::screenshot_watcher::ScreenshotWatcher;

/// ゲームプロセスの起動結果
pub struct LaunchResult {
    pub spawned_pid: Option<u32>,
    pub path_str: String,
}

/// プロセス検索の設定
pub struct ProcessSearchConfig {
    pub search_timeout: Duration,
    pub search_interval: Duration,
    pub system_folders: Vec<&'static str>,
    pub game_folders: Vec<&'static str>,
}

impl Default for ProcessSearchConfig {
    fn default() -> Self {
        Self {
            search_timeout: Duration::from_secs(180),
            search_interval: Duration::from_secs(2),
            system_folders: vec!["c:\\windows"],
            game_folders: vec!["VisualNovel", "steamapps", "dmmgameplayer"],
        }
    }
}

/// ゲームを起動する
///
/// # Arguments
/// * `element` - 起動するゲームのCollectionElement
///
/// # Returns
/// * `Ok(LaunchResult)` - 起動成功時
/// * `Err` - 起動失敗時
pub fn launch_game(element: &CollectionElement) -> anyhow::Result<LaunchResult> {
    let path_str = match (&element.exe_path, &element.lnk_path) {
        (Some(p), _) => p.clone(),
        (None, Some(p)) => p.clone(),
        (None, None) => {
            return Err(anyhow::anyhow!(
                "実行ファイルまたはショートカットが見つかりません"
            ))
        }
    };

    let path = std::path::Path::new(&path_str);
    let is_lnk = path_str.to_lowercase().ends_with(".lnk");

    let mut spawned_pid: Option<u32> = None;

    let spawn_result = if is_lnk {
        // .lnkファイルの場合、cmd /c startを使用
        std::process::Command::new("cmd")
            .args(&["/c", "start", "", &path_str])
            .spawn()
    } else {
        // .exeファイルの場合、直接起動
        if let Some(parent_dir) = path.parent() {
            match std::process::Command::new(path)
                .current_dir(parent_dir)
                .spawn()
            {
                Ok(child) => {
                    spawned_pid = Some(child.id());
                    Ok(child)
                }
                Err(e) => Err(e),
            }
        } else {
            return Err(anyhow::anyhow!("親ディレクトリが見つかりません"));
        }
    };

    match spawn_result {
        Ok(_) => Ok(LaunchResult {
            spawned_pid,
            path_str,
        }),
        Err(e) => Err(anyhow::anyhow!("Failed to launch game: {}", e)),
    }
}

/// 起動エラーがパス関連かどうかを判定
///
/// # Arguments
/// * `error` - 発生したエラー
///
/// # Returns
/// パス関連のエラー(ファイルが見つからない等)の場合true
pub fn is_path_related_error(error: &std::io::Error) -> bool {
    if let Some(code) = error.raw_os_error() {
        // 2: ERROR_FILE_NOT_FOUND
        // 3: ERROR_PATH_NOT_FOUND
        // 267: ERROR_DIRECTORY
        code == 2 || code == 3 || code == 267
    } else {
        false
    }
}

/// プロセス候補のスコアリング
///
/// より高いスコアは、ターゲットゲームである可能性が高いことを示します。
pub fn score_process_candidate(
    process: &sysinfo::Process,
    final_exe_path: &std::path::Path,
    game_name: &str,
    config: &ProcessSearchConfig,
) -> i32 {
    let exe_path = process.exe();
    let path_lower = exe_path.to_string_lossy().to_lowercase();
    let name_lower = process.name().to_lowercase();
    let game_name_lower = game_name.to_lowercase();

    // システムフォルダのプロセスは除外
    if config
        .system_folders
        .iter()
        .any(|folder| path_lower.starts_with(folder))
    {
        return 0;
    }

    let mut score = 0;

    // 最優先: 起動パスと完全一致
    if exe_path == final_exe_path {
        score += 100;
    }

    // 次点: プロセス名にゲーム名が含まれている
    if name_lower.contains(&game_name_lower) || game_name_lower.contains(&name_lower) {
        score += 50;
    }

    // 次点: 有名なゲームフォルダ
    if config
        .game_folders
        .iter()
        .any(|folder| path_lower.contains(folder))
    {
        score += 10;
    }

    // それ以外でも候補には入れる(スコア1)
    if score == 0 {
        score = 1;
    }

    score
}

/// 最適なプロセス候補を選択
pub fn select_best_process(
    candidates: &[(sysinfo::Pid, i32)],
    game_name: &str,
    process_names: &[(sysinfo::Pid, String)],
) -> Option<sysinfo::Pid> {
    if candidates.is_empty() {
        return None;
    }

    let max_score = candidates
        .iter()
        .map(|(_, score)| *score)
        .max()
        .unwrap_or(0);

    // スコアが同じ場合は、名前の長さで判定
    candidates
        .iter()
        .filter(|(_, score)| *score == max_score)
        .min_by_key(|(pid, _)| {
            process_names
                .iter()
                .find(|(p, _)| p == pid)
                .map(|(_, name)| (name.len() as i32 - game_name.len() as i32).abs())
                .unwrap_or(i32::MAX)
        })
        .map(|(pid, _)| *pid)
}

/// ゲームプロセスモニター
///
/// ゲームプロセスを監視し、プレイ時間を記録する責務を持つ構造体
pub struct GameProcessMonitor<R: RepositoriesExt + Send + Sync + 'static> {
    handle: Arc<AppHandle>,
    repositories: Arc<R>,
    pause_manager: Arc<PauseManager>,
    screenshot_watcher: Arc<ScreenshotWatcher<R>>,
    element_id: i32,
}

impl<R: RepositoriesExt + Send + Sync + 'static> GameProcessMonitor<R> {
    pub fn new(
        handle: Arc<AppHandle>,
        repositories: Arc<R>,
        pause_manager: Arc<PauseManager>,
        screenshot_watcher: Arc<ScreenshotWatcher<R>>,
        element_id: i32,
    ) -> Self {
        Self {
            handle,
            repositories,
            pause_manager,
            screenshot_watcher,
            element_id,
        }
    }

    /// ポーズショートカットを登録
    pub async fn register_pause_shortcut(&self) -> Option<Shortcut> {
        if let Ok(Some(pause_shortcut_key)) = self
            .repositories
            .collection_repository()
            .get_app_setting("pause_shortcut_key".to_string())
            .await
        {
            if !pause_shortcut_key.is_empty() {
                if let Ok(shortcut) = pause_shortcut_key.parse::<Shortcut>() {
                    if !self
                        .handle
                        .global_shortcut()
                        .is_registered(shortcut.clone())
                    {
                        if self
                            .handle
                            .global_shortcut()
                            .register(shortcut.clone())
                            .is_ok()
                        {
                            return Some(shortcut);
                        }
                    }
                }
            }
        }
        None
    }

    /// ゲームプロセスを検索
    pub async fn find_game_process(
        &self,
        path_str: &str,
        spawned_pid: Option<u32>,
        game_name: &str,
        config: &ProcessSearchConfig,
    ) -> Option<sysinfo::Pid> {
        // 直接起動したプロセスのPIDがある場合はそれを使う
        if let Some(pid) = spawned_pid {
            return Some(sysinfo::Pid::from(pid as usize));
        }

        let search_start_time = Instant::now();

        // LNKの解決
        let final_exe_path_str = if path_str.to_lowercase().ends_with(".lnk") {
            get_exe_path_from_lnk(path_str)
                .await
                .unwrap_or(path_str.to_string())
        } else {
            path_str.to_string()
        };
        let final_exe_path = std::path::Path::new(&final_exe_path_str);

        loop {
            if search_start_time.elapsed() > config.search_timeout {
                println!("[WARN] Game process search timed out. Play time may not be recorded.");
                return None;
            }

            tokio::time::sleep(config.search_interval).await;

            let mut system = System::new();
            system.refresh_processes();

            let processes: Vec<_> = system.processes().values().collect();
            if processes.is_empty() {
                continue;
            }

            let mut candidates: Vec<(sysinfo::Pid, i32)> = Vec::new();
            let mut process_names: Vec<(sysinfo::Pid, String)> = Vec::new();

            for process in &processes {
                let score = score_process_candidate(process, final_exe_path, game_name, config);
                if score > 0 {
                    candidates.push((process.pid(), score));
                    process_names.push((process.pid(), process.name().to_string()));
                }
            }

            if let Some(pid) = select_best_process(&candidates, game_name, &process_names) {
                return Some(pid);
            }
        }
    }

    /// プロセスを監視してプレイ時間を記録
    pub async fn monitor_process(&self, pid: sysinfo::Pid, registered_shortcut: Option<Shortcut>) {
        println!("Start monitoring process (PID: {}) for game", pid);

        // スクリーンショットウォッチャーを開始
        if let Err(e) = self
            .screenshot_watcher
            .start_watching(self.handle.clone(), self.element_id)
        {
            eprintln!("Failed to start screenshot watcher: {}", e);
        }

        let mut interval = interval(Duration::from_secs(10));
        let mut system = System::new();
        let mut last_check_time = Instant::now();

        loop {
            interval.tick().await;
            system.refresh_processes();

            if !self.pause_manager.is_paused() {
                let now = Instant::now();
                let duration = now.duration_since(last_check_time).as_secs() as i32;

                if duration > 0 {
                    let _ = self
                        .repositories
                        .collection_repository()
                        .add_play_time_seconds(&Id::new(self.element_id), duration)
                        .await;
                }
                last_check_time = now;
            } else {
                last_check_time = Instant::now();
            }

            // プロセスが終了したか確認
            if system.process(pid).is_none() {
                self.cleanup(registered_shortcut).await;
                break;
            }
        }
    }

    /// 監視終了時のクリーンアップ
    async fn cleanup(&self, registered_shortcut: Option<Shortcut>) {
        // スクリーンショットウォッチャーを停止
        self.screenshot_watcher.stop_watching();

        // 最終プレイ日時を更新
        let _ = self
            .repositories
            .collection_repository()
            .update_element_last_play_at_by_id(&Id::new(self.element_id), Local::now())
            .await;

        // 追跡状態をfalseに
        self.pause_manager.set_tracking(false);

        // ポーズショートカットを解除
        if let Some(shortcut) = registered_shortcut {
            let _ = self.handle.global_shortcut().unregister(shortcut);
        }

        // トレイメニューの「最近プレイしたゲーム」を更新
        let _ = self.handle.emit("recent-games-changed", ());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_search_config_default() {
        let config = ProcessSearchConfig::default();
        assert_eq!(config.search_timeout, Duration::from_secs(180));
        assert_eq!(config.search_interval, Duration::from_secs(2));
        assert!(config.system_folders.contains(&"c:\\windows"));
    }

    #[test]
    fn test_is_path_related_error() {
        // ERROR_FILE_NOT_FOUND (2)
        let error = std::io::Error::from_raw_os_error(2);
        assert!(is_path_related_error(&error));

        // ERROR_PATH_NOT_FOUND (3)
        let error = std::io::Error::from_raw_os_error(3);
        assert!(is_path_related_error(&error));

        // Other error
        let error = std::io::Error::from_raw_os_error(1);
        assert!(!is_path_related_error(&error));
    }

    #[test]
    fn test_select_best_process_empty() {
        let candidates: Vec<(sysinfo::Pid, i32)> = vec![];
        let process_names: Vec<(sysinfo::Pid, String)> = vec![];

        let result = select_best_process(&candidates, "TestGame", &process_names);
        assert!(result.is_none());
    }

    #[test]
    fn test_select_best_process_single() {
        let pid = sysinfo::Pid::from(1234);
        let candidates = vec![(pid, 100)];
        let process_names = vec![(pid, "TestGame.exe".to_string())];

        let result = select_best_process(&candidates, "TestGame", &process_names);
        assert_eq!(result, Some(pid));
    }
}
