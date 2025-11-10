use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use winapi::um::winuser::{GetWindowTextW, GetWindowThreadProcessId};
use winapi::um::processthreadsapi::{OpenProcess};
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use winapi::um::psapi::GetModuleFileNameExW;
use winapi::um::handleapi::CloseHandle;
use std::path::Path;

pub fn get_process_name(hwnd: winapi::shared::windef::HWND) -> Option<String> {
    let mut process_id: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, &mut process_id) };

    if process_id == 0 {
        return None;
    }

    let process_handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id) };
    if process_handle.is_null() {
        return None;
    }

    let mut exe_path_buf: [u16; 1024] = [0; 1024];
    let exe_path_len = unsafe { GetModuleFileNameExW(process_handle, std::ptr::null_mut(), exe_path_buf.as_mut_ptr(), 1024) };

    unsafe { CloseHandle(process_handle) };

    if exe_path_len > 0 {
        let exe_path = OsString::from_wide(&exe_path_buf[..exe_path_len as usize]);
        Path::new(&exe_path)
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    } else {
        None
    }
}

pub fn get_window_title(hwnd: winapi::shared::windef::HWND) -> String {
    let mut buffer: [u16; 256] = [0; 256];
    let len = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32) };
    if len > 0 {
        let os_string = OsString::from_wide(&buffer[..len as usize]);
        os_string.to_string_lossy().into_owned()
    } else {
        String::new()
    }
}
