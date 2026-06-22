/// Overlay for displaying gesture info on screen
pub struct Overlay {
    enabled: bool,
    show_confidence: bool,
    show_landmarks: bool,
}

impl Overlay {
    pub fn new() -> Self {
        Self {
            enabled: true,
            show_confidence: true,
            show_landmarks: false,
        }
    }
    
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    pub fn disable(&mut self) {
        self.enabled = false;
    }
    
    pub fn set_show_confidence(&mut self, show: bool) {
        self.show_confidence = show;
    }
    
    pub fn set_show_landmarks(&mut self, show: bool) {
        self.show_landmarks = show;
    }
    
    pub fn show_confidence(&self, gesture: &str, confidence: f32) {
        if self.enabled && self.show_confidence {
            println!("[Overlay] {}: {:.2}%", gesture, confidence * 100.0);
        }
    }
    
    pub fn show_calibration_status(&self, status: &str) {
        if self.enabled {
            println!("[Overlay] Calibration: {}", status);
        }
    }
}

impl Default for Overlay {
    fn default() -> Self {
        Self::new()
    }
}