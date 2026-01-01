use chrono::Local;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::domain::{collection::CollectionElement, Id};

#[derive(FromRow)]
pub struct CollectionElementTable {
    pub id: i32,
    pub gamename: String,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: i32,
    pub exe_path: Option<String>,
    pub lnk_path: Option<String>,
    pub install_at: Option<NaiveDateTime>,
    pub last_play_at: Option<NaiveDateTime>,
    pub like_at: Option<NaiveDateTime>,
    pub play_status: i32,
    pub total_play_time_seconds: i32,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl TryFrom<CollectionElementTable> for CollectionElement {
    type Error = anyhow::Error;
    fn try_from(st: CollectionElementTable) -> Result<Self, Self::Error> {
        Ok(CollectionElement::new(
            Id::new(st.id),
            st.gamename,
            st.gamename_ruby,
            st.brandname,
            st.brandname_ruby,
            st.sellday,
            st.is_nukige != 0,
            st.exe_path,
            st.lnk_path,
            st.install_at
                .and_then(|v| Some(v.and_utc().with_timezone(&Local))),
            st.last_play_at
                .and_then(|v| Some(v.and_utc().with_timezone(&Local))),
            st.like_at
                .and_then(|v| Some(v.and_utc().with_timezone(&Local))),
            st.play_status,
            st.total_play_time_seconds,
            st.thumbnail_width,
            st.thumbnail_height,
            st.created_at.and_utc().with_timezone(&Local),
            st.updated_at.and_utc().with_timezone(&Local),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn create_base_datetime() -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2023, 12, 25)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap()
    }

    #[test]
    fn test_collection_element_table_to_domain_conversion() {
        let table = CollectionElementTable {
            id: 123,
            gamename: "テストゲーム".to_string(),
            gamename_ruby: "てすとげーむ".to_string(),
            brandname: "テストブランド".to_string(),
            brandname_ruby: "てすとぶらんど".to_string(),
            sellday: "2023-12-25".to_string(),
            is_nukige: 0,
            exe_path: Some("C:/Games/test.exe".to_string()),
            lnk_path: None,
            install_at: None,
            last_play_at: None,
            like_at: None,
            play_status: 1,
            total_play_time_seconds: 3600,
            thumbnail_width: Some(256),
            thumbnail_height: Some(256),
            created_at: create_base_datetime(),
            updated_at: create_base_datetime(),
        };

        let domain: CollectionElement = table.try_into().unwrap();

        assert_eq!(domain.id.value, 123);
        assert_eq!(domain.gamename, "テストゲーム");
        assert_eq!(domain.is_nukige, false);
        assert_eq!(domain.play_status, 1);
        assert_eq!(domain.total_play_time_seconds, 3600);
    }

    #[test]
    fn test_collection_element_table_is_nukige_conversion() {
        let table = CollectionElementTable {
            id: 456,
            gamename: "Game".to_string(),
            gamename_ruby: "".to_string(),
            brandname: "Brand".to_string(),
            brandname_ruby: "".to_string(),
            sellday: "2024-01-01".to_string(),
            is_nukige: 1,
            exe_path: None,
            lnk_path: None,
            install_at: None,
            last_play_at: None,
            like_at: None,
            play_status: 0,
            total_play_time_seconds: 0,
            thumbnail_width: None,
            thumbnail_height: None,
            created_at: create_base_datetime(),
            updated_at: create_base_datetime(),
        };

        let domain: CollectionElement = table.try_into().unwrap();
        assert!(domain.is_nukige);
    }
}
