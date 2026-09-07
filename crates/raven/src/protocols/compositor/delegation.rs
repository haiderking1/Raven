//! Keep Smithay dispatch unchanged except the role-destruction notification.
use crate::state::State;
use smithay::{
    reexports::wayland_server::{
        delegate_dispatch, delegate_global_dispatch,
        protocol::{
            wl_callback::WlCallback, wl_compositor::WlCompositor, wl_region::WlRegion,
            wl_subcompositor::WlSubcompositor, wl_surface::WlSurface,
        },
    },
    wayland::compositor::{CompositorState, RegionUserData, SurfaceUserData},
};

delegate_global_dispatch!(State: [WlCompositor: ()] => CompositorState);
delegate_global_dispatch!(State: [WlSubcompositor: ()] => CompositorState);
delegate_dispatch!(State: [WlCompositor: ()] => CompositorState);
delegate_dispatch!(State: [WlSubcompositor: ()] => CompositorState);
delegate_dispatch!(State: [WlSurface: SurfaceUserData] => CompositorState);
delegate_dispatch!(State: [WlRegion: RegionUserData] => CompositorState);
delegate_dispatch!(State: [WlCallback: ()] => CompositorState);
