use crate::state::State;
use smithay::{
    desktop::Window,
    utils::{Logical, Point},
};

impl State {
    /// Move committed floating geometry without sending a size configure or
    /// entering pointer dispatch. This is also called under the pointer grab.
    pub(crate) fn move_floating_window(&mut self, window: &Window, position: Point<i32, Logical>) {
        if self.fullscreen_manages(window) {
            return;
        }
        if self
            .window_frame_geometry(window)
            .is_some_and(|frame| frame.loc != position)
        {
            self.leave_maximized_for_interaction(window);
        }
        let Some(index) = self.workspaces.index_of(window) else {
            return;
        };
        let Some(entry) = self.workspaces.entries[index]
            .floating
            .entries
            .get_mut(window)
        else {
            return;
        };
        let (Some(mut frame), Some(mut client)) = (entry.frame, entry.geometry) else {
            return;
        };
        let delta = position - frame.loc;
        if delta == (0, 0).into() {
            return;
        }
        entry.position = Some(position);
        frame.loc = position;
        client.loc += delta;
        entry.frame = Some(frame);
        entry.geometry = Some(client);
        self.workspaces.entries[index]
            .space
            .map_element(window.clone(), client.loc, false);
        self.refresh_reactive_popups(index);
        self.request_redraw();
    }
}
