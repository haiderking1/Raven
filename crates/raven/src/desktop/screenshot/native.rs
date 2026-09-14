use smithay::{
    backend::{allocator::Fourcc, renderer::element::memory::MemoryRenderBuffer},
    utils::Transform,
};
use std::ffi::{CStr, CString, c_char, c_void};
unsafe extern "C" {
    fn raven_screenshot_png(
        pixels: *const u8,
        w: i32,
        h: i32,
        data: *mut *mut c_char,
        length: *mut usize,
    ) -> bool;
    fn raven_screenshot_pictures() -> *const c_char;
    fn raven_screenshot_hud(
        pixels: *mut u8,
        w: i32,
        h: i32,
        scale: i32,
        text: *const c_char,
    ) -> bool;
    fn g_free(data: *mut c_void);
}
pub(super) fn encode(pixels: &[u8], w: i32, h: i32) -> Result<Vec<u8>, String> {
    if w <= 0 || h <= 0 || i64::from(w) * i64::from(h) * 4 != pixels.len() as i64 {
        return Err("invalid screenshot pixels".into());
    }
    let mut data = std::ptr::null_mut();
    let mut len = 0;
    let ok = unsafe { raven_screenshot_png(pixels.as_ptr(), w, h, &mut data, &mut len) };
    if !ok || data.is_null() {
        unsafe {
            g_free(data.cast());
        }
        return Err("PNG encoding failed".into());
    }
    let result = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), len).to_vec() };
    unsafe {
        g_free(data.cast());
    }
    Ok(result)
}
pub(super) fn pictures() -> Option<std::path::PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    let p = unsafe { raven_screenshot_pictures() };
    if p.is_null() {
        None
    } else {
        Some(std::path::PathBuf::from(std::ffi::OsStr::from_bytes(
            unsafe { CStr::from_ptr(p) }.to_bytes(),
        )))
    }
}
pub(super) const HUD_SIZE: (i32, i32) = (360, 64);
pub(super) fn hud_pixels(text: &str) -> Option<Vec<u8>> {
    let text = CString::new(text.replace('\0', "�")).ok()?;
    let (w, h) = (HUD_SIZE.0 * 2, HUD_SIZE.1 * 2);
    let mut pixels = vec![0; (w * h * 4) as usize];
    unsafe { raven_screenshot_hud(pixels.as_mut_ptr(), w, h, 2, text.as_ptr()) }.then_some(pixels)
}
pub(super) fn hud_buffer(pixels: &[u8]) -> MemoryRenderBuffer {
    MemoryRenderBuffer::from_slice(
        pixels,
        Fourcc::Argb8888,
        (HUD_SIZE.0 * 2, HUD_SIZE.1 * 2),
        2,
        Transform::Normal,
        None,
    )
}
