use base64::Engine;
use std::fs;
use winapi::um::winuser::{FindWindowW, GetWindowThreadProcessId, SendMessageW, WM_GETICON, ICON_BIG, ICON_SMALL, ICON_SMALL2, GCLP_HICON, GetForegroundWindow, FindWindowExW, GetClassLongPtrW};
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ, PROCESS_QUERY_LIMITED_INFORMATION};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::psapi::GetModuleFileNameExW;
use winapi::um::shellapi::ExtractIconExW;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use std::fs::create_dir_all;
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
use winapi::shared::windef::{HICON, HWND};

use crate::icon_utils;
use crate::window_utils;

#[derive(serde::Serialize, Clone)]
pub struct ActiveWindowInfo {
    title: String,
    process_name: String,
    display: String,
}

fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

#[tauri::command]
pub fn get_active_window() -> ActiveWindowInfo {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        return ActiveWindowInfo {
            title: String::new(),
            process_name: String::new(),
            display: String::new(),
        };
    }

    let window_title = window_utils::get_window_title(hwnd);
    let process_name = window_utils::get_process_name(hwnd).unwrap_or_default();
    let display = if process_name.is_empty() {
        window_title.clone()
    } else {
        format!("{} | {}", window_title, process_name)
    };

    ActiveWindowInfo {
        title: window_title,
        process_name,
        display,
    }
}

#[tauri::command]
pub fn get_app_icon(process_name: &str) -> Result<String, String> {
    let temp_dir = std::env::temp_dir();
    let app_images_dir = temp_dir.join("app_images");
    create_dir_all(&app_images_dir).map_err(|e| e.to_string())?;
    let icon_path = app_images_dir.join(format!("{}.png", process_name));

    if icon_path.exists() {
        let image_bytes = fs::read(&icon_path).map_err(|e| e.to_string())?;
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
        return Ok(format!("data:image/png;base64,{}", base64_image));
    }

    let mut hicon: Option<HICON> = None;
    let mut found_hwnd: Option<HWND> = None;

    unsafe {
        let mut current_hwnd: HWND = std::ptr::null_mut();
        loop {
            current_hwnd = FindWindowExW(std::ptr::null_mut(), current_hwnd, std::ptr::null(), std::ptr::null());
            if current_hwnd.is_null() {
                break;
            }

            if let Some(p_name) = window_utils::get_process_name(current_hwnd) {
                if p_name == process_name {
                    found_hwnd = Some(current_hwnd);
                    break;
                }
            }
        }
    }

    if let Some(hwnd) = found_hwnd {
        let mut hicon_val = unsafe { SendMessageW(hwnd, WM_GETICON, ICON_BIG as usize, 0) };
        if hicon_val == 0 {
            hicon_val = unsafe { SendMessageW(hwnd, WM_GETICON, ICON_SMALL as usize, 0) };
        }
        if hicon_val == 0 {
            hicon_val = unsafe { SendMessageW(hwnd, WM_GETICON, ICON_SMALL2 as usize, 0) };
        }
        if hicon_val == 0 {
            hicon_val = unsafe { GetClassLongPtrW(hwnd, GCLP_HICON) as isize };
        }
        if hicon_val != 0 {
            hicon = Some(hicon_val as HICON);
        }
    }

    if let Some(hicon_val) = hicon {
        unsafe {
            icon_utils::hicon_to_png(hicon_val, &icon_path)?;
        }
    } else {
        // Fallback to extracting from the executable if we have a process name
        let mut p_path_buf: [u16; 1024] = [0; 1024];
        let p_path_len = unsafe {
            let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, get_pid_by_name(process_name).unwrap_or(0));
            if !process_handle.is_null() {
                let len = GetModuleFileNameExW(process_handle, std::ptr::null_mut(), p_path_buf.as_mut_ptr(), 1024);
                CloseHandle(process_handle);
                len
            } else {
                0
            }
        };

        if p_path_len > 0 {
            let mut hicon_large: HICON = std::ptr::null_mut();
            let count = unsafe { ExtractIconExW(p_path_buf.as_ptr(), 0, &mut hicon_large, std::ptr::null_mut(), 1) };
            if count > 0 && !hicon_large.is_null() {
                unsafe {
                    icon_utils::hicon_to_png(hicon_large, &icon_path)?;
                    winapi::um::winuser::DestroyIcon(hicon_large);
                }
            }
        }
    }

    if !icon_path.exists() {
        return Err("Icon could not be extracted".into());
    }

    let image_bytes = fs::read(&icon_path).map_err(|e| e.to_string())?;
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_bytes);

    Ok(format!("data:image/png;base64,{}", base64_image))
}

fn get_pid_by_name(process_name: &str) -> Option<u32> {
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..unsafe { std::mem::zeroed() }
    };
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return None;
    }

    if unsafe { Process32FirstW(snapshot, &mut entry) } == 1 {
        loop {
            let current_name = unsafe {
                let pos = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(0);
                String::from_utf16_lossy(&entry.szExeFile[..pos])
            };
            if current_name == process_name {
                unsafe { CloseHandle(snapshot) };
                return Some(entry.th32ProcessID);
            }
            if unsafe { Process32NextW(snapshot, &mut entry) } != 1 {
                break;
            }
        }
    }

    unsafe { CloseHandle(snapshot) };
    None
}
