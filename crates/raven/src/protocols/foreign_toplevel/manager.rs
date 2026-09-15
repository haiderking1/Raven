use super::{Manager, Subscription};
use crate::state::State;
use smithay::reexports::{
    wayland_protocols_wlr::foreign_toplevel::v1::server::zwlr_foreign_toplevel_manager_v1::Request,
    wayland_server::{Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New},
};
impl GlobalDispatch<Manager, ()> for State {
    fn bind(
        state: &mut Self,
        _: &DisplayHandle,
        client: &Client,
        resource: New<Manager>,
        _: &(),
        init: &mut DataInit<'_, Self>,
    ) {
        let manager = init.init(resource, ());
        if state.foreign_toplevel.subscriptions.len() >= 32 {
            manager.finished();
            return;
        }
        state.foreign_toplevel.subscriptions.push(Subscription {
            manager,
            client: client.clone(),
            stopped: false,
            entries: Vec::new(),
        });
        state.refresh_foreign_toplevels();
    }
}
impl Dispatch<Manager, ()> for State {
    fn request(
        state: &mut Self,
        _: &Client,
        resource: &Manager,
        request: Request,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        match request {
            Request::Stop => {
                if let Some(s) = state
                    .foreign_toplevel
                    .subscriptions
                    .iter_mut()
                    .find(|s| s.manager == *resource)
                {
                    s.stopped = true;
                    s.manager.finished();
                }
            }
            _ => unreachable!(),
        }
    }
}
