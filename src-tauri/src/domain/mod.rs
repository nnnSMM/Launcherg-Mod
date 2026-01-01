use std::marker::PhantomData;

use derive_new::new;
use serde::{Deserialize, Serialize};

pub mod all_game_cache;
pub mod collection;
pub mod distance;
pub mod explored_cache;
pub mod file;

pub mod process;

pub mod explorer;
pub mod repository;
pub mod windows;

#[derive(new, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Id<T> {
    pub value: i32,
    _marker: PhantomData<T>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    struct TestEntity;

    #[test]
    fn test_id_new() {
        let id: Id<TestEntity> = Id::new(42);
        assert_eq!(id.value, 42);
    }

    #[test]
    fn test_id_clone() {
        let id1: Id<TestEntity> = Id::new(100);
        let id2 = id1.clone();
        assert_eq!(id1.value, id2.value);
    }

    #[test]
    fn test_id_copy() {
        let id1: Id<TestEntity> = Id::new(50);
        let id2 = id1;
        assert_eq!(id1.value, id2.value);
    }

    #[test]
    fn test_id_serialize_deserialize() {
        let id: Id<TestEntity> = Id::new(123);
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: Id<TestEntity> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id.value, deserialized.value);
    }

    #[test]
    fn test_id_debug_format() {
        let id: Id<TestEntity> = Id::new(999);
        let debug_str = format!("{:?}", id);
        assert!(debug_str.contains("999"));
    }

    #[test]
    fn test_id_json_format() {
        let id: Id<TestEntity> = Id::new(456);
        let json = serde_json::to_string(&id).unwrap();
        // JSONフォーマットを確認
        assert!(json.contains("\"value\":456"));
    }
}
