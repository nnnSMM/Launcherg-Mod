use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::domain::{collection::NewCollectionElementDetail, Id};

#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollectionElementDetail {
    pub collection_element_id: i32,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
}

impl From<CreateCollectionElementDetail> for NewCollectionElementDetail {
    fn from(c: CreateCollectionElementDetail) -> Self {
        NewCollectionElementDetail::new(
            Id::new(c.collection_element_id),
            c.gamename_ruby,
            c.brandname,
            c.brandname_ruby,
            c.sellday,
            c.is_nukige,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_collection_element_detail_conversion() {
        let create_detail = CreateCollectionElementDetail::new(
            123,
            "げーむるび".to_string(),
            "ブランド名".to_string(),
            "ぶらんどるび".to_string(),
            "2023-12-01".to_string(),
            true,
        );

        let new_detail: NewCollectionElementDetail = create_detail.into();

        assert_eq!(new_detail.collection_element_id.value, 123);
        assert_eq!(new_detail.gamename_ruby, "げーむるび");
        assert_eq!(new_detail.brandname, "ブランド名");
        assert_eq!(new_detail.brandname_ruby, "ぶらんどるび");
        assert_eq!(new_detail.sellday, "2023-12-01");
        assert!(new_detail.is_nukige);
    }

    #[test]
    fn test_create_collection_element_detail_conversion_non_nukige() {
        let create_detail = CreateCollectionElementDetail::new(
            456,
            "ruby".to_string(),
            "Brand".to_string(),
            "brand".to_string(),
            "2024-01-15".to_string(),
            false,
        );

        let new_detail: NewCollectionElementDetail = create_detail.into();

        assert_eq!(new_detail.collection_element_id.value, 456);
        assert!(!new_detail.is_nukige);
    }
}
