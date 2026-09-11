use super::RendererSurfaceState;
use crate::utils::Rectangle;

impl RendererSurfaceState {
    /// Invalidate the complete attached buffer without changing its contents,
    /// viewport, ownership, acquire synchronization, or scanout eligibility.
    pub fn damage_entire_buffer(&mut self) {
        if self.buffer.is_some() {
            if let Some(size) = self.buffer_dimensions {
                self.damage.add([Rectangle::from_size(size)]);
            }
        }
    }

    /// Re-submit the most recently recorded damage for a sibling-surface
    /// repaint. This advances the renderer damage counter, not wl_surface state.
    pub fn replay_buffer_damage(&mut self) {
        if self.buffer.is_none() {
            return;
        }
        let damage: Vec<_> = self
            .damage
            .raw()
            .next()
            .map(|rects| rects.copied().collect())
            .unwrap_or_default();
        if !damage.is_empty() {
            self.damage.add(damage);
        }
    }
}
