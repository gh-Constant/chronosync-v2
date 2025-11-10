// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod windows_tracker;

use std::fs;
use std::path::PathBuf;
use std::{mem, ptr};
use base64::Engine;
use image::{DynamicImage, ImageBuffer, Rgba};

use winapi::shared::minwindef::{DWORD, LPARAM, WPARAM};
use winapi::shared::windef::{HDC, HICON, HWND};
use winapi::um::winuser::{
    FindWindowW, GetClassLongPtrW, GetDC, GetIconInfo, SendMessageW,
    DestroyIcon, WM_GETICON, ICON_BIG, ICON_SMALL, ICON_SMALL2, GCLP_HICON, GetWindowThreadProcessId,
};
use winapi::um::winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::psapi::GetModuleFileNameExW;
use winapi::um::shellapi::ExtractIconExW;
use winapi::um::wingdi::{
    BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, GetDIBits, GetObjectW,
};
use winapi::um::handleapi::CloseHandle;

#[tauri::command]
fn get_active_window() -> String {
    windows_tracker::get_active_window()
}

fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

unsafe fn hicon_to_png(hicon: HICON, path: &PathBuf) -> Result<(), String> {
    if hicon.is_null() {
        return Err("null hicon".into());
    }

    // Get ICONINFO
    let mut iconinfo: winapi::um::winuser::ICONINFO = mem::zeroed();
    if GetIconInfo(hicon, &mut iconinfo) == 0 {
        return Err("GetIconInfo failed".into());
    }

    let hbm = iconinfo.hbmColor;
    if hbm.is_null() {
        return Err("no color bitmap in icon".into());
    }

    // Get BITMAP info
    let mut bmp: BITMAP = mem::zeroed();
    if GetObjectW(hbm as *mut _, mem::size_of::<BITMAP>() as i32, &mut bmp as *mut _ as *mut _) == 0 {
        return Err("GetObjectW failed".into());
    }

    let width = bmp.bmWidth as i32;
    let height = bmp.bmHeight as i32;

    // Prepare BITMAPINFO
    let mut bmi: BITMAPINFO = mem::zeroed();
    bmi.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = height;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32; // we request 32-bit
    bmi.bmiHeader.biCompression = BI_RGB as u32;

    // allocate buffer
    let row_bytes = (width * 4) as usize;
    let buf_size = (row_bytes * height as usize) as usize;
    let mut buf: Vec<u8> = vec![0u8; buf_size];

    // get DC
    let hdc: HDC = GetDC(ptr::null_mut());
    if hdc.is_null() {
        return Err("GetDC failed".into());
    }

    // GetDIBits (note: will return bottom-up DIB)
    let scanlines = GetDIBits(
        hdc,
        hbm,
        0,
        height as u32,
        buf.as_mut_ptr() as *mut _,
        &mut bmi,
        winapi::um::wingdi::DIB_RGB_COLORS,
    );

    if scanlines == 0 {
        return Err("GetDIBits failed".into());
    }

    // Convert BGRA bottom-up to RGBA top-down for image crate
    let mut img_buf: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for row in 0..height {
        let src_row = (height - 1 - row) as usize;
        let offset = src_row * row_bytes as usize;
        for col in 0..width as usize {
            let i = offset + col * 4;
            // BGRA -> RGBA
            let b = buf[i];
            let g = buf[i + 1];
            let r = buf[i + 2];
            let a = buf[i + 3];
            img_buf.push(r);
            img_buf.push(g);
            img_buf.push(b);
            img_buf.push(a);
        }
    }

    // create image::RgbaImage
    if let Some(image) = ImageBuffer::<Rgba<u8>, _>::from_raw(width as u32, height as u32, img_buf) {
        let dyn_img = DynamicImage::ImageRgba8(image);
        dyn_img.save(path).map_err(|e| format!("failed to save image: {}", e))?;
    } else {
        return Err("failed to create image buffer".into());
    }

    // cleanup
    if iconinfo.hbmColor != ptr::null_mut() {
        winapi::um::wingdi::DeleteObject(iconinfo.hbmColor as *mut _);
    }
    if iconinfo.hbmMask != ptr::null_mut() {
        winapi::um::wingdi::DeleteObject(iconinfo.hbmMask as *mut _);
    }
    DestroyIcon(hicon);

    Ok(())
}

fn find_hwnd_by_title(title: &str) -> Option<HWND> {
    unsafe {
        let wide = to_wide(title);
        let hwnd = FindWindowW(ptr::null(), wide.as_ptr());
        if !hwnd.is_null() {
            return Some(hwnd);
        }
    }
    None
}

fn get_exe_path_from_hwnd(hwnd: HWND) -> Option<String> {
    unsafe {
        let mut pid: DWORD = 0;
        GetWindowThreadProcessId(hwnd, &mut pid as *mut _ as *mut DWORD);
        if pid == 0 {
            return None;
        }
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut buf: [u16; 1024] = [0; 1024];
        let len = GetModuleFileNameExW(handle, ptr::null_mut(), buf.as_mut_ptr(), buf.len() as DWORD);
        CloseHandle(handle);
        if len == 0 {
            return None;
        }
        let os = String::from_utf16_lossy(&buf[..len as usize]);
        Some(os)
    }
}

#[tauri::command]
fn get_app_image(title: String) -> Result<String, String> {
    // Create path: %LOCALAPPDATA%\chronosync\app_images
    let local = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
    let mut dir = PathBuf::from(local);
    dir.push("chronosync");
    dir.push("app_images");

    if let Err(e) = fs::create_dir_all(&dir) {
        return Err(format!("failed to create dir: {}", e));
    }

    // Use md5 of the title as filename
    let hash = format!("{:x}", md5::compute(title.as_bytes()));
    let mut path = dir.clone();
    path.push(format!("{}.png", hash));

    // If file exists, return it
    if path.exists() {
        match fs::read(&path) {
            Ok(bytes) => {
                let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                let data_url = format!("data:image/png;base64,{}", encoded);
                return Ok(data_url);
            }
            Err(e) => return Err(format!("failed to read existing image: {}", e)),
        }
    }

    unsafe {
        // try to find HWND by title
        let mut hicon: HICON = ptr::null_mut();
        if let Some(hwnd) = find_hwnd_by_title(&title) {
            // try WM_GETICON
            let icons = [ICON_BIG as WPARAM, ICON_SMALL as WPARAM, ICON_SMALL2 as WPARAM];
            for &cmd in icons.iter() {
                let res = SendMessageW(hwnd, WM_GETICON, cmd as WPARAM, 0 as LPARAM) as isize;
                if res != 0 {
                    hicon = res as HICON;
                    break;
                }
            }
            // fallback to class icon
            if hicon.is_null() {
                let cls = GetClassLongPtrW(hwnd, GCLP_HICON);
                if cls != 0 {
                    hicon = cls as HICON;
                }
            }

            // if we have an icon, convert and save
            if !hicon.is_null() {
                if let Err(e) = hicon_to_png(hicon, &path) {
                    // continue to try extracting from exe
                    eprintln!("hicon->png failed: {}", e);
                } else {
                    // read and return
                    match fs::read(&path) {
                        Ok(bytes) => {
                            let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                            let data_url = format!("data:image/png;base64,{}", encoded);
                            return Ok(data_url);
                        }
                        Err(e) => return Err(format!("failed to read saved image: {}", e)),
                    }
                }
            }

            // try to get exe path and ExtractIconEx
            if let Some(exe) = get_exe_path_from_hwnd(hwnd) {
                let wide = to_wide(&exe);
                let mut large: [HICON; 1] = [ptr::null_mut()];
                let got = ExtractIconExW(wide.as_ptr(), 0, large.as_mut_ptr(), ptr::null_mut(), 1);
                if got > 0 && !large[0].is_null() {
                    if let Err(e) = hicon_to_png(large[0], &path) {
                        eprintln!("extract icon failed: {}", e);
                    } else {
                        match fs::read(&path) {
                            Ok(bytes) => {
                                let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                                let data_url = format!("data:image/png;base64,{}", encoded);
                                return Ok(data_url);
                            }
                            Err(e) => return Err(format!("failed to read saved image: {}", e)),
                        }
                    }
                }
            }
        }
    }

    // Last resort: create a tiny default placeholder (no border square like before)
    let imgx = 64;
    let imgy = 64;
    let mut imgbuf = image::RgbaImage::new(imgx, imgy);
    for (_x, _y, pixel) in imgbuf.enumerate_pixels_mut() {
        *pixel = image::Rgba([200, 200, 200, 255]);
    }
    if let Err(e) = imgbuf.save(&path) {
        return Err(format!("failed to save fallback image: {}", e));
    }

    match fs::read(&path) {
        Ok(bytes) => {
            let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
            let data_url = format!("data:image/png;base64,{}", encoded);
            Ok(data_url)
        }
        Err(e) => Err(format!("failed to read saved image: {}", e)),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_active_window, get_app_image])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
