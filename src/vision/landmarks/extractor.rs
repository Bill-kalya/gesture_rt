use anyhow::Result;
use nalgebra::Vector3;

/// Trait for extracting hand landmarks from an image.
pub trait LandmarkExtractor {
    /// Given an OpenCV `Mat` (BGR), returns 21 landmarks as `Vec<Vector3<f32>>`.
    fn extract(&mut self, frame_bgr: &opencv::core::Mat) -> Result<Vec<Vector3<f32>>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencv::core::Mat;

    struct Dummy;
    impl LandmarkExtractor for Dummy {
        fn extract(&mut self, _frame_bgr: &Mat) -> Result<Vec<Vector3<f32>>> {
            Ok(vec![])
        }
    }

    #[test]
    fn dummy_extracts() {
        let mut d = Dummy;
        let m = Mat::default();
        let out = d.extract(&m).unwrap();
        assert!(out.is_empty());
    }
}
