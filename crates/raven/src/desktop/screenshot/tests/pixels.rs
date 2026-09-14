use super::super::model::Selection;
use smithay::utils::Rectangle;
#[test]
fn selection_is_pixel_exact_reversible_and_clamped_to_the_output() {
    let mut s = Selection::new((200, 100).into()).unwrap();
    assert_eq!(s.rect, Rectangle::from_size((200, 100).into()));
    s.begin((190.0, 90.0).into());
    s.end((-8.0, -4.0).into());
    assert_eq!(s.rect, Rectangle::new((0, 0).into(), (191, 91).into()));
    s.begin((0.0, 0.0).into());
    s.end((500.0, 500.0).into());
    assert_eq!(s.rect, Rectangle::from_size((200, 100).into()));
    s.begin((20.0, 10.0).into());
    s.end((20.0, 10.0).into());
    assert_eq!(s.rect.size, (1, 1).into());
    assert!(Selection::new((0, 1).into()).is_none());
    assert!(Selection::new((65536, 65536).into()).is_none());
}
#[test]
fn png_encoding_preserves_rgba_and_saved_files_are_private_and_unique() {
    use std::{ffi::c_void, os::unix::fs::PermissionsExt};
    unsafe extern "C" {
        fn gdk_pixbuf_loader_new() -> *mut c_void;
        fn gdk_pixbuf_loader_write(
            loader: *mut c_void,
            bytes: *const u8,
            len: usize,
            error: *mut *mut c_void,
        ) -> i32;
        fn gdk_pixbuf_loader_close(loader: *mut c_void, error: *mut *mut c_void) -> i32;
        fn gdk_pixbuf_loader_get_pixbuf(loader: *mut c_void) -> *mut c_void;
        fn gdk_pixbuf_get_pixels(pixbuf: *mut c_void) -> *mut u8;
        fn gdk_pixbuf_get_width(pixbuf: *mut c_void) -> i32;
        fn gdk_pixbuf_get_height(pixbuf: *mut c_void) -> i32;
        fn gdk_pixbuf_get_rowstride(pixbuf: *mut c_void) -> i32;
        fn g_object_unref(object: *mut c_void);
    }
    let pixels = [
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 30, 40, 50, 128,
    ];
    let png = super::super::native::encode(&pixels, 2, 2).unwrap();
    unsafe {
        let loader = gdk_pixbuf_loader_new();
        assert_ne!(
            gdk_pixbuf_loader_write(loader, png.as_ptr(), png.len(), std::ptr::null_mut()),
            0
        );
        assert_ne!(gdk_pixbuf_loader_close(loader, std::ptr::null_mut()), 0);
        let image = gdk_pixbuf_loader_get_pixbuf(loader);
        assert_eq!(gdk_pixbuf_get_width(image), 2);
        assert_eq!(gdk_pixbuf_get_height(image), 2);
        let data = gdk_pixbuf_get_pixels(image);
        let stride = gdk_pixbuf_get_rowstride(image);
        for row in 0..2 {
            assert_eq!(
                std::slice::from_raw_parts(data.add(row * stride as usize), 8),
                &pixels[row * 8..row * 8 + 8]
            );
        }
        g_object_unref(loader);
    }
    let dir = std::env::temp_dir().join(format!(
        "raven-screenshot-files-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let a = super::super::files::save(&dir, &png).unwrap();
    let b = super::super::files::save(&dir, &png).unwrap();
    assert_ne!(a, b);
    assert_eq!(std::fs::read(&a).unwrap(), png);
    assert_eq!(
        std::fs::metadata(&a).unwrap().permissions().mode() & 0o077,
        0
    );
    assert!(super::super::files::save(&a, &png).is_err());
    assert_eq!(
        std::fs::read_dir(&dir).unwrap().count(),
        2,
        "temporary images must be removed after publication"
    );
    std::fs::remove_dir_all(dir).unwrap();
}
