#[cfg(test)]
mod tests {
    use crate::usecase::file::FileUseCase;
    use std::sync::Arc;

    // FileUseCase は R: ExplorersExt を要求するため、テスト用のダミー実装を用意する
    use crate::domain::explorer::file::FileExplorer;
    use crate::infrastructure::explorerimpl::explorer::ExplorersExt;

    use async_trait::async_trait;

    struct DummyFileExplorer;
    #[async_trait]
    impl FileExplorer for DummyFileExplorer {
        fn get_save_image_path(
            &self,
            _handle: &Arc<tauri::AppHandle>,
            _id: i32,
        ) -> anyhow::Result<String> {
            unimplemented!()
        }
        fn save_base64_image(&self, _path: &str, _base64_image: String) -> anyhow::Result<()> {
            unimplemented!()
        }
        fn get_save_screenshot_path_by_name(
            &self,
            _handle: &Arc<tauri::AppHandle>,
            _name: &str,
        ) -> anyhow::Result<String> {
            unimplemented!()
        }
        fn get_md_path(&self, _handle: &Arc<tauri::AppHandle>, _id: i32) -> anyhow::Result<String> {
            unimplemented!()
        }
        fn delete_file(&self, _path: &str) -> anyhow::Result<()> {
            unimplemented!()
        }
    }

    struct DummyExplorers;
    impl ExplorersExt for DummyExplorers {
        type FileExplorer = DummyFileExplorer;
        fn file_explorer(&self) -> &Self::FileExplorer {
            unimplemented!()
        }
    }

    #[test]
    fn test_dummy() {
        // コンパイルが通ることの確認用
        let _ = FileUseCase::new(Arc::new(DummyExplorers));
    }
}
