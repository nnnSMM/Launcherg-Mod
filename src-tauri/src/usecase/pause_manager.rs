use std::sync::{Arc, Mutex, MutexGuard};

fn lock_bool(lock: &Mutex<bool>) -> MutexGuard<'_, bool> {
    lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackingSession {
    pub game_id: i32,
    pub process_id: u32,
}

#[derive(Clone)]
pub struct PauseManager {
    is_paused: Arc<Mutex<bool>>,
    is_tracking: Arc<Mutex<bool>>,
    tracking_session: Arc<Mutex<Option<TrackingSession>>>,
}

impl PauseManager {
    pub fn new() -> Self {
        Self {
            is_paused: Arc::new(Mutex::new(false)),
            is_tracking: Arc::new(Mutex::new(false)),
            tracking_session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_paused(&self) -> bool {
        *lock_bool(&self.is_paused)
    }

    pub fn set_paused(&self, paused: bool) {
        *lock_bool(&self.is_paused) = paused;
    }

    pub fn is_tracking(&self) -> bool {
        *lock_bool(&self.is_tracking)
    }

    pub fn set_tracking(&self, tracking: bool) {
        *lock_bool(&self.is_tracking) = tracking;
        // Reset pause state when tracking changes
        if !tracking {
            self.set_paused(false);
            self.clear_tracking_session();
        }
    }

    pub fn set_tracking_session(&self, game_id: i32, process_id: u32) {
        *self
            .tracking_session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) =
            Some(TrackingSession {
                game_id,
                process_id,
            });
        self.set_tracking(true);
    }

    pub fn clear_tracking_session(&self) {
        *self
            .tracking_session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    }

    pub fn tracking_session(&self) -> Option<TrackingSession> {
        self.tracking_session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn toggle(&self) -> Result<bool, String> {
        if !self.is_tracking() {
            return Err("No active game tracking session".to_string());
        }

        let mut paused = lock_bool(&self.is_paused);
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
        manager.set_tracking_session(12, 34);
        manager.set_paused(true);
        assert!(manager.is_paused());
        assert_eq!(
            manager.tracking_session(),
            Some(TrackingSession {
                game_id: 12,
                process_id: 34
            })
        );

        manager.set_tracking(false);
        assert!(!manager.is_paused());
        assert_eq!(manager.tracking_session(), None);
    }

    #[test]
    fn test_set_tracking_session_marks_tracking() {
        let manager = PauseManager::new();

        manager.set_tracking_session(1, 2);

        assert!(manager.is_tracking());
        assert_eq!(
            manager.tracking_session(),
            Some(TrackingSession {
                game_id: 1,
                process_id: 2
            })
        );
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
