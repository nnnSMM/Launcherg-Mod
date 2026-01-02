pub struct File {}
pub struct LnkMetadata {
    pub path: String,
    pub icon: String,
}

use std::{collections::HashMap, fs, path::Path, sync::Arc};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use tauri::{async_runtime::JoinHandle, AppHandle};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;
use windows::{
    core::{ComInterface, PCWSTR},
    Win32::{
        Storage::FileSystem::WIN32_FIND_DATAW,
        System::Com::{CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_INPROC_SERVER},
        System::Com::{IPersistFile, STGM_READ},
        UI::Shell::{IShellLinkW, ShellLink},
    },
};

use crate::infrastructure::util::get_save_root_abs_dir;

use super::{
    all_game_cache::{AllGameCache, AllGameCacheOne},
    collection::CollectionElement,
    distance::get_comparable_distance,
    Id,
};

trait WString {
    fn to_wide_null_terminated(&self) -> Vec<u16>;
}

impl WString for &str {
    fn to_wide_null_terminated(&self) -> Vec<u16> {
        self.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

const NOT_GAME_EQUALLY_WORD: [&str; 1] = ["bgi"];

const NOT_GAME_TERMS: [&str; 16] = [
    "マニュアル",
    "詳細設定",
    "はじめに",
    "サポート",
    "セーブデータ",
    "インストール",
    "アンインストール",
    "体験版",
    "install",
    "uninstall",
    "autorun",
    "削除",
    "license",
    "ライセンス",
    "公式サイト",
    "ホームページ",
];
/// ファイル名がゲーム以外のファイルかどうかを判定
fn not_game(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    NOT_GAME_TERMS
        .iter()
        .any(|term| filename_lower.contains(term))
        || NOT_GAME_EQUALLY_WORD
            .iter()
            .any(|word| filename_lower == *word)
}

const REMOVE_WORDS: [&str; 9] = [
    "を起動",
    "の起動",
    "_単独動作版",
    "「",
    "」",
    " ",
    "　",
    "ダウンロード版",
    "DL版",
];
/// ファイル名から不要なワードを削除
fn remove_word(filename: &str) -> String {
    REMOVE_WORDS
        .iter()
        .fold(filename.to_string(), |acc, word| acc.replace(word, ""))
}

const IGNORE_GAME_ID: [i32; 4] = [2644, 63, 2797, 10419];

// (string, i32)でstringがファイル名といっちした場合はこのゲームにする
const EQUALLY_FILENAME_GAME_ID_PAIR: [(&str, i32); 1] = [("pieces", 27123)];

pub fn get_file_name_without_extension(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    if let Some(file_name) = path.file_name() {
        if let Some(file_name_str) = file_name.to_str() {
            let file_name_without_extension = Path::new(file_name_str)
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned());
            return file_name_without_extension;
        }
    }
    None
}

pub fn normalize(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        if ch >= 'Ａ' && ch <= 'Ｚ' || ch >= 'ａ' && ch <= 'ｚ' {
            result.push((ch as u32 - 'Ａ' as u32 + 'A' as u32) as u8 as char);
        } else if ch >= '０' && ch <= '９' {
            result.push((ch as u32 - '０' as u32 + '0' as u32) as u8 as char);
        } else {
            result.push(ch);
        }
    }
    result.to_lowercase()
}

pub fn get_file_paths_by_exts(
    explorer_dir_path: String,
    filter_exts: Vec<String>,
) -> anyhow::Result<Vec<String>> {
    let mut link_file_paths = Vec::new();

    for entry in WalkDir::new(explorer_dir_path) {
        let entry = entry?;

        if entry.file_type().is_file() {
            let path = entry.path();

            if let Some(extension) = path.extension() {
                let cmp_ext = extension.to_string_lossy().to_lowercase();
                for filter_ext in filter_exts.iter() {
                    if cmp_ext == *filter_ext {
                        let path_str = path.to_string_lossy().to_string();
                        link_file_paths.push(path_str);
                    }
                }
            }
        }
    }

    Ok(link_file_paths)
}

pub fn get_url_file_icon_path(url_file_path: &str) -> anyhow::Result<Option<String>> {
    let ini_contents = std::fs::read_to_string(url_file_path)?;
    Ok(get_ini_value(&ini_contents, "IconFile"))
}

fn get_ini_value(contents: &str, key: &str) -> Option<String> {
    let key_line = contents.lines().find(|&line| line.starts_with(key))?;
    let parts: Vec<&str> = key_line.splitn(2, '=').collect();
    if parts.len() == 2 {
        Some(parts[1].trim().to_string())
    } else {
        None
    }
}

pub fn get_lnk_metadatas(lnk_file_paths: Vec<&str>) -> anyhow::Result<HashMap<&str, LnkMetadata>> {
    let mut metadatas = HashMap::new();

    unsafe {
        CoInitialize(None)?;

        let mut target_path_vec: Vec<u16> = vec![0; 261];
        let target_path_slice =
            std::slice::from_raw_parts_mut(target_path_vec.as_mut_ptr(), target_path_vec.len());

        for file_path in lnk_file_paths {
            if file_path.to_lowercase().ends_with("lnk") {
                let shell_link: IShellLinkW =
                    CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

                let persist_file: IPersistFile = ComInterface::cast(&shell_link)?;
                persist_file.Load(
                    PCWSTR::from_raw(file_path.to_wide_null_terminated().as_ptr()),
                    STGM_READ,
                )?;

                shell_link.GetPath(target_path_slice, &mut WIN32_FIND_DATAW::default(), 0)?;
                let path = PCWSTR::from_raw(target_path_vec.as_mut_ptr())
                    .to_string()?
                    .clone();
                shell_link.GetIconLocation(target_path_slice, &mut 0)?;
                let icon = PCWSTR::from_raw(target_path_vec.as_mut_ptr())
                    .to_string()?
                    .clone();

                metadatas.insert(file_path, LnkMetadata { path, icon });
            } else if file_path.to_lowercase().ends_with("url") {
                let icon_file = get_url_file_icon_path(file_path)?;

                metadatas.insert(
                    file_path,
                    LnkMetadata {
                        path: file_path.to_string(),
                        icon: icon_file.unwrap_or_default(),
                    },
                );
            } else {
                return Err(anyhow::anyhow!("{} is not end lnk|url", file_path));
            }
        }

        CoUninitialize();
    }
    Ok(metadatas)
}

pub fn get_most_probable_game_candidate(
    id_name_pairs: &AllGameCache,
    filepath: String,
) -> anyhow::Result<Option<(AllGameCacheOne, String)>> {
    let candidates: AllGameCache =
        get_game_candidates_by_exe_path(id_name_pairs, &filepath, 0.8, 1)?;
    Ok(candidates
        .first()
        .map(|candidate| (candidate.clone(), filepath)))
}

pub fn get_game_candidates_by_exe_path(
    id_name_pairs: &AllGameCache,
    filepath: &str,
    threshould: f32,
    candidate_limit: usize,
) -> anyhow::Result<AllGameCache> {
    let parent = Path::new(&filepath)
        .parent()
        .and_then(|v| {
            v.file_name()
                .and_then(|name| Some(normalize(&name.to_string_lossy().to_string())))
        })
        .ok_or(anyhow::anyhow!("can not get parent"))?;

    let filename: String =
        get_file_name_without_extension(filepath).ok_or(anyhow::anyhow!("can not get filename"))?;
    let filename = normalize(&filename);
    if not_game(&filename) {
        return Ok(vec![]);
    }
    let filename = remove_word(&filename);

    let is_skip_filename_check = filename == "game" || filename == "start";

    let mut distance_pairs = vec![];

    for (equally_filename, id) in EQUALLY_FILENAME_GAME_ID_PAIR {
        if filename == *equally_filename {
            id_name_pairs.iter().find(|v| v.id == id).map(|v| {
                distance_pairs.push((v.clone(), 100.0));
            });
        }
    }

    for pair in id_name_pairs.iter() {
        let mut is_ignore = false;
        for ignore_id in IGNORE_GAME_ID {
            if pair.id == ignore_id {
                is_ignore = true;
            }
        }
        if is_ignore {
            continue;
        }

        let mut val: f32 = 0.0;
        if !is_skip_filename_check {
            val = val.max(get_comparable_distance(&filename, &pair.gamename));
        }

        val = val.max(get_comparable_distance(&parent, &pair.gamename));
        if val > threshould {
            distance_pairs.push((pair.clone(), val));
        }
    }

    if distance_pairs.len() == 0 {
        for pair in id_name_pairs.iter() {
            if filename.len() > 5 && pair.gamename.contains(&filename) {
                distance_pairs.push((pair.clone(), filename.len() as f32));
            }
            if parent.len() > 5 && pair.gamename.contains(&parent) {
                distance_pairs.push((pair.clone(), parent.len() as f32));
            }
        }
    }

    distance_pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut res = vec![];
    for (pair, _) in distance_pairs {
        if res.len() < candidate_limit {
            res.push(pair)
        }
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // not_game のテスト
    // ========================================
    #[test]
    fn test_not_game_returns_true_for_installer() {
        // Arrange & Act & Assert
        assert!(not_game("インストール"));
        assert!(not_game("アンインストール"));
        assert!(not_game("install"));
        assert!(not_game("uninstall"));
    }

    #[test]
    fn test_not_game_returns_true_for_manual() {
        assert!(not_game("マニュアル"));
        assert!(not_game("はじめに"));
        assert!(not_game("サポート"));
    }

    #[test]
    fn test_not_game_returns_false_for_game() {
        assert!(!not_game("game"));
        assert!(!not_game("start"));
        assert!(!not_game("サクラノ詩"));
    }

    #[test]
    fn test_not_game_case_insensitive() {
        // 大文字小文字を区別しない
        assert!(not_game("INSTALL"));
        assert!(not_game("Install"));
    }

    // ========================================
    // remove_word のテスト
    // ========================================
    #[test]
    fn test_remove_word_removes_launch_suffix() {
        assert_eq!(remove_word("ゲームを起動"), "ゲーム");
        assert_eq!(remove_word("ゲームの起動"), "ゲーム");
    }

    #[test]
    fn test_remove_word_removes_brackets() {
        assert_eq!(remove_word("「タイトル」"), "タイトル");
    }

    #[test]
    fn test_remove_word_removes_dl_suffix() {
        assert_eq!(remove_word("ゲームダウンロード版"), "ゲーム");
        assert_eq!(remove_word("ゲームDL版"), "ゲーム");
    }

    #[test]
    fn test_remove_word_no_change() {
        assert_eq!(remove_word("普通のゲーム名"), "普通のゲーム名");
    }

    // ========================================
    // normalize のテスト
    // ========================================
    #[test]
    fn test_normalize_fullwidth_to_halfwidth() {
        // 全角英字→半角
        assert_eq!(normalize("ＡＢＣＤ"), "abcd");
        assert_eq!(normalize("ａｂｃｄ"), "abcd");
    }

    #[test]
    fn test_normalize_fullwidth_numbers() {
        // 全角数字→半角
        assert_eq!(normalize("１２３"), "123");
    }

    #[test]
    fn test_normalize_mixed() {
        assert_eq!(normalize("Ｈｅｌｌｏ１２３"), "hello123");
    }

    #[test]
    fn test_normalize_japanese_unchanged() {
        // 日本語はそのまま
        assert_eq!(normalize("こんにちは"), "こんにちは");
    }

    // ========================================
    // get_file_name_without_extension のテスト
    // ========================================
    #[test]
    fn test_get_filename_windows_path() {
        let result = get_file_name_without_extension("C:\\Games\\Game.exe");
        assert_eq!(result, Some("Game".to_string()));
    }

    #[test]
    fn test_get_filename_unix_path() {
        let result = get_file_name_without_extension("/home/user/game.exe");
        assert_eq!(result, Some("game".to_string()));
    }

    #[test]
    fn test_get_filename_no_extension() {
        let result = get_file_name_without_extension("README");
        assert_eq!(result, Some("README".to_string()));
    }

    #[test]
    fn test_get_filename_multiple_dots() {
        let result = get_file_name_without_extension("game.v1.2.exe");
        assert_eq!(result, Some("game.v1.2".to_string()));
    }

    // ========================================
    // get_ini_value のテスト
    // ========================================
    #[test]
    fn test_get_ini_value_found() {
        let contents = "[InternetShortcut]\nURL=https://example.com\nIconFile=icon.ico";
        let result = get_ini_value(contents, "IconFile");
        assert_eq!(result, Some("icon.ico".to_string()));
    }

    #[test]
    fn test_get_ini_value_not_found() {
        let contents = "[InternetShortcut]\nURL=https://example.com";
        let result = get_ini_value(contents, "IconFile");
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_ini_value_with_spaces() {
        let contents = "IconFile = icon.ico";
        let result = get_ini_value(contents, "IconFile");
        assert_eq!(result, Some("icon.ico".to_string()));
    }

    // ========================================
    // 既存テスト
    // ========================================
    #[test]
    fn test_get_game_candidates_by_exe_path() {
        let res = get_game_candidates_by_exe_path(
            &vec![AllGameCacheOne::new(
                27123,
                "pieces/渡り鳥のソムニウム".to_string(),
            )],
            "W:/others/software/Whirlpool/pieces/pieces.exe",
            0.5,
            3,
        )
        .unwrap();
        let pieces = res.first().unwrap();
        assert_eq!(pieces.id, 27123);
    }

    #[test]
    fn test_get_game_candidates_empty_cache() {
        let res =
            get_game_candidates_by_exe_path(&vec![], "C:/Games/SomeGame/game.exe", 0.5, 3).unwrap();
        assert!(res.is_empty());
    }

    #[test]
    fn test_get_game_candidates_below_threshold() {
        // 全く関係ないゲーム名 -> 閾値未満でヒットしない
        let res = get_game_candidates_by_exe_path(
            &vec![AllGameCacheOne::new(1, "まったく別のゲーム".to_string())],
            "C:/Games/TotallyDifferent/game.exe",
            0.9, // 高い閾値
            3,
        )
        .unwrap();
        // 閾値0.9を超えるマッチがない可能性
        // (実際の結果はdistance計算次第だが、関係ないゲームなら空になるはず)
        assert!(res.is_empty());
    }

    #[test]
    fn test_get_game_candidates_multiple_candidates() {
        let cache = vec![
            AllGameCacheOne::new(1, "ゲームA".to_string()),
            AllGameCacheOne::new(2, "ゲームB".to_string()),
            AllGameCacheOne::new(3, "ゲームC".to_string()),
        ];
        // 親フォルダ名が "ゲームA" に近い場合
        let res =
            get_game_candidates_by_exe_path(&cache, "C:/Games/ゲームA/start.exe", 0.5, 5).unwrap();
        // 少なくとも1件ヒットするはず
        assert!(!res.is_empty());
        // 最初の候補が "ゲームA" であること
        assert_eq!(res.first().unwrap().id, 1);
    }

    #[test]
    fn test_get_game_candidates_not_game_filter() {
        // インストーラーやマニュアルは除外される
        let cache = vec![AllGameCacheOne::new(1, "ゲームA".to_string())];
        let res = get_game_candidates_by_exe_path(&cache, "C:/Games/ゲームA/install.exe", 0.5, 3)
            .unwrap();
        // "install" は not_game でtrue -> 空配列
        assert!(res.is_empty());
    }

    // ========================================
    // PlayHistory のテスト
    // ========================================
    #[test]
    fn test_play_history_serialization() {
        let history = PlayHistory {
            minutes: 30.5,
            start_date: "2023-12-25T10:00:00".to_string(),
        };

        let json = serde_json::to_string(&history).unwrap();
        assert!(json.contains("\"minutes\":30.5"));
        assert!(json.contains("\"startDate\":\"2023-12-25T10:00:00\"")); // camelCase
    }

    #[test]
    fn test_play_history_deserialization() {
        let json = r#"{"minutes":45.0,"startDate":"2024-01-01"}"#;
        let history: PlayHistory = serde_json::from_str(json).unwrap();

        assert_eq!(history.minutes, 45.0);
        assert_eq!(history.start_date, "2024-01-01");
    }

    #[test]
    fn test_play_history_jsonl_format() {
        // 複数のPlayHistoryをJSONL形式でシリアライズ・デシリアライズ
        let histories = vec![
            PlayHistory {
                minutes: 10.0,
                start_date: "2023-01-01".to_string(),
            },
            PlayHistory {
                minutes: 20.0,
                start_date: "2023-01-02".to_string(),
            },
        ];

        let jsonl: String = histories
            .iter()
            .map(|h| serde_json::to_string(h).unwrap())
            .collect::<Vec<_>>()
            .join("\n");

        // 各行をパースできることを確認
        let parsed: Vec<PlayHistory> = jsonl
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].minutes, 10.0);
        assert_eq!(parsed[1].minutes, 20.0);
    }

    // ========================================
    // 高画質化サムネイル関連のテスト
    // ========================================
    #[test]
    fn test_upscaled_thumbnails_directory_name() {
        // 高画質化サムネイルのディレクトリ名が正しいことを確認
        assert_eq!(super::UPSCALED_THUMBNAILS_ROOT_DIR, "thumbnails_upscaled");
    }

    #[test]
    fn test_thumbnails_directory_name() {
        // 元のサムネイルディレクトリ名が正しいことを確認
        assert_eq!(super::THUMBNAILS_ROOT_DIR, "thumbnails");
    }

    #[test]
    fn test_thumbnails_directories_are_different() {
        // 元画像と高画質版のディレクトリが異なることを確認
        assert_ne!(
            super::THUMBNAILS_ROOT_DIR,
            super::UPSCALED_THUMBNAILS_ROOT_DIR
        );
    }

    #[test]
    fn test_format_upscaled_thumbnail_filename_2x() {
        // 純粋関数: ファイル名生成（2x）
        let filename = super::format_upscaled_thumbnail_filename(123, 2);
        assert_eq!(filename, "123_2x.png");
    }

    #[test]
    fn test_format_upscaled_thumbnail_filename_4x() {
        // 純粋関数: ファイル名生成（4x）
        let filename = super::format_upscaled_thumbnail_filename(456, 4);
        assert_eq!(filename, "456_4x.png");
    }

    #[test]
    fn test_format_upscaled_thumbnail_filename_various_ids() {
        // 様々なIDでファイル名が正しく生成されることを確認
        assert_eq!(super::format_upscaled_thumbnail_filename(1, 2), "1_2x.png");
        assert_eq!(
            super::format_upscaled_thumbnail_filename(99999, 4),
            "99999_4x.png"
        );
    }

    #[test]
    fn test_calculate_next_scale_from_unprocessed() {
        // 純粋関数: 未処理（0）→ 2x
        assert_eq!(super::calculate_next_scale(0), 2);
    }

    #[test]
    fn test_calculate_next_scale_from_2x() {
        // 純粋関数: 2x → 4x
        assert_eq!(super::calculate_next_scale(2), 4);
    }

    #[test]
    fn test_calculate_next_scale_from_4x() {
        // 純粋関数: 4x → 8x（理論上、現在は4xが上限だがロジックとしては正しい）
        assert_eq!(super::calculate_next_scale(4), 8);
    }

    #[test]
    fn test_should_skip_upscale_at_target() {
        // 純粋関数: 目標と同じレベル → スキップ
        assert!(super::should_skip_upscale(4, 4));
        assert!(super::should_skip_upscale(2, 2));
    }

    #[test]
    fn test_should_skip_upscale_above_target() {
        // 純粋関数: 目標より高いレベル → スキップ
        assert!(super::should_skip_upscale(4, 2));
    }

    #[test]
    fn test_should_not_skip_upscale_below_target() {
        // 純粋関数: 目標より低いレベル → 処理続行
        assert!(!super::should_skip_upscale(0, 2));
        assert!(!super::should_skip_upscale(0, 4));
        assert!(!super::should_skip_upscale(2, 4));
    }
}

const ICONS_ROOT_DIR: &str = "game-icons";
pub fn get_icon_path(
    handle: &Arc<AppHandle>,
    collection_element_id: &Id<CollectionElement>,
) -> String {
    let dir = Path::new(&get_save_root_abs_dir(handle)).join(ICONS_ROOT_DIR);
    fs::create_dir_all(&dir).unwrap();
    Path::new(&dir)
        .join(format!("{}.png", collection_element_id.value))
        .to_string_lossy()
        .to_string()
}
pub fn save_icon_to_png(
    handle: &Arc<AppHandle>,
    file_path: &str,
    collection_element_id: &Id<CollectionElement>,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let save_png_path = get_icon_path(handle, collection_element_id);

    // If icon already exists, do not overwrite it.
    if Path::new(&save_png_path).exists() {
        return Ok(tauri::async_runtime::spawn(async { Ok(()) }));
    }

    let is_ico = file_path.to_lowercase().ends_with("ico");

    if is_ico {
        return save_ico_to_png(file_path, &save_png_path);
    }
    if Path::new(file_path).exists() {
        return save_exe_file_png(handle, file_path, &save_png_path);
    }
    return save_default_icon(&save_png_path);
}

pub fn save_default_icon(save_png_path: &str) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let save_p = save_png_path.to_string();
    let handle = tauri::async_runtime::spawn(async move {
        let default_icon = include_bytes!("../../icons/notfound.png");
        let mut file = tokio::fs::File::create(save_p).await?;
        file.write_all(default_icon).await?;
        return Ok(());
    });

    Ok(handle)
}

pub fn save_ico_to_png(
    file_path: &str,
    save_png_path: &str,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    assert!(file_path.to_lowercase().ends_with("ico"));

    let p = file_path.to_string();
    let save_p = save_png_path.to_string();
    let handle = tauri::async_runtime::spawn(async move {
        match save_ico_to_png_sync(&p, &save_p) {
            Err(_) => {
                if std::path::Path::new(&save_p).exists() {
                    return Ok(());
                }
                save_default_icon(&save_p)?.await?
            }
            _ => Ok(()),
        }
    });

    Ok(handle)
}

pub fn save_ico_to_png_sync(file_path: &str, save_png_path: &str) -> anyhow::Result<()> {
    // Read an ICO file from disk:
    let file = std::fs::File::open(file_path)?;
    let icon_dir = ico::IconDir::read(file)?;

    let largest_entry = icon_dir
        .entries()
        .into_iter()
        .fold(None, |largest, v| match largest {
            None => {
                return Some(v);
            }
            Some(largest) => {
                if largest.width() < v.width() {
                    return Some(v);
                }
                return Some(largest);
            }
        });

    if let Some(entry) = largest_entry {
        // Decode the first entry into an image:
        let image = entry.decode()?;
        // You can get raw RGBA pixel data to pass to another image library:
        let rgba = image.rgba_data();
        assert_eq!(rgba.len(), (4 * image.width() * image.height()) as usize);
        // Alternatively, you can save the image as a PNG file:
        let file = std::fs::File::create(save_png_path)?;
        Ok(image.write_png(file)?)
    } else {
        return Err(anyhow::anyhow!("icon_dir.entries() is empty"));
    }
}

/// EXEファイルからアイコンを抽出してPNGとして保存
pub fn save_exe_file_png(
    handle: &Arc<AppHandle>,
    file_path: &str,
    save_png_path: &str,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let save_png_path_cloned = save_png_path.to_string();
    let (mut rx, _) = handle
        .shell()
        .sidecar("extract-icon")?
        .args(vec!["48", file_path, save_png_path])
        .spawn()?;

    let handle: JoinHandle<anyhow::Result<()>> = tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(_) | CommandEvent::Stderr(_) => {
                    // アイコンが存在すればOK、なければデフォルトを保存
                    if std::path::Path::new(&save_png_path_cloned).exists() {
                        return Ok(());
                    }
                    return save_default_icon(&save_png_path_cloned)?.await?;
                }
                CommandEvent::Terminated(_) => return Ok(()),
                _ => {}
            }
        }
        Err(anyhow::anyhow!("extract-icon is not terminated"))
    });

    Ok(handle)
}

const PLAY_HISTORIES_ROOT_DIR: &str = "play-histories";
pub fn get_play_history_path(
    handle: &Arc<AppHandle>,
    collection_element_id: &Id<CollectionElement>,
) -> String {
    let dir = Path::new(&get_save_root_abs_dir(handle)).join(PLAY_HISTORIES_ROOT_DIR);
    fs::create_dir_all(dir).unwrap();
    Path::new(&get_save_root_abs_dir(handle))
        .join(format!("{}.jsonl", collection_element_id.value))
        .to_string_lossy()
        .to_string()
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayHistory {
    pub minutes: f32,
    pub start_date: String,
}

pub fn get_file_created_at_sync(path: &str) -> Option<DateTime<Local>> {
    let metadata = fs::metadata(path).ok();
    metadata.and_then(|meta| {
        meta.created()
            .ok()
            .and_then(|time| Some(DateTime::from(time)))
    })
}

const THUMBNAILS_ROOT_DIR: &str = "thumbnails";
pub fn get_thumbnail_path(
    handle: &Arc<AppHandle>,
    collection_element_id: &Id<CollectionElement>,
) -> String {
    let dir = Path::new(&get_save_root_abs_dir(handle)).join(THUMBNAILS_ROOT_DIR);
    fs::create_dir_all(&dir).unwrap();
    Path::new(&dir)
        .join(format!("{}.png", collection_element_id.value))
        .to_string_lossy()
        .to_string()
}

pub fn save_thumbnail(
    handle: &Arc<AppHandle>,
    collection_element_id: &Id<CollectionElement>,
    src_url: String,
) -> JoinHandle<anyhow::Result<()>> {
    let collection_element_id = collection_element_id.clone();
    let handle_cloned = handle.clone();
    tauri::async_runtime::spawn(async move {
        let save_path = get_thumbnail_path(&handle_cloned, &collection_element_id);
        if !(std::path::Path::new(&save_path).exists()) && src_url != "" {
            let client = reqwest::Client::new();
            let response = client.get(src_url).send().await?;
            let bytes = response.bytes().await?;

            // Offload CPU intensive work to Rayon thread pool
            let (tx, rx) = tokio::sync::oneshot::channel();
            rayon::spawn(move || {
                let res = (|| -> anyhow::Result<()> {
                    let img = image::load_from_memory(&bytes)?;
                    img.save(&save_path)?;
                    Ok(())
                })();
                let _ = tx.send(res);
            });
            rx.await??;
        }
        Ok(())
    })
}

const UPSCALED_THUMBNAILS_ROOT_DIR: &str = "thumbnails_upscaled";

// ========================================
// テスト可能な純粋関数
// ========================================

/// 高画質化サムネイルのファイル名を生成（純粋関数）
fn format_upscaled_thumbnail_filename(id: i32, scale: i32) -> String {
    format!("{}_{scale}x.png", id)
}

/// 次の高画質化スケールを計算（純粋関数）
/// 0 → 2, 2 → 4
fn calculate_next_scale(current_level: i32) -> i32 {
    if current_level == 0 {
        2
    } else {
        current_level * 2
    }
}

/// 高画質化をスキップすべきかどうか判定（純粋関数）
fn should_skip_upscale(current_level: i32, target_scale: i32) -> bool {
    current_level >= target_scale
}

// ========================================
// 高画質化サムネイル関連の公開関数
// ========================================

/// 指定した倍率の高画質化サムネイルのパスを取得
pub fn get_upscaled_thumbnail_path_with_scale(
    handle: &Arc<AppHandle>,
    collection_element_id: &Id<CollectionElement>,
    scale: i32,
) -> String {
    let dir = Path::new(&get_save_root_abs_dir(handle)).join(UPSCALED_THUMBNAILS_ROOT_DIR);
    fs::create_dir_all(&dir).unwrap();
    Path::new(&dir)
        .join(format_upscaled_thumbnail_filename(
            collection_element_id.value,
            scale,
        ))
        .to_string_lossy()
        .to_string()
}

/// 高画質化されたサムネイルのパスを取得（最高の倍率を返す）
pub fn get_upscaled_thumbnail_path(
    handle: &Arc<AppHandle>,
    collection_element_id: &Id<CollectionElement>,
) -> String {
    let level = get_upscale_level(handle, collection_element_id);
    if level > 0 {
        get_upscaled_thumbnail_path_with_scale(handle, collection_element_id, level)
    } else {
        // 高画質版がない場合は元のサムネイルパスを返す
        get_thumbnail_path(handle, collection_element_id)
    }
}

/// 現在の高画質化レベルを取得（0: 未処理, 2: 2x済み, 4: 4x済み）
pub fn get_upscale_level(
    handle: &Arc<AppHandle>,
    collection_element_id: &Id<CollectionElement>,
) -> i32 {
    // 4x → 2x の順でチェック（高い倍率を優先）
    let path_4x = get_upscaled_thumbnail_path_with_scale(handle, collection_element_id, 4);
    if std::path::Path::new(&path_4x).exists() {
        return 4;
    }
    let path_2x = get_upscaled_thumbnail_path_with_scale(handle, collection_element_id, 2);
    if std::path::Path::new(&path_2x).exists() {
        return 2;
    }
    0
}

/// サムネイルをArtCNNで高画質化
/// target_scale: 目標倍率（2 または 4）
pub fn upscale_thumbnail(
    handle: &Arc<AppHandle>,
    collection_element_id: &Id<CollectionElement>,
    target_scale: i32,
) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let current_level = get_upscale_level(handle, collection_element_id);

    // 既に目標倍率以上の場合はスキップ
    if should_skip_upscale(current_level, target_scale) {
        return Ok(tauri::async_runtime::spawn(async { Ok(()) }));
    }

    // 入力パスを決定（前段階の画像を使用）
    let input_path = if current_level == 0 {
        get_thumbnail_path(handle, collection_element_id)
    } else {
        get_upscaled_thumbnail_path_with_scale(handle, collection_element_id, current_level)
    };

    // 次の倍率を計算
    let next_scale = calculate_next_scale(current_level);
    let output_path =
        get_upscaled_thumbnail_path_with_scale(handle, collection_element_id, next_scale);

    // 入力ファイルが存在しない場合はエラー
    if !std::path::Path::new(&input_path).exists() {
        return Err(anyhow::anyhow!("Input thumbnail not found: {}", input_path));
    }

    // モデルのパス
    let model_path = "E:\\MyArtCNN\\models\\8x64\\artcnn_model.pth".to_string();

    let (mut rx, _) = handle
        .shell()
        .sidecar("artcnn_upscale")?
        .args(vec![
            "-i",
            &input_path,
            "-o",
            &output_path,
            "-m",
            &model_path,
            "--layers",
            "8",
            "--channels",
            "64",
            "--scale",
            "2",
        ])
        .spawn()?;

    let handle: JoinHandle<anyhow::Result<()>> = tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Terminated(status) => {
                    if status.code == Some(0) {
                        return Ok(());
                    } else {
                        return Err(anyhow::anyhow!(
                            "ArtCNN failed with exit code: {:?}",
                            status.code
                        ));
                    }
                }
                CommandEvent::Stderr(line) => {
                    eprintln!("{}", String::from_utf8_lossy(&line));
                }
                _ => {}
            }
        }
        Ok(())
    });

    Ok(handle)
}

pub async fn get_exe_path_from_lnk(path: &str) -> anyhow::Result<String> {
    if !path.to_lowercase().ends_with("lnk") {
        return Err(anyhow::anyhow!("filepath is not ends with lnk"));
    }
    let metadatas = get_lnk_metadatas(vec![path])?;
    if let Some(meta) = metadatas.get(path) {
        return Ok(meta.path.clone());
    } else {
        return Err(anyhow::anyhow!("cannot get lnk metadata"));
    }
}
