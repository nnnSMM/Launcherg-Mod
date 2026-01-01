#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use crate::{
        domain::all_game_cache::AllGameCacheOne, infrastructure::explorerimpl::explorer::Explorers,
        usecase::file::FileUseCase,
    };

    fn get_use_case() -> FileUseCase<Explorers> {
        FileUseCase::new(Arc::new(Explorers::new()))
    }

    const GAMENAME: &str = "gamename";

    fn get_base_cache() -> AllGameCacheOne {
        AllGameCacheOne::new(1, GAMENAME.to_string())
    }

    fn get_filepath(filename_without_extenstion: &str) -> String {
        format!("/path/to/{}.exe", filename_without_extenstion)
    }

    #[test]
    // 編集距離に基づいてupdate
    fn test_get_map_of_one_filepath_per_game_update_by_distance() {
        let u = get_use_case();
        let expect_filepath = get_filepath(GAMENAME);
        let input = vec![
            (get_base_cache(), expect_filepath.clone()),
            (
                get_base_cache(),
                get_filepath(format!("{}11", GAMENAME).as_str()),
            ),
        ];
        let mut expected_output: HashMap<i32, String> = HashMap::new();
        expected_output.insert(get_base_cache().id, expect_filepath);
        let actual = u.get_map_of_one_filepath_per_game(input);
        assert_eq!(actual, expected_output);
    }

    #[test]
    // ignore_word に基づいてupdate
    fn test_get_map_of_one_filepath_per_game_update_by_ignore_word() {
        let u = get_use_case();
        let expect_filepath = get_filepath("まったく関係のない名前あああああああ");
        let input = vec![
            (get_base_cache(), expect_filepath.clone()),
            (
                get_base_cache(),
                // 編集距離を近づけるために GAMENAME を入れてる
                get_filepath(format!("{}-{}", GAMENAME, "ファイル設定").as_str()),
            ),
        ];
        let mut expected_output: HashMap<i32, String> = HashMap::new();
        expected_output.insert(get_base_cache().id, expect_filepath);
        let actual = u.get_map_of_one_filepath_per_game(input);
        assert_eq!(actual, expected_output);
    }

    #[test]
    // should_update_word に基づいてupdate
    fn test_get_map_of_one_filepath_per_game_update_by_should_update_word() {
        let u = get_use_case();
        let expect_filepath = get_filepath("実行");
        let input = vec![
            (get_base_cache(), expect_filepath.clone()),
            (
                get_base_cache(),
                // 編集距離を近づけるために GAMENAME を入れてる
                get_filepath(GAMENAME),
            ),
        ];
        let mut expected_output: HashMap<i32, String> = HashMap::new();
        expected_output.insert(get_base_cache().id, expect_filepath);
        let actual = u.get_map_of_one_filepath_per_game(input);
        assert_eq!(actual, expected_output);
    }

    #[test]
    fn test_get_map_of_one_filepath_per_game_empty_input() {
        let u = get_use_case();
        let input: Vec<(AllGameCacheOne, String)> = vec![];
        let actual = u.get_map_of_one_filepath_per_game(input);
        assert!(actual.is_empty());
    }

    #[test]
    fn test_get_map_of_one_filepath_per_game_single_item() {
        let u = get_use_case();
        let filepath = get_filepath(GAMENAME);
        let input = vec![(get_base_cache(), filepath.clone())];
        let mut expected: HashMap<i32, String> = HashMap::new();
        expected.insert(get_base_cache().id, filepath);
        let actual = u.get_map_of_one_filepath_per_game(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_map_of_one_filepath_per_game_multiple_games() {
        // 異なるゲームIDのファイルは全て保持される
        let u = get_use_case();
        let cache1 = AllGameCacheOne::new(1, "GameA".to_string());
        let cache2 = AllGameCacheOne::new(2, "GameB".to_string());
        let cache3 = AllGameCacheOne::new(3, "GameC".to_string());

        let input = vec![
            (cache1, "/path/to/game_a.exe".to_string()),
            (cache2, "/path/to/game_b.exe".to_string()),
            (cache3, "/path/to/game_c.exe".to_string()),
        ];

        let actual = u.get_map_of_one_filepath_per_game(input);

        assert_eq!(actual.len(), 3);
        assert!(actual.contains_key(&1));
        assert!(actual.contains_key(&2));
        assert!(actual.contains_key(&3));
    }

    #[test]
    fn test_get_map_of_one_filepath_per_game_ignore_install_tool() {
        // "inst" を含むパスは無視される
        let u = get_use_case();
        let expect_filepath = get_filepath(GAMENAME);
        let input = vec![
            (get_base_cache(), get_filepath("install_tool")),
            (get_base_cache(), expect_filepath.clone()),
        ];

        let actual = u.get_map_of_one_filepath_per_game(input);
        assert_eq!(actual.get(&1), Some(&expect_filepath));
    }

    #[test]
    fn test_get_map_of_one_filepath_per_game_prefer_64bit() {
        // "64" を含むパスが優先される
        let u = get_use_case();
        let expect_filepath = get_filepath("game64");
        let input = vec![
            (get_base_cache(), get_filepath("game32")),
            (get_base_cache(), expect_filepath.clone()),
        ];

        let actual = u.get_map_of_one_filepath_per_game(input);
        assert_eq!(actual.get(&1), Some(&expect_filepath));
    }
}
