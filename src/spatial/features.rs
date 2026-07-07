use nalgebra::Vector3;
use crate::spatial::spatial_state::HandOrientation;

/// Engineered features for gesture recognition
/// These features improve accuracy with smaller ML models
#[derive(Debug, Clone, Default)]
pub struct GestureFeatures {
    // Hand-level features
    pub hand_velocity: Vector3<f32>,
    pub hand_acceleration: Vector3<f32>,
    pub hand_orientation: Vector3<f32>,
    pub hand_openness: f32,
    
    // Finger features
    pub finger_angles: [f32; 5],  // Five fingers
    pub finger_distances: [f32; 5],
    
    // Palm features
    pub palm_normal: Vector3<f32>,
    pub palm_center: Vector3<f32>,
    
    // Temporal features
    pub trajectory_curvature: f32,
    pub speed_variance: f32,
    pub direction_consistency: f32,
    
    // Spatial features (NEW)
    pub distance_from_camera: f32,
    pub hand_size: f32,
    pub palm_orientation: HandOrientation,
    pub normalized_motion: Vector3<f32>,
}

pub struct FeatureGenerator {
    landmark_indices: HandLandmarkIndices,
    prev_features: Option<GestureFeatures>,
}

impl FeatureGenerator {
    pub fn new() -> Self {
        Self {
            landmark_indices: HandLandmarkIndices::new(),
            prev_features: None,
        }
    }
    
    pub fn generate(&mut self, landmarks: &[Vector3<f32>], dt_secs: f32) -> Option<GestureFeatures> {
        if landmarks.len() < 21 {
            return None;
        }
        
        let mut features = GestureFeatures::default();
        
        // Palm center (average of wrist and MCPs)
        let wrist = landmarks[0];
        let index_mcp = landmarks[5];
        let pinky_mcp = landmarks[17];
        features.palm_center = (wrist + index_mcp + pinky_mcp) / 3.0;
        
        // Hand velocity and acceleration
        if let Some(prev) = &self.prev_features {
            features.hand_velocity = (features.palm_center - prev.palm_center) / dt_secs.max(0.001);
        }
        
        // Palm normal
        let palm_center = features.palm_center;
        let to_index = index_mcp - palm_center;
        let to_pinky = pinky_mcp - palm_center;
        features.palm_normal = to_index.cross(&to_pinky).normalize();
        
        // Hand openness (distance from fingertips to palm center)
        let finger_tips = [8, 12, 16, 20]; // Index, Middle, Ring, Pinky tips
        let mut avg_finger_distance = 0.0;
        for tip_idx in finger_tips.iter() {
            avg_finger_distance += (landmarks[*tip_idx] - palm_center).norm();
        }
        avg_finger_distance /= finger_tips.len() as f32;
        features.hand_openness = (avg_finger_distance / 0.1).min(1.0);
        
        // Finger angles (simplified - angle of each finger relative to palm)
        for i in 0..5 {
            let tip_idx = self.landmark_indices.finger_tip(i);
            let mcp_idx = self.landmark_indices.finger_mcp(i);
            let tip = landmarks[tip_idx];
            let mcp = landmarks[mcp_idx];
            let finger_vec = tip - mcp;
            features.finger_angles[i] = finger_vec.angle(&features.palm_normal);
            features.finger_distances[i] = finger_vec.norm();
        }
        
        // Direction consistency if we have previous features
        if let Some(prev) = &self.prev_features {
            let velocity_mag = features.hand_velocity.norm();
            if velocity_mag > 0.01 {
                let prev_vel_mag = prev.hand_velocity.norm();
                if prev_vel_mag > 0.01 {
                    let cos_angle = features.hand_velocity.dot(&prev.hand_velocity) / (velocity_mag * prev_vel_mag);
                    features.direction_consistency = cos_angle.clamp(-1.0, 1.0);
                }
            }
        }
        
        // Store for next frame
        self.prev_features = Some(features.clone());
        
        Some(features)
    }
}

struct HandLandmarkIndices {
    // Wrist
    wrist: usize,
    // Thumb
    thumb_mcp: usize,
    thumb_tip: usize,
    // Index
    index_mcp: usize,
    index_tip: usize,
    // Middle
    middle_mcp: usize,
    middle_tip: usize,
    // Ring
    ring_mcp: usize,
    ring_tip: usize,
    // Pinky
    pinky_mcp: usize,
    pinky_tip: usize,
}

impl HandLandmarkIndices {
    fn new() -> Self {
        Self {
            wrist: 0,
            thumb_mcp: 1, thumb_tip: 4,
            index_mcp: 5, index_tip: 8,
            middle_mcp: 9, middle_tip: 12,
            ring_mcp: 13, ring_tip: 16,
            pinky_mcp: 17, pinky_tip: 20,
        }
    }
    
    fn finger_tip(&self, finger: usize) -> usize {
        match finger {
            0 => self.thumb_tip,
            1 => self.index_tip,
            2 => self.middle_tip,
            3 => self.ring_tip,
            4 => self.pinky_tip,
            _ => 0,
        }
    }
    
    fn finger_mcp(&self, finger: usize) -> usize {
        match finger {
            0 => self.thumb_mcp,
            1 => self.index_mcp,
            2 => self.middle_mcp,
            3 => self.ring_mcp,
            4 => self.pinky_mcp,
            _ => 0,
        }
    }
}

impl Default for FeatureGenerator {
    fn default() -> Self {
        Self::new()
    }
}