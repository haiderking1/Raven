//! Run with an external timeout: a recursive mutex lock must fail, not hang CI.
mod appearance;
use crate::desktop::tests::fixture::Fixture;
use smithay::{
    backend::{
        egl::{EGLContext, EGLDevice, EGLDisplay},
        renderer::{element::Element, gles::GlesRenderer},
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    utils::Rectangle,
};

#[test]
#[ignore = "requires an EGL device; run under timeout to bound a render-lock regression"]
fn first_tiled_window_builds_real_scene_without_relocking_layer_map() {
    let mut fixture = Fixture::new();
    let output = Output::new(
        "render-lock-test".into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "test".into(),
            model: "test".into(),
        },
    );
    output.change_current_state(
        Some(Mode {
            size: (800, 600).into(),
            refresh: 60_000,
        }),
        None,
        None,
        Some((0, 0).into()),
    );
    fixture.state.space_mut().map_output(&output, (0, 0));
    fixture.state.output = Some(output.clone());
    let top = fixture.toplevel();
    fixture.configure(top);
    let buffer = fixture.buffer_sized(800, 600);
    fixture.attach(top, buffer);
    fixture.state.refresh();

    let device = EGLDevice::enumerate()
        .unwrap()
        .next()
        .expect("no EGL device");
    // This test owns its EGL display/context and never acquires DRM master.
    let display = unsafe { EGLDisplay::new(device) }.unwrap();
    let context = EGLContext::new(&display).unwrap();
    let mut renderer = unsafe { GlesRenderer::new(context) }.unwrap();
    let mut elements = Vec::new();
    eprintln!("Entering production scene assembly with one mapped tiled client");
    super::append(&mut renderer, &fixture.state, &output, &mut elements);
    assert_eq!(
        elements.len(),
        1,
        "the client's SHM buffer must become a render element"
    );
    assert_eq!(
        elements[0].geometry(1.0.into()),
        Rectangle::new((0, 0).into(), (800, 600).into())
    );
    // Preserve the original deadlock/client-buffer assertions before adding borders.
    elements.clear();
    appearance::assert_border_scene(&mut fixture, &mut renderer, &output);
}
