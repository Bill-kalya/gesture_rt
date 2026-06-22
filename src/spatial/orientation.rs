use nalgebra::{Vector3, Matrix3, Rotation3};

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