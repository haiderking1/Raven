//! Apply committed mapping and damage without changing buffer ownership.

use super::{
    BufferCoord, Damage, Rectangle, RendererSurfaceState, Size, SurfaceAttributes, SurfaceData, SurfaceView,
};

impl RendererSurfaceState {
    pub(super) fn update_view_and_damage(
        &mut self,
        states: &SurfaceData,
        attrs: &mut SurfaceAttributes,
        previous_dimensions: Option<Size<i32, BufferCoord>>,
    ) -> Option<(SurfaceView, bool)> {
        let mapping_changed = self.buffer_scale != attrs.buffer_scale
            || self.buffer_transform != attrs.buffer_transform.into()
            || self.buffer_dimensions != previous_dimensions;
        self.buffer_scale = attrs.buffer_scale;
        self.buffer_transform = attrs.buffer_transform.into();

        let Some(buffer_dimensions) = self.buffer_dimensions else {
            // Damage committed without content must not reach a later attachment.
            attrs.damage.clear();
            return None;
        };

        let surface_size = buffer_dimensions.to_logical(self.buffer_scale, self.buffer_transform);
        // Bounds validation must see the committed transform and integer scale.
        let surface_view = SurfaceView::from_states(states, surface_size, attrs.client_scale);
        let view_changed = self.surface_view.replace(surface_view) != Some(surface_view);

        if !attrs.damage.is_empty() {
            // Retained SHM content may have changed. Reimport through the normal
            // renderer path, retaining the Buffer and its acquire/release points.
            self.textures.clear();
        }

        let buffer_rect = Rectangle::from_size(buffer_dimensions);
        let buffer_damage = attrs.damage.drain(..).filter_map(|damage| {
            match damage {
                Damage::Buffer(rect) => rect,
                Damage::Surface(rect) => surface_view.rect_to_local(rect).to_i32_up().to_buffer(
                    self.buffer_scale,
                    self.buffer_transform,
                    &surface_size,
                ),
            }
            .intersection(buffer_rect)
        });
        // A crop or transform can change every displayed pixel without changing
        // the destination geometry or receiving client damage. Full buffer damage
        // advances the render-element counter and invalidates the old mapping.
        self.damage
            .add(buffer_damage.chain((mapping_changed || view_changed).then_some(buffer_rect)));

        Some((surface_view, view_changed || mapping_changed))
    }
}
