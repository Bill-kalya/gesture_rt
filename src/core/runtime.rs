use tokio::sync::mpsc::Receiver;
use log::{info, error, warn};
use coarsetime::Instant as CoarseInstant;
use crate::core::event_bus::{EventBus, RuntimeEvent};
use crate::spatial::filters::kalman::KalmanFilter;
use crate::gestures::temporal_engine::{MotionHistory, GestureFeatures};
use crate::gestures::confidence::{ConfidenceEngine, GestureConfidence};
use crate::gestures::gesture_graph::GestureStateMachine;
use nalgebra::Vector3;

pub struct RuntimeConfig {
    pub kalman_process_noise_pos: f32,
    pub kalman_process_noise_vel: f32,
    pub kalman_meas_noise: f32,
    pub confidence_threshold: f32,
    pub confidence_temporal_alpha: f32,
    pub fsm_cooldown_ms: u64,
    pub fsm_tracking_frames: usize,
    pub temporal_window_frames: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            kalman_process_noise_pos: 0.1,
            kalman_process_noise_vel: 1.0,
            kalman_meas_noise: 0.5,
            confidence_threshold: 0.75,
            confidence_temporal_alpha: 0.3,
            fsm_cooldown_ms: 500,
            fsm_tracking_frames: 3,
            temporal_window_frames: 10,
        }
    }
}

pub struct GestureRuntime {
    config: RuntimeConfig,
    kalman: KalmanFilter,
    motion_history: MotionHistory,
    confidence_engine: ConfidenceEngine,
    fsm: GestureStateMachine,
    event_bus: EventBus,
    event_receiver: Receiver<RuntimeEvent>,
    last_position: Option<Vector3<f32>>,
    low_confidence_counter: u32,
    calibration_mode: bool,
}

impl GestureRuntime {
    pub fn new(config: RuntimeConfig, event_bus: EventBus, event_receiver: Receiver<RuntimeEvent>) -> Self {
        let kalman = KalmanFilter::new(
            config.kalman_process_noise_pos,
            config.kalman_process_noise_vel,
            config.kalman_meas_noise,
        );
        
        let motion_history = MotionHistory::new(config.temporal_window_frames);
        let confidence_engine = ConfidenceEngine::new(config.confidence_threshold, config.confidence_temporal_alpha);
        let fsm = GestureStateMachine::new(config.fsm_cooldown_ms, config.fsm_tracking_frames);
        
        Self {
            config,
            kalman,
            motion_history,
            confidence_engine,
            fsm,
            event_bus,
            event_receiver,
            last_position: None,
            low_confidence_counter: 0,
            calibration_mode: false,
        }
    }
    
    pub async fn run(&mut self) {
        info!("GestureRT runtime started");
        
        while let Some(event) = self.event_receiver.recv().await {
            match event {
                RuntimeEvent::LandmarksExtracted(timestamp_ns, landmarks) => {
                    // Take wrist position (landmark 0) or palm center
                    if let Some(wrist) = landmarks.first() {
                        self.process_spatial_state(timestamp_ns, *wrist).await;
                    }
                }
                RuntimeEvent::CalibrationReset => {
                    self.calibration_mode = false;
                    self.low_confidence_counter = 0;
                    self.kalman.reset();
                    self.motion_history.clear();
                    self.confidence_engine.reset();
                    self.fsm.reset();
                    info!("Calibration reset complete");
                }
                RuntimeEvent::SystemShutdown => {
                    info!("Shutting down GestureRT");
                    break;
                }
                _ => {}
            }
        }
    }
    
    async fn process_spatial_state(&mut self, timestamp_ns: u64, raw_position: Vector3<f32>) {
        // Apply Kalman filter
        self.kalman.set_time(timestamp_ns);
        self.kalman.update(raw_position);
        
        let filtered_pos = self.kalman.position();
        let velocity = self.kalman.velocity();
        
        // Emit spatial state
        let _ = self.event_bus.try_emit(RuntimeEvent::SpatialState(timestamp_ns, filtered_pos, velocity));
        
        // Update motion history
        self.motion_history.push(filtered_pos, timestamp_ns);
        
        // Extract features and classify
        if let Some(features) = GestureFeatures::from_history(&self.motion_history) {
            let confidence = self.confidence_engine.classify(&features);
            let (top_gesture, top_prob) = confidence.highest();
            
            // Check for low confidence condition
            if top_prob < self.config.confidence_threshold * 0.5 {
                self.low_confidence_counter += 1;
                if self.low_confidence_counter > 30 && !self.calibration_mode {
                    self.calibration_mode = true;
                    let _ = self.event_bus.try_emit(RuntimeEvent::CalibrationRequested);
                    warn!("Low confidence for 30+ frames, calibration requested");
                }
            } else {
                self.low_confidence_counter = 0;
            }
            
            // State machine decides when to dispatch
            if !self.calibration_mode {
                if let Some(gesture) = self.fsm.update(&confidence, self.config.confidence_threshold) {
                    // Dispatch gesture
                    let _ = self.event_bus.try_emit(RuntimeEvent::GestureDetected(gesture, top_prob));
                    self.dispatch_gesture(gesture).await;
                    let _ = self.event_bus.try_emit(RuntimeEvent::GestureDispatched(gesture));
                }
            }
        }
    }
    
    async fn dispatch_gesture(&self, gesture: crate::gestures::confidence::GestureType) {
        // Platform-appropriate dispatch
        match gesture {
            crate::gestures::confidence::GestureType::SwipeLeft => {
                // ALT + TAB or media previous
                #[cfg(target_os = "windows")]
                {
                    use enigo::*;
                    let mut enigo = Enigo::new();
                    enigo.key_down(Key::Alt);
                    enigo.key_click(Key::Tab);
                    enigo.key_up(Key::Alt);
                }
                #[cfg(target_os = "linux")]
                {
                    // Linux implementation using enigo or xdotool
                }
                info!("Dispatched: Swipe Left");
            }
            crate::gestures::confidence::GestureType::SwipeRight => {
                info!("Dispatched: Swipe Right");
            }
            crate::gestures::confidence::GestureType::SwipeUp => {
                info!("Dispatched: Swipe Up");
            }
            crate::gestures::confidence::GestureType::SwipeDown => {
                info!("Dispatched: Swipe Down");
            }
            crate::gestures::confidence::GestureType::Pinch => {
                info!("Dispatched: Pinch");
            }
            crate::gestures::confidence::GestureType::Fist => {
                info!("Dispatched: Fist");
            }
            _ => {}
        }
    }
}