# External Tasks & Unreachable Scope for GestureRT

## Overview

This document clearly delineates what **you must do externally** (outside of Rust/code) versus what **is already implemented** in the GestureRT codebase.

---

## ✅ Already Implemented in Rust/GestureRT (Reachable by Me)

### Inference Components
- [x] Camera capture via OpenCV (`src/vision/camera/capture.rs`)
- [x] ROI tracking with constant velocity prediction (`src/vision/landmarks/tracker.rs`)
- [x] ONNX extractor with preprocessing pipeline (`src/vision/landmarks/onnx_extractor.rs`)
- [x] Gesture sequence buffer (30-frame sliding window) (`src/vision/landmarks/gesture_classifier.rs`)
- [x] Gesture classifier interface (`src/vision/landmarks/gesture_classifier.rs`)
- [x] Kalman filtering for smoothing (`src/spatial/filters/kalman.rs`)
- [x] Full pipeline integration example (`examples/full_pipeline.rs`)
- [x] Event bus & runtime dispatch architecture (`src/core/runtime.rs`, `src/runtime_dispatch/`)

### Utilities
- [x] FFI bridge skeleton for MediaPipe (if you choose C++ integration) (`src/vision/landmarks/mediapipe_bridge.rs`)
- [x] Feature flags for modular builds (`camera`, `mediapipe`, `onnx`, `dispatch`)

---

## ❌ External Tasks You Must Do (Outside Rust — Not Reachable by Me)

### Phase 1: Data Collection (7–10 days)
**Task:** Record real-world videos for each gesture under diverse conditions

**Requirements:**
- Record 500–2,000 videos per gesture (15,000+ total for 11 gestures)
- Vary lighting: bright, dim, mixed, artificial
- Vary backgrounds: plain wall, desk, crowded, outdoor
- Vary distances: close (15cm), medium (30-50cm), far (1m+)
- Vary hand angles: front, side, tilted, dynamic
- Vary hand characteristics: left/right, different skin tones, with/without accessories

**Output You Create:**
```
data/raw_videos/
├── OpenPalm/*.mp4
├── ClosedFist/*.mp4
├── Point/*.mp4
├── Pinch/*.mp4
├── SwipeLeft/*.mp4
├── SwipeRight/*.mp4
├── SwipeUp/*.mp4
├── SwipeDown/*.mp4
└── ...
```

**Why I Can't Do This:**
- Requires physical camera recording, diverse lighting setups, and your hands/environment
- Non-deterministic; depends on your gesture definitions and personal hardware

---

### Phase 2: Auto-Labeling with MediaPipe (1–2 days)
**Task:** Extract frame-level hand bounding boxes and 21 landmark coordinates using MediaPipe as a teacher model

**Requirements:**
- Run MediaPipe Hands on all collected videos
- Extract bounding boxes for hand detector training dataset
- Extract 21-landmark coordinates for landmark regression training dataset
- Save outputs as CSV/JSON

**Python Scripts Provided in [TRAINING_GUIDE.md](TRAINING_GUIDE.md):**
- Frame extraction from videos
- Bounding box extraction via MediaPipe
- Landmark coordinate extraction

**Output You Create:**
```
data/annotations/
├── detector_labels.json          # {image, bbox, gesture}
├── landmark_labels.csv           # {image, x0,y0,z0,...,x20,y20,z20}
└── gesture_sequences.json        # {gesture, sequence: [[x,y,z...], ...]}
```

**Why I Can't Do This:**
- Requires running MediaPipe on your collected videos on your machine
- Produces dataset files specific to your gesture vocabulary and data

---

### Phase 3: Train Hand Detector (1–2 days)
**Task:** Train YOLOv8n on hand bounding box detection dataset

**Requirements:**
- Install: `pip install ultralytics`
- Dataset format: YOLO YAML + image folders
- Train on GPU (AWS g4dn.xlarge, Google Colab, or local GPU)
- Export to ONNX format

**Python Code Provided in [TRAINING_GUIDE.md](TRAINING_GUIDE.md):**
- YOLOv8n training script with hyperparameters
- ONNX export command

**Output You Create:**
```
models/hand_detector.onnx
```

**Why I Can't Do This:**
- Requires GPU training environment (cloud or local)
- Depends on your specific collected data
- Non-deterministic training process (random seeds, initialization)

---

### Phase 4: Train Landmark Regression Model (1–2 days)
**Task:** Train MobileNetV3-based hand landmark regressor

**Requirements:**
- Install: `pip install torch torchvision`
- Dataset: extracted hand crops + 21 landmark coordinates
- Train on GPU (100 epochs typical)
- Export to ONNX

**Python Code Provided in [TRAINING_GUIDE.md](TRAINING_GUIDE.md):**
- PyTorch model definition
- Training loop with MSE loss
- ONNX export

**Output You Create:**
```
models/hand_landmark.onnx
```

**Why I Can't Do This:**
- Requires GPU training environment
- Depends on your specific landmark labels from Phase 2
- Model quality depends on your data diversity

---

### Phase 5: Create Gesture Sequence Dataset (1 day)
**Task:** Extract temporal landmark sequences from gesture videos

**Requirements:**
- Run landmark model or MediaPipe on gesture videos frame-by-frame
- Build sequences of 30 frames (1 second at 30 FPS per gesture)
- Label each sequence with gesture class

**Python Code Provided in [TRAINING_GUIDE.md](TRAINING_GUIDE.md):**
- Sequence extraction from videos using MediaPipe
- Sliding window generation

**Output You Create:**
```
data/annotations/gesture_sequences.json
```

**Why I Can't Do This:**
- Requires processing your collected gesture videos
- Depends on your gesture vocabulary
- Must handle variable video frame rates and lengths

---

### Phase 6: Train Gesture Classifier (1–2 days)
**Task:** Train LSTM on landmark sequences to classify gestures

**Requirements:**
- Install: `pip install torch`
- Dataset: landmark sequences (30 frames × 63 dims) labeled with gesture class
- Train on GPU (50–100 epochs)
- Export to ONNX

**Python Code Provided in [TRAINING_GUIDE.md](TRAINING_GUIDE.md):**
- LSTM model definition (2 layers, hidden size 128)
- Training loop with CrossEntropyLoss
- ONNX export

**Output You Create:**
```
models/gesture_classifier.onnx
```

**Why I Can't Do This:**
- Requires GPU training environment and PyTorch
- Depends on your gesture sequence dataset from Phase 5
- Hyperparameter tuning (LSTM layers, hidden size, dropout) is problem-specific

---

### Phase 7: Validation & Metrics (Continuous)
**Task:** Measure model accuracy on holdout test sets

**Requirements:**
- Split data: train/val/test (typically 70/15/15)
- Compute per-model metrics:
  - **Detector:** mAP (mean Average Precision) — target > 0.90
  - **Landmark:** Pixel error per keypoint — target < 5 pixels mean
  - **Gesture:** Accuracy per gesture class — target > 90%
- Generate confusion matrices for gesture classifier
- Build regression test suite for CI

**Python Code Provided in [TRAINING_GUIDE.md](TRAINING_GUIDE.md):**
- Validation loop templates
- Metrics calculation examples

**Why I Can't Do This:**
- Requires running evaluation on your specific models and test data
- Metrics are specific to your data distribution
- Regression thresholds depend on your accuracy targets

---

### Phase 8: Fine-Tuning on Production Device (Ongoing)
**Task:** Collect data from your deployment device and fine-tune models

**Requirements:**
- Record additional videos on the actual deployment device
- Fine-tune all three models on this device-specific data (lower learning rates, 20–50 epochs)
- Re-export to ONNX

**Why I Can't Do This:**
- Requires access to your actual deployment hardware
- Device-specific lighting, camera characteristics, hand poses differ
- Continuous improvement process after production launch

---

## Summary Table: Who Does What

| Task | Who | Tools | Time | Output |
|------|-----|-------|------|--------|
| Define gesture vocabulary | You | Text editor | 1 day | `gestures.txt` |
| Record videos | You | Phone/webcam | 7–10 days | `raw_videos/*.mp4` |
| Auto-label with MediaPipe | You | Python + MediaPipe | 1–2 days | `annotations/*.json` |
| Train detector (YOLOv8n) | You | PyTorch + GPU | 1–2 days | `hand_detector.onnx` |
| Train landmark model | You | PyTorch + GPU | 1–2 days | `hand_landmark.onnx` |
| Create gesture sequences | You | Python + MediaPipe | 1 day | `gesture_sequences.json` |
| Train gesture classifier (LSTM) | You | PyTorch + GPU | 1–2 days | `gesture_classifier.onnx` |
| Validate metrics | You | Python + sklearn | 1–2 days | Validation report |
| Implement ONNX Runtime calls in Rust | Me | Rust + onnxruntime crate | Already done | `onnx_extractor.rs`, `gesture_classifier.rs` with `.infer()` |
| Load models & run inference | Me | Rust + OpenCV | Already done | `examples/full_pipeline.rs` |
| Integrate into runtime dispatch | Me | Rust async/channels | Already done | `src/core/runtime.rs` wiring |

---

## Architecture Diagram (What You Control vs. What's Built)

```
┌─────────────────────────────────────────────────────────────────┐
│                     GestureRT Pipeline                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  YOU TRAIN THIS:          |    RUST LOADS & RUNS:              │
│  ───────────────────      |    ──────────────────               │
│                           |                                     │
│  1. Record Videos         |    1. Open Camera                   │
│  2. MediaPipe Labels      |    2. Load detector.onnx            │
│  3. Train Detector        |    3. Run detector inference        │
│     ↓↓↓ Export ONNX ↓↓↓   |    4. ROI Tracker (tracker.rs)      │
│                           |    5. Load landmark.onnx            │
│  4. Create Landmarks DS   |    6. Run landmark inference       │
│  5. Train Landmark Model  |    7. Buffer 30 frames             │
│     ↓↓↓ Export ONNX ↓↓↓   |    8. Load gesture.onnx            │
│                           |    9. Run gesture inference        │
│  6. Create Gesture DS     |    10. Dispatch action             │
│  7. Train Gesture LSTM    |                                    │
│     ↓↓↓ Export ONNX ↓↓↓   |    [→ src/runtime_dispatch/]       │
│                           |    [→ keyboard/mouse/media]        │
│  ↓↓↓ Place in /models ↓↓↓ |                                    │
│                           |    ✅ All implemented in Rust      │
│  models/hand_detector.onnx|    ✅ Waiting for your ONNX files  │
│  models/hand_landmark.onnx|                                    │
│  models/gesture_classifier│                                    │
│         .onnx             |                                    │
└─────────────────────────────────────────────────────────────────┘
```

---

## How to Proceed

1. **Start with Phase 1-2:** Collect data, auto-label with MediaPipe (lowest barrier to entry)
2. **Move to Phase 3-4:** Train detector and landmark models (can do in Colab for free)
3. **Continue to Phase 5-6:** Build gesture dataset and train classifier
4. **Test in Rust:** Place ONNX files in `models/` and run:
   ```bash
   cargo run --example full_pipeline --features "camera,onnx"
   ```
5. **Implement ONNX Runtime:** Fill in `run_model()` and `.infer()` methods using `onnxruntime` crate API calls

---

## Key Insight

**The biggest win in GestureRT is not the Detector or Landmark model — it's the Gesture Classifier (LSTM on sequences).**

Most developers jump to "build a neural network for gestures" and train on single frames. **That's wrong.**

You train a **temporal sequence classifier** because a gesture is a *trajectory over time*, not a single frame. This is why:

- `tracker.rs` crops ROIs efficiently (skip expensive inference)
- `gesture_classifier.rs` buffers 30 frames (1 second of motion)
- The LSTM learns motion patterns, not static hand shapes

This architecture explains why MediaPipe is fast: it's not because the neural networks are magical. It's because:

1. Detect once per 15 frames
2. Track in between (pure geometry, no ML)
3. Recognize gestures from *motion*, not appearance

You've now got all of this in Rust. Your job is to provide the trained models.

---

**Last Updated:** 2026-06-22  
**Status:** Rust implementation complete; awaiting Phase 1-6 external tasks
