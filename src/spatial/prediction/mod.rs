use nalgebra::Vector3;

/// Predict next position using constant acceleration model
pub fn predict_position(
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
    dt_secs: f32,
) -> Vector3<f32> {
    position + velocity * dt_secs + acceleration * 0.5 * dt_secs * dt_secs
}

/// Predict velocity using constant acceleration
pub fn predict_velocity(
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
    dt_secs: f32,
) -> Vector3<f32> {
    velocity + acceleration * dt_secs
}

/// Simple linear extrapolation (no acceleration)
pub fn linear_extrapolate(
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    dt_secs: f32,
) -> Vector3<f32> {
    position + velocity * dt_secs
}