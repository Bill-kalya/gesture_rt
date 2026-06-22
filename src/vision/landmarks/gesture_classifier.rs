use anyhow::{anyhow, Result};
use std::collections::VecDeque;

/// Gesture classification result.
#[derive(Debug, Clone)]
pub struct GestureInference {
    pub class_id: usize,
    pub confidence: f32,
    pub class_name: String,
}

/// Sliding window buffer for landmark sequences (T, 63) where T is temporal frames.
pub struct LandmarkSequenceBuffer {
    /// Stores flattened landmark tuples: [x1,y1,z1,...,x21,y21,z21]
    pub buffer: VecDeque<[f32; 63]>,
    pub max_frames: usize,
}

impl LandmarkSequenceBuffer {
    pub fn new(max_frames: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_frames),
            max_frames,
        }
    }

    /// Push a new landmark set (21 points × 3 coords).
    pub fn push(&mut self, landmarks: &[f32; 63]) {
        if self.buffer.len() >= self.max_frames {
            self.buffer.pop_front();
        }
        self.buffer.push_back(*landmarks);
    }

    /// Get the current buffer as a flattened tensor (frames * 63).
    pub fn as_tensor(&self) -> Vec<f32> {
        self.buffer.iter().flat_map(|l| l.iter().copied()).collect()
    }

    /// Return true if buffer is full (ready for inference).
    pub fn is_ready(&self) -> bool {
        self.buffer.len() >= self.max_frames
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

/// ONNX-backed gesture classifier that infers on landmark sequences.
pub struct GestureClassifier {
    pub model_path: String,
    pub class_names: Vec<String>,
    pub sequence_length: usize,
}

impl GestureClassifier {
    pub fn new<P: Into<String>>(model_path: P, class_names: Vec<String>, sequence_length: usize) -> Self {
        Self {
            model_path: model_path.into(),
            class_names,
            sequence_length,
        }
    }

    /// Run the gesture classifier on a landmark sequence tensor.
    /// `input_tensor` should be shape (1, sequence_length, 63) flattened to a vector.
    /// This method is a placeholder; it should call the ONNX Runtime session.
    pub fn infer(&self, _input_tensor: &[f32]) -> Result<GestureInference> {
        Err(anyhow!(
            "infer is not implemented: enable `onnx` feature and implement using onnxruntime crate"
        ))
    }
}
