/// Platform-agnostic gesture action definitions.
///
/// The gesture recognition engine emits these actions.
/// Each platform adapter (Windows, Linux, Android, macOS) interprets them
/// according to its capabilities and constraints.
///
/// This design enables: "Train once, run everywhere."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Platform-independent gesture action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GestureAction {
    // Navigation
    SwipeLeft,
    SwipeRight,
    SwipeUp,
    SwipeDown,

    // Hand states
    OpenPalm,
    ClosedFist,
    Point,

    // Fine control
    Pinch,
    Rotate,

    // Extended gestures
    ThreeFingerSwipe,
    TwoFingerRotate,

    // Reserved for future
    Custom(u32),
}

impl fmt::Display for GestureAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SwipeLeft => write!(f, "SwipeLeft"),
            Self::SwipeRight => write!(f, "SwipeRight"),
            Self::SwipeUp => write!(f, "SwipeUp"),
            Self::SwipeDown => write!(f, "SwipeDown"),
            Self::OpenPalm => write!(f, "OpenPalm"),
            Self::ClosedFist => write!(f, "ClosedFist"),
            Self::Point => write!(f, "Point"),
            Self::Pinch => write!(f, "Pinch"),
            Self::Rotate => write!(f, "Rotate"),
            Self::ThreeFingerSwipe => write!(f, "ThreeFingerSwipe"),
            Self::TwoFingerRotate => write!(f, "TwoFingerRotate"),
            Self::Custom(id) => write!(f, "Custom({})", id),
        }
    }
}

/// Context for a gesture action (metadata for platform adapters).
#[derive(Debug, Clone)]
pub struct GestureContext {
    pub action: GestureAction,
    pub confidence: f32,
    pub timestamp: u64,
    /// Hand position for pointer-based actions (normalized 0-1)
    pub hand_x: Option<f32>,
    pub hand_y: Option<f32>,
    /// Rotation or scale factor for applicable gestures
    pub magnitude: Option<f32>,
}

impl GestureContext {
    pub fn new(action: GestureAction, confidence: f32, timestamp: u64) -> Self {
        Self {
            action,
            confidence,
            timestamp,
            hand_x: None,
            hand_y: None,
            magnitude: None,
        }
    }
}

/// Platform-specific action result.
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
}

impl ActionResult {
    pub fn ok(msg: impl Into<String>) -> Self {
        Self { success: true, message: msg.into() }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self { success: false, message: msg.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gesture_action_display() {
        assert_eq!(GestureAction::SwipeLeft.to_string(), "SwipeLeft");
        assert_eq!(GestureAction::OpenPalm.to_string(), "OpenPalm");
    }

    #[test]
    fn gesture_context_creation() {
        let ctx = GestureContext::new(GestureAction::Pinch, 0.95, 1000);
        assert_eq!(ctx.action, GestureAction::Pinch);
        assert_eq!(ctx.confidence, 0.95);
    }
}
