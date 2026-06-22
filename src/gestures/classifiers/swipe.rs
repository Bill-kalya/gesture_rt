use nalgebra::Vector3;
use crate::gestures::temporal_engine::GestureFeatures;

#[derive(Debug, Clone)]
pub struct SwipeClassifier;

impl SwipeClassifier {
    pub fn new() -> Self {
        Self
    }
    
    pub fn detect(&self, features: &GestureFeatures) -> f32 {
        let disp = features.displacement;
        let path_len = features.path_length;
        
        if path_len < 0.05 {
            return 0.0; // Too small to be a swipe
        }
        
        let direction_consistency = features.direction_consistency;
        
        if direction_consistency < 0.6 {
            return 0.0; // Not directional enough
        }
        
        // Determine swipe axis
        let abs_disp = Vector3::new(disp.x.abs(), disp.y.abs(), disp.z.abs());
        let max_axis = if abs_disp.x > abs_disp.y && abs_disp.x > abs_disp.z {
            0 // X axis (left/right)
        } else if abs_disp.y > abs_disp.x && abs_disp.y > abs_disp.z {
            1 // Y axis (up/down)
        } else {
            2 // Z axis (in/out - not typically a swipe)
        };
        
        if max_axis == 2 {
            return 0.0;
        }
        
        // Confidence based on displacement magnitude and direction consistency
        let magnitude_confidence = (disp.norm() / 0.2).min(1.0);
        magnitude_confidence * direction_consistency
    }
    
    pub fn direction(&self, features: &GestureFeatures) -> Option<SwipeDirection> {
        let disp = features.displacement;
        
        if disp.x.abs() > disp.y.abs() && disp.x.abs() > 0.03 {
            if disp.x > 0.0 {
                Some(SwipeDirection::Right)
            } else {
                Some(SwipeDirection::Left)
            }
        } else if disp.y.abs() > disp.x.abs() && disp.y.abs() > 0.03 {
            if disp.y > 0.0 {
                Some(SwipeDirection::Up)
            } else {
                Some(SwipeDirection::Down)
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwipeDirection {
    Left,
    Right,
    Up,
    Down,
}