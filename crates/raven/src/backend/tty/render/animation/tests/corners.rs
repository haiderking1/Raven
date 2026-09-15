use super::super::{
    blend::{self, Blend},
    snapshot::Snapshot,
};
use crate::backend::tty::render::{SceneElement, rounded::shape::Shape};
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
    utils::{Rectangle, Transform},
};
use std::rc::Rc;
#[test]
#[ignore = "requires headless EGL"]
fn blended_resize_clips_only_the_final_frame_and_keeps_fullscreen_square() {
    let display = unsafe { EGLDisplay::new(EGLSurfacelessDisplay) }.unwrap();
    let context = EGLContext::new(&display).unwrap();
    let mut renderer = unsafe { GlesRenderer::new(context) }.unwrap();
    let bounds = Rectangle::new((20, 10).into(), (80, 60).into());
    let mut snapshot = |color: [u8; 4]| {
        let buffer = MemoryRenderBuffer::from_slice(
            &color.repeat(80 * 60),
            Fourcc::Abgr8888,
            (80, 60),
            1,
            Transform::Normal,
            None,
        );
        let element = MemoryRenderBufferRenderElement::from_buffer(
            &mut renderer,
            (20.0, 10.0),
            &buffer,
            None,
            None,
            None,
            Kind::Unspecified,
        )
        .unwrap();
        Rc::new(
            Snapshot::capture(
                &mut renderer,
                bounds,
                1.0,
                vec![SceneElement::Memory(element)],
            )
            .unwrap(),
        )
    };
    let old = snapshot([255, 0, 0, 255]);
    let current = snapshot([0, 0, 255, 255]);
    let program = blend::compile(&mut renderer).unwrap();
    let mut blend = Blend {
        corners: None,
        id: Id::new(),
        old,
        current,
        bounds,
        progress: 0.25,
        commit: Default::default(),
        program,
        live: Rc::from([]),
        opaque: Rc::from([Rectangle::from_size(bounds.size)]),
    };
    for rounded in [false, true] {
        blend.corners = rounded.then_some(Shape::new(bounds, 16.0));
        let mut target: GlesRenderbuffer = renderer
            .create_buffer(Fourcc::Abgr8888, (120, 90).into())
            .unwrap();
        let mut framebuffer = renderer.bind(&mut target).unwrap();
        let mut frame = renderer
            .render(&mut framebuffer, (120, 90).into(), Transform::Normal)
            .unwrap();
        frame
            .clear(
                [0.0, 0.0, 0.0, 1.0].into(),
                &[Rectangle::from_size((120, 90).into())],
            )
            .unwrap();
        blend
            .draw(
                &mut frame,
                blend.src(),
                bounds,
                &[Rectangle::from_size(bounds.size)],
                &[],
            )
            .unwrap();
        frame.finish().unwrap().wait().unwrap();
        let mapping = renderer
            .copy_framebuffer(
                &framebuffer,
                Rectangle::from_size((120, 90).into()),
                Fourcc::Abgr8888,
            )
            .unwrap();
        let bytes = renderer.map_texture(&mapping).unwrap();
        let pixel = |x: usize, y: usize| &bytes[(y * 120 + x) * 4..(y * 120 + x) * 4 + 4];
        assert_eq!(pixel(60, 40), &[191, 0, 64, 255]);
        assert_eq!(
            pixel(20, 10),
            if rounded {
                &[0, 0, 0, 255]
            } else {
                &[191, 0, 64, 255]
            }
        );
        assert_eq!(
            blend
                .opaque_regions(1.0.into())
                .iter()
                .any(|r| r.contains((0, 0))),
            !rounded
        );
    }
}
