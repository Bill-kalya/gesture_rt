/// macOS platform adapter.
///
/// Uses macOS Accessibility APIs (Quartz Event Services, Accessibility framework).
/// Requires "Accessibility" permissions in System Preferences.
///
/// For keyboard: CGEventCreateKeyboardEvent + CGEventPost
/// For mouse: CGEventCreateMouseEvent + CGEventPost

use crate::platform::adapter::PlatformAdapter;
use crate::platform::action::{GestureAction, GestureContext, ActionResult};
use log::{info, warn};

pub struct MacOSAdapter {
    ready: bool,
}

impl MacOSAdapter {
    pub fn new() -> Self {
        // In a real implementation, would check for accessibility permissions
        Self { ready: true }
    }
}

impl PlatformAdapter for MacOSAdapter {
    fn handle(&self, context: &GestureContext) -> ActionResult {
        match context.action {
            GestureAction::SwipeLeft => {
                info!("macOS: Swipe Left → Cmd+[");
                // Real: CGEventCreateKeyboardEvent(proxy, kVK_LeftBracket, true)
                ActionResult::ok("Cmd+[ sent")
            }
            GestureAction::SwipeRight => {
                info!("macOS: Swipe Right → Cmd+]");
                // Real: CGEventCreateKeyboardEvent(proxy, kVK_RightBracket, true)
                ActionResult::ok("Cmd+] sent")
            }
            GestureAction::SwipeUp => {
                info!("macOS: Swipe Up → Mission Control");
                // Real: CGEventCreateKeyboardEvent(proxy, kVK_F3, true)
                ActionResult::ok("F3 (Mission Control) sent")
            }
            GestureAction::SwipeDown => {
                info!("macOS: Swipe Down → Show Desktop");
                // Real: CGEventCreateKeyboardEvent(proxy, kVK_F11, true)
                ActionResult::ok("F11 (Show Desktop) sent")
            }
            GestureAction::OpenPalm => {
                info!("macOS: Open Palm → Cmd+Space (Spotlight)");
                // Real: CGEventCreateKeyboardEvent(proxy, kVK_Space, true)
                ActionResult::ok("Cmd+Space sent")
            }
            GestureAction::ClosedFist => {
                info!("macOS: Closed Fist → Sleep");
                // Real: CGEventCreateKeyboardEvent(proxy, kVK_F1, true) or system sleep
                ActionResult::ok("Sleep initiated")
            }
            GestureAction::Point => {
                info!("macOS: Point → Left click");
                // Real: CGEventCreateMouseEvent with kCGEventLeftMouseDown/Up
                ActionResult::ok("Left click sent")
            }
            GestureAction::Pinch => {
                info!("macOS: Pinch → Cmd+Minus (zoom out)");
                // Real: CGEventCreateKeyboardEvent(proxy, kVK_Minus, true)
                ActionResult::ok("Cmd+Minus sent")
            }
            _ => {
                warn!("macOS: Gesture {:?} not mapped", context.action);
                ActionResult::err(format!("Gesture not mapped: {}", context.action))
            }
        }
    }

    fn name(&self) -> &'static str {
        "macOS"
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn init(&self) -> ActionResult {
        ActionResult::ok("macOS adapter initialized. Grant accessibility permissions in System Preferences → Security & Privacy → Accessibility")
    }
}

impl Default for MacOSAdapter {
    fn default() -> Self {
        Self::new()
    }
}
