use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::domain::all_game_cache::AllGameCacheOne;

#[derive(FromRow)]
pub struct AllGameCacheTable {
    pub id: i32,
    pub gamename: String,
    #[allow(dead_code)]
    pub thumbnail_url: String,
    #[allow(dead_code)]
    pub created_at: NaiveDateTime,
}

impl TryFrom<AllGameCacheTable> for AllGameCacheOne {
    type Error = anyhow::Error;
    fn try_from(st: AllGameCacheTable) -> Result<Self, Self::Error> {
        Ok(AllGameCacheOne {
            id: st.id,
            gamename: st.gamename,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn create_test_table_row(id: i32, gamename: &str) -> AllGameCacheTable {
        AllGameCacheTable {
            id,
            gamename: gamename.to_string(),
            thumbnail_url: "https://example.com/thumb.jpg".to_string(),
            created_at: NaiveDate::from_ymd_opt(2023, 12, 25)
                .unwrap()
                .and_hms_opt(10, 0, 0)
                .unwrap(),
        }
    }

    #[test]
    fn test_all_game_cache_table_to_domain_conversion() {
        let table = create_test_table_row(12345, "テストゲーム");
        let domain: AllGameCacheOne = table.try_into().unwrap();

        assert_eq!(domain.id, 12345);
        assert_eq!(domain.gamename, "テストゲーム");
    }

    #[test]
    fn test_all_game_cache_table_conversion_ignores_thumbnail_and_created_at() {
        // thumbnail_url と created_at はドメインオブジェクトに含まれない
        let table = AllGameCacheTable {
            id: 99,
            gamename: "Game".to_string(),
            thumbnail_url: "https://ignored.com".to_string(),
            created_at: NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        };

        let domain: AllGameCacheOne = table.try_into().unwrap();
        assert_eq!(domain.id, 99);
        // thumbnail_url と created_at はドメインに存在しないためアサート不要
    }
}
