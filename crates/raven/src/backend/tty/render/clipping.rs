use super::SceneElement;
use smithay::{
    backend::renderer::{
        element::{surface::WaylandSurfaceRenderElement, utils::CropRenderElement},
        gles::GlesRenderer,
    },
    utils::{Logical, Rectangle},
};

pub(super) fn append(
    elements: &mut Vec<SceneElement>,
    surfaces: impl IntoIterator<Item = WaylandSurfaceRenderElement<GlesRenderer>>,
    clip: Rectangle<i32, Logical>,
    scale: f64,
) {
    // Round shared edges, not each tile's width independently. At fractional
    // scale the latter can let neighboring allocations overlap by one pixel.
    let clip = Rectangle::from_extremities(
        clip.loc.to_physical_precise_round(scale),
        (clip.loc + clip.size).to_physical_precise_round(scale),
    );
    elements.extend(surfaces.into_iter().filter_map(|element| {
        // Smithay forwards identity, damage, opaque regions and underlying storage.
        // Keeping the latter allows DRM to scan out a clipped client buffer.
        CropRenderElement::from_element(element, scale, clip).map(SceneElement::ClippedSurface)
    }));
}
