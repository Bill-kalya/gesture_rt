use super::keyboard::KeyboardDispatcher;

pub struct MediaDispatcher {
    keyboard: KeyboardDispatcher,
}

impl MediaDispatcher {
    pub fn new() -> Self {
        Self {
            keyboard: KeyboardDispatcher::new(),
        }
    }
    
    pub fn play_pause(&self) {
        self.keyboard.media_play_pause();
    }
    
    pub fn next_track(&self) {
        self.keyboard.media_next();
    }
    
    pub fn previous_track(&self) {
        self.keyboard.media_previous();
    }
    
    pub fn volume_up(&self) {
        self.keyboard.volume_up();
    }
    
    pub fn volume_down(&self) {
        self.keyboard.volume_down();
    }
}