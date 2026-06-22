use std::process::Command;
use std::fs::File;
use std::io::Write;

/// Linux-specific mouse dispatch using xdotool or uinput
pub fn send_mouse_move_relative(dx: i32, dy: i32) {
    // Use xdotool if available
    let _ = Command::new("xdotool")
        .args(&["mousemove_relative", "--", &dx.to_string(), &dy.to_string()])
        .output();
}

pub fn send_mouse_move_absolute(x: i32, y: i32) {
    let _ = Command::new("xdotool")
        .args(&["mousemove", "--", &x.to_string(), &y.to_string()])
        .output();
}

pub fn send_mouse_click_left() {
    let _ = Command::new("xdotool")
        .args(&["click", "1"])
        .output();
}

pub fn send_mouse_click_right() {
    let _ = Command::new("xdotool")
        .args(&["click", "3"])
        .output();
}