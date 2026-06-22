MediaPipe bridge integration notes
=================================

This document describes a minimal native bridge the Rust side expects when enabling the `mediapipe` feature.

Goal
----
Provide a small C API exported from a DLL/shared library that wraps MediaPipe C++ graph execution and exposes two functions:

- `int mp_initialize()` — initialize MediaPipe and load graph(s). Return 0 on success.
- `int mp_extract_landmarks(const uint8_t* frame_ptr, size_t frame_len, float* out_ptr, size_t out_len)`
   - Accepts an encoded raw frame buffer (e.g., BGR bytes or a simple raw bitmap agreed on by caller)
   - Writes landmark floats into `out_ptr` (e.g., 21 landmarks × 3 floats = 63 floats)
   - Returns the number of floats written, or a negative error code on failure.

Recommended approach
--------------------
1. Implement a small C++ wrapper that uses MediaPipe graph APIs to run single-frame inference.
2. Expose `extern "C"` functions above and build as a DLL (`mediapipe_bridge.dll`) on Windows.
3. Place the DLL where the Rust binary can load it (same folder or in PATH).

Minimal example header (C):

```c
// mediapipe_bridge.h
#include <stdint.h>

extern "C" {
    int mp_initialize();
    int mp_extract_landmarks(const uint8_t* frame_ptr, size_t frame_len, float* out_ptr, size_t out_len);
}
```

Notes
-----
- Building MediaPipe typically uses Bazel and can be heavy; consider building a small graph and packaging only the required binaries.
- Alternatively, use an ONNX model + `onnxruntime` for easier distribution if MediaPipe packaging is too heavy.
