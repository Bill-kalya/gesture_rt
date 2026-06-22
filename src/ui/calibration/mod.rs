use std::time::{Duration, Instant};
use nalgebra::Vector3;
use super::overlay::Overlay;

pub struct CalibrationUI {
    overlay: Overlay,
    state: CalibrationState,
    neutral_pose: Option<Vector3<f32>>,
    progress: f32,
    start_time: Option<Instant>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CalibrationState {
    Idle,
    WaitingForNeutral,
    CollectingSamples,
    Complete,
    Failed,
}

impl CalibrationUI {
    pub fn new() -> Self {
        Self {
            overlay: Overlay::new(),
            state: CalibrationState::Idle,
            neutral_pose: None,
            progress: 0.0,
            start_time: None,
        }
    }
    
    pub fn start_calibration(&mut self) {
        self.state = CalibrationState::WaitingForNeutral;
        self.progress = 0.0;
        self.start_time = Some(Instant::now());
        self.overlay.show_calibration_status("Please show neutral hand pose");
    }
    
    pub fn update(&mut self, hand_position: Vector3<f32>) -> Option<Vector3<f32>> {
        match self.state {
            CalibrationState::WaitingForNeutral => {
                self.progress = 0.0;
                // Check if hand is stable
                if let Some(start) = self.start_time {
                    if start.elapsed() > Duration::from_secs(2) {
                        self.state = CalibrationState::CollectingSamples;
                        self.neutral_pose = Some(hand_position);
                        self.overlay.show_calibration_status("Collecting samples...");
                    }
                }
                None
            }
            CalibrationState::CollectingSamples => {
                self.progress += 0.1;
                if self.progress >= 1.0 {
                    self.state = CalibrationState::Complete;
                    self.overlay.show_calibration_status("Calibration complete!");
                    return self.neutral_pose;
                }
                
                // Refine neutral pose with moving average
                if let Some(current) = self.neutral_pose {
                    self.neutral_pose = Some(current * 0.9 + hand_position * 0.1);
                }
                None
            }
            _ => None,
        }
    }
    
    pub fn reset(&mut self) {
        self.state = CalibrationState::Idle;
        self.neutral_pose = None;
        self.progress = 0.0;
        self.start_time = None;
        self.overlay.show_calibration_status("Calibration reset");
    }
    
    pub fn is_calibrated(&self) -> bool {
        matches!(self.state, CalibrationState::Complete)
    }
    
    pub fn get_neutral_pose(&self) -> Option<Vector3<f32>> {
        self.neutral_pose
    }
    
    pub fn progress(&self) -> f32 {
        self.progress
    }
}

impl Default for CalibrationUI {
    fn default() -> Self {
        Self::new()
    }
}