use nalgebra::Vector3;
use std::collections::VecDeque;
use crate::spatial::coordinate_system::WorldCoordinate;

/// Motion history for temporal analysis
pub struct MotionHistory {
    pub positions: VecDeque<Vector3<f32>>,
    pub velocities: VecDeque<Vector3<f32>>,
    pub timestamps: VecDeque<u64>,
    max_len: usize,
}

impl MotionHistory {
    pub fn new(max_frames: usize) -> Self {
        Self {
            positions: VecDeque::with_capacity(max_frames),
            velocities: VecDeque::with_capacity(max_frames),
            timestamps: VecDeque::with_capacity(max_frames),
            max_len: max_frames,
        }
    }

    pub fn push(&mut self, position: Vector3<f32>, timestamp_ns: u64) {
        // Calculate velocity if we have previous point
        if let Some(prev_pos) = self.positions.back() {
            if let Some(prev_time) = self.timestamps.back() {
                let dt = (timestamp_ns - prev_time) as f32 / 1_000_000_000.0;
                if dt > 0.0 {
                    let velocity = (position - *prev_pos) / dt;
                    self.velocities.push_back(velocity);
                }
            }
        }
        
        self.positions.push_back(position);
        self.timestamps.push_back(timestamp_ns);
        
        // Maintain max length
        while self.positions.len() > self.max_len {
            self.positions.pop_front();
            self.velocities.pop_front();
            self.timestamps.pop_front();
        }
    }

    pub fn mean_velocity(&self) -> Vector3<f32> {
        if self.velocities.is_empty() {
            return Vector3::zeros();
        }
        let sum = self.velocities.iter().fold(Vector3::zeros(), |acc, v| acc + v);
        sum / (self.velocities.len() as f32)
    }

    pub fn path_length(&self) -> f32 {
        if self.positions.len() < 2 {
            return 0.0;
        }
        let mut total = 0.0;
        for i in 1..self.positions.len() {
            total += (self.positions[i] - self.positions[i-1]).norm();
        }
        total
    }

    pub fn clear(&mut self) {
        self.positions.clear();
        self.velocities.clear();
        self.timestamps.clear();
    }
}

/// Feature vector for gesture classification
#[derive(Debug, Clone)]
pub struct GestureFeatures {
    pub mean_velocity: Vector3<f32>,
    pub path_length: f32,
    pub displacement: Vector3<f32>,
    pub direction_consistency: f32,
    pub duration_secs: f32,
}

impl GestureFeatures {
    pub fn from_history(history: &MotionHistory) -> Option<Self> {
        if history.positions.len() < 3 {
            return None;
        }
        
        let start = history.positions.front().unwrap();
        let end = history.positions.back().unwrap();
        let displacement = *end - *start;
        
        let mean_velocity = history.mean_velocity();
        let path_length = history.path_length();
        
        // Direction consistency: ratio of net displacement to path length
        let direction_consistency = if path_length > 0.0 {
            displacement.norm() / path_length
        } else {
            0.0
        };
        
        let duration_secs = if history.timestamps.len() >= 2 {
            (history.timestamps.back().unwrap() - history.timestamps.front().unwrap()) as f32 / 1_000_000_000.0
        } else {
            0.0
        };
        
        Some(GestureFeatures {
            mean_velocity,
            path_length,
            displacement,
            direction_consistency,
            duration_secs,
        })
    }
}