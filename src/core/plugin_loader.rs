use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use log::{info, warn, error};

/// Trait for plugins to implement
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn on_load(&self) -> Result<(), String>;
    fn on_unload(&self) -> Result<(), String>;
    fn on_gesture(&self, gesture_type: &str, confidence: f32) -> Result<(), String>;
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub enabled: bool,
}

/// Plugin loader for dynamic plugin management
pub struct PluginLoader {
    plugins: Arc<Mutex<HashMap<String, Box<dyn Plugin>>>>,
    metadata: Arc<Mutex<HashMap<String, PluginMetadata>>>,
    plugin_dir: String,
}

impl PluginLoader {
    pub fn new(plugin_dir: &str) -> Self {
        Self {
            plugins: Arc::new(Mutex::new(HashMap::new())),
            metadata: Arc::new(Mutex::new(HashMap::new())),
            plugin_dir: plugin_dir.to_string(),
        }
    }

    /// Load a plugin by name
    pub async fn load_plugin(&self, name: &str) -> Result<(), String> {
        // In Phase 1, we only support compile-time plugins
        // In Phase 2, we'll add dynamic loading via libloading
        
        info!("Loading plugin: {} (compile-time mode)", name);
        
        // Register built-in plugins
        let plugin: Box<dyn Plugin> = match name {
            "example" => Box::new(ExamplePlugin),
            "debug" => Box::new(DebugPlugin),
            _ => {
                return Err(format!("Plugin '{}' not found", name));
            }
        };
        
        let plugin_name = plugin.name().to_string();
        let mut plugins = self.plugins.lock().await;
        
        if plugins.contains_key(&plugin_name) {
            return Err(format!("Plugin '{}' already loaded", plugin_name));
        }
        
        // Call on_load
        if let Err(e) = plugin.on_load() {
            return Err(format!("Plugin '{}' failed to load: {}", plugin_name, e));
        }
        
        plugins.insert(plugin_name.clone(), plugin);
        
        // Add metadata
        let mut metadata = self.metadata.lock().await;
        metadata.insert(plugin_name.clone(), PluginMetadata {
            name: plugin_name.clone(),
            version: "0.1.0".to_string(),
            author: "GestureRT".to_string(),
            description: format!("{} plugin", plugin_name),
            enabled: true,
        });
        
        info!("Plugin '{}' loaded successfully", plugin_name);
        Ok(())
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, name: &str) -> Result<(), String> {
        let mut plugins = self.plugins.lock().await;
        
        if let Some(plugin) = plugins.remove(name) {
            if let Err(e) = plugin.on_unload() {
                warn!("Plugin '{}' unload warning: {}", name, e);
            }
            
            let mut metadata = self.metadata.lock().await;
            if let Some(mut meta) = metadata.remove(name) {
                meta.enabled = false;
            }
            
            info!("Plugin '{}' unloaded", name);
            Ok(())
        } else {
            Err(format!("Plugin '{}' not loaded", name))
        }
    }

    /// Get plugin by name
    pub async fn get_plugin(&self, name: &str) -> Option<Box<dyn Plugin>> {
        let plugins = self.plugins.lock().await;
        plugins.get(name).map(|p| unsafe { std::mem::transmute::<&dyn Plugin, Box<dyn Plugin>>(&**p) })
    }

    /// List all loaded plugins
    pub async fn list_plugins(&self) -> Vec<PluginMetadata> {
        let metadata = self.metadata.lock().await;
        metadata.values().cloned().collect()
    }

    /// Dispatch gesture to all plugins
    pub async fn dispatch_gesture(&self, gesture_type: &str, confidence: f32) {
        let plugins = self.plugins.lock().await;
        for (name, plugin) in plugins.iter() {
            if let Err(e) = plugin.on_gesture(gesture_type, confidence) {
                error!("Plugin '{}' failed to handle gesture: {}", name, e);
            }
        }
    }
}

/// Example plugin
struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn name(&self) -> &'static str { "example" }
    fn version(&self) -> &'static str { "0.1.0" }
    
    fn on_load(&self) -> Result<(), String> {
        info!("Example plugin loaded");
        Ok(())
    }
    
    fn on_unload(&self) -> Result<(), String> {
        info!("Example plugin unloaded");
        Ok(())
    }
    
    fn on_gesture(&self, gesture_type: &str, confidence: f32) -> Result<(), String> {
        info!("Example plugin received gesture: {} ({:.2})", gesture_type, confidence);
        Ok(())
    }
}

/// Debug plugin for logging
struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn name(&self) -> &'static str { "debug" }
    fn version(&self) -> &'static str { "0.1.0" }
    
    fn on_load(&self) -> Result<(), String> {
        info!("Debug plugin loaded");
        Ok(())
    }
    
    fn on_unload(&self) -> Result<(), String> {
        info!("Debug plugin unloaded");
        Ok(())
    }
    
    fn on_gesture(&self, gesture_type: &str, confidence: f32) -> Result<(), String> {
        debug!("[DEBUG] Gesture: {} (conf: {:.3})", gesture_type, confidence);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_loader() {
        let loader = PluginLoader::new("./plugins");
        
        // Load example plugin
        let result = loader.load_plugin("example").await;
        assert!(result.is_ok());
        
        // List plugins
        let plugins = loader.list_plugins().await;
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "example");
        
        // Unload plugin
        let result = loader.unload_plugin("example").await;
        assert!(result.is_ok());
        
        let plugins = loader.list_plugins().await;
        assert_eq!(plugins.len(), 0);
    }
}