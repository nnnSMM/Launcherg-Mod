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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_unpaused_not_tracking() {
        let manager = PauseManager::new();
        assert!(!manager.is_paused());
        assert!(!manager.is_tracking());
    }

    #[test]
    fn test_set_paused() {
        let manager = PauseManager::new();

        manager.set_paused(true);
        assert!(manager.is_paused());

        manager.set_paused(false);
        assert!(!manager.is_paused());
    }

    #[test]
    fn test_set_tracking_resets_pause_when_stopped() {
        let manager = PauseManager::new();

        manager.set_tracking(true);
        manager.set_paused(true);
        assert!(manager.is_paused());

        manager.set_tracking(false);
        assert!(!manager.is_paused());
    }

    #[test]
    fn test_toggle_fails_when_not_tracking() {
        let manager = PauseManager::new();

        let result = manager.toggle();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No active game tracking session");
    }

    #[test]
    fn test_toggle_switches_pause_state() {
        let manager = PauseManager::new();
        manager.set_tracking(true);

        let result1 = manager.toggle();
        assert!(result1.is_ok());
        assert!(result1.unwrap()); // false -> true

        let result2 = manager.toggle();
        assert!(result2.is_ok());
        assert!(!result2.unwrap()); // true -> false
    }
}
