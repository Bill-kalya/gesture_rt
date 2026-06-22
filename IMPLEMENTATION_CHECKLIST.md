# Implementation Checklist: What's Done, What's Next

## ✅ Completed in Rust (You Can Use Immediately)

### Core Inference Components
- [x] **Camera Capture** (`src/vision/camera/capture.rs`)
  - OpenCV VideoCapture wrapper
  - Frame capture with error handling
  - Used in `examples/camera_smoke.rs`

- [x] **ROI Tracker** (`src/vision/landmarks/tracker.rs`)
  - Bounding box prediction with constant velocity model
  - Update with new detections
  - Expand ROI for landmark inference context
  - Auto-reset after N missed frames

- [x] **ONNX Landmark Extractor** (`src/vision/landmarks/onnx_extractor.rs`)
  - BGR → RGB color conversion
  - Resize to model input (224×224)
  - CHW float normalization
  - Placeholder `run_model()` method (you implement with onnxruntime)

- [x] **Landmark Sequence Buffer** (`src/vision/landmarks/gesture_classifier.rs`)
  - VecDeque for 30-frame sliding window
  - Per-frame: 63 floats (21 landmarks × 3 coords)
  - Converts to flattened tensor for gesture model
  - Ready-state check

- [x] **Gesture Classifier Interface** (`src/vision/landmarks/gesture_classifier.rs`)
  - Accepts class names and model path
  - Placeholder `infer()` method (you implement with onnxruntime)
  - Returns gesture ID, confidence, class name

- [x] **Kalman Filter** (`src/spatial/filters/kalman.rs`)
  - 3D position + velocity smoothing
  - Configurable process/measurement noise
  - Used in runtime for temporal stability

- [x] **Runtime Dispatcher** (`src/runtime_dispatch/keyboard/`, `mouse/`, `media/`)
  - Keyboard (Alt+Tab, shortcuts)
  - Mouse (movement, clicks)
  - Media (play, pause, volume)
  - Platform-specific (Windows via `enigo`)

### Examples
- [x] **camera_smoke.rs** — Verify OpenCV camera works
- [x] **onnx_pipeline.rs** — ROI tracker + landmark pipeline example
- [x] **full_pipeline.rs** — Complete pipeline with gesture classifier (reference architecture)

### Dependency Additions
- [x] Added `opencv = "0.76"` to Cargo.toml with buildtime-bindgen
- [x] Added `onnxruntime = "0.17"` (optional) to Cargo.toml
- [x] Added `onnx` feature flag

### Documentation
- [x] **TRAINING_GUIDE.md** — Complete 10-phase training pipeline with Python scripts
- [x] **EXTERNAL_TASKS.md** — Clear delineation of what you do vs. what's implemented
- [x] **Implementation checklist** (this file)

---

## ⚠️ Placeholder Methods (You Must Implement)

### 1. **`OnnxLandmarkExtractor::run_model()`** 
**File:** `src/vision/landmarks/onnx_extractor.rs:72`

**Current Status:** Returns `Err("not implemented")`

**What You Need to Do:**
```rust
pub fn run_model(&self, input_tensor: &[f32]) -> Result<Vec<f32>> {
    // TODO: Implement using onnxruntime crate
    // 1. Create Session from self.model_path
    // 2. Run inference with input_tensor (shape: 1,3,224,224)
    // 3. Extract output tensor (shape: 1,63 for 21 landmarks)
    // 4. Return as Vec<f32>
}
```

**Dependencies You'll Add:**
```toml
[dev-dependencies]
onnxruntime = "0.17"
```

**Pseudocode:**
```rust
use onnxruntime::Environment;

let env = Environment::new()?;
let session = env.new_session_builder()?
    .with_model_from_file(self.model_path)?;
let inputs = ndarray::Array::from_shape_vec((1, 3, 224, 224), input_tensor.to_vec())?;
let outputs = session.run(vec![inputs])?;
// Convert outputs to Vec<f32> and return
```

---

### 2. **`GestureClassifier::infer()`**
**File:** `src/vision/landmarks/gesture_classifier.rs:67`

**Current Status:** Returns `Err("not implemented")`

**What You Need to Do:**
```rust
pub fn infer(&self, input_tensor: &[f32]) -> Result<GestureInference> {
    // TODO: Implement using onnxruntime crate
    // 1. Create Session from self.model_path
    // 2. Run inference with input_tensor (shape: 1,30,63)
    // 3. Get logits output (shape: 1, num_classes)
    // 4. Apply softmax to get confidence
    // 5. Find argmax for class_id
    // 6. Return GestureInference
}
```

**Pseudocode:**
```rust
let env = Environment::new()?;
let session = env.new_session_builder()?
    .with_model_from_file(self.model_path)?;
let inputs = ndarray::Array::from_shape_vec((1, 30, 63), input_tensor.to_vec())?;
let outputs = session.run(vec![inputs])?;

// Apply softmax
let logits = outputs[0].iter().copied().collect::<Vec<_>>();
let exps: Vec<f32> = logits.iter().map(|l| l.exp()).collect();
let sum_exp: f32 = exps.iter().sum();
let probs: Vec<f32> = exps.iter().map(|e| e / sum_exp).collect();

let (class_id, &confidence) = probs.iter().enumerate()
    .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
    .ok_or(anyhow!("empty probabilities"))?;

Ok(GestureInference {
    class_id,
    confidence,
    class_name: self.class_names.get(class_id).cloned().unwrap_or_default(),
})
```

---

## 📋 What You Must Do Externally

### Phase 1-6: Model Training (See TRAINING_GUIDE.md)

**Deliverables:**
1. `models/hand_detector.onnx` — YOLOv8n for hand detection (320×320)
2. `models/hand_landmark.onnx` — MobileNetV3 for landmarks (224×224 → 63 outputs)
3. `models/gesture_classifier.onnx` — LSTM for gesture class (1,30,63 → num_classes)

**Estimated Timeline:**
- Data collection: 7–10 days
- Auto-labeling: 1–2 days
- Detector training: 1–2 days
- Landmark training: 1–2 days
- Gesture sequence creation: 1 day
- Gesture classifier training: 1–2 days
- **Total: 3–4 weeks**

### Phase 7+: Validation & Deployment

**Metrics to Track:**
- Detector mAP > 0.90
- Landmark error < 5 pixels
- Gesture accuracy > 90%

---

## 🚀 Quick Start (After Models Are Trained)

### 1. Install Dependencies
```powershell
# Windows with vcpkg
git clone https://github.com/microsoft/vcpkg.git
.\vcpkg\bootstrap-vcpkg.bat
.\vcpkg\vcpkg.exe install opencv4[contrib,ffmpeg]:x64-windows

# Install ONNX Runtime (system-level)
# Option A: Use vcpkg
.\vcpkg\vcpkg.exe install onnxruntime:x64-windows

# Option B: Download from https://github.com/microsoft/onnxruntime/releases
```

### 2. Build with Features
```powershell
# Verify camera works
cargo run --example camera_smoke --features camera

# Verify OpenCV pipeline
cargo run --example onnx_pipeline --features "camera,onnx"

# Run full pipeline (after implementing .infer() methods)
cargo run --example full_pipeline --features "camera,onnx"
```

### 3. Implement ONNX Runtime Hooks
Edit these two methods:
- `src/vision/landmarks/onnx_extractor.rs` → `run_model()`
- `src/vision/landmarks/gesture_classifier.rs` → `infer()`

### 4. Place Models
```
gesture_rt/
└── models/
    ├── hand_detector.onnx
    ├── hand_landmark.onnx
    └── gesture_classifier.onnx
```

### 5. Run Full System
```powershell
cargo run --bin gesture_rt --features "camera,onnx"
```

---

## 📊 Architecture Recap

```
Camera (OpenCV)
    ↓
Hand Detector (ONNX, every 15 frames)
    ↓
ROI Tracker (Rust geometry)
    ↓
Landmark Extractor (ONNX on crop)
    ↓
30-Frame Buffer
    ↓
Gesture Classifier (LSTM ONNX)
    ↓
Dispatch (Keyboard/Mouse/Media)
```

**Why This Design Wins:**
- **Speed:** Only run expensive models on small ROIs
- **Stability:** Kalman + temporal buffering reduces jitter
- **Flexibility:** Swap ONNX models or providers without Rust changes
- **Safety:** Single language (Rust) for critical path

---

## 🔗 File Dependencies

### Must Implement Before Running Examples

1. **onnx_extractor.rs::run_model()**
   - Called by: `extract()` method
   - Used by: `onnx_pipeline.rs`, `full_pipeline.rs`

2. **gesture_classifier.rs::infer()**
   - Called by: `full_pipeline.rs`
   - Depends on: gesture sequence buffer ready state

### Must Have Before Running Full Pipeline

1. `models/hand_detector.onnx`
2. `models/hand_landmark.onnx`
3. `models/gesture_classifier.onnx`

---

## ❓ Troubleshooting

### Issue: Build fails with "opencv not found"
**Fix:** Ensure vcpkg installed and `VCPKG_ROOT` set:
```powershell
setx VCPKG_ROOT "C:\path\to\vcpkg"
```

### Issue: "onnxruntime is not found"
**Fix:** Implement `.run_model()` using onnxruntime crate:
```toml
# Add to Cargo.toml
[dependencies]
onnxruntime = "0.17"
ndarray = "0.15"  # For tensor operations
```

### Issue: ONNX model shape mismatch errors
**Common Fix:** Verify model export with Python:
```python
import onnx
model = onnx.load("hand_landmark.onnx")
print([inp.name for inp in model.graph.input])  # Should be (1,3,224,224)
print([out.name for out in model.graph.output])  # Should be (1,63)
```

---

## 📞 Support & Next Steps

1. **Review** `EXTERNAL_TASKS.md` → understand what you own
2. **Follow** `TRAINING_GUIDE.md` → train three models (Phases 1-6)
3. **Implement** placeholder methods → `run_model()` and `infer()`
4. **Place** ONNX files in `models/` directory
5. **Test** with `examples/full_pipeline.rs`
6. **Deploy** with `cargo run --bin gesture_rt --features "camera,onnx"`

---

**Last Updated:** 2026-06-22  
**Status:** Rust implementation complete; awaiting ONNX models from training pipeline
