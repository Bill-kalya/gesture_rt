use tokio::sync::mpsc::Receiver;
use log::{info, error, warn};
use coarsetime::Instant as CoarseInstant;
use crate::core::event_bus::{EventBus, RuntimeEvent};
use crate::spatial::filters::one_euro::OneEuroFilter;
use crate::spatial::features::{FeatureGenerator, GestureFeatures};
use crate::spatial::depth::DepthEstimator;
use crate::spatial::normalization::SpatialNormalizer;
use crate::spatial::orientation::OrientationEstimator;
use crate::spatial::spatial_state::{SpatialHand, SpatialConfidence, FrameState, UserProfile};
use crate::gestures::temporal_engine::MotionHistory;
use crate::gestures::confidence::{ConfidenceEngine, GestureConfidence};
use crate::gestures::confidence::fuser::ConfidenceFuser;
use crate::gestures::session::{GestureSessionManager, SessionConfig};
use crate::spatial::depth::DepthMethod;
use nalgebra::Vector3;

pub struct RuntimeConfig {
    pub confidence_threshold: f32,
    pub confidence_temporal_alpha: f32,
    pub temporal_window_frames: usize,
    pub session_config: SessionConfig,
    pub depth_method: DepthMethod,
    pub min_confidence: f32,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.75,
            confidence_temporal_alpha: 0.3,
            temporal_window_frames: 10,
            session_config: SessionConfig::default(),
            depth_method: DepthMethod::HandSize,
            min_confidence: 0.3,
        }
    }
}

pub struct GestureRuntime {
    config: RuntimeConfig,
    one_euro: OneEuroFilter,
    feature_generator: FeatureGenerator,
    depth_estimator: DepthEstimator,
    spatial_normalizer: SpatialNormalizer,
    orientation_estimator: OrientationEstimator,
    motion_history: MotionHistory,
    confidence_engine: ConfidenceEngine,
    confidence_fuser: ConfidenceFuser,
    session_manager: GestureSessionManager,
    event_bus: EventBus,
    event_receiver: Receiver<RuntimeEvent>,
    low_confidence_counter: u32,
    calibration_mode: bool,
    frame_count: u64,
    user_profile: UserProfile,
    prev_frame_state: Option<FrameState>,
}

impl GestureRuntime {
    pub fn new(config: RuntimeConfig, event_bus: EventBus, event_receiver: Receiver<RuntimeEvent>) -> Self {
        let depth_estimator = DepthEstimator::new(config.depth_method);
        
        Self {
            config: config.clone(),
            one_euro: OneEuroFilter::default_hand(),
            feature_generator: FeatureGenerator::new(),
            depth_estimator,
            spatial_normalizer: SpatialNormalizer::new(),
            orientation_estimator: OrientationEstimator::new(),
            motion_history: MotionHistory::new(config.temporal_window_frames),
            confidence_engine: ConfidenceEngine::new(config.confidence_threshold, config.confidence_temporal_alpha),
            confidence_fuser: ConfidenceFuser::default(),
            session_manager: GestureSessionManager::new(config.session_config),
            event_bus,
            event_receiver,
            low_confidence_counter: 0,
            calibration_mode: false,
            frame_count: 0,
            user_profile: UserProfile::default(),
            prev_frame_state: None,
        }
    }
    
    pub async fn run(&mut self) {
        info!("GestureRT runtime started with depth estimation and spatial normalization");
        info!("Depth method: {:?}", self.config.depth_method);
        
        while let Some(event) = self.event_receiver.recv().await {
            match event {
                RuntimeEvent::LandmarksExtracted(timestamp_ns, landmarks) => {
                    self.frame_count += 1;
                    let timestamp_secs = timestamp_ns as f64 / 1_000_000_000.0;
                    
                    // Process with full spatial pipeline
                    if let Some(wrist) = landmarks.first() {
                        self.process_spatial_pipeline(timestamp_ns, timestamp_secs, *wrist, &landmarks).await;
                    }
                }
                RuntimeEvent::CalibrationReset => {
                    self.calibration_mode = false;
                    self.low_confidence_counter = 0;
                    self.one_euro = OneEuroFilter::default_hand();
                    self.motion_history.clear();
                    self.confidence_engine.reset();
                    self.session_manager.cancel_session();
                    self.spatial_normalizer = SpatialNormalizer::new();
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
    
    async fn process_spatial_pipeline(&mut self, timestamp_ns: u64, timestamp_secs: f64, raw_position: Vector3<f32>, landmarks: &[Vector3<f32>]) {
        // Step 1: One-Euro filter
        let filtered_pos = self.one_euro.filter(raw_position, timestamp_secs);
        
        // Step 2: Depth estimation
        let distance_cm = self.depth_estimator.estimate_distance(landmarks).unwrap_or(45.0);
        let hand_size_cm = self.depth_estimator.estimate_hand_size(landmarks).unwrap_or(8.5);
        
        // Step 3: Spatial normalization
        let normalized_pos = self.spatial_normalizer.normalize_position(filtered_pos);
        
        // Step 4: Orientation estimation
        let palm_center = self.calculate_palm_center(landmarks);
        let palm_normal = self.calculate_palm_normal(landmarks);
        let orientation = self.orientation_estimator.estimate_orientation(
            palm_center,
            palm_normal,
            landmarks[0], // wrist
        );
        
        // Step 5: Create spatial hand state
        let spatial_confidence = SpatialConfidence {
            tracking: 0.9,
            depth: 0.85,
            visibility: 0.9,
            overall: 0.85,
        };
        
        let spatial_hand = SpatialHand {
            landmarks: landmarks.to_vec(),
            distance_cm,
            hand_size_cm,
            palm_center,
            palm_normal,
            palm_rotation: OrientationEstimator::orientation_to_rotation(&orientation),
            orientation,
            confidence: spatial_confidence,
        };
        
        // Step 6: Feature generation
        let dt = if self.frame_count > 1 { 0.033 } else { 0.001 };
        if let Some(mut features) = self.feature_generator.generate(landmarks, dt) {
            // Add spatial features
            features.distance_from_camera = distance_cm;
            features.hand_size = hand_size_cm;
            features.palm_orientation = orientation;
            
            // Step 7: Update motion history with normalized positions
            self.motion_history.push(normalized_pos, timestamp_ns);
            
            // Step 8: Temporal features
            if let Some(temporal_features) = GestureFeatures::from_history(&self.motion_history) {
                // Step 9: Confidence engine
                let raw_confidence = self.confidence_engine.classify(&temporal_features);
                
                // Step 10: Fuse with spatial confidence
                let fused_confidence = self.confidence_fuser.fuse(&raw_confidence, &spatial_confidence);
                
                // Step 11: Session manager
                self.session_manager.set_position(normalized_pos);
                if let Some(session) = self.session_manager.update(&fused_confidence) {
                    if let Some(gesture_type) = session.gesture_type {
                        let confidence_val = fused_confidence.highest().1;
                        let _ = self.event_bus.try_emit(RuntimeEvent::GestureDetected(gesture_type, confidence_val));
                        self.dispatch_gesture(gesture_type).await;
                        let _ = self.event_bus.try_emit(RuntimeEvent::GestureDispatched(gesture_type));
                    }
                }
                
                // Step 12: Check for calibration
                let (_, top_prob) = fused_confidence.highest();
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
            }
        }
        
        // Store frame state
        let frame_state = FrameState {
            left_hand: None,
            right_hand: Some(spatial_hand),
            frame_id: self.frame_count,
            timestamp_ns,
        };
        self.prev_frame_state = Some(frame_state);
    }
    
    fn calculate_palm_center(&self, landmarks: &[Vector3<f32>]) -> Vector3<f32> {
        if landmarks.len() < 21 {
            return Vector3::zeros();
        }
        (landmarks[0] + landmarks[5] + landmarks[17]) / 3.0
    }
    
    fn calculate_palm_normal(&self, landmarks: &[Vector3<f32>]) -> Vector3<f32> {
        if landmarks.len() < 21 {
            return Vector3::new(0.0, 0.0, 1.0);
        }
        let palm_center = self.calculate_palm_center(landmarks);
        let to_index = landmarks[5] - palm_center;
        let to_pinky = landmarks[17] - palm_center;
        to_index.cross(&to_pinky).normalize()
    }
    
    async fn dispatch_gesture(&self, gesture: crate::gestures::confidence::GestureType) {
        match gesture {
            crate::gestures::confidence::GestureType::SwipeLeft => {
                #[cfg(target_os = "windows")]
                {
                    use enigo::*;
                    let mut enigo = Enigo::new();
                    enigo.key_down(Key::Alt);
                    enigo.key_click(Key::Tab);
                    enigo.key_up(Key::Alt);
                }
                info!("💡 Dispatched: Swipe Left (distance-normalized)");
            }
            crate::gestures::confidence::GestureType::SwipeRight => {
                info!("💡 Dispatched: Swipe Right (distance-normalized)");
            }
            crate::gestures::confidence::GestureType::SwipeUp => {
                info!("💡 Dispatched: Swipe Up (distance-normalized)");
            }
            crate::gestures::confidence::GestureType::SwipeDown => {
                info!("💡 Dispatched: Swipe Down (distance-normalized)");
            }
            crate::gestures::confidence::GestureType::Pinch => {
                info!("💡 Dispatched: Pinch (distance-normalized)");
            }
            crate::gestures::confidence::GestureType::Fist => {
                info!("💡 Dispatched: Fist (distance-normalized)");
            }
            _ => {}
        }
    }
}
