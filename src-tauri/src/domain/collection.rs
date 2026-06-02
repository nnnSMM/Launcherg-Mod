use chrono::{DateTime, Local};
use derive_new::new;
use serde::{Deserialize, Serialize};

use super::Id;

#[allow(dead_code)]
#[derive(new, Debug)]
pub struct NewCollection {
    pub name: String,
}
#[allow(clippy::too_many_arguments)]
#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionElement {
    pub id: Id<CollectionElement>,
    pub gamename: String,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
    pub exe_path: Option<String>,
    pub lnk_path: Option<String>,
    pub install_at: Option<DateTime<Local>>,
    pub first_play_at: Option<DateTime<Local>>,
    pub last_play_at: Option<DateTime<Local>>,
    pub like_at: Option<DateTime<Local>>,
    pub play_status: i32,
    pub total_play_time_seconds: i32,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(new, Debug)]
pub struct NewCollectionElement {
    pub id: Id<CollectionElement>,
    pub gamename: String,
    pub exe_path: Option<String>,
    pub lnk_path: Option<String>,
    pub install_at: Option<DateTime<Local>>,
    // play_status は初期登録時はデフォルト0とし、更新で対応するためここには含めない
}

#[derive(new, Debug, Clone, Serialize, Deserialize)]
pub struct NewCollectionElementDetail {
    pub collection_element_id: Id<CollectionElement>,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_collection_element_creation() {
        let id = Id::new(123);
        let element = NewCollectionElement::new(
            id,
            "テストゲーム".to_string(),
            Some("C:/Games/test.exe".to_string()),
            None,
            None,
        );

        assert_eq!(element.id.value, 123);
        assert_eq!(element.gamename, "テストゲーム");
        assert_eq!(element.exe_path, Some("C:/Games/test.exe".to_string()));
        assert!(element.lnk_path.is_none());
        assert!(element.install_at.is_none());
    }

    #[test]
    fn test_new_collection_element_detail_creation() {
        let detail = NewCollectionElementDetail::new(
            Id::new(456),
            "てすとげーむ".to_string(),
            "テストブランド".to_string(),
            "てすとぶらんど".to_string(),
            "2023-12-25".to_string(),
            false,
        );

        assert_eq!(detail.collection_element_id.value, 456);
        assert_eq!(detail.gamename_ruby, "てすとげーむ");
        assert_eq!(detail.brandname, "テストブランド");
        assert!(!detail.is_nukige);
    }

    #[test]
    fn test_new_collection_element_detail_serialization() {
        let detail = NewCollectionElementDetail::new(
            Id::new(789),
            "ruby".to_string(),
            "Brand".to_string(),
            "brand".to_string(),
            "2024-01-01".to_string(),
            true,
        );

        // JSON serializeができることを確認
        let json = serde_json::to_string(&detail).unwrap();
        assert!(json.contains("789"));
        assert!(json.contains("ruby"));
        assert!(json.contains("Brand"));

        // deserializeも確認
        let deserialized: NewCollectionElementDetail = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.collection_element_id.value, 789);
        assert!(deserialized.is_nukige);
    }

    #[test]
    fn test_new_collection_creation() {
        let collection = NewCollection::new("マイコレクション".to_string());
        assert_eq!(collection.name, "マイコレクション");
    }
}
