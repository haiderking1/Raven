use super::super::Capture;
use crate::{backend::tty::render::SceneElement, state::State};
use smithay::{
    backend::{
        allocator::Fourcc,
        egl::{EGLContext, EGLDisplay, native::EGLSurfacelessDisplay},
        renderer::{
            Bind, ExportMem, Frame, Offscreen, Renderer,
            element::{
                Element, Kind, RenderElement,
                memory::{MemoryRenderBuffer, MemoryRenderBufferRenderElement},
            },
            gles::{GlesRenderbuffer, GlesRenderer},
        },
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{calloop::EventLoop, wayland_server::Display},
    utils::{Rectangle, Transform},
};
#[test]
#[ignore = "requires headless EGL"]
fn selector_preview_starts_unframed_and_never_obscures_the_selected_area() {
    let display = unsafe { EGLDisplay::new(EGLSurfacelessDisplay) }.unwrap();
    let context = EGLContext::new(&display).unwrap();
    let mut renderer = unsafe { GlesRenderer::new(context) }.unwrap();
    let wayland = Display::<State>::new().unwrap();
    let event_loop = EventLoop::try_new().unwrap();
    let mut state = State::new(wayland.handle(), event_loop.get_signal()).unwrap();
    state.install_screenshot(event_loop.handle());
    let output = Output::new(
        "preview".into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "test".into(),
            model: "test".into(),
        },
    );
    output.change_current_state(
        Some(Mode {
            size: (960, 540).into(),
            refresh: 60_000,
        }),
        None,
        None,
        Some((0, 0).into()),
    );
    state.space_mut().map_output(&output, (0, 0));
    state.output = Some(output.clone());
    let color = |x: i32, y: i32| {
        [
            (40 + x * 130 / 960) as u8,
            (50 + y * 140 / 540) as u8,
            (60 + ((x / 24 + y / 24) & 1) * 12) as u8,
            255,
        ]
    };
    let pixels: Vec<_> = (0..540)
        .flat_map(|y| (0..960).flat_map(move |x| color(x, y)))
        .collect();
    let memory = MemoryRenderBuffer::from_slice(
        &pixels,
        Fourcc::Abgr8888,
        (960, 540),
        1,
        Transform::Normal,
        None,
    );
    let element = MemoryRenderBufferRenderElement::from_buffer(
        &mut renderer,
        (0.0, 0.0),
        &memory,
        None,
        None,
        None,
        Kind::Unspecified,
    )
    .unwrap();
    state.open_screenshot();
    let mut capture = Capture::default();
    capture.after_frame(
        &mut renderer,
        &mut state,
        &mut vec![SceneElement::Memory(element)],
    );
    for stage in 0..3 {
        if stage == 1 {
            state.screenshot_motion((160.0, 100.0).into());
            state.screenshot_button(0x110, smithay::backend::input::ButtonState::Pressed);
            state.screenshot_motion((559.0, 519.0).into());
        } else if stage == 2 {
            state.screenshot_button(0x110, smithay::backend::input::ButtonState::Released);
        }
        let mut elements = Vec::new();
        capture
            .append(&mut renderer, &state, &output, &mut elements)
            .unwrap();
        assert_eq!(
            elements.len(),
            1,
            "the selector must not have a hint overlay"
        );
        let mut target: GlesRenderbuffer = renderer
            .create_buffer(Fourcc::Abgr8888, (960, 540).into())
            .unwrap();
        let mut framebuffer = renderer.bind(&mut target).unwrap();
        let mut frame = renderer
            .render(&mut framebuffer, (960, 540).into(), Transform::Normal)
            .unwrap();
        frame
            .clear(
                [0.0, 0.0, 0.0, 1.0].into(),
                &[Rectangle::from_size((960, 540).into())],
            )
            .unwrap();
        for element in elements.iter().rev() {
            element
                .draw(
                    &mut frame,
                    element.src(),
                    element.geometry(1.0.into()),
                    &[Rectangle::from_size(element.geometry(1.0.into()).size)],
                    &[],
                )
                .unwrap();
        }
        frame.finish().unwrap().wait().unwrap();
        let mapping = renderer
            .copy_framebuffer(
                &framebuffer,
                Rectangle::from_size((960, 540).into()),
                Fourcc::Abgr8888,
            )
            .unwrap();
        let result = renderer.map_texture(&mapping).unwrap();
        let pixel = |x: usize, y: usize| &result[(y * 960 + x) * 4..(y * 960 + x) * 4 + 4];
        if stage == 0 {
            for y in 0..540 {
                for x in 0..960 {
                    for (actual, original) in pixel(x, y)[..3].iter().zip(color(x as i32, y as i32))
                    {
                        assert!(
                            (f64::from(*actual) - f64::from(original) * 0.5).abs() < 2.0,
                            "opening pixel {x},{y} must be dimmed without a frame or hint"
                        );
                    }
                }
            }
        } else {
            assert_eq!(pixel(350, 210), &color(350, 210));
            assert_eq!(
                pixel(310, 484),
                &color(310, 484),
                "the former hint location must stay unobstructed"
            );
            assert_eq!(
                pixel(158, 98),
                &[255, 255, 255, 255],
                "dragging must show a selection outline"
            );
        }
        for (actual, original) in pixel(20, 20)[..3].iter().zip(color(20, 20)) {
            assert!((f64::from(*actual) - f64::from(original) * 0.5).abs() < 2.0);
        }
        if let Some(path) = std::env::var_os("RAVEN_SCREENSHOT_PREVIEW") {
            std::fs::write(path, result).unwrap();
        }
    }
}
