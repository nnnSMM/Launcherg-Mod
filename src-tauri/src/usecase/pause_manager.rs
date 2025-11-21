use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct PauseManager {
    is_paused: Arc<Mutex<bool>>,
    is_tracking: Arc<Mutex<bool>>,
}

impl PauseManager {
    pub fn new() -> Self {
        Self {
            is_paused: Arc::new(Mutex::new(false)),
            is_tracking: Arc::new(Mutex::new(false)),
        }
    }

    pub fn is_paused(&self) -> bool {
        *self.is_paused.lock().unwrap()
    }

    pub fn set_paused(&self, paused: bool) {
        *self.is_paused.lock().unwrap() = paused;
    }

    pub fn is_tracking(&self) -> bool {
        *self.is_tracking.lock().unwrap()
    }

    pub fn set_tracking(&self, tracking: bool) {
        *self.is_tracking.lock().unwrap() = tracking;
        // Reset pause state when tracking changes
        if !tracking {
            self.set_paused(false);
        }
    }

    pub fn toggle(&self) -> Result<bool, String> {
        if !self.is_tracking() {
            return Err("No active game tracking session".to_string());
        }

        let mut paused = self.is_paused.lock().unwrap();
        *paused = !*paused;
        Ok(*paused)
    }
}
