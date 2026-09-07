//! Run explicitly on a machine with EGL device support and timer queries.

use super::GpuTime;
use smithay::{
    backend::{
        allocator::Fourcc,
        egl::{EGLContext, EGLDevice, EGLDisplay},
        renderer::{
            Bind, Frame, Offscreen, Renderer,
            gles::{GlesRenderbuffer, GlesRenderer, ffi},
        },
    },
    utils::{Rectangle, Transform},
};
use std::{
    ffi::CStr,
    thread,
    time::{Duration, Instant},
};

fn state(renderer: &mut GlesRenderer) -> [i32; 5] {
    renderer
        .with_context(|gl| unsafe {
            let mut state = [0; 5];
            gl.GetIntegerv(ffi::FRAMEBUFFER_BINDING, state.as_mut_ptr());
            gl.GetIntegerv(ffi::VIEWPORT, state[1..].as_mut_ptr());
            assert_eq!(gl.GetError(), ffi::NO_ERROR);
            state
        })
        .unwrap()
}

#[test]
#[ignore = "requires an EGL device advertising GL_EXT_disjoint_timer_query"]
fn real_elapsed_query_preserves_renderer_state_and_releases_its_lease() {
    let device = EGLDevice::enumerate()
        .unwrap()
        .next()
        .expect("no EGL device");
    // This test owns the display and context and never binds them elsewhere.
    let display = unsafe { EGLDisplay::new(device) }.unwrap();
    let context = EGLContext::new(&display).unwrap();
    let mut renderer = unsafe { GlesRenderer::new(context) }.unwrap();
    renderer
        .with_context(|gl| unsafe {
            let name = CStr::from_ptr(gl.GetString(ffi::RENDERER).cast());
            eprintln!("EGL test renderer: {}", name.to_string_lossy());
        })
        .unwrap();
    let mut buffer: GlesRenderbuffer = renderer
        .create_buffer(Fourcc::Argb8888, (256, 256).into())
        .unwrap();
    let mut target = renderer.bind(&mut buffer).unwrap();
    let before = state(&mut renderer);
    let mut timer = GpuTime::new(&mut renderer).unwrap();
    assert!(
        timer.is_enabled(),
        "elapsed timer unavailable on this EGL device"
    );
    assert_eq!(state(&mut renderer), before);
    assert!(
        !GpuTime::new(&mut renderer).unwrap().is_enabled(),
        "duplicate disjoint reader"
    );

    let mut measured = None;
    let deadline = Instant::now() + Duration::from_secs(2);
    // Retry a bounded number of our own render jobs if power-up is disjoint.
    for _ in 0..4 {
        timer.reset(&mut renderer).unwrap();
        let before = state(&mut renderer);
        timer.begin(&mut renderer).unwrap();
        assert_eq!(state(&mut renderer), before);
        let mut frame = renderer
            .render(&mut target, (256, 256).into(), Transform::Normal)
            .unwrap();
        frame
            .clear(
                [0.2, 0.4, 0.6, 1.0].into(),
                &[Rectangle::from_size((256, 256).into())],
            )
            .unwrap();
        let fence = frame.finish().unwrap();
        let before = state(&mut renderer);
        timer.end(&mut renderer).unwrap();
        assert_eq!(state(&mut renderer), before);
        // Polling drives query progress. Only the test sleeps, never GpuTime.
        for _ in 0..100 {
            measured = timer.sample(&mut renderer).unwrap();
            if measured.is_some() || Instant::now() >= deadline {
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        if measured.is_some() {
            assert!(
                fence.is_reached(),
                "elapsed completion must follow the render fence"
            );
        }
        // There is no external buffer consumer. Reuse in this same GL
        // context is ordered, including when a disjoint discarded the sample.
        if measured.is_some() || Instant::now() >= deadline {
            break;
        }
    }
    assert!(
        measured.is_some_and(|duration| duration > Duration::ZERO),
        "no nonzero elapsed GL interval"
    );
    let before = state(&mut renderer);
    timer.destroy(&mut renderer).unwrap();
    timer.destroy(&mut renderer).unwrap();
    assert!(!timer.is_enabled());
    assert_eq!(state(&mut renderer), before);
    // Re-creation checks both release of the lease and an idle query target.
    let mut replacement = GpuTime::new(&mut renderer).unwrap();
    assert!(replacement.is_enabled());
    replacement.destroy(&mut renderer).unwrap();
}
