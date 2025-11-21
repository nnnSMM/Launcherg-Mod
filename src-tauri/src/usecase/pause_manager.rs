use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct PauseManager {
    is_paused: Arc<Mutex<bool>>,
}

impl PauseManager {
    pub fn new() -> Self {
        Self {
            is_paused: Arc::new(Mutex::new(false)),
        }
    }

    pub fn is_paused(&self) -> bool {
        *self.is_paused.lock().unwrap()
    }

    pub fn set_paused(&self, paused: bool) {
        *self.is_paused.lock().unwrap() = paused;
    }

    pub fn toggle(&self) -> bool {
        let mut paused = self.is_paused.lock().unwrap();
        *paused = !*paused;
        *paused
    }
}
