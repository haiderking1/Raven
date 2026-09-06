use smithay::{
    backend::{allocator::Fourcc, renderer::element::memory::MemoryRenderBuffer},
    utils::Transform,
};

const WIDTH: usize = 24;
const HEIGHT: usize = 32;

fn arrow_pixel(x: i32, y: i32) -> bool {
    // Arrowhead, followed by a slanted stem. Coordinates include a border margin.
    let (x, y) = (x - 2, y - 2);
    (0..=21).contains(&y) && x >= 0 && x <= y / 2 && (y <= 15 || x <= 5)
        || (15..=27).contains(&y) && (x - (y - 15) / 2 >= 4) && (x - (y - 15) / 2 <= 8)
}

pub(super) fn default_arrow() -> MemoryRenderBuffer {
    let mut pixels = vec![0_u8; WIDTH * HEIGHT * 4];
    for y in 0..HEIGHT as i32 {
        for x in 0..WIDTH as i32 {
            if !arrow_pixel(x, y) {
                continue;
            }
            let edge = [(-1, 0), (1, 0), (0, -1), (0, 1)]
                .iter()
                .any(|(dx, dy)| !arrow_pixel(x + dx, y + dy));
            let shade = if edge { 24 } else { 245 };
            let offset = (y as usize * WIDTH + x as usize) * 4;
            // ARGB8888 native little-endian bytes are BGRA. The RGB channels
            // are equal, so this opaque grayscale cursor has no channel ambiguity.
            pixels[offset..offset + 4].copy_from_slice(&[shade, shade, shade, 255]);
        }
    }
    MemoryRenderBuffer::from_slice(
        &pixels,
        Fourcc::Argb8888,
        (WIDTH as i32, HEIGHT as i32),
        1,
        Transform::Normal,
        None,
    )
}
