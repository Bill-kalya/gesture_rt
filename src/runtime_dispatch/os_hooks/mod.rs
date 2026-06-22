pub enum DispatchMode {
    Global,
    Focused,
}

pub struct OSDispatcher {
    mode: DispatchMode,
}

impl OSDispatcher {
    pub fn new(mode: DispatchMode) -> Self {
        Self { mode }
    }
    
    pub fn set_mode(&mut self, mode: DispatchMode) {
        self.mode = mode;
    }
    
    pub fn dispatch_key(&self, key: &str) {
        match self.mode {
            DispatchMode::Global => {
                // Global dispatch across entire OS
                #[cfg(target_os = "windows")]
                {
                    use enigo::*;
                    let mut enigo = Enigo::new();
                    match key {
                        "ALT+TAB" => {
                            enigo.key_down(Key::Alt);
                            enigo.key_click(Key::Tab);
                            enigo.key_up(Key::Alt);
                        }
                        _ => {}
                    }
                }
            }
            DispatchMode::Focused => {
                // Only dispatch when app is focused
                // Implementation would check window focus
            }
        }
    }
}