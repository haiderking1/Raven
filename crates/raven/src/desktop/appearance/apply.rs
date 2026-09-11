use super::{Appearance, InvalidAppearance};
use crate::state::State;

impl State {
    /// Current validated settings. Change them through set_appearance.
    pub fn appearance(&self) -> Appearance {
        self.appearance
    }

    /// Validate before changing anything. Invalid settings send no configures and
    /// leave layout, focus and redraw state untouched. Equal settings are a no-op.
    pub fn set_appearance(&mut self, appearance: Appearance) -> Result<(), InvalidAppearance> {
        appearance.validate()?;
        if self.appearance == appearance {
            return Ok(());
        }
        self.begin_resize_batch(self.workspaces.active);
        self.appearance = appearance;
        for index in 0..self.workspaces.entries.len() {
            self.retile_workspace(index);
        }
        self.end_resize_batch();
        // Retiling updates mapped clients, floating/opening clients and pointer
        // clipping. Unmapped tiled clients also need their prospective allocation.
        let opening: Vec<_> = self
            .windows
            .iter()
            .filter(|window| {
                self.workspaces.index_of(window).is_some_and(|index| {
                    self.workspaces.entries[index]
                        .space
                        .element_location(window)
                        .is_none()
                })
            })
            .cloned()
            .collect();
        for window in opening {
            if self.fullscreen_manages(&window) {
                self.configure_fullscreen(&window, false);
            } else if window
                .toplevel()
                .is_some_and(|top| top.is_initial_configure_sent())
            {
                self.configure_initial_tile(&window);
                if let Some(top) = window.toplevel() {
                    top.send_pending_configure();
                }
            }
        }
        Ok(())
    }
}
