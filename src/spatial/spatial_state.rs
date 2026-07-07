use nalgebra::{Vector3, Matrix3};
use serde::{Serialize, Deserialize};

/// Complete spatial state of a hand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialHand {
    pub landmarks: Vec<Vector3<f32>>,
    pub distance_cm: f32,
    pub hand_size_cm: f32,
    pub palm_center: Vector3<f32>,
    pub palm_normal: Vector3<f32>,
    pub palm_rotation: Matrix3<f32>,
    pub orientation: HandOrientation,
    pub confidence: SpatialConfidence,
}

/// Hand orientation in 3D space
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandOrientation {
    pub pitch: f32,  // Tilt forward/backward
    pub yaw: f32,    // Rotation left/right
    pub roll: f32,   // Rotation around palm normal
}

/// Confidence scores for spatial data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialConfidence {
    pub tracking: f32,    // Tracking confidence
    pub depth: f32,       // Depth estimation confidence
    pub visibility: f32,  // Hand visibility
    pub overall: f32,     // Fused confidence
}

/// Frame state with both hands (future-proof)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameState {
    pub left_hand: Option<SpatialHand>,
    pub right_hand: Option<SpatialHand>,
    pub frame_id: u64,
    pub timestamp_ns: u64,
}

/// User profile for calibration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub preferred_distance_cm: f32,
    pub arm_length_cm: f32,
    pub hand_size_cm: f32,
    pub dominant_hand: DominantHand,
    pub calibrated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DominantHand {
    Left,
    Right,
    Ambidextrous,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            preferred_distance_cm: 45.0,
            arm_length_cm: 65.0,
            hand_size_cm: 8.5,
            dominant_hand: DominantHand::Right,
            calibrated: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn test_spatial_hand_creation() {
        let hand = SpatialHand {
            landmarks: vec![Vector3::new(0.0, 0.0, 0.0); 21],
            distance_cm: 45.0,
            hand_size_cm: 8.5,
            palm_center: Vector3::new(0.0, 0.0, 0.0),
            palm_normal: Vector3::new(0.0, 0.0, 1.0),
            palm_rotation: Matrix3::identity(),
            orientation: HandOrientation {
                pitch: 0.0,
                yaw: 0.0,
                roll: 0.0,
            },
            confidence: SpatialConfidence {
                tracking: 0.9,
                depth: 0.85,
                visibility: 0.9,
                overall: 0.85,
            },
        };
        assert_eq!(hand.distance_cm, 45.0);
        assert_eq!(hand.landmarks.len(), 21);
    }

    #[test]
    fn test_user_profile_default() {
        let profile = UserProfile::default();
        assert_eq!(profile.preferred_distance_cm, 45.0);
        assert_eq!(profile.hand_size_cm, 8.5);
        assert!(!profile.calibrated);
    }
}