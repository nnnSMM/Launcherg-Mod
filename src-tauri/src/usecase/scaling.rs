use crate::infrastructure::windowsimpl::scaling::processor::ScalingProcessor;
use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

#[derive(Clone)]
pub struct ScalingUseCase {
    processor: Arc<Mutex<ScalingProcessor>>,
}

impl ScalingUseCase {
    pub fn new() -> Self {
        Self {
            processor: Arc::new(Mutex::new(ScalingProcessor::new())),
        }
    }

    pub fn start_scaling(&self, shader_name: String) -> Result<()> {
        let mut processor = self.processor.lock().unwrap();
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.0 == 0 {
            return Err(anyhow!("No foreground window found"));
        }
        processor.start(hwnd, shader_name)
    }

    pub fn stop_scaling(&self) -> Result<()> {
        let mut processor = self.processor.lock().unwrap();
        processor.stop()
    }

    pub fn is_scaling(&self) -> bool {
        let processor = self.processor.lock().unwrap();
        processor.is_running()
    }

    pub fn toggle_scaling(&self, shader_name: String) -> Result<bool> {
        if self.is_scaling() {
            self.stop_scaling()?;
            Ok(false)
        } else {
            self.start_scaling(shader_name)?;
            Ok(true)
        }
    }
}
