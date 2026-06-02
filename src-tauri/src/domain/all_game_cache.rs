#[derive(derive_new::new, Debug, Clone)]
pub struct AllGameCacheOne {
    pub id: i32,
    pub gamename: String,
}

#[derive(derive_new::new, Debug, Clone)]
pub struct NewAllGameCacheOne {
    pub id: i32,
    pub gamename: String,
    pub thumbnail_url: String,
}

#[derive(Debug, Clone)]
pub struct AllGameCacheOneWithThumbnailUrl {
    pub id: i32,
    pub gamename: String,
    pub thumbnail_url: String,
}

pub type AllGameCache = Vec<AllGameCacheOne>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_game_cache_one_creation() {
        let cache = AllGameCacheOne::new(12345, "テストゲーム".to_string());
        assert_eq!(cache.id, 12345);
        assert_eq!(cache.gamename, "テストゲーム");
    }

    #[test]
    fn test_new_all_game_cache_one_creation() {
        let cache = NewAllGameCacheOne::new(
            67890,
            "Another Game".to_string(),
            "https://example.com/thumbnail.jpg".to_string(),
        );
        assert_eq!(cache.id, 67890);
        assert_eq!(cache.gamename, "Another Game");
        assert_eq!(cache.thumbnail_url, "https://example.com/thumbnail.jpg");
    }

    #[test]
    fn test_all_game_cache_vec_operations() {
        let cache: AllGameCache = vec![
            AllGameCacheOne::new(1, "Game A".to_string()),
            AllGameCacheOne::new(2, "Game B".to_string()),
        ];

        assert_eq!(cache.len(), 2);
        assert_eq!(cache[0].id, 1);
        assert_eq!(cache[1].gamename, "Game B");
    }

    #[test]
    fn test_all_game_cache_one_clone() {
        let original = AllGameCacheOne::new(100, "Original".to_string());
        let cloned = original.clone();

        assert_eq!(original.id, cloned.id);
        assert_eq!(original.gamename, cloned.gamename);
    }
}
