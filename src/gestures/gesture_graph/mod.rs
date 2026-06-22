// Re-export FSM from earlier
pub use crate::gestures::fsm::*;

// This file is a stub - the main FSM implementation is in the module we created
// but we need to make sure it's accessible

use std::time::{Duration, Instant};
use crate::gestures::confidence::{GestureType, GestureConfidence};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GestureFSMState {
    Idle,
    Tracking,
    ActiveGesture(GestureType),
    Cooldown,
}

pub struct GestureStateMachine {
    pub state: GestureFSMState,
    pub current_gesture: Option<GestureType>,
    pub confidence_history: Vec<f32>,
    pub last_trigger_time: Instant,
    pub cooldown_duration: Duration,
    pub tracking_required_frames: usize,
    frame_counter: usize,
}

impl GestureStateMachine {
    pub fn new(cooldown_ms: u64, tracking_frames: usize) -> Self {
        Self {
            state: GestureFSMState::Idle,
            current_gesture: None,
            confidence_history: Vec::new(),
            last_trigger_time: Instant::now(),
            cooldown_duration: Duration::from_millis(cooldown_ms),
            tracking_required_frames: tracking_frames,
            frame_counter: 0,
        }
    }
    
    pub fn update(&mut self, confidence: &GestureConfidence, dispatch_threshold: f32) -> Option<GestureType> {
        let (gesture, prob) = confidence.highest();
        
        match self.state {
            GestureFSMState::Idle => {
                if prob > dispatch_threshold && gesture != GestureType::None {
                    self.state = GestureFSMState::Tracking;
                    self.current_gesture = Some(gesture);
                    self.frame_counter = 1;
                    self.confidence_history.clear();
                    self.confidence_history.push(prob);
                    None
                } else {
                    None
                }
            }
            
            GestureFSMState::Tracking => {
                if let Some(current) = self.current_gesture {
                    if gesture == current && prob > dispatch_threshold * 0.7 {
                        self.frame_counter += 1;
                        self.confidence_history.push(prob);
                        
                        if self.frame_counter >= self.tracking_required_frames {
                            self.state = GestureFSMState::ActiveGesture(current);
                            self.last_trigger_time = Instant::now();
                            Some(current)
                        } else {
                            None
                        }
                    } else {
                        self.state = GestureFSMState::Idle;
                        self.current_gesture = None;
                        None
                    }
                } else {
                    self.state = GestureFSMState::Idle;
                    None
                }
            }
            
            GestureFSMState::ActiveGesture(gesture_type) => {
                self.state = GestureFSMState::Cooldown;
                Some(gesture_type)
            }
            
            GestureFSMState::Cooldown => {
                if self.last_trigger_time.elapsed() >= self.cooldown_duration {
                    self.state = GestureFSMState::Idle;
                    self.current_gesture = None;
                }
                None
            }
        }
    }
    
    pub fn reset(&mut self) {
        self.state = GestureFSMState::Idle;
        self.current_gesture = None;
        self.confidence_history.clear();
        self.frame_counter = 0;
    }
}