#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;

use enigo::*;

#[derive(Clone)]
pub struct MouseDispatcher {
    sensitivity: f32,
}

impl MouseDispatcher {
    pub fn new() -> Self {
        Self {
            sensitivity: 1.0,
        }
    }
    
    pub fn set_sensitivity(&mut self, sensitivity: f32) {
        self.sensitivity = sensitivity.clamp(0.1, 5.0);
    }
    
    pub fn move_relative(&self, dx: f32, dy: f32) {
        let dx = (dx * self.sensitivity) as i32;
        let dy = (dy * self.sensitivity) as i32;
        
        #[cfg(target_os = "windows")]
        {
            windows::send_mouse_move_relative(dx, dy);
        }
        #[cfg(target_os = "linux")]
        {
            linux::send_mouse_move_relative(dx, dy);
        }
        #[cfg(target_os = "macos")]
        {
            macos::send_mouse_move_relative(dx, dy);
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            let mut enigo = Enigo::new();
            enigo.mouse_move_relative(dx, dy);
        }
    }
    
    pub fn click_left(&self) {
        #[cfg(target_os = "windows")]
        {
            windows::send_mouse_click_left();
        }
        #[cfg(target_os = "linux")]
        {
            linux::send_mouse_click_left();
        }
        #[cfg(target_os = "macos")]
        {
            macos::send_mouse_click_left();
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            let mut enigo = Enigo::new();
            enigo.mouse_click(MouseButton::Left);
        }
    }
    
    pub fn click_right(&self) {
        #[cfg(target_os = "linux")]
        {
            linux::send_mouse_click_right();
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            let mut enigo = Enigo::new();
            enigo.mouse_click(MouseButton::Right);
        }
    }
}

impl Default for MouseDispatcher {
    fn default() -> Self {
        Self::new()
    }
}