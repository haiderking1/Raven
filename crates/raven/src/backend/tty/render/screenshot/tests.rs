mod preview;
use super::{capture, download::Download, view::View};
use crate::backend::tty::render::SceneElement;
use smithay::{
    backend::{
        allocator::Fourcc,
        egl::{EGLContext, EGLDisplay, native::EGLSurfacelessDisplay},
        renderer::{
            Bind, ExportMem, Frame, Offscreen, Renderer,
            element::{
                Element, Id, Kind, RenderElement,
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            },
            gles::{GlesRenderbuffer, GlesRenderer},
        },
    },
    reexports::calloop::EventLoop,
    utils::{Rectangle, Transform},
};
#[test]
#[ignore = "requires headless EGL"]
fn screenshot_freezes_the_scene_excludes_cursor_and_exports_upright_pixels_on_every_transform() {
    let display = unsafe { EGLDisplay::new(EGLSurfacelessDisplay) }.unwrap();
    let context = EGLContext::new(&display).unwrap();
    let mut renderer = unsafe { GlesRenderer::new(context) }.unwrap();
    let event_loop: EventLoop<()> = EventLoop::try_new().unwrap();
    for (transform, scale) in [
        Transform::Normal,
        Transform::_90,
        Transform::_180,
        Transform::_270,
        Transform::Flipped,
        Transform::Flipped90,
        Transform::Flipped180,
        Transform::Flipped270,
    ]
    .into_iter()
    .map(|t| (t, 1.0))
    .chain([(Transform::Normal, 1.5), (Transform::_90, 2.0)])
    {
        let physical = (192, 96).into();
        let size = transform.transform_size(physical);
        let pixels: Vec<u8> = (0..size.h)
            .flat_map(|y| (0..size.w).flat_map(move |x| [x as u8, y as u8, 80, 255]))
            .collect();
        let buffer = MemoryRenderBuffer::from_slice(
            &pixels,
            Fourcc::Abgr8888,
            (size.w, size.h),
            1,
            Transform::Normal,
            None,
        );
        let background = MemoryRenderBufferRenderElement::from_buffer(
            &mut renderer,
            (0.0, 0.0),
            &buffer,
            None,
            Some(Rectangle::from_size((size.w as f64, size.h as f64).into())),
            Some(
                (
                    (size.w as f64 / scale).round() as i32,
                    (size.h as f64 / scale).round() as i32,
                )
                    .into(),
            ),
            Kind::Unspecified,
        )
        .unwrap();
        let marker_pixels = [255, 0, 0, 255].repeat(12 * 12);
        let marker_buffer = MemoryRenderBuffer::from_slice(
            &marker_pixels,
            Fourcc::Abgr8888,
            (12, 12),
            1,
            Transform::Normal,
            None,
        );
        let marker = MemoryRenderBufferRenderElement::from_buffer(
            &mut renderer,
            (20.0, 20.0),
            &marker_buffer,
            None,
            Some(Rectangle::from_size((12.0, 12.0).into())),
            Some(((12.0 / scale) as i32, (12.0 / scale) as i32).into()),
            Kind::Unspecified,
        )
        .unwrap();
        let cursor = MemoryRenderBufferRenderElement::from_buffer(
            &mut renderer,
            (10.0, 10.0),
            &marker_buffer,
            None,
            None,
            None,
            Kind::Cursor,
        )
        .unwrap();
        let elements = vec![
            SceneElement::Memory(cursor),
            SceneElement::Memory(marker),
            SceneElement::Memory(background),
        ];
        let mut frozen =
            capture::capture(&mut renderer, size, scale, elements, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let crop = Rectangle::new((10, 10).into(), (25, 25).into());
        let download = Download::start(
            &mut renderer,
            &mut frozen.image,
            crop,
            1,
            event_loop.get_signal(),
        )
        .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !download.ready() {
            assert!(std::time::Instant::now() < deadline, "GPU fence timed out");
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        let image = download.finish(&mut renderer).unwrap();
        for y in 0..25 {
            for x in 0..25 {
                let expected = if (10..22).contains(&x) && (10..22).contains(&y) {
                    [255, 0, 0, 255]
                } else {
                    [(x + 10) as u8, (y + 10) as u8, 80, 255]
                };
                assert_eq!(
                    &image.rgba[((y * 25 + x) * 4) as usize..((y * 25 + x) * 4 + 4) as usize],
                    &expected,
                    "{transform:?} crop {x},{y}"
                );
            }
        }
        let view = View {
            id: Id::new(),
            commit: Default::default(),
            texture: frozen.image.clone(),
            size,
            selection: Some(Rectangle::from_size(size)),
            scale,
        };
        let mut target: GlesRenderbuffer = renderer
            .create_buffer(Fourcc::Abgr8888, (192, 96).into())
            .unwrap();
        let mut framebuffer = renderer.bind(&mut target).unwrap();
        let mut captures = Vec::new();
        for frozen_view in [false, true] {
            let mut frame = renderer
                .render(&mut framebuffer, physical, transform)
                .unwrap();
            frame
                .clear([0.0, 0.0, 0.0, 1.0].into(), &[Rectangle::from_size(size)])
                .unwrap();
            if frozen_view {
                view.draw(
                    &mut frame,
                    view.src(),
                    view.geometry(scale.into()),
                    &[Rectangle::from_size(size)],
                    &[],
                )
                .unwrap();
            } else {
                for element in frozen.inputs[1..].iter().rev() {
                    element
                        .draw(
                            &mut frame,
                            element.src(),
                            element.geometry(scale.into()),
                            &[Rectangle::from_size(element.geometry(scale.into()).size)],
                            &[],
                        )
                        .unwrap();
                }
            }
            frame.finish().unwrap().wait().unwrap();
            let mapping = renderer
                .copy_framebuffer(
                    &framebuffer,
                    Rectangle::from_size((192, 96).into()),
                    Fourcc::Abgr8888,
                )
                .unwrap();
            captures.push(renderer.map_texture(&mapping).unwrap().to_vec());
        }
        for y in 4..92 {
            for x in 4..188 {
                let at = (y * 192 + x) * 4;
                assert_eq!(
                    &captures[0][at..at + 4],
                    &captures[1][at..at + 4],
                    "{transform:?} frozen preview {x},{y}"
                );
            }
        }
    }
}
