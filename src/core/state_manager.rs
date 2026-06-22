use std::collections::HashMap;
use nalgebra::Vector3;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use std::sync::Arc;

/// Global state of the gesture runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeState {
    pub hand_position: Vector3<f32>,
    pub hand_velocity: Vector3<f32>,
    pub hand_acceleration: Vector3<f32>,
    pub palm_normal: Vector3<f32>,
    pub confidence: f32,
    pub gesture_state: GestureState,
    pub timestamp_ns: u64,
    pub frame_id: u64,
    pub is_calibrated: bool,
    pub calibration_baseline: Vector3<f32>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            hand_position: Vector3::zeros(),
            hand_velocity: Vector3::zeros(),
            hand_acceleration: Vector3::zeros(),
            palm_normal: Vector3::new(0.0, 0.0, 1.0),
            confidence: 0.0,
            gesture_state: GestureState::Idle,
            timestamp_ns: 0,
            frame_id: 0,
            is_calibrated: false,
            calibration_baseline: Vector3::zeros(),
        }
    }
}

/// Gesture state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GestureState {
    Idle,
    Tracking,
    Active,
    Cooldown,
    Calibrating,
}

/// State manager for maintaining runtime state
pub struct StateManager {
    state: Arc<RwLock<RuntimeState>>,
    history: Arc<RwLock<Vec<RuntimeState>>>,
    max_history: usize,
}

impl StateManager {
    pub fn new(max_history: usize) -> Self {
        Self {
            state: Arc::new(RwLock::new(RuntimeState::default())),
            history: Arc::new(RwLock::new(Vec::with_capacity(max_history))),
            max_history,
        }
    }

    /// Update current state
    pub async fn update(&self, new_state: RuntimeState) {
        let mut state = self.state.write().await;
        *state = new_state;
        
        // Store in history
        let mut history = self.history.write().await;
        history.push(state.clone());
        if history.len() > self.max_history {
            history.remove(0);
        }
    }

    /// Get current state
    pub async fn get(&self) -> RuntimeState {
        self.state.read().await.clone()
    }

    /// Get state at specific index in history
    pub async fn get_history(&self, index: usize) -> Option<RuntimeState> {
        let history = self.history.read().await;
        if index < history.len() {
            Some(history[index].clone())
        } else {
            None
        }
    }

    /// Get recent history (last N frames)
    pub async fn get_recent_history(&self, n: usize) -> Vec<RuntimeState> {
        let history = self.history.read().await;
        let start = if history.len() > n { history.len() - n } else { 0 };
        history[start..].to_vec()
    }

    /// Clear history
    pub async fn clear_history(&self) {
        let mut history = self.history.write().await;
        history.clear();
    }

    /// Reset state to default
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        *state = RuntimeState::default();
        self.clear_history().await;
    }

    /// Calibrate with neutral pose
    pub async fn calibrate(&self, neutral_position: Vector3<f32>) {
        let mut state = self.state.write().await;
        state.calibration_baseline = neutral_position;
        state.is_calibrated = true;
        state.gesture_state = GestureState::Idle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_manager() {
        let manager = StateManager::new(10);
        
        let state = RuntimeState {
            hand_position: Vector3::new(1.0, 2.0, 3.0),
            ..Default::default()
        };
        
        manager.update(state).await;
        
        let retrieved = manager.get().await;
        assert_eq!(retrieved.hand_position.x, 1.0);
        assert_eq!(retrieved.hand_position.y, 2.0);
        assert_eq!(retrieved.hand_position.z, 3.0);
        
        // Test history
        let history = manager.get_recent_history(5).await;
        assert_eq!(history.len(), 1);
    }
}