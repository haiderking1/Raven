use super::{button, geometry, motion, output};
use crate::{protocols::virtual_pointer::Event, state::State};
use smithay::{
    backend::input::ButtonState,
    input::pointer::RelativeMotionEvent,
    output::Output,
    reexports::wayland_server::backend::ObjectId,
    utils::{Clock, Monotonic},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::input) enum Source {
    Physical(String),
    Virtual(ObjectId),
}

impl State {
    pub(crate) fn virtual_pointer_event(
        &mut self,
        id: ObjectId,
        mapped_output: Option<&Output>,
        event: Event,
    ) {
        match event {
            Event::Motion { time, dx, dy } => {
                let Some(bounds) = output::bounds(self) else {
                    return;
                };
                let delta = (dx, dy).into();
                motion::send(
                    self,
                    geometry::clamp(self.pointer_location + delta, bounds),
                    time,
                    Some(RelativeMotionEvent {
                        delta,
                        delta_unaccel: delta,
                        utime: u64::from(time) * 1000,
                    }),
                    false,
                );
            }
            Event::Absolute {
                time,
                x,
                y,
                x_extent,
                y_extent,
            } => {
                let bounds = match mapped_output {
                    Some(output) => self.space().output_geometry(output),
                    None => output::bounds(self),
                };
                let Some(bounds) = bounds else {
                    return;
                };
                let Some(location) = geometry::absolute(x, y, x_extent, y_extent, bounds) else {
                    return;
                };
                motion::send(self, location, time, None, false);
            }
            Event::Button {
                time,
                button: code,
                pressed,
            } => {
                // wl_pointer button codes use Linux evdev's BTN range.
                if !(0x100..=0x2ff).contains(&code) {
                    return;
                }
                if self
                    .input
                    .pointer_buttons
                    .change(Source::Virtual(id), code, pressed)
                {
                    button::send(
                        self,
                        code,
                        if pressed {
                            ButtonState::Pressed
                        } else {
                            ButtonState::Released
                        },
                        time,
                        false,
                    );
                }
            }
            Event::Frame(axis) => {
                if let Some(pointer) = self.seat.get_pointer() {
                    if let Some(axis) = axis {
                        pointer.axis(self, axis);
                    }
                    pointer.frame(self);
                }
            }
        }
    }

    pub(crate) fn remove_virtual_pointer(&mut self, id: ObjectId) {
        let released = self.input.pointer_buttons.remove(&Source::Virtual(id));
        self.release_pointer_buttons(released);
    }
    pub(crate) fn remove_physical_pointer(&mut self, name: &str) {
        let released = self
            .input
            .pointer_buttons
            .remove(&Source::Physical(name.to_owned()));
        self.release_pointer_buttons(released);
    }
    pub(crate) fn suspend_pointer_input(&mut self) {
        self.input.virtual_pointer_epoch = self.input.virtual_pointer_epoch.wrapping_add(1);
        let released = self.input.pointer_buttons.clear();
        self.release_pointer_buttons(released);
    }
    fn release_pointer_buttons(&mut self, released: Vec<u32>) {
        if released.is_empty() {
            return;
        }
        let time = Clock::<Monotonic>::new().now().as_millis();
        for code in released {
            button::send(self, code, ButtonState::Released, time, false);
        }
        if let Some(pointer) = self.seat.get_pointer() {
            pointer.frame(self);
        }
    }
}
