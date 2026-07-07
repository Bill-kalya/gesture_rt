use nalgebra::Vector3;
use crate::gestures::temporal_engine::GestureFeatures;

pub mod fuser;

#[derive(Debug, Clone)]
pub struct GestureLogits {
    pub swipe_left: f32,
    pub swipe_right: f32,
    pub swipe_up: f32,
    pub swipe_down: f32,
    pub pinch: f32,
    pub fist: f32,
    pub none: f32,
}

impl GestureLogits {
    pub fn zero() -> Self {
        Self {
            swipe_left: 0.0,
            swipe_right: 0.0,
            swipe_up: 0.0,
            swipe_down: 0.0,
            pinch: 0.0,
            fist: 0.0,
            none: 1.0,
        }
    }
}

/// Softmax function for probability distribution
pub fn softmax(logits: &mut [f32]) {
    let max_logit = logits.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
    let mut sum = 0.0;
    for logit in logits.iter_mut() {
        *logit = (*logit - max_logit).exp();
        sum += *logit;
    }
    for logit in logits.iter_mut() {
        *logit /= sum;
    }
}

pub struct ConfidenceEngine {
    /// Threshold for action dispatch (0.0 to 1.0)
    pub dispatch_threshold: f32,
    /// Temporal smoothing factor (0.0 to 1.0)
    pub temporal_alpha: f32,
    /// Previous probabilities for smoothing
    prev_probs: Option<[f32; 7]>,
}

impl ConfidenceEngine {
    pub fn new(threshold: f32, temporal_alpha: f32) -> Self {
        Self {
            dispatch_threshold: threshold,
            temporal_alpha,
            prev_probs: None,
        }
    }

    /// Classify gesture from features using logits
    pub fn classify(&mut self, features: &GestureFeatures) -> GestureConfidence {
        let mut logits = GestureLogits::zero();
        
        // Heuristic logit calculation (Phase 1 - will be replaced by learned weights)
        // Swipe detection based on displacement and direction consistency
        let disp = features.displacement;
        let path_len = features.path_length;
        
        if path_len > 0.05 && features.direction_consistency > 0.7 {
            // Strong directional gesture
            let magnitude = disp.norm();
            
            if disp.x.abs() > disp.y.abs() && disp.x.abs() > disp.z.abs() {
                // Horizontal swipe
                if disp.x > 0.03 {
                    logits.swipe_right = magnitude * 5.0;
                } else if disp.x < -0.03 {
                    logits.swipe_left = magnitude * 5.0;
                }
            } else if disp.y.abs() > disp.x.abs() && disp.y.abs() > disp.z.abs() {
                // Vertical swipe
                if disp.y > 0.03 {
                    logits.swipe_up = magnitude * 5.0;
                } else if disp.y < -0.03 {
                    logits.swipe_down = magnitude * 5.0;
                }
            }
        }
        
        // Convert logits to array for softmax
        let mut logit_array = [
            logits.swipe_left,
            logits.swipe_right,
            logits.swipe_up,
            logits.swipe_down,
            logits.pinch,
            logits.fist,
            logits.none,
        ];
        
        // Apply softmax
        softmax(&mut logit_array);
        
        // Temporal smoothing
        let probs = if let Some(prev) = self.prev_probs {
            let mut smoothed = [0.0; 7];
            for i in 0..7 {
                smoothed[i] = self.temporal_alpha * logit_array[i] + (1.0 - self.temporal_alpha) * prev[i];
            }
            smoothed
        } else {
            logit_array
        };
        
        self.prev_probs = Some(probs);
        
        GestureConfidence {
            swipe_left: probs[0],
            swipe_right: probs[1],
            swipe_up: probs[2],
            swipe_down: probs[3],
            pinch: probs[4],
            fist: probs[5],
            none: probs[6],
        }
    }
    
    pub fn reset(&mut self) {
        self.prev_probs = None;
    }
}

#[derive(Debug, Clone)]
pub struct GestureConfidence {
    pub swipe_left: f32,
    pub swipe_right: f32,
    pub swipe_up: f32,
    pub swipe_down: f32,
    pub pinch: f32,
    pub fist: f32,
    pub none: f32,
}

impl GestureConfidence {
    pub fn highest(&self) -> (GestureType, f32) {
        let variants = [
            (GestureType::SwipeLeft, self.swipe_left),
            (GestureType::SwipeRight, self.swipe_right),
            (GestureType::SwipeUp, self.swipe_up),
            (GestureType::SwipeDown, self.swipe_down),
            (GestureType::Pinch, self.pinch),
            (GestureType::Fist, self.fist),
            (GestureType::None, self.none),
        ];
        
        variants.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap_or((GestureType::None, 0.0))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GestureType {
    SwipeLeft,
    SwipeRight,
    SwipeUp,
    SwipeDown,
    Pinch,
    Fist,
    None,
}