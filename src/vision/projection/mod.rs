use nalgebra::{Vector2, Vector3};

/// Pinhole camera model for perspective projection
pub struct PinholeCamera {
    pub focal_length_x: f32,
    pub focal_length_y: f32,
    pub principal_x: f32,
    pub principal_y: f32,
}

impl PinholeCamera {
    /// Create from image dimensions and FOV
    pub fn from_fov(width: f32, height: f32, fov_degrees: f32) -> Self {
        let fov_rad = fov_degrees.to_radians();
        let focal_length = width / (2.0 * (fov_rad / 2.0).tan());
        Self {
            focal_length_x: focal_length,
            focal_length_y: focal_length,
            principal_x: width / 2.0,
            principal_y: height / 2.0,
        }
    }

    /// Project world point to image coordinates (pixels)
    /// World point: X right, Y up, Z toward user (positive)
    /// Returns (u, v) pixel coordinates with origin at top-left
    pub fn project(&self, world_point: Vector3<f32>) -> Option<Vector2<f32>> {
        if world_point.z <= 0.0 {
            return None; // Behind camera
        }
        
        let u = self.focal_length_x * world_point.x / world_point.z + self.principal_x;
        let v = -self.focal_length_y * world_point.y / world_point.z + self.principal_y;
        
        Some(Vector2::new(u, v))
    }

    /// Unproject image point to ray in world coordinates
    /// Returns direction vector (not normalized)
    pub fn unproject(&self, image_point: Vector2<f32>, z: f32) -> Vector3<f32> {
        let x = (image_point.x - self.principal_x) * z / self.focal_length_x;
        let y = -(image_point.y - self.principal_y) * z / self.focal_length_y;
        Vector3::new(x, y, z)
    }
}

/// Orthographic approximation (for MVP when depth is unknown)
pub fn orthographic_approx(image_point: Vector2<f32>, scale: f32) -> Vector3<f32> {
    Vector3::new(
        image_point.x * scale,
        -image_point.y * scale,
        1.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_project_unproject_roundtrip() {
        let cam = PinholeCamera::from_fov(640.0, 480.0, 60.0);
        let world = Vector3::new(0.1, 0.05, 1.0);
        
        let proj = cam.project(world).unwrap();
        let back = cam.unproject(proj, world.z);
        
        assert_relative_eq!(back.x, world.x, epsilon = 1e-5);
        assert_relative_eq!(back.y, world.y, epsilon = 1e-5);
    }
}