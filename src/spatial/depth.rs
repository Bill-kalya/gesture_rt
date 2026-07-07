use nalgebra::Vector3;
use anyhow::Result;

/// Distance estimation strategies
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DepthMethod {
    /// Estimate from hand size in pixels
    HandSize,
    /// Use monocular depth model (MiDaS, DepthAnything)
    MonocularDepth,
    /// Use known focal length and hand size
    Perspective,
    /// Use depth sensor (if available)
    DepthSensor,
}

/// Depth estimator for hand distance
pub struct DepthEstimator {
    method: DepthMethod,
    focal_length_pixels: f32,
    real_palm_width_cm: f32,
    calibration_factor: f32,
}

impl DepthEstimator {
    pub fn new(method: DepthMethod) -> Self {
        Self {
            method,
            focal_length_pixels: 720.0, // Typical for 720p cameras
            real_palm_width_cm: 8.5,    // Average adult palm width
            calibration_factor: 1.0,
        }
    }
    
    /// Set camera focal length from camera calibration
    pub fn set_focal_length(&mut self, focal_length_pixels: f32) {
        self.focal_length_pixels = focal_length_pixels;
    }
    
    /// Calibrate palm width for specific user
    pub fn calibrate_palm_width(&mut self, palm_width_cm: f32) {
        self.real_palm_width_cm = palm_width_cm;
    }
    
    /// Set calibration factor from user calibration
    pub fn set_calibration_factor(&mut self, factor: f32) {
        self.calibration_factor = factor;
    }
    
    /// Estimate distance from hand landmarks
    pub fn estimate_distance(&self, landmarks: &[Vector3<f32>]) -> Option<f32> {
        if landmarks.len() < 21 {
            return None;
        }
        
        match self.method {
            DepthMethod::HandSize => self.estimate_from_hand_size(landmarks),
            DepthMethod::Perspective => self.estimate_from_perspective(landmarks),
            DepthMethod::MonocularDepth => {
                // Would require depth model inference
                // For now, fallback to hand size
                self.estimate_from_hand_size(landmarks)
            }
            DepthMethod::DepthSensor => {
                // Would use depth sensor data
                self.estimate_from_hand_size(landmarks)
            }
        }
    }
    
    /// Estimate distance from hand size in pixels
    fn estimate_from_hand_size(&self, landmarks: &[Vector3<f32>]) -> Option<f32> {
        // Use thumb MCP (2) and pinky MCP (17) for palm width
        let thumb_mcp = landmarks[2];
        let pinky_mcp = landmarks[17];
        let pixel_width = (thumb_mcp - pinky_mcp).norm();
        
        if pixel_width < 1.0 {
            return None;
        }
        
        // Distance = (real_width * focal_length) / pixel_width
        let distance = (self.real_palm_width_cm * self.focal_length_pixels) / pixel_width;
        Some(distance * self.calibration_factor)
    }
    
    /// Estimate from perspective projection
    fn estimate_from_perspective(&self, landmarks: &[Vector3<f32>]) -> Option<f32> {
        // Use relative depth from landmarks if available
        // MediaPipe provides relative Z, we can scale it
        let wrist = landmarks[0];
        let index_tip = landmarks[8];
        
        let relative_z = (index_tip.z - wrist.z).abs();
        if relative_z < 0.001 {
            return None;
        }
        
        // Scale relative Z to absolute distance
        // This requires calibration
        let distance = relative_z * 100.0 * self.calibration_factor;
        Some(distance.clamp(10.0, 200.0))
    }
    
    /// Get hand size in cm from landmarks
    pub fn estimate_hand_size(&self, landmarks: &[Vector3<f32>]) -> Option<f32> {
        if landmarks.len() < 21 {
            return None;
        }
        
        let thumb_mcp = landmarks[2];
        let pinky_mcp = landmarks[17];
        let pixel_width = (thumb_mcp - pinky_mcp).norm();
        
        // If we know the distance, we can estimate real size
        if let Some(distance_cm) = self.estimate_distance(landmarks) {
            let real_size = (pixel_width * distance_cm) / self.focal_length_pixels;
            Some(real_size)
        } else {
            None
        }
    }
}

impl Default for DepthEstimator {
    fn default() -> Self {
        Self::new(DepthMethod::HandSize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_distance_estimation() {
        let estimator = DepthEstimator::default();
        let mut landmarks = Vec::new();
        
        // Create simulated landmarks
        for i in 0..21 {
            let x = (i as f32) * 0.01;
            let y = (i as f32) * 0.01;
            let z = 0.5 + (i as f32) * 0.001;
            landmarks.push(Vector3::new(x, y, z));
        }
        
        let distance = estimator.estimate_distance(&landmarks);
        assert!(distance.is_some());
        assert!(distance.unwrap() > 0.0);
    }
}