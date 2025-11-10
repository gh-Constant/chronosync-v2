use image::{ImageBuffer, Rgba};
use std::mem;
use std::path::PathBuf;
use std::ptr;
use winapi::shared::windef::HICON;
use winapi::um::wingdi::{
    BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, GetDIBits, GetObjectW,
};
use winapi::um::winuser::{GetDC, GetIconInfo};

pub unsafe fn hicon_to_png(hicon: HICON, path: &PathBuf) -> Result<(), String> {
    println!("Starting hicon_to_png for path: {:?}", path);
    if hicon.is_null() {
        println!("hicon is null");
        return Err("null hicon".into());
    }

    let mut iconinfo: winapi::um::winuser::ICONINFO = mem::zeroed();
    if GetIconInfo(hicon, &mut iconinfo) == 0 {
        println!("GetIconInfo failed");
        return Err("GetIconInfo failed".into());
    }
    println!("GetIconInfo successful: fIcon={}, xHotspot={}, yHotspot={}", iconinfo.fIcon, iconinfo.xHotspot, iconinfo.yHotspot);

    let hbm = iconinfo.hbmColor;
    if hbm.is_null() {
        println!("hbmColor is null");
        return Err("no color bitmap in icon".into());
    }

    let mut bmp: BITMAP = mem::zeroed();
    if GetObjectW(hbm as *mut _, mem::size_of::<BITMAP>() as i32, &mut bmp as *mut _ as *mut _) == 0 {
        println!("GetObjectW failed");
        return Err("GetObjectW failed".into());
    }
    println!("GetObjectW successful: width={}, height={}, width_bytes={}", bmp.bmWidth, bmp.bmHeight, bmp.bmWidthBytes);

    let width = bmp.bmWidth;
    let height = bmp.bmHeight;
    println!("Icon dimensions: {}x{}", width, height);

    let mut bmi: BITMAPINFO = mem::zeroed();
    bmi.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = -height; // Negative height to indicate top-down DIB
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;

    let mut buf: Vec<u8> = vec![0; (width * height * 4) as usize];
    let hdc = GetDC(ptr::null_mut());
    let get_dibits_result = GetDIBits(
        hdc,
        hbm,
        0,
        height as u32,
        buf.as_mut_ptr() as *mut _,
        &mut bmi,
        winapi::um::wingdi::DIB_RGB_COLORS,
    );

    if get_dibits_result == 0 {
        println!("GetDIBits failed");
        return Err("GetDIBits failed".into());
    }
    println!("GetDIBits successful, read {} scanlines", get_dibits_result);

    let mut img_buf: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for chunk in buf.chunks_exact(4) {
        img_buf.push(chunk[2]); // R
        img_buf.push(chunk[1]); // G
        img_buf.push(chunk[0]); // B
        img_buf.push(chunk[3]); // A
    }

    println!("Attempting to save image to {:?}", path);
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(width as u32, height as u32, img_buf)
        .ok_or("Failed to create image buffer")?;

    image.save(path).map_err(|e| {
        println!("Failed to save image: {}", e);
        e.to_string()
    })?;

    println!("Image saved successfully");
    Ok(())
}
