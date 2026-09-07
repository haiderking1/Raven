use smithay::{
    desktop::LayerSurface,
    output::Output,
    utils::{Logical, Rectangle, Serial, Size},
    wayland::shell::wlr_layer::{Anchor, ExclusiveZone, LayerSurfaceCachedState},
};

/// Match Smithay 0.7 LayerMap::arrange without adding a pending client to the map.
fn initial_size(
    state: LayerSurfaceCachedState,
    output: &Output,
    zone: Rectangle<i32, Logical>,
) -> Size<i32, Logical> {
    let output_size = output.current_mode().map_or_else(
        || (0, 0).into(),
        |mode| {
            let logical = mode
                .size
                .to_f64()
                .to_logical(output.current_scale().fractional_scale())
                .to_i32_round();
            output.current_transform().transform_size(logical)
        },
    );
    let mut source = match state.exclusive_zone {
        ExclusiveZone::DontCare => output_size,
        ExclusiveZone::Exclusive(_) | ExclusiveZone::Neutral => zone.size,
    };
    if state.anchor.contains(Anchor::LEFT) {
        source.w -= state.margin.left;
    }
    if state.anchor.contains(Anchor::RIGHT) {
        source.w -= state.margin.right;
    }
    if state.anchor.contains(Anchor::TOP) {
        source.h -= state.margin.top;
    }
    if state.anchor.contains(Anchor::BOTTOM) {
        source.h -= state.margin.bottom;
    }
    let mut size = state.size;
    size.w = size.w.min(source.w);
    size.h = size.h.min(source.h);
    if size.w == 0 {
        size.w = source.w / 2;
    }
    if size.h == 0 {
        size.h = source.h / 2;
    }
    if state.anchor.anchored_horizontally() {
        size.w = source.w;
    }
    if state.anchor.anchored_vertically() {
        size.h = source.h;
    }
    size
}

fn prepare_configure(
    layer: &LayerSurface,
    output: &Output,
    zone: Rectangle<i32, Logical>,
) -> Result<(), ()> {
    let size = initial_size(layer.cached_state(), output, zone);
    // A negative extent cannot be represented by the protocol's unsigned configure size.
    if size.w < 0 || size.h < 0 {
        return Err(());
    }
    let surface = layer.layer_surface();
    surface.with_pending_state(|pending| pending.size = Some(size));
    Ok(())
}

pub(super) fn configure_initial(
    layer: &LayerSurface,
    output: &Output,
    zone: Rectangle<i32, Logical>,
) -> Result<Serial, ()> {
    prepare_configure(layer, output, zone)?;
    // Force a fresh serial even when the previous mapping had the same size.
    Ok(layer.layer_surface().send_configure())
}

pub(super) fn configure_pending(
    layer: &LayerSurface,
    output: &Output,
    zone: Rectangle<i32, Logical>,
) -> Result<(), ()> {
    prepare_configure(layer, output, zone)?;
    layer.layer_surface().send_pending_configure();
    Ok(())
}
