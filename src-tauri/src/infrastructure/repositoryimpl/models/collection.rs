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
