use crate::spatial::spatial_state::SpatialConfidence;
use crate::gestures::confidence::GestureConfidence;

/// Fuse multiple confidence sources for robust gesture detection
pub struct ConfidenceFuser {
    weights: ConfidenceWeights,
    min_confidence: f32,
}

#[derive(Debug, Clone)]
pub struct ConfidenceWeights {
    pub gesture: f32,
    pub tracking: f32,
    pub depth: f32,
    pub visibility: f32,
}

impl Default for ConfidenceWeights {
    fn default() -> Self {
        Self {
            gesture: 0.5,
            tracking: 0.2,
            depth: 0.15,
            visibility: 0.15,
        }
    }
}

impl ConfidenceFuser {
    pub fn new(weights: ConfidenceWeights, min_confidence: f32) -> Self {
        Self { weights, min_confidence }
    }
    
    /// Fuse gesture confidence with spatial confidences
    pub fn fuse(
        &self,
        gesture_confidence: &GestureConfidence,
        spatial_confidence: &SpatialConfidence,
    ) -> GestureConfidence {
        let base_confidence = gesture_confidence.highest().1;
        
        // Calculate overall spatial confidence
        let spatial_confidence_score = spatial_confidence.overall;
        
        // Fuse confidences
        let fused_confidence = base_confidence * (0.5 + 0.5 * spatial_confidence_score);
        
        // Apply minimum threshold
        let final_confidence = if fused_confidence < self.min_confidence {
            0.0
        } else {
            fused_confidence
        };
        
        // Create new confidence with fused values
        let mut fused = gesture_confidence.clone();
        // Scale all confidences by the fused factor
        let scale = final_confidence / base_confidence.max(0.001);
        fused.swipe_left *= scale;
        fused.swipe_right *= scale;
        fused.swipe_up *= scale;
        fused.swipe_down *= scale;
        fused.pinch *= scale;
        fused.fist *= scale;
        fused.none = 1.0 - fused.swipe_left.max(fused.swipe_right)
            .max(fused.swipe_up)
            .max(fused.swipe_down)
            .max(fused.pinch)
            .max(fused.fist);
        
        fused
    }
    
    /// Calculate overall confidence from components
    pub fn calculate_overall_confidence(
        &self,
        gesture_conf: f32,
        spatial_conf: &SpatialConfidence,
    ) -> f32 {
        let spatial_score = spatial_conf.overall;
        
        let weighted = self.weights.gesture * gesture_conf
            + self.weights.tracking * spatial_conf.tracking
            + self.weights.depth * spatial_conf.depth
            + self.weights.visibility * spatial_conf.visibility;
        
        weighted.clamp(0.0, 1.0)
    }
}

impl Default for ConfidenceFuser {
    fn default() -> Self {
        Self::new(ConfidenceWeights::default(), 0.3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gestures::confidence::GestureConfidence;
    
    #[test]
    fn test_confidence_fusion() {
        let fuser = ConfidenceFuser::default();
        
        let gesture_conf = GestureConfidence {
            swipe_left: 0.0,
            swipe_right: 0.8,
            swipe_up: 0.0,
            swipe_down: 0.0,
            pinch: 0.0,
            fist: 0.0,
            none: 0.2,
        };
        
        let spatial_conf = SpatialConfidence {
            tracking: 0.9,
            depth: 0.8,
            visibility: 0.9,
            overall: 0.85,
        };
        
        let fused = fuser.fuse(&gesture_conf, &spatial_conf);
        assert!(fused.swipe_right > 0.0);
        assert!(fused.swipe_right < 0.8); // Should be reduced by spatial confidence
    }
}