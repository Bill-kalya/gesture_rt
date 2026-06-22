/// Windows platform adapter.
///
/// Uses `enigo` for keyboard/mouse control.
/// Keyboard: Alt+Tab, global shortcuts
/// Mouse: movement, clicks

use crate::platform::adapter::PlatformAdapter;
use crate::platform::action::{GestureAction, GestureContext, ActionResult};
use log::{info, warn};

pub struct WindowsAdapter {
    ready: bool,
}

impl WindowsAdapter {
    pub fn new() -> Self {
        Self { ready: true }
    }
}

impl PlatformAdapter for WindowsAdapter {
    fn handle(&self, context: &GestureContext) -> ActionResult {
        use enigo::{Enigo, Key, KeyboardControllable, MouseControllable};

        let mut enigo = match Enigo::new() {
            Ok(e) => e,
            Err(e) => return ActionResult::err(format!("Failed to initialize Enigo: {}", e)),
        };

        match context.action {
            GestureAction::SwipeLeft => {
                info!("Windows: Swipe Left → Alt+Tab");
                let _ = enigo.key_down(Key::Alt);
                let _ = enigo.key_click(Key::Tab);
                let _ = enigo.key_up(Key::Alt);
                ActionResult::ok("Alt+Tab sent")
            }
            GestureAction::SwipeRight => {
                info!("Windows: Swipe Right → Shift+Alt+Tab");
                let _ = enigo.key_down(Key::Alt);
                let _ = enigo.key_down(Key::Shift);
                let _ = enigo.key_click(Key::Tab);
                let _ = enigo.key_up(Key::Shift);
                let _ = enigo.key_up(Key::Alt);
                ActionResult::ok("Shift+Alt+Tab sent")
            }
            GestureAction::SwipeUp => {
                info!("Windows: Swipe Up → Win+Up");
                let _ = enigo.key_down(Key::Meta);
                let _ = enigo.key_click(Key::UpArrow);
                let _ = enigo.key_up(Key::Meta);
                ActionResult::ok("Win+Up sent")
            }
            GestureAction::SwipeDown => {
                info!("Windows: Swipe Down → Win+Down");
                let _ = enigo.key_down(Key::Meta);
                let _ = enigo.key_click(Key::DownArrow);
                let _ = enigo.key_up(Key::Meta);
                ActionResult::ok("Win+Down sent")
            }
            GestureAction::OpenPalm => {
                info!("Windows: Open Palm → Win+D (desktop)");
                let _ = enigo.key_down(Key::Meta);
                let _ = enigo.key_click(Key::KeyD);
                let _ = enigo.key_up(Key::Meta);
                ActionResult::ok("Win+D sent")
            }
            GestureAction::ClosedFist => {
                info!("Windows: Closed Fist → Win+Esc (lock)");
                let _ = enigo.key_down(Key::Meta);
                let _ = enigo.key_click(Key::Escape);
                let _ = enigo.key_up(Key::Meta);
                ActionResult::ok("Win+Esc sent")
            }
            GestureAction::Point => {
                info!("Windows: Point → Left click");
                let _ = enigo.mouse_click(enigo::MouseButton::Left);
                ActionResult::ok("Left click sent")
            }
            GestureAction::Pinch => {
                info!("Windows: Pinch → Ctrl+Minus (zoom out)");
                let _ = enigo.key_down(Key::Control);
                let _ = enigo.key_click(Key::Minus);
                let _ = enigo.key_up(Key::Control);
                ActionResult::ok("Ctrl+Minus sent")
            }
            _ => {
                warn!("Windows: Gesture {:?} not mapped", context.action);
                ActionResult::err(format!("Gesture not mapped: {}", context.action))
            }
        }
    }

    fn name(&self) -> &'static str {
        "Windows"
    }

    fn is_ready(&self) -> bool {
        self.ready
    }
}

impl Default for WindowsAdapter {
    fn default() -> Self {
        Self::new()
    }
}
