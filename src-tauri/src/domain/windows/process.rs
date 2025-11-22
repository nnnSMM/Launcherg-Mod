use async_trait::async_trait;

#[async_trait]
pub trait ProcessWindows {
    fn save_screenshot_by_process_id(&self, process_id: u32, filepath: &str) -> anyhow::Result<()>;
    #[allow(dead_code)]
    fn save_top_window_screenshot(&self, filepath: &str) -> anyhow::Result<()>;
    #[allow(dead_code)]
    fn get_top_window_name(&self) -> anyhow::Result<String>;
}
