use nalgebra::Vector3;

/// Normalize landmarks to a canonical space
pub fn normalize_landmarks(
    landmarks: &[Vector3<f32>],
    reference_point: usize,
) -> Vec<Vector3<f32>> {
    if landmarks.is_empty() || reference_point >= landmarks.len() {
        return landmarks.to_vec();
    }
    
    let origin = landmarks[reference_point];
    landmarks
        .iter()
        .map(|p| *p - origin)
        .collect()
}

/// Scale normalization based on hand size
pub fn scale_normalize(
    landmarks: &mut [Vector3<f32>],
    scale: f32,
) {
    for point in landmarks.iter_mut() {
        *point /= scale;
    }
}

/// Compute hand scale from wrist to middle finger tip
pub fn compute_hand_scale(landmarks: &[Vector3<f32>]) -> f32 {
    if landmarks.len() < 13 {
        return 1.0;
    }
    // Wrist is index 0, middle finger tip is index 12
    (landmarks[12] - landmarks[0]).norm()
}