use super::blocker::AcquireBlocker;
use crate::state::{ClientState, State};
use smithay::{
    reexports::{
        calloop::Interest,
        wayland_server::{
            Client, DisplayHandle, Resource, backend::protocol::ProtocolError,
            protocol::wl_surface::WlSurface,
        },
    },
    wayland::{
        compositor::{
            BufferAssignment, SurfaceAttributes, add_blocker, add_destruction_hook,
            add_pre_commit_hook, with_states,
        },
        dmabuf::get_dmabuf,
    },
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

/// Call once from CompositorHandler::new_surface, for every surface role.
/// CompositorHandler::commit is too late: Smithay has already applied state.
pub(crate) fn install(surface: &WlSurface) {
    add_pre_commit_hook::<State, _>(surface, pre_commit);
    add_destruction_hook::<State, _>(surface, |state, surface| {
        if let Some(backend) = &mut state.backend {
            backend.cancel_dmabuf_surface(&surface.id());
        }
        // A destroyed synchronized child must not stall its parent's transaction.
        if let Some(client) = surface.client() {
            wake_client(state, &client);
        }
    });
}

fn wake_client(state: &mut State, client: &Client) {
    if let Some(client_state) = client.get_data::<ClientState>() {
        let display = state.display_handle.clone();
        client_state
            .compositor_state
            .blocker_cleared(state, &display);
    }
}

fn pre_commit(state: &mut State, display: &DisplayHandle, surface: &WlSurface) {
    // Do not take the assignment: the renderer's commit handler still owns it.
    let dmabuf = with_states(surface, |states| {
        let mut attributes = states.cached_state.get::<SurfaceAttributes>();
        match &attributes.pending().buffer {
            Some(BufferAssignment::NewBuffer(buffer)) => get_dmabuf(buffer).ok().cloned(),
            _ => None,
        }
    });
    let Some(dmabuf) = dmabuf else {
        return;
    };
    let Some(client) = surface.client() else {
        return;
    };
    // READ waits for all planes' last writers, not readers. Generate a fresh
    // source for each attachment; an earlier import/readiness result is stale.
    let Ok((fence, source)) = dmabuf.generate_blocker(Interest::READ) else {
        return; // Smithay polled the planes and found them already ready.
    };
    let mut resize_acquire = Some(crate::desktop::resize::acquire_started(surface));
    let cancelled = Arc::new(AtomicBool::new(false));
    let blocker = AcquireBlocker {
        fence,
        surface: surface.downgrade(),
        cancelled: cancelled.clone(),
    };
    let wake = client.clone();
    let result = match state.backend.as_mut() {
        Some(backend) => {
            backend.wait_for_dmabuf(source, surface.id(), cancelled.clone(), move |_, state| {
                // DmabufSource releases its blocker before invoking this callback.
                if let Some(acquire) = resize_acquire.take() {
                    acquire.complete(state);
                }
                wake_client(state, &wake);
                Ok(())
            })
        }
        None => Err(std::io::Error::other("DMA-BUF backend is unavailable")),
    };
    if let Err(error) = result {
        // Never apply an unready buffer when registering its fd fails.
        cancelled.store(true, Ordering::SeqCst);
        client.kill(
            display,
            ProtocolError {
                code: 3, // wl_display.error.implementation
                object_id: 1,
                object_interface: "wl_display".into(),
                message: format!("cannot monitor DMA-BUF acquire readiness: {error}"),
            },
        );
    }
    add_blocker(surface, blocker);
}
