use crate::gestures::confidence::{GestureType, GestureConfidence};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SessionState {
    Idle,
    Acquiring,    // Building confidence
    Tracking,     // Gesture in progress
    Recognized,   // Gesture identified
    Executing,    // Action being performed
    Cooldown,     // Debounce period
}

#[derive(Debug, Clone)]
pub struct GestureSession {
    pub id: u64,
    pub gesture_type: Option<GestureType>,
    pub start_time: Instant,
    pub last_update: Instant,
    pub confidence_history: Vec<f32>,
    pub state: SessionState,
    pub duration: Duration,
    pub data: GestureSessionData,
}

#[derive(Debug, Clone, Default)]
pub struct GestureSessionData {
    pub start_position: Option<nalgebra::Vector3<f32>>,
    pub current_position: Option<nalgebra::Vector3<f32>>,
    pub velocity_history: Vec<nalgebra::Vector3<f32>>,
    pub distance_traveled: f32,
    pub direction_changes: u32,
}

pub struct GestureSessionManager {
    current_session: Option<GestureSession>,
    session_counter: u64,
    config: SessionConfig,
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub acquire_timeout_ms: u64,
    pub tracking_timeout_ms: u64,
    pub cooldown_ms: u64,
    pub min_confidence_threshold: f32,
    pub max_session_duration_ms: u64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            acquire_timeout_ms: 500,
            tracking_timeout_ms: 1000,
            cooldown_ms: 300,
            min_confidence_threshold: 0.7,
            max_session_duration_ms: 2000,
        }
    }
}

impl GestureSessionManager {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            current_session: None,
            session_counter: 0,
            config,
        }
    }
    
    pub fn update(&mut self, confidence: &GestureConfidence) -> Option<GestureSession> {
        let (gesture, prob) = confidence.highest();
        let now = Instant::now();
        
        match &mut self.current_session {
            Some(session) => {
                // Update existing session
                session.last_update = now;
                session.confidence_history.push(prob);
                session.duration = now - session.start_time;
                
                if let Some(pos) = session.data.current_position {
                    if let Some(start_pos) = session.data.start_position {
                        session.data.distance_traveled += (pos - start_pos).norm();
                    }
                }
                
                match session.state {
                    SessionState::Acquiring => {
                        if prob >= self.config.min_confidence_threshold {
                            // Gesture recognized
                            session.state = SessionState::Recognized;
                            session.gesture_type = Some(gesture);
                            Some(session.clone())
                        } else if session.duration > Duration::from_millis(self.config.acquire_timeout_ms) {
                            // Timeout - abort
                            session.state = SessionState::Idle;
                            self.current_session = None;
                            None
                        } else {
                            None
                        }
                    }
                    SessionState::Tracking => {
                        if prob >= self.config.min_confidence_threshold * 0.7 {
                            // Continue tracking
                            None
                        } else {
                            // Lost tracking
                            session.state = SessionState::Cooldown;
                            None
                        }
                    }
                    SessionState::Recognized => {
                        // Execute and move to cooldown
                        session.state = SessionState::Executing;
                        Some(session.clone())
                    }
                    SessionState::Executing => {
                        session.state = SessionState::Cooldown;
                        None
                    }
                    SessionState::Cooldown => {
                        if session.duration > Duration::from_millis(self.config.cooldown_ms) {
                            self.current_session = None;
                        }
                        None
                    }
                    _ => {
                        self.current_session = None;
                        None
                    }
                }
            }
            None => {
                // Start new session if confidence is high enough
                if prob >= self.config.min_confidence_threshold && gesture != GestureType::None {
                    self.session_counter += 1;
                    let session = GestureSession {
                        id: self.session_counter,
                        gesture_type: Some(gesture),
                        start_time: now,
                        last_update: now,
                        confidence_history: vec![prob],
                        state: SessionState::Acquiring,
                        duration: Duration::default(),
                        data: GestureSessionData::default(),
                    };
                    self.current_session = Some(session);
                    None
                } else {
                    None
                }
            }
        }
    }
    
    pub fn set_position(&mut self, position: nalgebra::Vector3<f32>) {
        if let Some(session) = &mut self.current_session {
            session.data.current_position = Some(position);
            if session.data.start_position.is_none() {
                session.data.start_position = Some(position);
            }
            
            // Update velocity history
            if let Some(prev) = session.data.current_position {
                let vel = position - prev;
                session.data.velocity_history.push(vel);
                if session.data.velocity_history.len() > 30 {
                    session.data.velocity_history.remove(0);
                }
            }
        }
    }
    
    pub fn current_session(&self) -> Option<&GestureSession> {
        self.current_session.as_ref()
    }
    
    pub fn cancel_session(&mut self) {
        self.current_session = None;
    }
    
    pub fn is_active(&self) -> bool {
        matches!(
            self.current_session,
            Some(ref s) if s.state != SessionState::Idle && s.state != SessionState::Cooldown
        )
    }
}

impl Default for GestureSessionManager {
    fn default() -> Self {
        Self::new(SessionConfig::default())
    }
}