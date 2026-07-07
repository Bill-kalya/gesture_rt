use nalgebra::{Vector3, Matrix3, Rotation3};
use crate::spatial::spatial_state::HandOrientation;

/// Quaternion-based orientation tracking
#[derive(Debug, Clone)]
pub struct OrientationTracker {
    current_rotation: Rotation3<f32>,
}

impl OrientationTracker {
    pub fn new() -> Self {
        Self {
            current_rotation: Rotation3::identity(),
        }
    }
    
    pub fn update_from_vectors(&mut self, from: Vector3<f32>, to: Vector3<f32>) {
        if from.norm() < 1e-6 || to.norm() < 1e-6 {
            return;
        }
        
        let from_norm = from.normalize();
        let to_norm = to.normalize();
        
        let axis = from_norm.cross(&to_norm);
        let angle = from_norm.dot(&to_norm).acos();
        
        if axis.norm() > 1e-6 {
            let rotation = Rotation3::from_axis_angle(&axis.normalize(), angle);
            self.current_rotation = rotation * self.current_rotation;
        }
    }
    
    pub fn rotation_matrix(&self) -> Matrix3<f32> {
        self.current_rotation.matrix()
    }
}

/// Hand orientation estimator with temporal smoothing
#[derive(Debug, Clone)]
pub struct OrientationEstimator {
    prev_orientation: Option<HandOrientation>,
}

impl OrientationEstimator {
    pub fn new() -> Self {
        Self {
            prev_orientation: None,
        }
    }
    
    /// Estimate orientation from palm landmarks
    pub fn estimate_orientation(
        &mut self,
        palm_center: Vector3<f32>,
        palm_normal: Vector3<f32>,
        wrist: Vector3<f32>,
    ) -> HandOrientation {
        // Calculate palm direction (from wrist to palm center)
        let palm_direction = (palm_center - wrist).normalize();
        
        // Calculate pitch (tilt forward/backward)
        let pitch = palm_direction.y.atan2(palm_direction.x);
        
        // Calculate yaw (rotation around vertical axis)
        let yaw = palm_direction.z.atan2(palm_direction.x);
        
        // Calculate roll (rotation around palm normal)
        let roll = palm_normal.z.atan2(palm_normal.x);
        
        // Smooth orientation changes
        let orientation = HandOrientation {
            pitch: pitch.clamp(-std::f32::consts::PI, std::f32::consts::PI),
            yaw: yaw.clamp(-std::f32::consts::PI, std::f32::consts::PI),
            roll: roll.clamp(-std::f32::consts::PI, std::f32::consts::PI),
        };
        
        // Apply temporal smoothing
        if let Some(prev) = &self.prev_orientation {
            let alpha = 0.3;
            let smoothed = HandOrientation {
                pitch: orientation.pitch * alpha + prev.pitch * (1.0 - alpha),
                yaw: orientation.yaw * alpha + prev.yaw * (1.0 - alpha),
                roll: orientation.roll * alpha + prev.roll * (1.0 - alpha),
            };
            self.prev_orientation = Some(smoothed.clone());
            smoothed
        } else {
            self.prev_orientation = Some(orientation.clone());
            orientation
        }
    }
    
    /// Get rotation matrix from orientation
    pub fn orientation_to_rotation(orientation: &HandOrientation) -> Matrix3<f32> {
        let rot = Rotation3::from_euler_angles(orientation.pitch, orientation.yaw, orientation.roll);
        rot.matrix()
    }
}

impl Default for OrientationEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn test_orientation_estimation() {
        let mut estimator = OrientationEstimator::new();
        
        let palm_center = Vector3::new(0.0, 0.0, 0.0);
        let palm_normal = Vector3::new(0.0, 0.0, 1.0);
        let wrist = Vector3::new(-10.0, 0.0, 0.0);
        
        let orientation = estimator.estimate_orientation(palm_center, palm_normal, wrist);
        
        assert!(orientation.pitch >= -std::f32::consts::PI);
        assert!(orientation.pitch <= std::f32::consts::PI);
        assert!(orientation.yaw >= -std::f32::consts::PI);
        assert!(orientation.yaw <= std::f32::consts::PI);
    }

    #[test]
    fn test_temporal_smoothing() {
        let mut estimator = OrientationEstimator::new();
        
        let palm_center = Vector3::new(0.0, 0.0, 0.0);
        let palm_normal = Vector3::new(0.0, 0.0, 1.0);
        let wrist = Vector3::new(-10.0, 0.0, 0.0);
        
        let first = estimator.estimate_orientation(palm_center, palm_normal, wrist);
        let second = estimator.estimate_orientation(palm_center, palm_normal, wrist);
        
        // Second orientation should be smoothed towards first
        assert!((second.pitch - first.pitch).abs() < 0.1);
    }
}
