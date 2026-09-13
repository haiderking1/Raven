use smithay::utils::{Physical, Rectangle};

/// Hyprland SurfacePassElement::getTexBox: interactive undersized roots keep
/// their buffer extent, and children retain their offsets and natural sizes.
/// Oversized surfaces are squeezed only at the allocation's right/bottom edge.
/// Unlike animated mapping, a root resize does not rescale every child subtree.
pub(super) fn destination(
    source: Rectangle<i32, Physical>,
    window: Rectangle<i32, Physical>,
    target: Rectangle<i32, Physical>,
    main: bool,
    small: bool,
) -> Rectangle<i32, Physical> {
    let mut destination = if main {
        Rectangle::new(target.loc, if small { source.size } else { target.size })
    } else {
        Rectangle::new(target.loc + (source.loc - window.loc), source.size)
    };
    destination.size.w = destination
        .size
        .w
        .min(target.loc.x + target.size.w - destination.loc.x);
    destination.size.h = destination
        .size
        .h
        .min(target.loc.y + target.size.h - destination.loc.y);
    destination
}
