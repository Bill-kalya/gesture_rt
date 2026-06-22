/// Platform adapter trait.
///
/// Each platform implementation translates GestureAction into platform-specific operations.
/// This trait ensures consistent behavior across Windows, Linux, Android, macOS.

use crate::platform::action::{GestureAction, GestureContext, ActionResult};
use std::sync::Arc;

/// Trait for platform-specific gesture action handlers.
pub trait PlatformAdapter: Send + Sync {
    /// Handle a gesture action on the current platform.
    fn handle(&self, context: &GestureContext) -> ActionResult;

    /// Platform name (for logging).
    fn name(&self) -> &'static str;

    /// Check if this adapter is initialized and ready.
    fn is_ready(&self) -> bool;

    /// Optional: initialize platform-specific resources.
    fn init(&self) -> ActionResult {
        ActionResult::ok("Initialized")
    }

    /// Optional: cleanup platform-specific resources.
    fn shutdown(&self) -> ActionResult {
        ActionResult::ok("Shutdown complete")
    }
}

/// Convenience function for dispatching to platform adapter.
pub fn dispatch(adapter: &Arc<dyn PlatformAdapter>, context: &GestureContext) -> ActionResult {
    if !adapter.is_ready() {
        return ActionResult::err("Platform adapter not ready");
    }
    adapter.handle(context)
}
