use super::super::{server::Server, x11::FloatingKind};
use smithay::{
    backend::renderer::utils::with_renderer_surface_state,
    desktop::Window,
    reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
    utils::{Logical, Point, Rectangle},
    wayland::{compositor::with_states, shell::xdg::SurfaceCachedState},
};
use std::error::Error;

pub(super) fn imported(window: &Window) -> bool {
    window.toplevel().is_some_and(|top| {
        with_renderer_surface_state(top.wl_surface(), |state| state.buffer().is_some())
            .unwrap_or(false)
    })
}

pub(super) fn floating(
    server: &Server,
    main: &Window,
    window: &Window,
    kind: FloatingKind,
) -> Result<Rectangle<i32, Logical>, Box<dyn Error>> {
    let state = &server.state;
    let top = window.toplevel().ok_or("floating toplevel disappeared")?;
    let (min, max) = with_states(top.wl_surface(), |states| {
        let mut cached = states.cached_state.get::<SurfaceCachedState>();
        let current = cached.current();
        (current.min_size, current.max_size)
    });
    let (width, height) = kind.size();
    let size = (i32::from(width), i32::from(height)).into();
    let expected_parent = match kind {
        FloatingKind::Transient => Some(
            main.toplevel()
                .ok_or("main disappeared")?
                .wl_surface()
                .clone(),
        ),
        FloatingKind::Splash => None,
    };
    let hints_match = match kind {
        FloatingKind::Transient => min == (0, 0).into() && max == (0, 0).into(),
        FloatingKind::Splash => min == size && max == size,
    };
    if top.parent() != expected_parent || !hints_match {
        return Err(format!("Satellite {kind:?} forwarding mismatch: parent={:?}, expected={expected_parent:?}, min={min:?}, max={max:?}", top.parent()).into());
    }
    let area = state.tiling_area().ok_or("missing workarea")?;
    let anchor = match kind {
        FloatingKind::Transient => state
            .window_layout_geometry(main)
            .ok_or("missing parent allocation")?,
        FloatingKind::Splash => area,
    };
    let expected = Rectangle::new(
        (
            (anchor.loc.x + (anchor.size.w - size.w) / 2)
                .clamp(area.loc.x, area.loc.x + area.size.w - size.w),
            (anchor.loc.y + (anchor.size.h - size.h) / 2)
                .clamp(area.loc.y, area.loc.y + area.size.h - size.h),
        )
            .into(),
        size,
    );
    let frame = Rectangle::new(
        expected.loc - smithay::utils::Point::from((2, 2)),
        (size.w + 4, size.h + 4).into(),
    );
    let current = top.current_state();
    let tiled = [
        xdg_toplevel::State::TiledLeft,
        xdg_toplevel::State::TiledRight,
        xdg_toplevel::State::TiledTop,
        xdg_toplevel::State::TiledBottom,
    ];
    if !state.window_is_floating(window)
        || state.floating_geometry(window) != Some(expected)
        || state.window_layout_geometry(window) != Some(expected)
        || state.window_frame_geometry(window) != Some(frame)
        || state.window_tile_geometry(window).is_some()
        || window.geometry().size != size
        || state.space().element_location(window) != Some(expected.loc - window.geometry().loc)
        || state.workspaces.index_of(window) != state.workspaces.index_of(main)
        || tiled.iter().any(|flag| current.states.contains(*flag))
        || current.states.contains(xdg_toplevel::State::Fullscreen)
        || !imported(window)
    {
        return Err(format!("{kind:?} is not mapped floating at its natural bounded size: floating={}, allocation={:?}, expected={expected:?}, geometry={:?}, state={current:?}",
            state.window_is_floating(window), state.floating_geometry(window), window.geometry()).into());
    }
    let order: Vec<_> = state.visible_windows().collect();
    let main_index = order.iter().position(|candidate| *candidate == main);
    let index = order.iter().position(|candidate| *candidate == window);
    if !matches!((main_index, index), (Some(parent), Some(child)) if parent < child) {
        return Err(format!("{kind:?} is not stacked above the main window").into());
    }
    Ok(expected)
}

pub(super) fn center(area: Rectangle<i32, Logical>) -> Point<f64, Logical> {
    area.loc.to_f64() + Point::from((f64::from(area.size.w) / 2.0, f64::from(area.size.h) / 2.0))
}

pub(super) fn hit(
    server: &Server,
    window: &Window,
    point: Point<f64, Logical>,
) -> Result<(), Box<dyn Error>> {
    let hit = server.state.window_under(point);
    let surface = window
        .toplevel()
        .ok_or("hit target disappeared")?
        .wl_surface();
    if hit.as_ref().map(|hit| hit.window) != Some(window)
        || server
            .state
            .surface_under(point)
            .map(|(surface, _)| surface)
            .as_ref()
            != Some(surface)
    {
        return Err(
            format!("floating window is not hittable above its parent at {point:?}").into(),
        );
    }
    Ok(())
}
