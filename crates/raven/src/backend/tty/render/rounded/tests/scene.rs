use crate::{
    backend::tty::render::{self, SceneElement},
    desktop::{appearance::Appearance, tests::fixture::Fixture},
};
use smithay::{
    backend::{
        allocator::Fourcc,
        egl::{EGLContext, EGLDisplay, native::EGLSurfacelessDisplay},
        renderer::{
            Bind, Offscreen,
            damage::OutputDamageTracker,
            element::Element,
            gles::{GlesRenderbuffer, GlesRenderer},
        },
    },
    output::{Mode, Output, PhysicalProperties, Subpixel},
    utils::Transform,
};
#[test]
#[ignore = "requires headless EGL"]
fn live_window_keeps_identity_idle_damage_and_corner_input_consistent() {
    let mut f = Fixture::new();
    f.state.set_appearance(Appearance::default()).unwrap();
    let output = Output::new(
        "corners".into(),
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
            refresh: 60000,
        }),
        None,
        None,
        Some((0, 0).into()),
    );
    f.state.space_mut().map_output(&output, (0, 0));
    f.state.output = Some(output.clone());
    let top = f.toplevel();
    f.configure(top);
    let buffer = f.buffer_sized(800, 600);
    f.attach(top, buffer);
    f.state.refresh();
    let window = f.state.visible_windows().next().unwrap();
    let client = f.state.window_client_geometry(window).unwrap();
    assert!(
        f.state
            .window_under((client.loc.x as f64 + 1.0, client.loc.y as f64 + 1.0).into())
            .is_none()
    );
    assert!(f.state.window_under((400.0, 300.0).into()).is_some());
    let display = unsafe { EGLDisplay::new(EGLSurfacelessDisplay) }.unwrap();
    let context = EGLContext::new(&display).unwrap();
    let mut renderer = unsafe { GlesRenderer::new(context) }.unwrap();
    let mut animations = render::animation::Animations::default();
    let mut elements = Vec::new();
    render::desktop::append(
        &mut renderer,
        &f.state,
        &output,
        &mut animations,
        &mut elements,
    )
    .unwrap();
    assert_eq!(elements.len(), 2);
    assert!(matches!(elements[0], SceneElement::RoundedBorder(_)));
    assert!(matches!(elements[1], SceneElement::Rounded(_)));
    let ids: Vec<_> = elements
        .iter()
        .map(|e| (e.id().clone(), e.current_commit()))
        .collect();
    let mut target: GlesRenderbuffer = renderer
        .create_buffer(Fourcc::Abgr8888, (800, 600).into())
        .unwrap();
    let mut framebuffer = renderer.bind(&mut target).unwrap();
    let mut tracker = OutputDamageTracker::new((800, 600), 1.0, Transform::Normal);
    let result = tracker
        .render_output(
            &mut renderer,
            &mut framebuffer,
            0,
            &elements,
            [0.0, 0.0, 0.0, 1.0],
        )
        .unwrap();
    assert!(result.damage.is_some());
    result.sync.wait().unwrap();
    drop(result);
    elements.clear();
    render::desktop::append(
        &mut renderer,
        &f.state,
        &output,
        &mut animations,
        &mut elements,
    )
    .unwrap();
    for (e, (id, commit)) in elements.iter().zip(ids) {
        assert_eq!(*e.id(), id);
        assert_eq!(e.current_commit(), commit);
    }
    assert!(
        tracker
            .render_output(
                &mut renderer,
                &mut framebuffer,
                1,
                &elements,
                [0.0, 0.0, 0.0, 1.0]
            )
            .unwrap()
            .damage
            .is_none()
    );
}
