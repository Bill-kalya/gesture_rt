use nalgebra::Vector3;

#[derive(Debug, Clone)]
pub struct FistClassifier;

impl FistClassifier {
    pub fn new() -> Self {
        Self
    }
    
    /// Detect fist based on finger tip distances from palm
    pub fn detect(&self, finger_tips: &[Vector3<f32>], palm_center: Vector3<f32>) -> f32 {
        if finger_tips.len() < 5 {
            return 0.0;
        }
        
        // Average distance from finger tips to palm center
        let avg_distance: f32 = finger_tips
            .iter()
            .map(|tip| (tip - palm_center).norm())
            .sum::<f32>()
            / finger_tips.len() as f32;
        
        // Fist = all fingers close to palm (< 5cm)
        if avg_distance < 0.05 {
            let confidence = 1.0 - (avg_distance / 0.05);
            confidence.min(1.0)
        } else {
            0.0
        }
    }
}