use winapi::um::winuser::{mouse_event, MOUSEEVENTF_MOVE, MOUSEEVENTF_ABSOLUTE};
use winapi::um::winuser::{SendInput, INPUT, INPUT_MOUSE, MOUSEINPUT};
use std::mem;

/// Windows-specific mouse dispatch using SendInput
pub fn send_mouse_move_relative(dx: i32, dy: i32) {
    unsafe {
        let mut input = INPUT {
            r#type: INPUT_MOUSE,
            u: std::mem::zeroed(),
        };
        
        *input.u.mi_mut() = MOUSEINPUT {
            dx: dx,
            dy: dy,
            mouseData: 0,
            dwFlags: MOUSEEVENTF_MOVE,
            time: 0,
            dwExtraInfo: 0,
        };
        
        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
    }
}

pub fn send_mouse_move_absolute(x: i32, y: i32) {
    unsafe {
        let mut input = INPUT {
            r#type: INPUT_MOUSE,
            u: std::mem::zeroed(),
        };
        
        *input.u.mi_mut() = MOUSEINPUT {
            dx: x,
            dy: y,
            mouseData: 0,
            dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
            time: 0,
            dwExtraInfo: 0,
        };
        
        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
    }
}

pub fn send_mouse_click_left() {
    unsafe {
        let mut down = INPUT {
            r#type: INPUT_MOUSE
            u: std::mem::zeroed(),
        };
        *down.u.mi_mut() = MOUSEINPUT {
            dx: 0,
            dy: 0,
            mouseData: 0,
            dwFlags: 0x0002, // MOUSEEVENTF_LEFTDOWN
            time: 0,
            dwExtraInfo: 0,
        };
        SendInput(1, &mut down, mem::size_of::<INPUT>() as i32);
        
        let mut up = INPUT {
            r#type: INPUT_MOUSE,
            u: std::mem::zeroed(),
        };
        *up.u.mi_mut() = MOUSEINPUT {
            dx: 0,
            dy: 0,
            mouseData: 0,
            dwFlags: 0x0004, // MOUSEEVENTF_LEFTUP
            time: 0,
            dwExtraInfo: 0,
        };
        SendInput(1, &mut up, mem::size_of::<INPUT>() as i32);
    }
}