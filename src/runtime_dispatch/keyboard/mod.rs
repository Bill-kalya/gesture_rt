#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

use enigo::*;

pub struct KeyboardDispatcher;

impl KeyboardDispatcher {
    pub fn new() -> Self {
        Self
    }
    
    pub fn alt_tab(&self) {
        let mut enigo = Enigo::new();
        enigo.key_down(Key::Alt);
        enigo.key_click(Key::Tab);
        enigo.key_up(Key::Alt);
    }
    
    pub fn media_next(&self) {
        let mut enigo = Enigo::new();
        enigo.key_click(Key::MediaNextTrack);
    }
    
    pub fn media_previous(&self) {
        let mut enigo = Enigo::new();
        enigo.key_click(Key::MediaPrevTrack);
    }
    
    pub fn media_play_pause(&self) {
        let mut enigo = Enigo::new();
        enigo.key_click(Key::MediaPlayPause);
    }
    
    pub fn volume_up(&self) {
        let mut enigo = Enigo::new();
        enigo.key_click(Key::VolumeUp);
    }
    
    pub fn volume_down(&self) {
        let mut enigo = Enigo::new();
        enigo.key_click(Key::VolumeDown);
    }
}