use core_graphics::event::{
    CGEvent, CGEventCreateMouseEvent, CGEventPost, 
    kCGEventMouseMoved, kCGEventLeftMouseDown, kCGEventLeftMouseUp,
    kCGHIDEventTap,
};
use core_graphics::geometry::CGPoint;
use core_graphics::event_source::CGEventSource;

/// macOS-specific mouse dispatch using CoreGraphics
pub fn send_mouse_move_absolute(x: f64, y: f64) {
    let point = CGPoint::new(x, y);
    let event = CGEvent::new_mouse_event(
        None,
        kCGEventMouseMoved,
        point,
        0, // mouse button
    );
    event.post(kCGHIDEventTap);
}

pub fn send_mouse_move_relative(dx: i32, dy: i32) {
    // Get current mouse position first
    if let Some(event) = CGEvent::new(None) {
        let location = event.location();
        let new_x = location.x + dx as f64;
        let new_y = location.y + dy as f64;
        send_mouse_move_absolute(new_x, new_y);
    }
}

pub fn send_mouse_click_left() {
    let point = if let Some(event) = CGEvent::new(None) {
        event.location()
    } else {
        CGPoint::new(0.0, 0.0)
    };
    
    let down = CGEvent::new_mouse_event(
        None,
        kCGEventLeftMouseDown,
        point,
        1, // mouse button
    );
    down.post(kCGHIDEventTap);
    
    let up = CGEvent::new_mouse_event(
        None,
        kCGEventLeftMouseUp,
        point,
        1,
    );
    up.post(kCGHIDEventTap);
}