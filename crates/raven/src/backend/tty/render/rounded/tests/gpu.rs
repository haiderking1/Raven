use super::super::{
    element::{Ring, Rounded},
    program::Programs,
    shape::Shape,
};
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
    utils::{Rectangle, Transform},
};
fn coverage(x: f32, y: f32, w: f32, h: f32, r: f32) -> f32 {
    let qx = (x - w / 2.0).abs() - w / 2.0 + r;
    let qy = (y - h / 2.0).abs() - h / 2.0 + r;
    let d = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - r;
    (0.5 - d).clamp(0.0, 1.0)
}
#[test]
#[ignore = "requires headless EGL"]
fn rounded_pixels_match_cpu_reference_across_transforms_scales_and_output_edges() {
    let display = unsafe { EGLDisplay::new(EGLSurfacelessDisplay) }.unwrap();
    let context = EGLContext::new(&display).unwrap();
    let mut renderer = unsafe { GlesRenderer::new(context) }.unwrap();
    let programs = Programs::get(&mut renderer).unwrap();
    for transform in [
        Transform::Normal,
        Transform::_90,
        Transform::_180,
        Transform::_270,
        Transform::Flipped,
        Transform::Flipped90,
        Transform::Flipped180,
        Transform::Flipped270,
    ] {
        for scale in [1.0, 1.5, 2.0] {
            for left in [12, -8] {
                let w = (64.0 * scale) as i32;
                let h = (40.0 * scale) as i32;
                let r = 16.0 * scale as f32;
                let width = scale as f32;
                let bounds = Rectangle::new(
                    ((left as f64 * scale) as i32, (8.0 * scale) as i32).into(),
                    (w, h).into(),
                );
                let shape = Shape::new(bounds, r);
                let inner_bounds = Rectangle::from_extremities(
                    (
                        ((left + 1) as f64 * scale).round() as i32,
                        (9.0 * scale).round() as i32,
                    ),
                    (
                        ((left + 63) as f64 * scale).round() as i32,
                        (47.0 * scale).round() as i32,
                    ),
                );
                let inner_shape = Shape::new(inner_bounds, r - width);
                let original = [80u8, 120, 200, 255].repeat((w * h) as usize);
                let expected: Vec<u8> = (0..h)
                    .flat_map(|y| {
                        (0..w).flat_map(move |x| {
                            let outer =
                                coverage(x as f32 + 0.5, y as f32 + 0.5, w as f32, h as f32, r);
                            let inner = coverage(
                                x as f32 + 0.5 - (inner_bounds.loc.x - bounds.loc.x) as f32,
                                y as f32 + 0.5 - (inner_bounds.loc.y - bounds.loc.y) as f32,
                                inner_bounds.size.w as f32,
                                inner_bounds.size.h as f32,
                                r - width,
                            );
                            let ring = (outer - inner).max(0.0);
                            [80.0, 120.0, 200.0, 255.0]
                                .map(|c| (255.0 * ring + c * outer * (1.0 - ring)).round() as u8)
                        })
                    })
                    .collect();
                let mut target: GlesRenderbuffer = renderer
                    .create_buffer(Fourcc::Abgr8888, (192, 128).into())
                    .unwrap();
                let mut framebuffer = renderer.bind(&mut target).unwrap();
                let mut outputs = Vec::new();
                for reference in [false, true] {
                    let buffer = MemoryRenderBuffer::from_slice(
                        if reference { &expected } else { &original },
                        Fourcc::Abgr8888,
                        (w, h),
                        1,
                        Transform::Normal,
                        None,
                    );
                    let memory = MemoryRenderBufferRenderElement::from_buffer(
                        &mut renderer,
                        bounds.loc.to_f64(),
                        &buffer,
                        None,
                        Some(Rectangle::from_size((w as f64, h as f64).into())),
                        Some((64, 40).into()),
                        Kind::Unspecified,
                    )
                    .unwrap();
                    let element = if reference {
                        SceneElement::Memory(memory)
                    } else {
                        SceneElement::Rounded(Rounded {
                            element: Box::new(SceneElement::Memory(memory)),
                            shape,
                            commit: Default::default(),
                            previous: None,
                            scale,
                            programs: programs.clone(),
                        })
                    };
                    let mut frame = renderer
                        .render(&mut framebuffer, (192, 128).into(), transform)
                        .unwrap();
                    frame
                        .clear(
                            [0.0, 0.0, 0.0, 1.0].into(),
                            &[Rectangle::from_size(
                                transform.transform_size((192, 128).into()),
                            )],
                        )
                        .unwrap();
                    element
                        .draw(
                            &mut frame,
                            element.src(),
                            element.geometry(scale.into()),
                            &[Rectangle::from_size((w, h).into())],
                            &[],
                        )
                        .unwrap();
                    if !reference {
                        let ring = Ring {
                            id: Id::new(),
                            commit: Default::default(),
                            shape,
                            color: [1.0; 4],
                            inner: inner_shape,
                            programs: programs.clone(),
                        };
                        ring.draw(
                            &mut frame,
                            ring.src(),
                            bounds,
                            &[Rectangle::from_size((w, h).into())],
                            &[],
                        )
                        .unwrap();
                    }
                    frame.finish().unwrap().wait().unwrap();
                    let mapping = renderer
                        .copy_framebuffer(
                            &framebuffer,
                            Rectangle::from_size((192, 128).into()),
                            Fourcc::Abgr8888,
                        )
                        .unwrap();
                    outputs.push(renderer.map_texture(&mapping).unwrap().to_vec());
                }
                for (i, (actual, expected)) in outputs[0].iter().zip(&outputs[1]).enumerate() {
                    assert!(
                        (i16::from(*actual) - i16::from(*expected)).abs() <= 2,
                        "{transform:?} scale {scale} left {left} byte {i}: {actual} != {expected}"
                    );
                }
            }
        }
    }
}
