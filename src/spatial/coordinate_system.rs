use nalgebra as na;
use na::{Vector3, Matrix3};

/// Right-handed world coordinate system:
/// X = right, Y = up, Z = out of screen (toward user)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldCoordinate;

impl WorldCoordinate {
    /// Convert MediaPipe 2.5D landmark to world coordinates
    /// MediaPipe: X right, Y down, Z depth (forward from camera, but camera-relative)
    pub fn from_mediapipe(mediapipe_x: f32, mediapipe_y: f32, mediapipe_z: f32) -> Vector3<f32> {
        Vector3::new(
            mediapipe_x,                        // X: same direction (right)
            -mediapipe_y,                       // Y: invert (MediaPipe down → world up)
            mediapipe_z,                        // Z: depth, positive toward user
        )
    }

    /// Convert world coordinates to OpenCV projection space (for display)
    pub fn to_opencv(world: Vector3<f32>) -> (f32, f32) {
        (world.x, -world.y)  // OpenCV Y down
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_mediapipe_conversion() {
        let mediapipe = (0.5, 0.3, 0.2);
        let world = WorldCoordinate::from_mediapipe(mediapipe.0, mediapipe.1, mediapipe.2);
        
        assert_relative_eq!(world.x, 0.5);
        assert_relative_eq!(world.y, -0.3);
        assert_relative_eq!(world.z, 0.2);
    }
}