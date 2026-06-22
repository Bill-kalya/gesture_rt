// Lightweight MediaPipe FFI bridge skeleton.
// This file declares the C symbols the native MediaPipe bridge DLL should expose.

#[cfg(feature = "mediapipe")]
extern "C" {
    // Initialize MediaPipe runtime. Returns 0 on success.
    pub fn mp_initialize() -> i32;

    // Extract landmarks from an encoded frame buffer.
    // `frame_ptr` points to raw image bytes (e.g., BGR or RGB), `frame_len` is byte length.
    // `out_ptr` points to a preallocated float buffer and `out_len` its capacity (number of floats).
    // Returns number of floats written on success, negative on error.
    pub fn mp_extract_landmarks(frame_ptr: *const u8, frame_len: usize, out_ptr: *mut f32, out_len: usize) -> i32;
}

pub fn initialize_bridge() -> Result<(), String> {
    #[cfg(feature = "mediapipe")]
    unsafe {
        if mp_initialize() == 0 {
            Ok(())
        } else {
            Err("mp_initialize failed".into())
        }
    }

    #[cfg(not(feature = "mediapipe"))]
    {
        Err("mediapipe feature not enabled; compile with --features mediapipe".into())
    }
}
