use crate::state::State;
use smithay::desktop::Window;

impl State {
    /// Parent assignment before mapping transfers intent without claiming either
    /// workspace or displacing its displayed owner.
    pub(crate) fn transfer_unmapped_fullscreen(
        &mut self,
        window: &Window,
        source: usize,
        destination: usize,
    ) {
        if let Some(entry) = self.workspaces.entries[source]
            .fullscreen
            .entries
            .remove(window)
        {
            self.workspaces.entries[destination]
                .fullscreen
                .entries
                .insert(window.clone(), entry);
        }
    }

    /// Assignment has already changed, but the destination has not mapped yet.
    /// Workspace moves never activate the destination or its remembered focus.
    pub(crate) fn transfer_fullscreen(
        &mut self,
        window: &Window,
        source: usize,
        destination: usize,
    ) {
        let full = &mut self.workspaces.entries[source].fullscreen;
        let entry = full.entries.remove(window);
        let displayed = full.displayed.as_ref() == Some(window);
        if displayed {
            full.displayed = None;
        }
        if full.requested.as_ref() == Some(window) {
            full.requested = None;
        }
        let Some(entry) = entry else { return };
        let intent = entry.intent;
        let full = &mut self.workspaces.entries[destination].fullscreen;
        let previous = if intent || displayed {
            full.requested.take()
        } else {
            None
        };
        if intent {
            full.requested = Some(window.clone());
        }
        if displayed {
            full.displayed = Some(window.clone());
        }
        full.entries.insert(window.clone(), entry);
        if let Some(previous) = previous {
            if let Some(entry) = full.entries.get_mut(&previous) {
                entry.intent = false;
            }
            self.configure_fullscreen(&previous, true);
        }
    }
}
