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
