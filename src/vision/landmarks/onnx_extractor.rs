use anyhow::{anyhow, Result};
use nalgebra::Vector3;
use opencv::{core, imgproc};

use crate::vision::landmarks::extractor::LandmarkExtractor;

/// ONNX-backed landmark extractor skeleton.
///
/// This implementation performs ROI cropping, resizing and normalization using OpenCV,
/// and expects a model that consumes an input tensor of shape (1,3,H,W) with float32 values.
/// The actual ONNX Runtime inference call is left to the `run_model` method which must
/// be implemented to call into `onnxruntime` session APIs. This keeps the preprocessing
/// and ROI/tracking logic in pure Rust and OpenCV.
pub struct OnnxLandmarkExtractor {
    pub model_path: String,
    pub input_w: i32,
    pub input_h: i32,
    pub mean: [f32; 3],
    pub std: [f32; 3],
}

impl OnnxLandmarkExtractor {
    pub fn new<P: Into<String>>(model_path: P, input_w: i32, input_h: i32) -> Self {
        Self {
            model_path: model_path.into(),
            input_w,
            input_h,
            mean: [0.0, 0.0, 0.0],
            std: [1.0, 1.0, 1.0],
        }
    }

    /// Preprocess crop (BGR Mat) -> CHW f32 tensor
    pub fn preprocess(&self, crop_bgr: &core::Mat) -> Result<Vec<f32>> {
        // Convert BGR -> RGB
        let mut rgb = core::Mat::default();
        imgproc::cvt_color(crop_bgr, &mut rgb, imgproc::COLOR_BGR2RGB, 0)?;

        // Resize to model input
        let mut resized = core::Mat::default();
        imgproc::resize(&rgb, &mut resized, core::Size::new(self.input_w, self.input_h), 0.0, 0.0, imgproc::INTER_LINEAR)?;

        // Convert to f32 and CHW order
        let mut float_mat = core::Mat::default();
        resized.convert_to(&mut float_mat, core::CV_32F, 1.0, 0.0)?;

        let (h, w) = (self.input_h as usize, self.input_w as usize);
        let channels = 3usize;
        let mut tensor: Vec<f32> = vec![0.0; channels * h * w];

        for y in 0..h {
            for x in 0..w {
                let pix = resized.at_2d::<core::Vec3b>(y as i32, x as i32)?;
                // RGB order
                for c in 0..3 {
                    let v = pix[c] as f32 / 255.0;
                    let v = (v - self.mean[c]) / self.std[c];
                    // CHW index
                    let idx = c * h * w + y * w + x;
                    tensor[idx] = v;
                }
            }
        }

        Ok(tensor)
    }

    /// Run the ONNX model. This method should call the ONNX Runtime session and return a
    /// vector of floats representing model outputs (e.g., landmark coordinates). It's left
    /// unimplemented here so you can choose the ONNX runtime backend and session options.
    pub fn run_model(&self, _input_tensor: &[f32]) -> Result<Vec<f32>> {
        Err(anyhow!("run_model is not implemented: enable `onnx` feature and implement inference using onnxruntime crate"))
    }
}

impl LandmarkExtractor for OnnxLandmarkExtractor {
    fn extract(&mut self, frame_bgr: &core::Mat) -> Result<Vec<Vector3<f32>>> {
        // In a typical pipeline you'd be given an ROI; here we assume full-frame input and
        // the caller will crop using a tracker when available. For safety, use the whole frame.
        let rect = core::Rect::new(0, 0, frame_bgr.cols(), frame_bgr.rows());
        let crop = core::Mat::roi(frame_bgr, rect)?;
        let tensor = self.preprocess(&crop)?;
        let out = self.run_model(&tensor)?;

        // Expecting out to be flat floats: [x1,y1,z1, x2,y2,z2, ...]
        if out.len() % 3 != 0 {
            return Err(anyhow!("unexpected model output length"));
        }

        let mut landmarks = Vec::with_capacity(out.len() / 3);
        for i in 0..(out.len() / 3) {
            landmarks.push(Vector3::new(out[i*3], out[i*3+1], out[i*3+2]));
        }

        Ok(landmarks)
    }
}
