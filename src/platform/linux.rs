/// Linux platform adapter.
///
/// Uses xdotool or X11/Wayland APIs for keyboard/mouse control.
/// For simplicity, this implementation assumes xdotool is available.

use crate::platform::adapter::PlatformAdapter;
use crate::platform::action::{GestureAction, GestureContext, ActionResult};
use log::{info, warn};
use std::process::Command;

pub struct LinuxAdapter {
    ready: bool,
}

impl LinuxAdapter {
    pub fn new() -> Self {
        // Check if xdotool is available
        let xdotool_available = Command::new("which")
            .arg("xdotool")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        Self {
            ready: xdotool_available,
        }
    }

    fn run_xdotool(&self, args: &[&str]) -> ActionResult {
        match Command::new("xdotool").args(args).output() {
            Ok(output) => {
                if output.status.success() {
                    ActionResult::ok("Command executed")
                } else {
                    ActionResult::err("xdotool failed")
                }
            }
            Err(e) => ActionResult::err(format!("Failed to run xdotool: {}", e)),
        }
    }
}

impl PlatformAdapter for LinuxAdapter {
    fn handle(&self, context: &GestureContext) -> ActionResult {
        match context.action {
            GestureAction::SwipeLeft => {
                info!("Linux: Swipe Left → Alt+Tab");
                self.run_xdotool(&["key", "alt+Tab"])
            }
            GestureAction::SwipeRight => {
                info!("Linux: Swipe Right → Shift+Alt+Tab");
                self.run_xdotool(&["key", "shift+alt+Tab"])
            }
            GestureAction::SwipeUp => {
                info!("Linux: Swipe Up → Super+Up");
                self.run_xdotool(&["key", "super+Up"])
            }
            GestureAction::SwipeDown => {
                info!("Linux: Swipe Down → Super+Down");
                self.run_xdotool(&["key", "super+Down"])
            }
            GestureAction::OpenPalm => {
                info!("Linux: Open Palm → Super+D (desktop)");
                self.run_xdotool(&["key", "super+d"])
            }
            GestureAction::ClosedFist => {
                info!("Linux: Closed Fist → Super+L (lock)");
                self.run_xdotool(&["key", "super+l"])
            }
            GestureAction::Point => {
                info!("Linux: Point → Left click");
                self.run_xdotool(&["click", "1"])
            }
            GestureAction::Pinch => {
                info!("Linux: Pinch → Ctrl+Minus (zoom out)");
                self.run_xdotool(&["key", "ctrl+minus"])
            }
            _ => {
                warn!("Linux: Gesture {:?} not mapped", context.action);
                ActionResult::err(format!("Gesture not mapped: {}", context.action))
            }
        }
    }

    fn name(&self) -> &'static str {
        "Linux"
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn init(&self) -> ActionResult {
        if self.ready {
            ActionResult::ok("Linux adapter initialized (xdotool available)")
        } else {
            ActionResult::err("xdotool not found. Install with: sudo apt install xdotool")
        }
    }
}

impl Default for LinuxAdapter {
    fn default() -> Self {
        Self::new()
    }
}
