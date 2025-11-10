use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use winapi::um::winuser::{GetForegroundWindow, GetWindowTextW};

pub fn get_active_window() -> String {
    let hwnd = unsafe { GetForegroundWindow() };
    let mut buffer: [u16; 256] = [0; 256];
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    if len > 0 {
        let os_string = OsString::from_wide(&buffer[..len as usize]);
        os_string.to_string_lossy().into_owned()
    } else {
        String::new()
    }
}