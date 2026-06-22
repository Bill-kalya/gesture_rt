/// Full production pipeline example: Camera → Detector → Tracker → Landmarks → Gesture → Dispatch
///
/// To run:
/// cargo run --example full_pipeline --features "camera,onnx"
///
/// This example demonstrates the complete three-stage inference pipeline:
/// 1. Hand Detector (ONNX) → bounding box
/// 2. Landmark Extractor (ONNX) → 21 landmarks per ROI
/// 3. Gesture Classifier (ONNX) → gesture class + confidence
///
/// NOTE: This is a reference architecture. Actual ONNX Runtime integration
/// (implementing run_model and infer methods) requires system ONNX Runtime installation
/// and filling in the onnxruntime API calls.

use std::collections::VecDeque;

#[cfg(all(feature = "camera", feature = "onnx"))]
fn main() {
    use gesture_rt::vision::landmarks::{
        RoiTracker, LandmarkExtractor, OnnxLandmarkExtractor,
        GestureClassifier, LandmarkSequenceBuffer,
    };
    use opencv::{prelude::*, videoio, core};
    use anyhow::Result;

    fn run() -> Result<()> {
        println!("🎯 GestureRT Full Production Pipeline");
        println!("=====================================\n");

        // Initialize models
        println!("📦 Initializing models...");
        let detector = OnnxLandmarkExtractor::new("models/hand_detector.onnx", 320, 320);
        let mut landmark_extractor = OnnxLandmarkExtractor::new("models/hand_landmark.onnx", 224, 224);
        
        let gesture_classifier = GestureClassifier::new(
            "models/gesture_classifier.onnx",
            vec![
                "OpenPalm".to_string(),
                "ClosedFist".to_string(),
                "Point".to_string(),
                "Pinch".to_string(),
                "SwipeLeft".to_string(),
                "SwipeRight".to_string(),
                "SwipeUp".to_string(),
                "SwipeDown".to_string(),
            ],
            30, // sequence length
        );
        println!("✅ Models initialized\n");

        // Open camera
        println!("📷 Opening camera...");
        let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
        if !videoio::VideoCapture::is_opened(&cam)? {
            eprintln!("❌ Failed to open camera");
            return Ok(());
        }
        let w = cam.get(videoio::CAP_PROP_FRAME_WIDTH)? as i32;
        let h = cam.get(videoio::CAP_PROP_FRAME_HEIGHT)? as i32;
        println!("✅ Camera opened: {}x{}\n", w, h);

        // Initialize tracking and buffering
        let mut tracker = RoiTracker::default();
        let mut landmark_buffer = LandmarkSequenceBuffer::new(30);
        let mut frame_idx: u64 = 0;

        // FPS tracking
        let mut fps_counter = 0;
        let start_time = std::time::Instant::now();

        println!("▶️  Pipeline running. Press Ctrl+C to stop.\n");
        println!("{:>6} | {:>8} | {:>15} | {:>8} | {:>10}",
            "Frame", "Detector", "Landmarks", "Gesture", "Latency");
        println!("{:-<60}", "");

        let mut frame = core::Mat::default();
        loop {
            let frame_start = std::time::Instant::now();
            frame_idx += 1;

            // Capture frame
            cam.read(&mut frame)?;
            if frame.empty() {
                continue;
            }

            // Stage 1: Hand Detector (rarely, or when tracker is lost)
            let mut detector_result = "no-run";
            if frame_idx % 15 == 0 || tracker.bbox.is_none() {
                // In production, this would call the ONNX detector model
                // For now, it's a placeholder showing the architecture
                // detector_result = detector.extract(&frame)
                //     .ok()
                //     .map(|_| "detect")
                //     .unwrap_or("fail");
                detector_result = "run";
            }

            // Stage 2: ROI Tracking
            let mut landmark_count = 0;
            if let Some(expanded) = tracker.expanded_bbox(w, h, 1.6) {
                let roi = core::Mat::roi(&frame, expanded)?;

                // Stage 3: Landmark Extraction (ONNX on ROI)
                if let Ok(landmarks) = landmark_extractor.extract(&roi) {
                    landmark_count = landmarks.len();

                    // Convert Vec<Vector3> to [f32; 63]
                    let mut landmark_array = [0.0f32; 63];
                    for (i, lm) in landmarks.iter().enumerate() {
                        if i < 21 {
                            landmark_array[i * 3] = lm.x;
                            landmark_array[i * 3 + 1] = lm.y;
                            landmark_array[i * 3 + 2] = lm.z;
                        }
                    }

                    // Stage 4: Landmark Sequence Buffering
                    landmark_buffer.push(&landmark_array);

                    // Stage 5: Gesture Classification (when buffer is full)
                    let mut gesture_name = "buffering";
                    if landmark_buffer.is_ready() {
                        let tensor = landmark_buffer.as_tensor();
                        if let Ok(gesture_result) = gesture_classifier.infer(&tensor) {
                            gesture_name = &gesture_result.class_name;
                            
                            // Stage 6: Action Dispatch (would happen here)
                            // For this example, we just print
                            if gesture_result.confidence > 0.8 {
                                println!("🎯 GESTURE DETECTED: {} (conf: {:.2})",
                                    gesture_result.class_name, gesture_result.confidence);
                            }
                        } else {
                            gesture_name = "inference-error";
                        }
                    }

                    // Update tracker with new position
                    tracker.update(expanded, frame_idx);
                } else {
                    tracker.missed();
                    gesture_name = "landmark-fail";
                }
            } else {
                tracker.missed();
                gesture_name = "no-roi";
            }

            // Latency measurement
            let frame_latency_ms = frame_start.elapsed().as_millis();

            // Print status every 30 frames
            if frame_idx % 30 == 0 {
                let gesture_status = if landmark_buffer.is_ready() { "ready" } else { "filling" };
                println!("{:>6} | {:>8} | {:>15} | {:>8} | {:>10}ms",
                    frame_idx,
                    detector_result,
                    landmark_count,
                    gesture_status,
                    frame_latency_ms
                );

                // FPS calculation
                fps_counter += 30;
                let elapsed = start_time.elapsed().as_secs_f64();
                let fps = fps_counter as f64 / elapsed;
                if frame_idx % 300 == 0 {
                    println!("\n📊 Running at ~{:.1} FPS\n", fps);
                    println!("{:>6} | {:>8} | {:>15} | {:>8} | {:>10}",
                        "Frame", "Detector", "Landmarks", "Gesture", "Latency");
                    println!("{:-<60}", "");
                }
            }

            // Throttle to ~30 FPS (33ms per frame)
            let remaining = 33i64 - frame_start.elapsed().as_millis() as i64;
            if remaining > 0 {
                std::thread::sleep(std::time::Duration::from_millis(remaining as u64));
            }
        }
    }

    if let Err(e) = run() {
        eprintln!("❌ Pipeline error: {}", e);
    }
}

#[cfg(not(all(feature = "camera", feature = "onnx")))]
fn main() {
    println!("Build with --features \"camera,onnx\" to run the full pipeline example");
    println!("cargo run --example full_pipeline --features \"camera,onnx\"");
}
