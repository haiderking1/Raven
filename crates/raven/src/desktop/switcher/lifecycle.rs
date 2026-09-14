use super::layout::Layout;
use crate::state::State;
use smithay::{
    backend::input::ButtonState,
    utils::{IsAlive, Logical, Point},
};

impl State {
    pub(crate) fn update_switcher_artwork(&mut self) {
        let Some(session) = &self.switcher.session else {
            return;
        };
        let Some(output) = &self.output else {
            self.cancel_app_switcher();
            return;
        };
        let Some(area) = self.space().output_geometry(output) else {
            self.cancel_app_switcher();
            return;
        };
        let Some(layout) = Layout::new(area, session.apps.len(), session.selected) else {
            self.cancel_app_switcher();
            return;
        };
        let scale = output
            .current_scale()
            .fractional_scale()
            .ceil()
            .clamp(1.0, 4.0) as i32;
        let apps = session
            .apps
            .iter()
            .map(|app| (app.id.clone(), app.title.clone()))
            .collect();
        self.switcher.artwork.request(
            layout,
            scale,
            session.selected,
            apps,
            self.loop_signal.clone(),
        );
        self.request_redraw();
    }
    pub(crate) fn refresh_app_switcher(&mut self) {
        self.switcher.history.retain(|window| window.alive());
        if self.switcher.active()
            && (self.exclusive_keyboard_layer().is_some()
                || self.seat.get_keyboard().is_some_and(|k| k.is_grabbed())
                || self.seat.get_pointer().is_some_and(|p| p.is_grabbed()))
        {
            self.cancel_app_switcher();
        }
        let workspaces = &self.workspaces;
        let changed = self.switcher.session.as_mut().is_some_and(|s| {
            s.retain(|w| {
                w.alive()
                    && workspaces.index_of(w).is_some_and(|index| {
                        workspaces.entries[index]
                            .space
                            .element_location(w)
                            .is_some()
                    })
            })
        });
        if self
            .switcher
            .session
            .as_ref()
            .is_some_and(|s| s.apps.is_empty())
        {
            self.cancel_app_switcher();
        } else if changed {
            self.switcher.artwork.hide();
            self.update_switcher_artwork();
        }
        match self.switcher.artwork.poll() {
            Ok(true) => self.request_redraw(),
            Err(error) => {
                eprintln!("raven: {error}");
                self.cancel_app_switcher();
            }
            _ => {}
        }
        self.refresh_switcher_activation();
    }
    pub(crate) fn switcher_motion(&mut self, point: Point<f64, Logical>) -> bool {
        if !self.switcher.active() {
            return false;
        }
        let changed = self.pointer_location != point;
        self.pointer_location = point;
        if changed {
            self.request_redraw();
            if let Some(index) = self
                .switcher
                .artwork
                .frame
                .as_ref()
                .and_then(|frame| frame.layout.hit(point))
            {
                if let Some(session) = &mut self.switcher.session {
                    if session.selected != index {
                        session.selected = index;
                        self.update_switcher_artwork();
                    }
                }
            }
        }
        true
    }
    pub(crate) fn switcher_button(&mut self, button: u32, state: ButtonState) -> bool {
        if state == ButtonState::Released {
            return self.switcher.swallowed_buttons.remove(&button);
        }
        self.switcher.pending = None;
        self.switcher.exiting = None;
        if !self.switcher.active() {
            return false;
        }
        self.switcher.swallowed_buttons.insert(button);
        let hit = self
            .switcher
            .artwork
            .frame
            .as_ref()
            .and_then(|frame| frame.layout.hit(self.pointer_location));
        if button == 0x110 && hit.is_some() {
            if let Some(session) = &mut self.switcher.session {
                session.selected = hit.unwrap();
            }
            self.confirm_app_switcher();
        } else {
            self.cancel_app_switcher();
        }
        true
    }
}
