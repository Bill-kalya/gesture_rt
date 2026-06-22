use nalgebra::Vector3;

/// Compute velocity from position history
pub fn compute_velocity(
    current_pos: Vector3<f32>,
    previous_pos: Vector3<f32>,
    dt_secs: f32,
) -> Vector3<f32> {
    if dt_secs <= 0.0 {
        return Vector3::zeros();
    }
    (current_pos - previous_pos) / dt_secs
}

/// Compute acceleration from velocity history
pub fn compute_acceleration(
    current_vel: Vector3<f32>,
    previous_vel: Vector3<f32>,
    dt_secs: f32,
) -> Vector3<f32> {
    if dt_secs <= 0.0 {
        return Vector3::zeros();
    }
    (current_vel - previous_vel) / dt_secs
}

/// Compute hand orientation from palm landmarks
pub fn compute_palm_normal(
    wrist: Vector3<f32>,
    index_mcp: Vector3<f32>,
    pinky_mcp: Vector3<f32>,
) -> Vector3<f32> {
    let palm_center = (wrist + index_mcp + pinky_mcp) / 3.0;
    let to_index = index_mcp - palm_center;
    let to_pinky = pinky_mcp - palm_center;
    to_index.cross(&to_pinky).normalize()
}