/// Platform abstraction layer.
///
/// Each platform provides an adapter that translates GestureAction → platform-specific commands.

pub mod action;
pub mod adapter;
pub mod windows;
pub mod linux;
pub mod android;
pub mod macos;

pub use action::{GestureAction, GestureContext, ActionResult};
pub use adapter::PlatformAdapter;

use std::sync::Arc;

/// Get the platform adapter for the current OS.
pub fn get_adapter() -> Arc<dyn PlatformAdapter> {
    #[cfg(target_os = "windows")]
    return Arc::new(windows::WindowsAdapter::new());

    #[cfg(target_os = "linux")]
    return Arc::new(linux::LinuxAdapter::new());

    #[cfg(target_os = "android")]
    return Arc::new(android::AndroidAdapter::new());

    #[cfg(target_os = "macos")]
    return Arc::new(macos::MacOSAdapter::new());

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "android", target_os = "macos")))]
    compile_error!("Unsupported platform");
}
