use crate::state::State;

impl State {
    pub(crate) fn cycle_applications(&mut self, reverse: bool) {
        self.cancel_management_requests();
        if let Some(session) = &mut self.switcher.session {
            session.step(reverse);
        } else {
            if self.exclusive_keyboard_layer().is_some()
                || self.seat.get_keyboard().is_some_and(|k| k.is_grabbed())
                || self.seat.get_pointer().is_some_and(|p| p.is_grabbed())
            {
                return;
            }
            self.switcher.pending = None;
            self.switcher.session = super::model::Session::new(
                self.switcher_candidates(),
                &self.switcher.history,
                reverse,
            );
        }
        self.switcher.reverse = reverse;
        self.update_switcher_artwork();
    }
    pub(crate) fn cancel_app_switcher(&mut self) {
        self.switcher.pending = None;
        self.switcher.exiting = None;
        self.stop_switcher_repeat();
        if self.switcher.session.take().is_some() {
            self.switcher.artwork.hide();
            self.request_redraw();
            self.refresh_tiling_pointer();
        }
    }
    pub(crate) fn confirm_app_switcher(&mut self) {
        let target = self
            .switcher
            .session
            .as_ref()
            .and_then(|session| {
                let target = session.target()?;
                if !self.window_is_minimized(target) {
                    Some(target)
                } else {
                    session
                        .apps
                        .get(session.selected)?
                        .windows
                        .iter()
                        .find(|window| !self.window_is_minimized(window))
                }
            })
            .cloned();
        self.cancel_app_switcher();
        let Some(target) = target else {
            return;
        };
        let Some(index) = self.workspaces.index_of(&target) else {
            return;
        };
        self.switcher.activating = true;
        self.switch_workspace(index);
        self.switcher.activating = false;
        if self.workspaces.active != index {
            return;
        }
        // An exiting fullscreen transaction retains visibility until settlement.
        // Activate only after the normal transaction makes the target visible.
        self.switcher.pending = Some(target);
        self.refresh_switcher_activation();
    }
    // Taskbar restoration shares the same fullscreen-exit and focus transaction.
    pub(in crate::desktop) fn refresh_switcher_activation(&mut self) {
        let Some(target) = self.switcher.pending.clone() else {
            return;
        };
        if self.window_is_minimized(&target)
            || self.workspaces.index_of(&target) != Some(self.workspaces.active)
            || self.space().element_location(&target).is_none()
        {
            self.switcher.pending = None;
            self.switcher.exiting = None;
            return;
        }
        if self.window_is_visible(&target) {
            self.switcher.pending = None;
            self.switcher.exiting = None;
            self.activate_window(Some(target.clone()));
            // Workspace restoration may already have installed this focus while
            // history updates were suppressed, so no seat callback follows.
            if self.focused_window().as_ref() == Some(&target) {
                super::model::remember(&mut self.switcher.history, target);
            }
            self.refresh_tiling_pointer();
        } else if let Some(fullscreen) = self.fullscreen_window().filter(|w| *w != &target).cloned()
        {
            // A pending fullscreen handoff may display a different owner after
            // the first exit request. Follow settlement without resending the
            // same configure every frame or bypassing any transaction.
            if self.switcher.exiting.as_ref() != Some(&fullscreen) {
                self.switcher.exiting = Some(fullscreen.clone());
                if let Some(surface) = fullscreen.toplevel() {
                    self.request_fullscreen(surface, false);
                }
            }
        }
    }
}
