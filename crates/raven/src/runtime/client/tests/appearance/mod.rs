use super::server::Server;
use smithay::{
    desktop::Window,
    utils::{Logical, Rectangle},
};
use std::error::Error;

/// Hard-coded expected pixels catch accidental changes to the default appearance.
pub(super) fn tiled(
    server: &Server,
    window: &Window,
    workarea: Rectangle<i32, Logical>,
) -> Result<(), Box<dyn Error>> {
    let appearance = server.state.appearance();
    let frame = Rectangle::new(
        workarea.loc + smithay::utils::Point::from((8, 8)),
        (workarea.size.w - 16, workarea.size.h - 16).into(),
    );
    let client = Rectangle::new(
        frame.loc + smithay::utils::Point::from((2, 2)),
        (frame.size.w - 4, frame.size.h - 4).into(),
    );
    if appearance.inner.horizontal != 8
        || appearance.inner.vertical != 8
        || [
            appearance.outer.top,
            appearance.outer.right,
            appearance.outer.bottom,
            appearance.outer.left,
        ] != [8; 4]
        || appearance.border.width != 2
        || server.state.window_frame_geometry(window) != Some(frame)
        || server.state.window_client_geometry(window) != Some(client)
        || server.state.window_tile_geometry(window) != Some(client)
        || server.state.space().element_location(window) != Some(client.loc - window.geometry().loc)
        || window.geometry().size != client.size
        || window
            .toplevel()
            .is_none_or(|top| top.current_state().size != Some(client.size))
    {
        return Err(format!(
            "default appearance mismatch: frame={:?}, client={:?}, expected={frame:?}/{client:?}",
            server.state.window_frame_geometry(window),
            server.state.window_client_geometry(window)
        )
        .into());
    }
    Ok(())
}
