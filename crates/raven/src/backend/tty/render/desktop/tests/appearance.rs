use crate::{
    backend::tty::render::SceneElement,
    desktop::{appearance::Appearance, tests::fixture::Fixture},
};
use smithay::{
    backend::{
        allocator::Fourcc,
        renderer::{
            Bind, Offscreen,
            damage::OutputDamageTracker,
            element::Element,
            gles::{GlesRenderbuffer, GlesRenderer},
        },
    },
    output::Output,
    utils::{Rectangle, Transform},
};

pub(super) fn assert_border_scene(f: &mut Fixture, renderer: &mut GlesRenderer, output: &Output) {
    f.state.set_appearance(Appearance::default()).unwrap();
    let mut elements = Vec::new();
    super::super::append(renderer, &f.state, output, &mut elements);
    assert_eq!(elements.len(), 5);
    assert!(
        elements[..4]
            .iter()
            .all(|e| matches!(e, SceneElement::Border(_)))
    );
    assert_eq!(
        elements[4].geometry(1.0.into()),
        Rectangle::new((10, 10).into(), (780, 580).into())
    );
    let identity: Vec<_> = elements[..4]
        .iter()
        .map(|e| (e.id().clone(), e.current_commit()))
        .collect();
    let mut target: GlesRenderbuffer = renderer
        .create_buffer(Fourcc::Abgr8888, (800, 600).into())
        .unwrap();
    let mut framebuffer = renderer.bind(&mut target).unwrap();
    let mut damage = OutputDamageTracker::new((800, 600), 1.0, Transform::Normal);
    {
        let result = damage
            .render_output(
                renderer,
                &mut framebuffer,
                0,
                &elements,
                [0.0, 0.0, 0.0, 1.0],
            )
            .unwrap();
        assert!(
            result.damage.is_some(),
            "the real EGL path draws the border scene"
        );
        result.sync.wait().unwrap();
    }
    elements.clear();
    super::super::append(renderer, &f.state, output, &mut elements);
    for (element, (id, commit)) in elements[..4].iter().zip(&identity) {
        assert_eq!(element.id(), id);
        assert_eq!(element.current_commit(), *commit);
    }
    assert!(
        damage
            .render_output(
                renderer,
                &mut framebuffer,
                1,
                &elements,
                [0.0, 0.0, 0.0, 1.0]
            )
            .unwrap()
            .damage
            .is_none(),
        "stable borders must not create continuous damage"
    );

    f.state.activate_window(None);
    elements.clear();
    super::super::append(renderer, &f.state, output, &mut elements);
    for (element, (id, commit)) in elements[..4].iter().zip(&identity) {
        assert_eq!(
            element.id(),
            id,
            "focus keeps each stripe's damage identity"
        );
        assert_ne!(
            element.current_commit(),
            *commit,
            "inactive color damages the stripe"
        );
    }
    assert!(
        damage
            .render_output(
                renderer,
                &mut framebuffer,
                1,
                &elements,
                [0.0, 0.0, 0.0, 1.0]
            )
            .unwrap()
            .damage
            .is_some()
    );
    elements.clear();
    super::super::append(renderer, &f.state, output, &mut elements);
    assert!(
        damage
            .render_output(
                renderer,
                &mut framebuffer,
                1,
                &elements,
                [0.0, 0.0, 0.0, 1.0]
            )
            .unwrap()
            .damage
            .is_none()
    );

    let mut appearance = Appearance::default();
    appearance.border.inactive = [0.8, 0.4, 0.2, 0.5];
    f.state.set_appearance(appearance).unwrap();
    elements.clear();
    super::super::append(renderer, &f.state, output, &mut elements);
    assert!(
        elements[..4]
            .iter()
            .all(|e| e.alpha() == 0.5 && e.opaque_regions(1.0.into()).is_empty())
    );
    let result = damage
        .render_output(
            renderer,
            &mut framebuffer,
            1,
            &elements,
            [0.0, 0.0, 0.0, 1.0],
        )
        .unwrap();
    assert!(result.damage.is_some());
    result.sync.wait().unwrap();
    f.state.set_appearance(Appearance::disabled()).unwrap();
    elements.clear();
    super::super::append(renderer, &f.state, output, &mut elements);
    assert_eq!(elements.len(), 1);
    assert!(
        damage
            .render_output(
                renderer,
                &mut framebuffer,
                1,
                &elements,
                [0.0, 0.0, 0.0, 1.0]
            )
            .unwrap()
            .damage
            .is_some(),
        "removing frames damages their old pixels"
    );
}
