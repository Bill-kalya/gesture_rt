use anyhow::Result;
use opencv::prelude::*;
use opencv::{videoio, core};

use gesture_rt::vision::landmarks::{OnnxLandmarkExtractor, RoiTracker, LandmarkExtractor};

fn main() -> Result<()> {
    #[cfg(not(all(feature = "camera", feature = "onnx")))]
    {
        println!("Build with --features "camera,onnx" to run the ONNX pipeline example");
        return Ok(());
    }

    #[cfg(all(feature = "camera", feature = "onnx"))]
    {
        // Open camera
        let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
        if !videoio::VideoCapture::is_opened(&cam)? {
            eprintln!("Failed to open camera");
            return Ok(());
        }

        let mut frame = core::Mat::default();
        let mut extractor = OnnxLandmarkExtractor::new("models/hand_landmark.onnx", 224, 224);
        let mut tracker = RoiTracker::default();

        let mut frame_idx: u64 = 0;
        loop {
            cam.read(&mut frame)?;
            if frame.empty() {
                continue;
            }

            frame_idx += 1;

            // If tracker has bbox, crop and run landmark model on expanded ROI
            if let Some(expanded) = tracker.expanded_bbox(frame.cols(), frame.rows(), 1.6) {
                let roi = core::Mat::roi(&frame, expanded)?;
                match extractor.extract(&roi) {
                    Ok(landmarks) => {
                        // Here you would transform landmark coords back to full-frame pixels
                        println!("Got {} landmarks", landmarks.len());
                        // For demo, pretend detection updated bbox center
                        tracker.update(expanded, frame_idx);
                    }
                    Err(e) => {
                        eprintln!("Landmark inference error: {}", e);
                        tracker.missed();
                    }
                }
            } else {
                // No tracker bbox: expensive fallback — run on full frame every N frames or use detector
                if frame_idx % 15 == 0 {
                    match extractor.extract(&frame) {
                        Ok(landmarks) => {
                            println!("Full-frame landmarks: {}", landmarks.len());
                            // You'd set tracker bbox here based on returned landmarks
                        }
                        Err(e) => {
                            eprintln!("Full-frame inference error: {}", e);
                        }
                    }
                }
            }
        }
    }
}
