use super::parser::ASTNode;
use std::collections::HashMap;
use log::info;

pub struct Interpreter {
    gesture_map: HashMap<String, Vec<ASTNode>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            gesture_map: HashMap::new(),
        }
    }
    
    pub fn load_script(&mut self, ast: Vec<ASTNode>) -> Result<(), String> {
        for node in ast {
            match node {
                ASTNode::GestureDefinition { name, body } => {
                    self.gesture_map.insert(name, body);
                    info!("Loaded gesture definition");
                }
                ASTNode::Comment { text } => {
                    info!("Comment: {}", text);
                }
                _ => {
                    // Ignore other nodes
                }
            }
        }
        Ok(())
    }
    
    pub fn execute_gesture(&self, name: &str) -> Result<(), String> {
        if let Some(body) = self.gesture_map.get(name) {
            for node in body {
                match node {
                    ASTNode::ExecuteStatement { command } => {
                        self.execute_command(command)?;
                    }
                    _ => {
                        // Ignore other node types
                    }
                }
            }
            Ok(())
        } else {
            Err(format!("Gesture '{}' not found", name))
        }
    }
    
    fn execute_command(&self, command: &str) -> Result<(), String> {
        info!("Executing command: {}", command);
        
        // Parse and execute OS-level commands
        match command {
            "ALT+TAB" => {
                #[cfg(target_os = "windows")]
                {
                    use enigo::*;
                    let mut enigo = Enigo::new();
                    enigo.key_down(Key::Alt);
                    enigo.key_click(Key::Tab);
                    enigo.key_up(Key::Alt);
                }
                #[cfg(not(target_os = "windows"))]
                {
                    // Linux/macOS implementation
                    info!("ALT+TAB not implemented on this platform");
                }
            }
            "SPOTIFY" => {
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("cmd")
                        .args(&["/C", "start spotify:"])
                        .spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("xdg-open")
                        .arg("spotify:")
                        .spawn();
                }
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open")
                        .arg("spotify:")
                        .spawn();
                }
            }
            _ => {
                // Try to execute as a shell command
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("cmd")
                        .args(&["/C", command])
                        .spawn();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = std::process::Command::new("sh")
                        .args(&["-c", command])
                        .spawn();
                }
            }
        }
        
        Ok(())
    }
    
    pub fn list_gestures(&self) -> Vec<String> {
        self.gesture_map.keys().cloned().collect()
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}