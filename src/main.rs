mod core;
mod vision;
mod spatial;
mod gestures;
mod runtime_dispatch;
mod dsl;
mod ui;
mod platform;

use log::{info, error};
use env_logger;
use tokio;

use crate::core::runtime::{GestureRuntime, RuntimeConfig};
use crate::core::event_bus::EventBus;
use crate::core::plugin_loader::PluginLoader;

#[cfg(feature = "camera")]
use crate::vision::camera::capture::CameraCapture;
#[cfg(feature = "camera")]
use crate::vision::landmarks::mediapipe_adapter::MediaPipeLandmarkExtractor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    info!("╔═══════════════════════════════════════╗");
    info!("║  GestureRT v0.1.0                   ║");
    info!("║  Real-time Spatial Operating Runtime║");
    info!("╚═══════════════════════════════════════╝");
    info!("");
    info!("Coordinate system: Right-handed (X=right, Y=up, Z=toward user)");
    info!("Kalman filter: Linear, position+velocity");
    info!("Confidence: Softmax + temporal smoothing");
    info!("FSM: Idle → Tracking → Active → Cooldown");
    info!("");
    
    // Initialize event bus
    let (event_bus, event_receiver) = EventBus::new();
    
    // Initialize runtime
    let config = RuntimeConfig::default();
    let mut runtime = GestureRuntime::new(config, event_bus.clone(), event_receiver);
    
    // Initialize plugin loader
    let plugin_loader = PluginLoader::new("./plugins");
    let _ = plugin_loader.load_plugin("debug").await;
    
    info!("Runtime initialized.");
    
    #[cfg(feature = "camera")]
    {
        info!("📷 Camera feature enabled. Initializing camera...");
        let mut camera = match CameraCapture::new(0) {
            Ok(cam) => cam,
            Err(e) => {
                error!("Failed to initialize camera: {}", e);
                return Ok(());
            }
        };
        
        let mut landmark_extractor = MediaPipeLandmarkExtractor::new();
        
        info!("▶️ Starting main capture loop...");
        
        // Spawn runtime task
        let runtime_handle = tokio::spawn(async move {
            runtime.run().await;
        });
        
        let mut frame_count = 0;
        loop {
            match camera.capture_frame() {
                Ok(frame) => {
                    frame_count += 1;
                    if frame_count % 30 == 0 {
                        info!("📊 Captured {} frames", frame_count);
                    }
                    
                    // Extract landmarks
                    if let Ok(landmarks) = landmark_extractor.extract(&frame) {
                        let timestamp = coarsetime::Instant::now().as_nanos() as u64;
                        let _ = event_bus.emit(core::event_bus::RuntimeEvent::LandmarksExtracted(timestamp, landmarks)).await;
                    }
                }
                Err(e) => {
                    error!("Camera error: {}", e);
                    break;
                }
            }
            
            // Throttle to ~30 FPS
            tokio::time::sleep(tokio::time::Duration::from_millis(33)).await;
        }
        
        // Shutdown
        let _ = event_bus.emit(core::event_bus::RuntimeEvent::SystemShutdown).await;
        runtime_handle.await?;
    }
    
    #[cfg(not(feature = "camera"))]
    {
        info!("🎮 Camera feature disabled. Running in demo mode with synthetic landmarks.");
        info!("   (This demonstrates the gesture recognition pipeline without hardware)");
        
        // Spawn runtime task
        let runtime_handle = tokio::spawn(async move {
            runtime.run().await;
        });
        
        // Generate synthetic landmarks for testing
        let mut synthetic_position = nalgebra::Vector3::new(0.0, 0.0, 1.0);
        let mut direction = 1.0;
        
        info!("▶️ Starting demo loop (press Ctrl+C to stop)");
        
        for frame_idx in 0..300 {
            // Simulate a swipe left/right motion
            synthetic_position.x += 0.005 * direction;
            if synthetic_position.x.abs() > 0.2 {
                direction *= -1.0;
                info!("🔄 Direction change at frame {}", frame_idx);
            }
            
            // Occasionally simulate a vertical swipe
            if frame_idx % 50 == 0 && frame_idx > 0 {
                synthetic_position.y = 0.1;
                info!("↕️ Simulating vertical movement at frame {}", frame_idx);
            } else {
                synthetic_position.y *= 0.98; // Return to center
            }
            
            let timestamp = coarsetime::Instant::now().as_nanos() as u64;
            
            // Create 21 landmarks (all at same position for simplicity in demo)
            let landmarks: Vec<nalgebra::Vector3<f32>> = (0..21)
                .map(|_| synthetic_position)
                .collect();
            
            let _ = event_bus.emit(core::event_bus::RuntimeEvent::LandmarksExtracted(timestamp, landmarks)).await;
            
            // Print status every 30 frames
            if frame_idx % 30 == 0 && frame_idx > 0 {
                info!("📊 Processed {} frames", frame_idx);
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(33)).await;
        }
        
        info!("⏹️ Demo complete. Shutting down...");
        let _ = event_bus.emit(core::event_bus::RuntimeEvent::SystemShutdown).await;
        runtime_handle.await?;
    }
    
    info!("👋 GestureRT shutdown complete");
    Ok(())
}