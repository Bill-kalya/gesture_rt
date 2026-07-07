use nalgebra::Vector3;
use crate::spatial::spatial_state::SpatialHand;

/// Normalize spatial data for consistent gesture recognition
pub struct SpatialNormalizer {
    calibration_baseline: Option<Vector3<f32>>,
    distance_scale: f32,
    position_scale: f32,
}

impl SpatialNormalizer {
    pub fn new() -> Self {
        Self {
            calibration_baseline: None,
            distance_scale: 1.0,
            position_scale: 1.0,
        }
    }
    
    /// Calibrate with neutral hand position
    pub fn calibrate(&mut self, neutral_position: Vector3<f32>, distance_cm: f32) {
        self.calibration_baseline = Some(neutral_position);
        self.distance_scale = 45.0 / distance_cm; // Normalize to 45cm
        self.position_scale = 0.1; // Scale to reasonable range
    }
    
    /// Normalize hand position to canonical space
    pub fn normalize_position(&self, position: Vector3<f32>) -> Vector3<f32> {
        if let Some(baseline) = self.calibration_baseline {
            let diff = position - baseline;
            diff * self.distance_scale * self.position_scale
        } else {
            position * self.position_scale
        }
    }
    
    /// Normalize motion based on distance
    pub fn normalize_motion(&self, motion: Vector3<f32>, distance_cm: f32) -> Vector3<f32> {
        let scale = 45.0 / distance_cm.max(10.0);
        motion * scale * self.position_scale
    }
    
    /// Normalize hand size
    pub fn normalize_hand_size(&self, hand_size_cm: f32) -> f32 {
        hand_size_cm * self.distance_scale
    }
    
    /// Get normalized spatial hand
    pub fn normalize_spatial_hand(&self, hand: &SpatialHand) -> SpatialHand {
        let mut normalized = hand.clone();
        
        // Normalize positions
        normalized.landmarks = normalized.landmarks
            .iter()
            .map(|&pos| self.normalize_position(pos))
            .collect();
        
        normalized.palm_center = self.normalize_position(hand.palm_center);
        
        // Scale distance to normalized range
        normalized.distance_cm = 45.0; // Normalized distance
        
        normalized
    }
}

impl Default for SpatialNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn test_normalize_position_without_calibration() {
        let normalizer = SpatialNormalizer::new();
        let pos = Vector3::new(100.0, 200.0, 300.0);
        let normalized = normalizer.normalize_position(pos);
        
        // Without calibration, should just scale by position_scale
        assert_eq!(normalized.x, 10.0);
        assert_eq!(normalized.y, 20.0);
        assert_eq!(normalized.z, 30.0);
    }

    #[test]
    fn test_normalize_position_with_calibration() {
        let mut normalizer = SpatialNormalizer::new();
        let baseline = Vector3::new(100.0, 200.0, 300.0);
        normalizer.calibrate(baseline, 50.0); // Calibrate at 50cm
        
        let pos = Vector3::new(110.0, 210.0, 310.0);
        let normalized = normalizer.normalize_position(pos);
        
        // distance_scale = 45.0 / 50.0 = 0.9
        // position_scale = 0.1
        // diff = (10, 10, 10)
        // normalized = (10, 10, 10) * 0.9 * 0.1 = (0.9, 0.9, 0.9)
        assert!((normalized.x - 0.9).abs() < 0.001);
        assert!((normalized.y - 0.9).abs() < 0.001);
        assert!((normalized.z - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_normalize_motion() {
        let normalizer = SpatialNormalizer::new();
        let motion = Vector3::new(10.0, 20.0, 30.0);
        let normalized = normalizer.normalize_motion(motion, 50.0);
        
        // scale = 45.0 / 50.0 = 0.9
        // normalized = (10, 20, 30) * 0.9 * 0.1 = (0.9, 1.8, 2.7)
        assert!((normalized.x - 0.9).abs() < 0.001);
        assert!((normalized.y - 1.8).abs() < 0.001);
        assert!((normalized.z - 2.7).abs() < 0.001);
    }

    #[test]
    fn test_normalize_hand_size() {
        let normalizer = SpatialNormalizer::new();
        let size = normalizer.normalize_hand_size(10.0);
        assert_eq!(size, 10.0); // distance_scale is 1.0 by default
    }
}