# GestureRT Android Roadmap

## Strategic Direction

**Android as a first-class, primary platform.**

Not "GestureRT on Android" — but **"GestureRT *is* Android, also runs on desktop."**

The killer feature: **Any Android phone becomes a wireless spatial controller for any desktop.**

---

## Use Case: Phone-as-Controller

```
┌─────────────┐
│   Android   │
│    Phone    │
│ (Camera +   │
│  Rust Core) │
└──────┬──────┘
       │ ONNX inference
       │ GestureAction
       │
       ├─ Local dispatch (Accessibility)
       │  └─ Control *this* phone
       │
       └─ Network dispatch (Wi-Fi/BLE)
          └─ Control *another* device
              (PC/Mac/Linux)
```

### Advantages

1. **Everyone has an Android phone.** No need to buy a camera.
2. **Mobile devices are always with you.** Spontaneous control anywhere.
3. **Accessibility APIs are designed for this.** No root required.
4. **ONNX Runtime Mobile is production-proven.** Used in production by major apps.

---

## Phase 1: Android Prototype (2–3 weeks)

### Goal
Proof-of-concept: Android app that recognizes gestures locally and executes accessibility actions on the same device.

### Architecture

```
Flutter App
    ↓
 [UI Layer]
    ↓
flutter_rust_bridge
    ↓
 [Rust Core + ONNX]
    ↓
[Kotlin Bridge]
    ↓
Android Accessibility Service
```

### What You Build

#### 1. Rust Core (Unchanged)
- Existing ONNX inference pipeline
- Gesture classification
- **No changes** to camera → ONNX → GestureAction pipeline

#### 2. Kotlin Bridge (`android/` folder)
```kotlin
// AccessibilityService that receives GestureAction from Rust
class GestureAccessibilityService : AccessibilityService() {
    override fun onAccessibilityEvent(event: AccessibilityEvent) {
        // Receive gesture from Rust via JNI
        handleGestureAction(gestureAction)
    }
    
    fun handleGestureAction(action: String) {
        when (action) {
            "SwipeLeft" -> performGlobalAction(GLOBAL_ACTION_BACK)
            "SwipeRight" -> performGlobalAction(GLOBAL_ACTION_RECENTS)
            "OpenPalm" -> performGlobalAction(GLOBAL_ACTION_HOME)
            else -> {}
        }
    }
}
```

#### 3. Flutter UI (`flutter/` folder)
```dart
class GestureRecognizerApp extends StatefulWidget {
  // Camera preview
  // Gesture visualization
  // Settings (model path, confidence threshold)
}
```

#### 4. FFI Bridge (`flutter_rust_bridge`)
```rust
// Rust side: exposed to Dart/Flutter
#[flutter_rust_bridge::frb(sync)]
pub fn start_gesture_recognition() -> String {
    // Initialize camera + ONNX
}

#[flutter_rust_bridge::frb(sync)]
pub fn get_last_gesture() -> Option<GestureAction> {
    // Return detected gesture
}
```

### Dependencies

```yaml
# pubspec.yaml
flutter_rust_bridge: ^1.80
```

```toml
# Cargo.toml (android feature)
[dependencies]
flutter_rust_bridge = "1.80"
```

### Android Manifest

```xml
<uses-permission android:name="android.permission.CAMERA" />
<uses-permission android:name="android.permission.BIND_ACCESSIBILITY_SERVICE" />

<service
    android:name=".GestureAccessibilityService"
    android:permission="android.permission.BIND_ACCESSIBILITY_SERVICE">
    <intent-filter>
        <action android:name="android.accessibilityservice.AccessibilityService" />
    </intent-filter>
</service>
```

### Steps

1. **Create Android project structure**
   ```
   gesture_rt/
   ├── rust/ (existing Rust core)
   ├── flutter/ (new Flutter UI)
   ├── android/
   │   ├── app/src/main/kotlin/...AccessibilityService.kt
   │   └── AndroidManifest.xml
   ```

2. **Implement Kotlin AccessibilityService**
   - Receives gesture actions from Rust
   - Executes accessibility commands
   - Logs/debugging UI

3. **Set up flutter_rust_bridge**
   - Generate FFI bindings
   - Test Rust ↔ Flutter communication

4. **Build APK**
   ```bash
   flutter build apk
   ```

5. **Test on device/emulator**
   - Enable accessibility service
   - Point camera at hand
   - Perform gestures
   - Verify accessibility actions trigger

### Testing Checklist

- [ ] Phone camera capture works (30 FPS)
- [ ] ONNX inference runs on device (< 100 ms latency)
- [ ] Gesture detected and logged
- [ ] Accessibility service running
- [ ] SwipeLeft → Back works
- [ ] SwipeRight → Recent apps works
- [ ] OpenPalm → Home works
- [ ] Battery drain acceptable (< 1% per 5 minutes idle with recognition running)

---

## Phase 2: Network Protocol (1–2 weeks)

### Goal
Android phone sends gestures to desktop over Wi-Fi.

### Architecture

```
Android Phone (GestureRT)
    ↓
 [Gesture Engine]
    ↓
 [Network Protocol]
    ↓
 Wi-Fi/Local Network
    ↓
 [Desktop Receiver]
    ↓
 Platform Adapter (Windows/Linux/macOS)
    ↓
 Desktop Action
```

### Protocol

**Simple JSON over TCP:**

```json
{
  "type": "gesture",
  "action": "SwipeLeft",
  "confidence": 0.95,
  "timestamp": 1687123456789,
  "device_id": "phone-12345"
}
```

### Implementation

#### Rust Side (Server on desktop, Client on phone)

```rust
// Desktop receiver
async fn start_gesture_server(adapter: Arc<dyn PlatformAdapter>) {
    let listener = TcpListener::bind("0.0.0.0:9999").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        let msg = read_json_from_socket(socket).await?;
        
        let action = serde_json::from_str::<GestureAction>(&msg)?;
        let context = GestureContext::new(action, 0.95, /* ... */);
        adapter.handle(&context)?;
    }
}

// Phone sender
async fn send_gesture_to_desktop(
    desktop_ip: &str,
    port: u16,
    gesture: GestureAction
) -> anyhow::Result<()> {
    let mut socket = TcpStream::connect(format!("{}:{}", desktop_ip, port)).await?;
    let json = serde_json::to_string(&gesture)?;
    socket.write_all(json.as_bytes()).await?;
    Ok(())
}
```

#### Flutter UI (Desktop pairing)

```dart
// Android app shows:
// - Local network IP
// - "Pair with desktop" button
// - Paired device list
```

### Steps

1. **Add networking to Rust core**
   - tokio TCP server (desktop)
   - tokio TCP client (Android)
   - Serialization: serde_json

2. **Update Flutter UI**
   - Settings: desktop IP address
   - Connection status
   - Local network discovery (mDNS optional)

3. **Test**
   - Both on same Wi-Fi network
   - Desktop listening on port 9999
   - Android sends gesture
   - Desktop receives and dispatches

### Performance

- **Latency:** < 50 ms network + local processing
- **Bandwidth:** ~1 KB per gesture
- **Jitter:** Negligible (gestures are sporadic)

---

## Phase 3: Phone Optimization (1 week)

### Battery

Typical gesture recognition:
- Camera: 50-100 mW (continuous)
- ONNX inference: 100-200 mW (intermittent)
- Display: 500+ mW (but usually off when using as controller)

**Target:** 5% battery per hour with screen off, continuous recognition.

**Optimizations:**
- Frame skipping (process every 2nd frame)
- Power-efficient ONNX provider (NNAPI)
- Battery saver mode awareness

### Model Optimization

For Android, optimize models for mobile:

```python
# TensorFlow Lite Quantization
converter = tf.lite.TFLiteConverter.from_saved_model(saved_model_dir)
converter.optimizations = [tf.lite.Optimize.DEFAULT]
converter.target_spec.supported_ops = [
    tf.lite.OpsSet.TFLITE_BUILTINS_INT8
]
tflite_model = converter.convert()
```

**Or use ONNX quantization:**

```bash
python -m onnxruntime.transformers.convert_optimizer \
    --model_type bert \
    --model_path hand_landmark.onnx \
    --optimized_model_path hand_landmark_optimized.onnx
```

### Testing Checklist

- [ ] Battery drain < 5% per hour
- [ ] Works on low-end Android (Snapdragon 600 series)
- [ ] Works on midrange (Snapdragon 700 series)
- [ ] Works on flagship (Snapdragon 8 series)
- [ ] Memory usage < 200 MB
- [ ] Thermal management (doesn't overheat)

---

## Phase 4: Desktop Pairing (1 week)

### Goal
Simple desktop app that receives gestures from Android phone.

### Components

#### Desktop Receiver (`desktop/`)

```rust
// Standalone desktop app that:
// 1. Listens for incoming gestures
// 2. Pairs with Android phones
// 3. Routes to platform adapter

fn main() {
    let adapter = platform::get_adapter();
    
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        start_gesture_server(&adapter).await
    });
}
```

#### Configuration

```yaml
# ~/.config/gesturert/config.yaml
server:
  port: 9999
  bind: 0.0.0.0

devices:
  - name: "My Phone"
    id: "phone-12345"
    trusted: true

actions:
  swipe_left: alt+tab
  swipe_right: shift+alt+tab
  open_palm: win+d
```

### Steps

1. **Build minimal desktop receiver**
2. **Support device pairing** (optional auth token)
3. **Test on Windows, Linux, macOS**

---

## Phase 5: Production Hardening (2 weeks)

### Reliability

- [ ] Graceful disconnection handling
- [ ] Reconnection on network glitches
- [ ] Timeout mechanisms
- [ ] Error logging
- [ ] Crash recovery

### Security

- [ ] Encrypted network (TLS/mTLS)
- [ ] Device pairing tokens
- [ ] Rate limiting
- [ ] Audit logging

### Performance

- [ ] 99th percentile latency < 100 ms
- [ ] Jitter < 50 ms
- [ ] Loss rate < 1%

### Testing

- [ ] Stress test: 1000 gestures/sec
- [ ] Network degradation: 2G, 3G, 4G, 5G
- [ ] Multi-device: control PC from 3+ phones
- [ ] Failover: network reconnection

---

## Phase 6: Distribution (Ongoing)

### Android App

```
Google Play Store
    ↓
 [GestureRT]
 ├─ Free tier: Local control only
 └─ Premium tier: Remote control ($2.99)
```

### Desktop Software

```
Windows: MSI installer
Linux: Snap/AppImage/native packages
macOS: DMG installer
```

---

## Timeline Estimate

| Phase | Duration | Owner | Complexity |
|-------|----------|-------|------------|
| 1: Android MVP | 2–3 weeks | You | Medium |
| 2: Network Protocol | 1–2 weeks | You | Medium |
| 3: Phone Optimization | 1 week | You | Low |
| 4: Desktop Pairing | 1 week | You | Low |
| 5: Production Hardening | 2 weeks | You | High |
| 6: Distribution | Ongoing | Community | High |
| **Total** | **~8–9 weeks** | | |

---

## Contingency Plans

### If ONNX Runtime Mobile is too slow
**Fallback:** Use TensorFlow Lite Mobile (same models can be converted)

### If accessibility API too restrictive
**Fallback:** Device owner mode (enterprise Android)

### If network latency unacceptable
**Fallback:** Bluetooth Low Energy (BLE) for direct device control

### If battery drain too high
**Fallback:** On-device frame rate reduction (15 FPS instead of 30)

---

## Success Metrics

### Phase 1 Success
- APK < 100 MB
- Latency < 100 ms (local)
- Gesture recognition > 85% accuracy
- Accessibility service reliable

### Phase 2 Success
- Network latency < 50 ms
- Multi-device pairing works
- Desktop receiver stable

### Overall Success
- 10K+ downloads on Play Store
- < 2% crash rate
- > 4.0 star rating
- Users report "best spatial input on mobile"

---

## Competitive Positioning

| Product | Desktop | Mobile | Accuracy | Open-Source |
|---------|---------|--------|----------|------------|
| MediaPipe | ✅ | ✅ | High | ✅ |
| Leap Motion | ✅ | ❌ | Very High | ❌ |
| Microsoft Kinect | ✅ | ❌ | High | ❌ |
| **GestureRT** | ✅ | **✅** | High | **✅** |

**Unique proposition:** Only open-source gesture system that prioritizes Android as a first-class platform + works everywhere with same models.

---

## Technology Stack

### Android

| Layer | Technology |
|-------|-----------|
| UI | Flutter |
| Rust Bridge | flutter_rust_bridge |
| Inference | ONNX Runtime Mobile (Rust binding) |
| Dispatch | Android Accessibility Service |
| Network | tokio + serde |

### Desktop (Linux/Windows/macOS)

| Layer | Technology |
|-------|-----------|
| Inference | ONNX Runtime |
| Dispatch | Platform adapters |
| Network | tokio + serde |

---

## References

- [ARCHITECTURE_CROSSPLATFORM.md](ARCHITECTURE_CROSSPLATFORM.md) — General cross-platform design
- [ONNX Runtime Mobile](https://onnxruntime.ai/docs/execution-providers/CoreML-ExecutionProvider.html)
- [flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge)
- [Android Accessibility Service API](https://developer.android.com/reference/android/accessibilityservice/AccessibilityService)

---

**Last Updated:** 2026-06-22  
**Strategic Priority:** HIGHEST — This enables "run everywhere" and "phone as controller"
