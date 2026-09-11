mod mode;
mod parent;
mod popup;
mod position;

use crate::state::State;
use smithay::{
    delegate_xdg_shell,
    desktop::Window,
    reexports::wayland_server::protocol::{wl_output::WlOutput, wl_seat::WlSeat},
    utils::Serial,
    wayland::shell::xdg::{
        PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
    },
};

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        // The first configure follows the initial wl_surface.commit, not get_toplevel.
        let window = Window::new_wayland_window(surface);
        self.workspaces
            .assign(window.clone(), self.workspaces.active);
        self.windows.push(window);
    }

    fn parent_changed(&mut self, surface: ToplevelSurface) {
        self.reparent_toplevel(surface);
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        self.cancel_surface_animation(surface.wl_surface());
        let window = self
            .windows
            .iter()
            .find(|w| w.toplevel() == Some(&surface))
            .cloned();
        if let Some(window) = window {
            self.remove_window(&window);
        }
    }

    fn ack_configure(
        &mut self,
        surface: smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
        configure: smithay::wayland::shell::xdg::Configure,
    ) {
        if let smithay::wayland::shell::xdg::Configure::Toplevel(configure) = configure {
            self.acknowledge_resize_animation(&surface, configure);
        }
    }

    fn maximize_request(&mut self, surface: ToplevelSurface) {
        mode::reply_unchanged(&surface);
    }

    fn unmaximize_request(&mut self, surface: ToplevelSurface) {
        mode::reply_unchanged(&surface);
    }

    fn fullscreen_request(&mut self, surface: ToplevelSurface, _output: Option<WlOutput>) {
        // Optional output hints are preferences. Raven uses its sole live output
        // and never retains or dereferences a possibly destroyed client resource.
        self.request_fullscreen(&surface, true);
    }

    fn unfullscreen_request(&mut self, surface: ToplevelSurface) {
        self.request_fullscreen(&surface, false);
    }

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        self.position_popup(&surface, positioner);
        if self
            .popup_manager
            .track_popup(surface.clone().into())
            .is_err()
        {
            surface.send_popup_done();
        }
    }

    fn grab(&mut self, surface: PopupSurface, seat: WlSeat, serial: Serial) {
        self.grab_popup(surface, seat, serial);
    }

    fn reposition_request(
        &mut self,
        surface: PopupSurface,
        positioner: PositionerState,
        token: u32,
    ) {
        self.position_popup(&surface, positioner);
        surface.send_repositioned(token);
    }

    fn popup_destroyed(&mut self, _surface: PopupSurface) {
        self.request_redraw();
        self.popup_manager.cleanup();
    }
}
delegate_xdg_shell!(State);
