use base64::Engine;
use std::fs::{self, create_dir_all};
use winapi::um::winuser::{GetWindowThreadProcessId, SendMessageW, WM_GETICON, ICON_BIG, ICON_SMALL, ICON_SMALL2, GCLP_HICON, GetForegroundWindow, GetClassLongPtrW, EnumWindows, GetWindowLongW, GWL_EXSTYLE, WS_EX_APPWINDOW, IsWindowVisible};
use winapi::um::winnt::PROCESS_QUERY_LIMITED_INFORMATION;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::psapi::GetModuleFileNameExW;
use winapi::um::shellapi::ExtractIconExW;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::tlhelp32::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
use winapi::shared::windef::{HICON, HWND};
use winapi::shared::minwindef::LPARAM;

use crate::icon_utils;
use crate::window_utils;

#[derive(serde::Serialize, Clone)]
pub struct ActiveWindowInfo {
    title: String,
    process_name: String,
    display: String,
}

struct EnumData {
    process_name: String,
    hwnd: Option<HWND>,
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
    println!("Attempting to get icon for process: {}", process_name);
    let temp_dir = std::env::temp_dir();
    let app_images_dir = temp_dir.join("app_images");
    create_dir_all(&app_images_dir).map_err(|e| e.to_string())?;
    let icon_path = app_images_dir.join(format!("{}.png", process_name));

    if icon_path.exists() {
        println!("Icon for {} already exists, returning cached version.", process_name);
        let image_bytes = fs::read(&icon_path).map_err(|e| e.to_string())?;
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_bytes);
        return Ok(format!("data:image/png;base64,{}", base64_image));
    }

    let mut hicon: Option<HICON> = None;
    let mut found_hwnd: Option<HWND> = None;

    println!("Searching for a window for process: {}", process_name);

    let mut data = EnumData {
        process_name: process_name.to_string(),
        hwnd: None,
    };

    unsafe {
        EnumWindows(Some(enum_windows_proc), &mut data as *mut _ as LPARAM);
    }

    found_hwnd = data.hwnd;

    if let Some(hwnd) = found_hwnd {
        println!("Found window for {}: {:?}", process_name, hwnd);
        println!("Attempting to get icon from window handle for {}", process_name);
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
            println!("Successfully retrieved icon from window handle for {}", process_name);
            hicon = Some(hicon_val as HICON);
        } else {
            println!("Failed to retrieve icon from window handle for {}", process_name);
        }
    } else {
        println!("No window found for process: {}", process_name);
    }

    if let Some(hicon_val) = hicon {
        println!("Converting HICON to PNG for {}", process_name);
        unsafe {
            icon_utils::hicon_to_png(hicon_val, &icon_path)?;
        }
    } else {
        println!("No icon from window handle, falling back to executable for {}", process_name);
        // Fallback to extracting from the executable if we have a process name
        let mut p_path_buf: [u16; 1024] = [0; 1024];
        let p_path_len = unsafe {
            let pid = get_pid_by_name(process_name).unwrap_or(0);
            println!("PID for {}: {}", process_name, pid);
            let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if !process_handle.is_null() {
                println!("Successfully opened process handle for {}", process_name);
                let len = GetModuleFileNameExW(process_handle, std::ptr::null_mut(), p_path_buf.as_mut_ptr(), 1024);
                CloseHandle(process_handle);
                len
            } else {
                println!("Failed to open process handle for {}", process_name);
                0
            }
        };

        if p_path_len > 0 {
            println!("Executable path length for {}: {}", process_name, p_path_len);
            let mut hicon_large: HICON = std::ptr::null_mut();
            let count = unsafe { ExtractIconExW(p_path_buf.as_ptr(), 0, &mut hicon_large, std::ptr::null_mut(), 1) };
            println!("Extracted {} icons from executable for {}", count, process_name);
            if count > 0 && !hicon_large.is_null() {
                println!("Successfully extracted icon from executable for {}", process_name);
                unsafe {
                    icon_utils::hicon_to_png(hicon_large, &icon_path)?;
                    winapi::um::winuser::DestroyIcon(hicon_large);
                }
            } else {
                println!("Failed to extract icon from executable for {}", process_name);
            }
        } else {
            println!("Could not get executable path for {}", process_name);
        }
    }

    if !icon_path.exists() {
        println!("Icon file was not created for {}", process_name);
        return Err("Icon could not be extracted".into());
    }

    println!("Successfully created icon for {}", process_name);
    let image_bytes = fs::read(&icon_path).map_err(|e| e.to_string())?;
    let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_bytes);

    Ok(format!("data:image/png;base64,{}", base64_image))
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
    let data = &mut *(lparam as *mut EnumData);

    let mut process_id: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut process_id);

    if process_id != 0 {
        if let Some(p_name) = window_utils::get_process_name(hwnd) {
            if p_name == data.process_name {
                if IsWindowVisible(hwnd) != 0 {
                    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                    if (ex_style as u32 & WS_EX_APPWINDOW) != 0 || GetWindowLongW(hwnd, winapi::um::winuser::GWL_HWNDPARENT) == 0 {
                        data.hwnd = Some(hwnd);
                        return 0; // Stop enumeration
                    }
                }
            }
        }
    }
    1 // Continue enumeration
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
            let current_name = {
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
