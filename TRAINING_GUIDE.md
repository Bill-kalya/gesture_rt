# GestureRT Three-Model Training & Deployment Guide

## Architecture Overview

GestureRT uses a **three-stage ONNX inference pipeline**:

```
Camera Frame
    ↓
[1] Hand Detector (YOLOv8n) → Bounding Box
    ↓
[2] ROI Tracker (Rust) → Tracked Hand Region
    ↓
[3] Landmark Regressor (MobileNetV3) → 21 Landmarks per frame
    ↓
[4] Landmark Sequence Buffer (30 frames × 63 dims) → Temporal Sequence
    ↓
[5] Gesture Classifier (LSTM) → Gesture Class + Confidence
    ↓
Application Actions (Keyboard, Mouse, etc.)
```

**Rust Implementation:** Stages [2], [4], [5] (tracking, buffering, dispatch)  
**Your Task:** Train stages [1], [3], [5] → export as ONNX models

---

## Phase 0: Before You Start

### Gesture Vocabulary Definition

Define **all** gestures your system must recognize. Example:

```
ID  Name                    Purpose
0   OpenPalm                Navigate
1   ClosedFist              Stop/Pause
2   Point                   Select
3   Pinch                   Zoom/Fine Control
4   SwipeLeft               Previous
5   SwipeRight              Next
6   SwipeUp                 Scroll Up
7   SwipeDown               Scroll Down
8   RotateClockwise         Rotate Object
9   RotateCounterclockwise  Rotate Object (reverse)
10  ThreeFingerSwipe        Advanced Gesture
11  TwoFingerRotate         Advanced Rotation
```

**Critical:** Lock this vocabulary before collecting data. Changing it mid-training is expensive.

---

## Phase 1: Data Collection (Your Task — 7-10 days)

### 1.1 Record Videos for Each Gesture

For each gesture, record videos from:

#### Different Lighting Conditions
- Bright (outdoor, sunlight)
- Dim (indoor, evening)
- Mixed (mixed lighting, shadows)
- Artificial (fluorescent, LED)

#### Different Backgrounds
- Plain wall (white, gray, colored)
- Desk with objects
- Crowded room
- Outdoor

#### Different Distances from Camera
- Close (15cm from camera)
- Medium (30-50cm)
- Far (1+ meter)

#### Different Hand Angles & Poses
- Front-facing
- Side angle
- Tilted/rotated
- Dynamic (mid-motion)

#### Different Hand Characteristics
- Left hand
- Right hand
- Different skin tones
- With/without jewelry or accessories

### 1.2 Recording Setup

**Recommended:** 30-60 second video per scenario per gesture

**Format:** MP4 (H.264) or WebM

**Resolution:** 1280×720 or higher

**Frame Rate:** 30 FPS

**Total Data:** Aim for 500–2000 examples per gesture
- If 11 gestures: 5,500–22,000 video clips

**Minimum:** 10 videos per gesture for testing (quick iteration)

### 1.3 File Organization

```
data/
├── raw_videos/
│   ├── OpenPalm/
│   │   ├── bright_1.mp4
│   │   ├── dim_1.mp4
│   │   └── ...
│   ├── ClosedFist/
│   │   ├── bright_1.mp4
│   │   └── ...
│   └── ...
├── extracted_frames/
│   ├── OpenPalm/
│   │   └── *.jpg
│   └── ...
└── annotations/
    ├── detector_labels.csv
    ├── landmark_labels.csv
    └── gesture_sequences.csv
```

---

## Phase 2: Auto-Labeling with MediaPipe (Your Task — 2-3 days)

### 2.1 Extract Frames from Videos

```python
import cv2
import os

video_dir = "data/raw_videos"
output_dir = "data/extracted_frames"

for gesture in os.listdir(video_dir):
    os.makedirs(f"{output_dir}/{gesture}", exist_ok=True)
    for video_file in os.listdir(f"{video_dir}/{gesture}"):
        cap = cv2.VideoCapture(f"{video_dir}/{gesture}/{video_file}")
        frame_idx = 0
        while cap.isOpened():
            ret, frame = cap.read()
            if not ret:
                break
            cv2.imwrite(f"{output_dir}/{gesture}/frame_{frame_idx:06d}.jpg", frame)
            frame_idx += 1
        cap.release()
```

### 2.2 Generate Detector Labels (Bounding Boxes) using MediaPipe

```python
import mediapipe as mp
import json

mp_hands = mp.solutions.hands.Hands(
    static_image_mode=True,
    max_num_hands=1,
    min_detection_confidence=0.5
)

detector_labels = []
for gesture in os.listdir("data/extracted_frames"):
    for img_file in os.listdir(f"data/extracted_frames/{gesture}"):
        img_path = f"data/extracted_frames/{gesture}/{img_file}"
        image = cv2.imread(img_path)
        results = mp_hands.process(cv2.cvtColor(image, cv2.COLOR_BGR2RGB))
        
        if results.multi_hand_landmarks:
            hand = results.multi_hand_landmarks[0]
            # Get bounding box from landmarks
            xs = [lm.x for lm in hand.landmark]
            ys = [lm.y for lm in hand.landmark]
            x_min, x_max = min(xs), max(xs)
            y_min, y_max = min(ys), max(ys)
            h, w = image.shape[:2]
            x, y, x2, y2 = int(x_min*w), int(y_min*h), int(x_max*w), int(y_max*h)
            
            detector_labels.append({
                "image": img_path,
                "x": x, "y": y,
                "width": x2-x, "height": y2-y,
                "gesture": gesture
            })

# Save as YOLO format or COCO format
with open("data/annotations/detector_labels.json", "w") as f:
    json.dump(detector_labels, f)
```

### 2.3 Generate Landmark Labels using MediaPipe

```python
import pandas as pd

landmark_data = []
for gesture in os.listdir("data/extracted_frames"):
    for img_file in os.listdir(f"data/extracted_frames/{gesture}"):
        img_path = f"data/extracted_frames/{gesture}/{img_file}"
        image = cv2.imread(img_path)
        results = mp_hands.process(cv2.cvtColor(image, cv2.COLOR_BGR2RGB))
        
        if results.multi_hand_landmarks:
            hand = results.multi_hand_landmarks[0]
            row = {"image": img_path, "gesture": gesture}
            for i, lm in enumerate(hand.landmark):
                row[f"x{i}"] = lm.x
                row[f"y{i}"] = lm.y
                row[f"z{i}"] = lm.z
            landmark_data.append(row)

df = pd.DataFrame(landmark_data)
df.to_csv("data/annotations/landmark_labels.csv", index=False)
```

---

## Phase 3: Train Hand Detector (Your Task — AWS g4dn.xlarge or Colab GPU — 1-2 days)

### 3.1 Prepare YOLO Dataset Format

```yaml
# data/hand_detect.yaml
path: /path/to/data
train: annotations/train_images
val: annotations/val_images
test: annotations/test_images

nc: 1  # 1 class: hand
names: ['hand']
```

### 3.2 Train with YOLOv8n

```python
from ultralytics import YOLO

# Load a pretrained model
model = YOLO("yolov8n.pt")

# Train the model
results = model.train(
    data="data/hand_detect.yaml",
    epochs=100,
    imgsz=320,
    batch=32,
    patience=20,
    device=0  # GPU index
)

# Export to ONNX
model.export(format="onnx")
```

**Output:**
- `runs/detect/train/weights/best.pt` (PyTorch)
- `runs/detect/train/weights/best.onnx` (ONNX)

**Copy to project:**
```bash
cp runs/detect/train/weights/best.onnx models/hand_detector.onnx
```

---

## Phase 4: Train Landmark Regressor (Your Task — Colab GPU — 1-2 days)

### 4.1 PyTorch Training Script

```python
import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import DataLoader, Dataset
import torchvision.models as models
import pandas as pd
import cv2
from pathlib import Path

class LandmarkDataset(Dataset):
    def __init__(self, csv_path, img_dir, transform=None):
        self.df = pd.read_csv(csv_path)
        self.img_dir = img_dir
        self.transform = transform

    def __len__(self):
        return len(self.df)

    def __getitem__(self, idx):
        row = self.df.iloc[idx]
        img_path = row['image']
        image = cv2.imread(img_path)
        image = cv2.resize(image, (224, 224))
        image = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
        if self.transform:
            image = self.transform(image)
        
        landmarks = []
        for i in range(21):
            landmarks.extend([row[f'x{i}'], row[f'y{i}'], row[f'z{i}']])
        
        landmarks = torch.tensor(landmarks, dtype=torch.float32)
        return image, landmarks

# Model: MobileNetV3 → FC → 63 outputs
class LandmarkModel(nn.Module):
    def __init__(self):
        super().__init__()
        mobilenet = models.mobilenet_v3_small(pretrained=True)
        self.features = mobilenet.features
        self.avgpool = nn.AdaptiveAvgPool2d(1)
        self.classifier = nn.Sequential(
            nn.Linear(576, 256),
            nn.ReLU(),
            nn.Dropout(0.5),
            nn.Linear(256, 63)
        )

    def forward(self, x):
        x = self.features(x)
        x = self.avgpool(x)
        x = torch.flatten(x, 1)
        x = self.classifier(x)
        return x

# Training loop
device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
model = LandmarkModel().to(device)
optimizer = optim.Adam(model.parameters(), lr=1e-3)
criterion = nn.MSELoss()

dataset = LandmarkDataset("data/annotations/landmark_labels.csv", "data/extracted_frames")
dataloader = DataLoader(dataset, batch_size=32, shuffle=True)

for epoch in range(100):
    for images, landmarks in dataloader:
        images, landmarks = images.to(device), landmarks.to(device)
        
        optimizer.zero_grad()
        outputs = model(images)
        loss = criterion(outputs, landmarks)
        loss.backward()
        optimizer.step()
    
    print(f"Epoch {epoch}, Loss: {loss.item():.6f}")

# Export to ONNX
torch.onnx.export(
    model,
    torch.randn(1, 3, 224, 224).to(device),
    "models/hand_landmark.onnx",
    input_names=['image'],
    output_names=['landmarks'],
    opset_version=12
)
```

---

## Phase 5: Create Gesture Sequence Dataset (Your Task — 1 day)

### 5.1 Extract Landmark Sequences from Videos

```python
import mediapipe as mp
import numpy as np
import json

mp_hands = mp.solutions.hands.Hands(
    static_image_mode=False,
    max_num_hands=1,
    min_detection_confidence=0.7
)

gesture_sequences = []

for gesture in os.listdir("data/raw_videos"):
    for video_file in os.listdir(f"data/raw_videos/{gesture}"):
        cap = cv2.VideoCapture(f"data/raw_videos/{gesture}/{video_file}")
        landmarks_seq = []
        
        while cap.isOpened():
            ret, frame = cap.read()
            if not ret:
                break
            
            results = mp_hands.process(cv2.cvtColor(frame, cv2.COLOR_BGR2RGB))
            if results.multi_hand_landmarks:
                hand = results.multi_hand_landmarks[0]
                landmarks_flat = []
                for lm in hand.landmark:
                    landmarks_flat.extend([lm.x, lm.y, lm.z])
                landmarks_seq.append(landmarks_flat)
        
        cap.release()
        
        # Keep sequences of length 30 (1 second at 30 FPS)
        if len(landmarks_seq) >= 30:
            for i in range(len(landmarks_seq) - 30):
                gesture_sequences.append({
                    "gesture": gesture,
                    "sequence": landmarks_seq[i:i+30]
                })

with open("data/annotations/gesture_sequences.json", "w") as f:
    json.dump(gesture_sequences, f)
```

---

## Phase 6: Train Gesture Classifier (Your Task — Colab GPU — 1-2 days)

### 6.1 PyTorch LSTM Model

```python
import torch
import torch.nn as nn
import json
from torch.utils.data import DataLoader, Dataset

class GestureDataset(Dataset):
    def __init__(self, json_path, gesture_names):
        with open(json_path, 'r') as f:
            data = json.load(f)
        
        self.gesture_to_idx = {g: i for i, g in enumerate(gesture_names)}
        self.sequences = []
        self.labels = []
        
        for item in data:
            gesture = item['gesture']
            sequence = torch.tensor(item['sequence'], dtype=torch.float32)
            self.sequences.append(sequence)
            self.labels.append(self.gesture_to_idx[gesture])

    def __len__(self):
        return len(self.sequences)

    def __getitem__(self, idx):
        return self.sequences[idx], self.labels[idx]

class GestureClassifier(nn.Module):
    def __init__(self, num_classes, sequence_length=30, landmark_dim=63):
        super().__init__()
        self.lstm = nn.LSTM(
            input_size=landmark_dim,
            hidden_size=128,
            num_layers=2,
            dropout=0.5,
            batch_first=True
        )
        self.fc = nn.Sequential(
            nn.Linear(128, 64),
            nn.ReLU(),
            nn.Dropout(0.5),
            nn.Linear(64, num_classes)
        )

    def forward(self, x):
        # x shape: (batch, sequence_length, landmark_dim)
        lstm_out, (h_n, c_n) = self.lstm(x)
        # Use last hidden state
        last_hidden = h_n[-1]  # (batch, hidden_size)
        logits = self.fc(last_hidden)
        return logits

# Training
gesture_names = [
    "OpenPalm", "ClosedFist", "Point", "Pinch",
    "SwipeLeft", "SwipeRight", "SwipeUp", "SwipeDown",
    "RotateClockwise", "RotateCounterclockwise", "ThreeFingerSwipe"
]

device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
model = GestureClassifier(num_classes=len(gesture_names)).to(device)
optimizer = optim.Adam(model.parameters(), lr=1e-3)
criterion = nn.CrossEntropyLoss()

dataset = GestureDataset("data/annotations/gesture_sequences.json", gesture_names)
dataloader = DataLoader(dataset, batch_size=32, shuffle=True)

for epoch in range(100):
    for sequences, labels in dataloader:
        sequences, labels = sequences.to(device), labels.to(device)
        
        optimizer.zero_grad()
        logits = model(sequences)
        loss = criterion(logits, labels)
        loss.backward()
        optimizer.step()
    
    print(f"Epoch {epoch}, Loss: {loss.item():.6f}")

# Export to ONNX
torch.onnx.export(
    model,
    torch.randn(1, 30, 63).to(device),
    "models/gesture_classifier.onnx",
    input_names=['landmark_sequence'],
    output_names=['gesture_logits'],
    opset_version=12
)
```

---

## Phase 7: Place ONNX Models in Project

After training and exporting all three models:

```
gesture_rt/
├── models/
│   ├── hand_detector.onnx
│   ├── hand_landmark.onnx
│   └── gesture_classifier.onnx
└── src/
    └── ...
```

---

## Phase 8: Implement ONNX Runtime in Rust (Already Done)

The Rust side is ready. Key files:

- `src/vision/landmarks/tracker.rs` — ROI tracking (Stage 2)
- `src/vision/landmarks/gesture_classifier.rs` — Gesture sequence buffer & classifier interface (Stage 4-5)
- `src/vision/landmarks/onnx_extractor.rs` — Landmark model preprocessing (Stage 3)
- `examples/onnx_pipeline.rs` — Full pipeline example

To enable inference, implement the `run_model` methods using `onnxruntime` crate (you'll need to install system ONNX Runtime).

---

## Phase 9: Validation & Metrics (Your Task — continuous)

### Detector Metrics
- mAP (mean Average Precision) on validation set
- Target: > 0.90 mAP

### Landmark Metrics
- Per-keypoint pixel error on validation set
- Target: < 5 pixels mean error
- Excellent: < 3 pixels

### Gesture Classifier Metrics
- Accuracy on holdout test set per gesture
- Confusion matrix (which gestures are confused?)
- Target: > 90% accuracy
- Excellent: > 95% accuracy

```python
from sklearn.metrics import accuracy_score, confusion_matrix
import numpy as np

predictions = []
ground_truth = []

# Evaluate on test set
model.eval()
with torch.no_grad():
    for sequences, labels in test_dataloader:
        sequences = sequences.to(device)
        logits = model(sequences)
        preds = torch.argmax(logits, dim=1)
        predictions.extend(preds.cpu().numpy())
        ground_truth.extend(labels.numpy())

acc = accuracy_score(ground_truth, predictions)
cm = confusion_matrix(ground_truth, predictions)
print(f"Accuracy: {acc:.4f}")
print("Confusion Matrix:")
print(cm)
```

---

## Phase 10: Fine-Tuning on Your Device

Once models are in production, collect data from your actual deployment device and fine-tune:

```python
# Fine-tuning loop (smaller learning rate)
model.load_state_dict(torch.load("models/gesture_classifier.pt"))
optimizer = optim.Adam(model.parameters(), lr=1e-4)  # Lower LR

for epoch in range(20):  # Fewer epochs
    for sequences, labels in fine_tune_dataloader:
        # Training loop (same as Phase 6)
```

---

## Timeline Estimate

| Phase | Task | Time | Difficulty |
|-------|------|------|------------|
| 1 | Data collection | 7–10 days | Medium (tedious but straightforward) |
| 2 | Auto-labeling with MediaPipe | 1–2 days | Low |
| 3 | Train detector (YOLOv8n) | 1–2 days | Low (use pretrained) |
| 4 | Train landmark model | 1–2 days | Low (use pretrained MobileNetV3) |
| 5 | Create gesture sequences | 1 day | Low |
| 6 | Train gesture classifier (LSTM) | 1–2 days | Medium |
| 7–10 | Validation & fine-tuning | 3–5 days | High |

**Total: 3–4 weeks of work** (mostly data collection)

---

## Hardware Requirements

### For Training
- GPU: NVIDIA with CUDA support (or cloud)
  - Recommended: AWS g4dn.xlarge, Google Colab, or Kaggle
- RAM: 16GB+
- Storage: 100GB+ for raw videos

### For Inference (Rust/GestureRT)
- CPU: Modern multi-core (Intel i5+, AMD Ryzen 5+)
- GPU: Optional (DirectML on Windows, CUDA, Metal on macOS)
- RAM: 4GB+ for full pipeline
- Storage: ~50MB for three ONNX models

---

## Troubleshooting

### Problem: Low detector accuracy
- **Cause:** Insufficient diverse training data
- **Fix:** Collect more videos with varied lighting, distances, backgrounds
- **Test:** Evaluate on held-out validation set before training classifier

### Problem: Landmarks jittery/unstable
- **Cause:** Inadequate temporal smoothing or poor landmark model accuracy
- **Fix:** 
  - Increase Kalman filter smoothness (`kalman_process_noise_pos`)
  - Collect more landmark training data
  - Use larger padding in ROI expansion

### Problem: Gestures confused (swipe left → swipe right)
- **Cause:** Overlapping gesture patterns or insufficient landmark sequence data
- **Fix:**
  - Collect more diverse sequences for each gesture
  - Review landmark quality (ensure landmarks track fingertips accurately)
  - Increase LSTM hidden size or sequence length

### Problem: ONNX Runtime binding errors
- **Cause:** ONNX model incompatibility or missing system library
- **Fix:**
  - Verify opset_version == 12 in export
  - Ensure ONNX Runtime system library installed
  - Check input/output tensor shapes match Rust expectations

---

## Production Checklist

- [ ] Collected 500+ examples per gesture
- [ ] Detector mAP > 0.90 on validation set
- [ ] Landmark model < 5 pixel error
- [ ] Gesture classifier > 90% accuracy on held-out test set
- [ ] ONNX models validated (inference runs in < 50ms on target hardware)
- [ ] Regression dataset defined for continuous CI testing
- [ ] Device-specific calibration (focal length, lens distortion) implemented
- [ ] Fallback graceful degradation when models fail
- [ ] Confidence thresholds tuned on production hardware
- [ ] Documentation for future model updates

---

## References

- [MediaPipe Hands](https://mediapipe.dev/solutions/hands)
- [YOLOv8 Training](https://docs.ultralytics.com/modes/train/)
- [PyTorch ONNX Export](https://pytorch.org/docs/stable/onnx.html)
- [ONNX Runtime Rust Bindings](https://crates.io/crates/onnxruntime)
- [MobileNetV3 Paper](https://arxiv.org/abs/1905.02175)

---

**Last Updated:** 2026-06-22  
**Author:** GestureRT Development Team
