# GestureRT Cross-Platform Architecture

## Core Principle

**Train Once, Run Everywhere**

The gesture recognition engine (ONNX models + inference pipeline) is identical across all platforms. Only the **platform adapter** changes, translating `GestureAction` to OS-specific commands.

---

## Architecture Layers

```
┌─────────────────────────────────────────────────────┐
│           Gesture Recognition Engine                │
│  (Camera → ONNX Detector → ONNX Landmarks           │
│   → ONNX Gesture Classifier)                        │
└────────────────────┬────────────────────────────────┘
                     │
                     ↓
            ┌────────────────────┐
            │  GestureAction     │
            │  (Platform-agnostic)
            │  ├─ SwipeLeft      │
            │  ├─ SwipeRight     │
            │  ├─ OpenPalm       │
            │  ├─ Pinch          │
            │  └─ ...            │
            └────────────────────┘
                     │
         ┌───────────┼───────────┬──────────┐
         ↓           ↓           ↓          ↓
    ┌────────┐ ┌────────┐ ┌──────────┐ ┌───────┐
    │Windows │ │ Linux  │ │ Android  │ │ macOS │
    │Adapter │ │Adapter │ │ Adapter  │ │Adapter│
    └────────┘ └────────┘ └──────────┘ └───────┘
         │           │           │          │
         ↓           ↓           ↓          ↓
    Keyboard    xdotool    Accessibility  Quartz
     Mouse                    Service      Events
```

---

## File Organization

```
gesture_rt/
├── src/
│   ├── platform/                   # NEW: Platform abstraction
│   │   ├── mod.rs                  # Module root + get_adapter()
│   │   ├── action.rs               # GestureAction enum
│   │   ├── adapter.rs              # PlatformAdapter trait
│   │   ├── windows.rs              # Windows implementation
│   │   ├── linux.rs                # Linux implementation
│   │   ├── android.rs              # Android implementation
│   │   └── macos.rs                # macOS implementation
│   │
│   ├── vision/                     # Unchanged
│   ├── gestures/                   # Unchanged
│   ├── core/                       # Refactor: emit GestureAction
│   └── runtime_dispatch/           # Deprecated: replaced by platform/
│
└── docs/
    ├── ARCHITECTURE_CROSSPLATFORM.md    # This file
    ├── ANDROID_ROADMAP.md               # Android implementation guide
    └── PLATFORM_PORTING.md              # How to add a new platform
```

---

## Platform Adapter Interface

```rust
pub trait PlatformAdapter: Send + Sync {
    fn handle(&self, context: &GestureContext) -> ActionResult;
    fn name(&self) -> &'static str;
    fn is_ready(&self) -> bool;
    fn init(&self) -> ActionResult;
    fn shutdown(&self) -> ActionResult;
}
```

### GestureContext

```rust
pub struct GestureContext {
    pub action: GestureAction,
    pub confidence: f32,
    pub timestamp: u64,
    pub hand_x: Option<f32>,  // Normalized 0-1 for pointer gestures
    pub hand_y: Option<f32>,
    pub magnitude: Option<f32>, // For rotation/scale gestures
}
```

---

## Platform Implementations

### Windows (`platform/windows.rs`)

| Gesture | Action |
|---------|--------|
| SwipeLeft | Alt+Tab |
| SwipeRight | Shift+Alt+Tab |
| SwipeUp | Win+Up (snap) |
| SwipeDown | Win+Down (snap) |
| OpenPalm | Win+D (desktop) |
| ClosedFist | Win+Esc (lock) |
| Point | Left click |
| Pinch | Ctrl+Minus (zoom) |

Uses: **enigo** crate (already in dependencies)

---

### Linux (`platform/linux.rs`)

| Gesture | Action |
|---------|--------|
| SwipeLeft | Alt+Tab |
| SwipeRight | Shift+Alt+Tab |
| SwipeUp | Super+Up |
| SwipeDown | Super+Down |
| OpenPalm | Super+D |
| ClosedFist | Super+L (lock) |
| Point | Click |
| Pinch | Ctrl+Minus |

Uses: **xdotool** (external command)

Requires: `sudo apt install xdotool`

---

### macOS (`platform/macos.rs`)

| Gesture | Action |
|---------|--------|
| SwipeLeft | Cmd+[ (back) |
| SwipeRight | Cmd+] (forward) |
| SwipeUp | F3 (Mission Control) |
| SwipeDown | F11 (Show Desktop) |
| OpenPalm | Cmd+Space (Spotlight) |
| ClosedFist | Sleep |
| Point | Left click |
| Pinch | Cmd+Minus (zoom) |

Uses: **Quartz Event Services** (macOS native framework)

Requires: Accessibility permissions

---

### Android (`platform/android.rs`)

| Gesture | Action |
|---------|--------|
| SwipeLeft | Back |
| SwipeRight | Recent Apps |
| SwipeUp | Notifications |
| SwipeDown | Quick Settings |
| OpenPalm | Home |
| ClosedFist | Lock Screen |
| Point | Tap at hand position |
| Pinch | Screenshot |

Uses: **Android AccessibilityService** (via Kotlin/JNI bridge)

Requires: Accessibility Service enabled + flutter_rust_bridge

See: [ANDROID_ROADMAP.md](ANDROID_ROADMAP.md)

---

## Runtime Integration

### Before (Tightly Coupled)

```rust
// Old: Hard-coded dispatch
match gesture_type {
    SWIPE_LEFT => keyboard::alt_tab(),
    SWIPE_RIGHT => keyboard::shift_alt_tab(),
    // ...
}
```

### After (Platform-Agnostic)

```rust
// New: Platform adapter dispatch
let adapter = platform::get_adapter();
let context = GestureContext::new(GestureAction::SwipeLeft, confidence, timestamp);
adapter.handle(&context)?;
```

The runtime doesn't know if it's Windows, Linux, Android, or macOS.

---

## Runtime Flow Refactor

### Current (Gesture Engine only):

```
Camera
  ↓
Detector ONNX
  ↓
Tracker
  ↓
Landmark ONNX
  ↓
Gesture Classifier ONNX
  ↓
Action (hardcoded per platform)
```

### Refactored (Clean Separation):

```
Camera
  ↓
Detector ONNX
  ↓
Tracker
  ↓
Landmark ONNX
  ↓
Gesture Classifier ONNX
  ↓
GestureAction enum
  ↓
Platform Adapter (runtime-selected)
  ↓
OS-Specific Command
```

---

## How to Add a New Platform

1. Create `src/platform/newos.rs`
2. Implement `PlatformAdapter` trait
3. Add pattern match in `platform::get_adapter()` for `#[cfg(target_os = "newos")]`
4. Implement gesture mappings for your OS
5. Add system-level integration (keyboard, mouse, accessibility APIs as needed)

Example:

```rust
// src/platform/newos.rs
pub struct NewOSAdapter { ready: bool }

impl PlatformAdapter for NewOSAdapter {
    fn handle(&self, context: &GestureContext) -> ActionResult {
        match context.action {
            GestureAction::SwipeLeft => { /* Your implementation */ }
            // ...
        }
    }
    // ...
}
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_swipe_left() {
        let adapter = WindowsAdapter::new();
        let ctx = GestureContext::new(GestureAction::SwipeLeft, 0.95, 1000);
        let result = adapter.handle(&ctx);
        assert!(result.success);
    }
}
```

### Cross-Platform CI

```yaml
test:
  matrix:
    - os: ubuntu-latest
      run: cargo test --target x86_64-unknown-linux-gnu
    
    - os: windows-latest
      run: cargo test --target x86_64-pc-windows-msvc
    
    - os: macos-latest
      run: cargo test --target x86_64-apple-darwin
    
    - os: ubuntu-latest
      run: cargo test --target x86_64-linux-android
```

---

## Benefits of This Design

1. **Modularity:** Each platform is isolated; changes to one don't affect others
2. **Testability:** Mock adapters for unit tests; platform-specific adapters for integration tests
3. **Extensibility:** Add new platforms without touching gesture engine
4. **Maintainability:** Clear responsibility boundaries
5. **Reusability:** Same ONNX models, same training pipeline, different platforms

---

## Model Distribution

All platforms use **identical ONNX model files**:

```
models/
├── hand_detector.onnx          (2-5 MB)
├── hand_landmark.onnx          (5-10 MB)
└── gesture_classifier.onnx     (1-2 MB)
Total: ~10-17 MB per platform
```

### Platform Packaging

| Platform | Package Size | Includes |
|----------|--------------|----------|
| Windows | ~150 MB | App + ONNX Runtime + Models |
| Linux | ~120 MB | App + ONNX Runtime + Models |
| macOS | ~140 MB | App + ONNX Runtime + Models |
| Android APK | ~80 MB | App + ONNX Runtime Mobile + Models |

---

## Performance Targets

### Desktop (Windows/Linux/macOS)

| Component | Target | Typical |
|-----------|--------|---------|
| Camera capture | 30 FPS | 30 FPS |
| Detector | < 20 ms | 10-15 ms |
| Landmarks | < 10 ms | 5-8 ms |
| Gesture classifier | < 5 ms | 2-3 ms |
| Platform dispatch | < 5 ms | 1-2 ms |
| **Total latency** | **< 50 ms** | **30-40 ms** |

### Android

| Component | Target | Note |
|-----------|--------|------|
| Camera capture | 24-30 FPS | Device-dependent |
| Full pipeline | < 100 ms | Include phone overhead |
| Platform dispatch | < 10 ms | Accessibility Service |

---

## Security Considerations

### Windows
- Requires no special permissions for Alt+Tab, Win+X
- System hotkeys protected by Windows

### Linux
- xdotool requires X11/Wayland access
- User must be in `input` group for some operations

### macOS
- Requires "Accessibility" permission grant
- User explicitly authorizes in System Preferences

### Android
- Requires "Accessibility Service" permission
- User explicitly authorizes in Settings
- Cannot access system UI directly without root (by design)
- Accessibility API is the intended interface

---

## Migration Path from Current System

### Phase 1: Introduce Platform Module
- Keep existing `runtime_dispatch/` as-is
- Add `platform/` module alongside
- Runtime can use either (feature flag)

### Phase 2: Refactor Core Runtime
- Update `core::runtime.rs` to emit `GestureAction` instead of direct dispatch
- Wire to platform adapter
- Deprecate `runtime_dispatch/`

### Phase 3: Platform Parity
- Verify all platforms work
- Performance testing on each OS
- User testing

### Phase 4: Cleanup
- Remove `runtime_dispatch/` module
- Ship platform-based system

---

## Future: Remote Gesture Control

Extend the design to support **remote Android-to-Desktop**:

```
Android Phone
    ↓
GestureRT (Rust core)
    ↓
Network (Wi-Fi/Bluetooth)
    ↓
Desktop GestureRT
    ↓
Platform Adapter (Windows/Linux/macOS)
    ↓
Desktop Action
```

This requires:
- Network protocol for gesture serialization
- Desktop "remote adapter" receiving over network
- Low-latency synchronization

The abstraction already supports this — just add a `remote::RemoteAdapter` that sends gestures over the network.

---

## References

- [GestureRT Main README](README.md)
- [Android Implementation Roadmap](ANDROID_ROADMAP.md)
- [Platform Porting Guide](PLATFORM_PORTING.md)
- [TRAINING_GUIDE.md](TRAINING_GUIDE.md) — Model training (unchanged)

---

**Last Updated:** 2026-06-22  
**Status:** Architecture complete; awaiting implementation feedback
