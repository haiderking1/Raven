use super::{Request, raster::Raster};
use crate::desktop::switcher::layout::Layout;
use smithay::utils::Rectangle;
#[test]
fn artwork_has_transparent_corners_and_a_visible_selected_tile() {
    let apps: Vec<_> = [
        ("com.mitchellh.ghostty", "Ghostty"),
        ("firefox", "Firefox"),
        ("com.t3tools.t3code", "T3 Code"),
        ("cursor", "Cursor"),
        ("brave-browser", "Brave"),
        ("chromium", "Chromium"),
        ("gimp", "GIMP"),
        ("foot", "Foot"),
    ]
    .into_iter()
    .map(|(id, title)| (id.to_owned(), title.to_owned()))
    .collect();
    let layout = Layout::new(Rectangle::from_size((1400, 800).into()), apps.len(), 1).unwrap();
    let request = Request {
        generation: 1,
        layout,
        scale: 1,
        selected: 1,
        apps,
    };
    let mut raster = Raster::new();
    let pixels = raster.paint(&request).unwrap();
    let size = request.layout.rect.size;
    assert_eq!(pixels.len(), (size.w * size.h * 4) as usize);
    let alpha = |x: i32, y: i32| pixels[((y * size.w + x) * 4 + 3) as usize];
    assert_eq!(alpha(0, 0), 0);
    for y in 16..size.h - 16 {
        assert_eq!(alpha(size.w / 2, y), 255, "panel interior must be opaque");
    }
    let brightness = |x: i32, y: i32| pixels[((y * size.w + x) * 4) as usize];
    let unselected = brightness(24 + request.layout.slot / 2, 30);
    let selected = brightness(24 + request.layout.slot + request.layout.slot / 2, 30);
    assert!(
        selected > unselected,
        "selected tile must be distinct from panel"
    );
    if let Some(path) = std::env::var_os("RAVEN_SWITCHER_ARTWORK_PREVIEW") {
        std::fs::write(&path, pixels).unwrap();
        std::fs::write(
            std::path::PathBuf::from(path).with_extension("size"),
            format!("{} {}", size.w, size.h),
        )
        .unwrap();
    }
}
