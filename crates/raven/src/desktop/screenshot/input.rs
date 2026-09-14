use super::Phase;
use crate::state::State;
use smithay::{
    backend::input::ButtonState,
    utils::{Logical, Physical, Point},
};
impl State {
    fn screenshot_point(&self) -> Point<f64, Physical> {
        let p = self.pointer_location - self.screenshot.area.loc.to_f64();
        let mut result = p.to_physical(self.screenshot.scale);
        if p.x >= f64::from(self.screenshot.area.size.w - 1) {
            result.x = f64::from(self.screenshot.size.w - 1);
        }
        if p.y >= f64::from(self.screenshot.area.size.h - 1) {
            result.y = f64::from(self.screenshot.size.h - 1);
        }
        result
    }
    pub(crate) fn screenshot_motion(&mut self, point: Point<f64, Logical>) -> bool {
        if !self.screenshot.active() {
            return false;
        }
        if self.pointer_location == point {
            return true;
        }
        self.pointer_location = point;
        let position = self.screenshot_point();
        let changed = self.screenshot.selection_mut().is_some_and(|selection| {
            let previous = selection.rect;
            selection.motion(position);
            selection.rect != previous
        });
        if changed {
            self.screenshot.commit.increment();
        }
        self.request_redraw();
        true
    }
    pub(crate) fn screenshot_button(&mut self, button: u32, state: ButtonState) -> bool {
        if state == ButtonState::Released {
            if !self.screenshot.buttons.remove(&button) {
                return false;
            }
            let point = self.screenshot_point();
            if button == 0x110 {
                if let Some(selection) = self.screenshot.selection_mut() {
                    selection.end(point);
                }
            }
        } else {
            if !self.screenshot.active() {
                return false;
            }
            self.screenshot.buttons.insert(button);
            if button == 0x111 {
                self.cancel_screenshot();
            }
            if button == 0x110 {
                let point = self.screenshot_point();
                if let Some(selection) = self.screenshot.selection_mut() {
                    selection.begin(point);
                }
            }
        }
        self.screenshot.commit.increment();
        self.request_redraw();
        true
    }
}

impl super::Screenshot {
    fn selection_mut(&mut self) -> Option<&mut super::model::Selection> {
        match &mut self.phase {
            Phase::Requested => self.pending_selection.as_mut(),
            Phase::Selecting(s) => Some(s),
            _ => None,
        }
    }
}
