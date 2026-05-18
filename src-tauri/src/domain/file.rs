pub struct File {}
pub struct LnkMetadata {
    pub path: String,
    pub icon: String,
    #[allow(dead_code)]
    pub icon_index: i32,
}

use std::{collections::HashMap, fs, path::Path, sync::Arc};

use chrono::{DateTime, Local};
use image::{codecs::jpeg::JpegEncoder, imageops::FilterType, ColorType, GenericImageView};
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

const NOT_GAME_EQUALLY_WORD: [&str; 3] = ["bgi", "siglusengine", "nscripter"];

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
/// ファイル名がインストーラーやマニュアル等に該当するか（完全に除外すべきワード）
fn not_game_strictly(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    NOT_GAME_TERMS
        .iter()
        .any(|term| filename_lower.contains(term))
}

/// ファイル名がBGI等の汎用エンジン名か（ファイル名マッチのみスキップ、親フォルダ推定は継続）
fn is_engine_name(filename: &str) -> bool {
    let filename_lower = filename.to_lowercase();
    NOT_GAME_EQUALLY_WORD
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

const BASE_TITLE_SEPARATORS: [&str; 11] = [
    " -", " ～", " ~", "（", "(", "【", "[", "「", "『", "：", ":",
];

/// ファイル名から不要なワードを削除
fn remove_word(filename: &str) -> String {
    REMOVE_WORDS
        .iter()
        .fold(filename.to_string(), |acc, word| acc.replace(word, ""))
}

fn get_base_title(gamename: &str) -> &str {
    let index = BASE_TITLE_SEPARATORS
        .iter()
        .filter_map(|separator| gamename.find(separator))
        .filter(|index| *index > 0)
        .min();

    match index {
        Some(index) => gamename[..index].trim(),
        None => gamename.trim(),
    }
}

fn get_game_path_part_score(path_part: &str, gamename: &str) -> f32 {
    let score = get_comparable_distance(path_part, gamename);
    let base_title = get_base_title(gamename);
    if base_title == path_part && base_title.len() < gamename.trim().len() && base_title.len() > 5 {
        return 2.0 + score;
    }
    score
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

fn expand_env_vars(path: &str) -> String {
    use windows::Win32::System::Environment::ExpandEnvironmentStringsW;
    let input = path.to_wide_null_terminated();
    let mut buffer: Vec<u16> = vec![0; 261];
    unsafe {
        let size = ExpandEnvironmentStringsW(PCWSTR::from_raw(input.as_ptr()), Some(&mut buffer));
        if size > 261 {
            buffer = vec![0; size as usize];
            ExpandEnvironmentStringsW(PCWSTR::from_raw(input.as_ptr()), Some(&mut buffer));
        }
        PCWSTR::from_raw(buffer.as_ptr())
            .to_string()
            .unwrap_or_else(|_| path.to_string())
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
                let path =
                    expand_env_vars(&PCWSTR::from_raw(target_path_vec.as_mut_ptr()).to_string()?);

                let mut icon_index = 0i32;
                shell_link.GetIconLocation(target_path_slice, &mut icon_index)?;
                let icon =
                    expand_env_vars(&PCWSTR::from_raw(target_path_vec.as_mut_ptr()).to_string()?);

                metadatas.insert(
                    file_path,
                    LnkMetadata {
                        path,
                        icon,
                        icon_index,
                    },
                );
            } else if file_path.to_lowercase().ends_with("url") {
                let icon_file = get_url_file_icon_path(file_path)?;

                metadatas.insert(
                    file_path,
                    LnkMetadata {
                        path: file_path.to_string(),
                        icon: expand_env_vars(&icon_file.unwrap_or_default()),
                        icon_index: 0,
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
    // 親フォルダ名（直接の親）
    let parent = Path::new(&filepath)
        .parent()
        .and_then(|v| {
            v.file_name()
                .and_then(|name| Some(normalize(&name.to_string_lossy().to_string())))
        })
        .ok_or(anyhow::anyhow!("can not get parent"))?;

    // 祖父母フォルダ名／ブランド名相当のフォルダ（親の親）
    let grandparent = Path::new(&filepath)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|v| {
            v.file_name()
                .and_then(|name| Some(normalize(&name.to_string_lossy().to_string())))
        });

    let filename: String =
        get_file_name_without_extension(filepath).ok_or(anyhow::anyhow!("can not get filename"))?;
    let filename = normalize(&filename);

    // インストーラー・マニュアル等は親フォルダ名に関わらず完全除外
    if not_game_strictly(&filename) {
        return Ok(vec![]);
    }

    // BGI・SiglusEngine等の汎用エンジン名はファイル名判定のみスキップ、親フォルダ名判定は継続
    let is_skip_filename_check =
        is_engine_name(&filename) || filename == "game" || filename == "start";

    let filename = remove_word(&filename);

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
            val = val.max(get_game_path_part_score(&filename, &pair.gamename));
        }

        // 親フォルダ名でゲーム名と比較
        val = val.max(get_game_path_part_score(&parent, &pair.gamename));
        // 祖父母フォルダ名でゲーム名と比較（ブランド名フォルダの下にゲームフォルダがある場合のケア）
        if let Some(ref gp) = grandparent {
            val = val.max(get_game_path_part_score(gp, &pair.gamename));
        }
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
        assert!(not_game_strictly("インストール"));
        assert!(not_game_strictly("アンインストール"));
        assert!(not_game_strictly("install"));
        assert!(not_game_strictly("uninstall"));
    }

    #[test]
    fn test_not_game_returns_true_for_manual() {
        assert!(not_game_strictly("マニュアル"));
        assert!(not_game_strictly("はじめに"));
        assert!(not_game_strictly("サポート"));
    }

    #[test]
    fn test_not_game_returns_false_for_game() {
        assert!(!not_game_strictly("game"));
        assert!(!not_game_strictly("start"));
        assert!(!not_game_strictly("サクラノ詩"));
    }

    #[test]
    fn test_not_game_case_insensitive() {
        // 大文字小文字を区別しない
        assert!(not_game_strictly("INSTALL"));
        assert!(not_game_strictly("Install"));
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
            )]
            .into_iter()
            .map(|pair| AllGameCacheOne {
                id: pair.id,
                gamename: normalize(&pair.gamename),
            })
            .collect(),
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
    // 汎用エンジン名ファイルでの親フォルダ名推定のテスト（新規）
    // ========================================

    // RED: BGI.exeはnot_game対象だが、親フォルダ名「サクラノ詩」から正しく推定できるべき
    #[test]
    fn test_get_game_candidates_bgi_from_parent_folder() {
        let cache = vec![
            AllGameCacheOne::new(4529, "サクラノ詩 -櫻の森の上を舞う-".to_string()),
            AllGameCacheOne::new(11396, "サクラノ詩 春ノ雪".to_string()),
            AllGameCacheOne::new(
                39075,
                "サクラノ詩 -櫻の森の上を舞う- 10th Anniversary Edition".to_string(),
            ),
        ];
        // パス：e:\VisualNovel\枕\サクラノ詩\BGI.exe
        let res = get_game_candidates_by_exe_path(
            &cache,
            "E:\\VisualNovel\\枕\\サクラノ詩\\BGI.exe",
            0.8,
            1,
        )
        .unwrap();
        assert!(
            !res.is_empty(),
            "BGI.exeの親フォルダからサクラノ詩が推定できるべき"
        );
        assert_eq!(res.first().unwrap().id, 4529);
    }

    #[test]
    fn test_get_game_candidates_bgi_from_exact_derived_parent_folder() {
        let cache = vec![
            AllGameCacheOne::new(4529, "サクラノ詩 -櫻の森の上を舞う-".to_string()),
            AllGameCacheOne::new(11396, "サクラノ詩 春ノ雪".to_string()),
        ];
        let res = get_game_candidates_by_exe_path(
            &cache,
            "E:\\VisualNovel\\枕\\サクラノ詩 春ノ雪\\BGI.exe",
            0.8,
            1,
        )
        .unwrap();

        assert_eq!(res.first().unwrap().id, 11396);
    }

    // RED: SiglusEngine.exe はnot_game対象ではないが、親フォルダ名から正しく推定できるべき
    #[test]
    fn test_get_game_candidates_siglusengine_from_parent_folder() {
        let cache = vec![AllGameCacheOne::new(
            29016,
            "summer pockets reflection blue".to_string(),
        )];
        // パス：e:\VisualNovel\key\Summer Pockets REFLECTION BLUE\SiglusEngine.exe
        let res = get_game_candidates_by_exe_path(
            &cache,
            "E:\\VisualNovel\\key\\Summer Pockets REFLECTION BLUE\\SiglusEngine.exe",
            0.8,
            1,
        )
        .unwrap();
        assert!(
            !res.is_empty(),
            "SiglusEngine.exeの親フォルダからSummer Pocketsが推定できるべき"
        );
        assert_eq!(res.first().unwrap().id, 29016);
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

pub fn find_icon_in_dir_recursive(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    let parent = path.parent()?;

    for entry in WalkDir::new(parent).into_iter().flatten() {
        let p = entry.path();
        if p.is_file()
            && p.extension()
                .map_or(false, |ext| ext.to_string_lossy().to_lowercase() == "ico")
        {
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
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

    // EXE/LNKなどの場合、子孫フォルダ含め .ico があればそれを優先
    if let Some(sibling_ico) = find_icon_in_dir_recursive(file_path) {
        return save_ico_to_png(&sibling_ico, &save_png_path);
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
    let file_path_cloned = file_path.to_string();
    let handle_cloned = handle.clone();

    let join_handle: JoinHandle<anyhow::Result<()>> = tauri::async_runtime::spawn(async move {
        let spawn_result = handle_cloned
            .shell()
            .sidecar("extract-icon")
            .and_then(|cmd| {
                cmd.args(vec!["48", &file_path_cloned, &save_png_path_cloned])
                    .spawn()
            });

        let (mut rx, _child) = match spawn_result {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[save_exe_file_png] extract-icon spawn failed: {}", e);
                return save_default_icon(&save_png_path_cloned)?.await?;
            }
        };

        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Terminated(_) => {
                    if std::path::Path::new(&save_png_path_cloned).exists() {
                        return Ok(());
                    }
                    return save_default_icon(&save_png_path_cloned)?.await?;
                }
                _ => {}
            }
        }
        // イベントループが終了してもTerminatedが来なかった場合
        if std::path::Path::new(&save_png_path_cloned).exists() {
            return Ok(());
        }
        save_default_icon(&save_png_path_cloned)?.await?
    });

    Ok(join_handle)
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

const SCREENSHOTS_ROOT_DIR: &str = "game-memos";
const SCREENSHOT_THUMBNAILS_DIR: &str = "thumbnails";
const SCREENSHOT_THUMBNAIL_MAX_WIDTH: u32 = 640;
const SCREENSHOT_THUMBNAIL_QUALITY: u8 = 78;

pub fn get_screenshot_file_path(
    save_root_dir: &str,
    game_id: i32,
    filename: &str,
) -> std::path::PathBuf {
    Path::new(save_root_dir)
        .join(SCREENSHOTS_ROOT_DIR)
        .join(game_id.to_string())
        .join(filename)
}

pub fn get_screenshot_thumbnail_path(
    save_root_dir: &str,
    game_id: i32,
    filename: &str,
) -> std::path::PathBuf {
    let dir = Path::new(save_root_dir)
        .join(SCREENSHOTS_ROOT_DIR)
        .join(game_id.to_string())
        .join(SCREENSHOT_THUMBNAILS_DIR);

    let safe_name = filename.replace(['\\', '/'], "_");
    dir.join(format!("{safe_name}.thumb.jpg"))
}

pub fn ensure_screenshot_thumbnail(
    save_root_dir: &str,
    game_id: i32,
    filename: &str,
) -> anyhow::Result<Option<String>> {
    let source_path = get_screenshot_file_path(save_root_dir, game_id, filename);
    if !source_path.exists() {
        return Ok(None);
    }

    let thumbnail_path = get_screenshot_thumbnail_path(save_root_dir, game_id, filename);
    if thumbnail_path.exists() {
        return Ok(Some(thumbnail_path.to_string_lossy().to_string()));
    }

    if let Some(parent) = thumbnail_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let img = image::open(&source_path)?;
    let (width, height) = img.dimensions();
    let resized = if width > SCREENSHOT_THUMBNAIL_MAX_WIDTH {
        let resized_height = ((height as u64 * SCREENSHOT_THUMBNAIL_MAX_WIDTH as u64
            + (width as u64 / 2))
            / width as u64) as u32;
        img.resize_exact(
            SCREENSHOT_THUMBNAIL_MAX_WIDTH,
            resized_height.max(1),
            FilterType::Triangle,
        )
    } else {
        img
    };

    let rgb = resized.to_rgb8();
    let mut file = fs::File::create(&thumbnail_path)?;
    let mut encoder = JpegEncoder::new_with_quality(&mut file, SCREENSHOT_THUMBNAIL_QUALITY);
    encoder.encode(&rgb, rgb.width(), rgb.height(), ColorType::Rgb8)?;

    Ok(Some(thumbnail_path.to_string_lossy().to_string()))
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
