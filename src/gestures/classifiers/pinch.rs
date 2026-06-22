use nalgebra::Vector3;

#[derive(Debug, Clone)]
pub struct PinchClassifier;

impl PinchClassifier {
    pub fn new() -> Self {
        Self
    }
    
    /// Detect pinch gesture based on thumb-index finger distance
    pub fn detect(&self, thumb_tip: Vector3<f32>, index_tip: Vector3<f32>) -> f32 {
        let distance = (thumb_tip - index_tip).norm();
        
        // Pinch when distance < 0.02 meters (2cm)
        if distance < 0.02 {
            let confidence = 1.0 - (distance / 0.02);
            confidence.min(1.0)
        } else {
            0.0
        }
    }
    
    /// Detect pinch with velocity (faster pinch = higher confidence)
    pub fn detect_with_velocity(
        &self,
        thumb_tip: Vector3<f32>,
        index_tip: Vector3<f32>,
        thumb_vel: Vector3<f32>,
        index_vel: Vector3<f32>,
    ) -> f32 {
        let distance = (thumb_tip - index_tip).norm();
        let closing_velocity = (thumb_vel - index_vel).norm();
        
        if distance < 0.02 {
            let distance_conf = 1.0 - (distance / 0.02);
            let velocity_conf = (closing_velocity / 0.5).min(1.0);
            (distance_conf * 0.7 + velocity_conf * 0.3).min(1.0)
        } else {
            0.0
        }
    }
}