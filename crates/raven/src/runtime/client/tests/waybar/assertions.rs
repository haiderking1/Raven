use super::super::{appearance, server::Server};
use smithay::{
    backend::renderer::utils::with_renderer_surface_state,
    desktop::{LayerSurface, Window, layer_map_for_output},
    utils::{Logical, Rectangle},
    wayland::shell::wlr_layer::Layer,
};
use std::error::Error;

pub(super) fn mapped(server: &Server) -> Option<LayerSurface> {
    let map = layer_map_for_output(server.state.output.as_ref()?);
    let mut layers = map.layers_on(Layer::Top);
    let layer = layers.next()?.clone();
    if layers.next().is_some()
        || map.len() != 1
        || layer.namespace() != "waybar"
        || !server.state.layer_is_mapped(&layer)
        || !with_renderer_surface_state(layer.wl_surface(), |state| state.buffer().is_some())
            .unwrap_or(false)
    {
        return None;
    }
    Some(layer)
}

pub(super) fn workarea(server: &Server) -> Rectangle<i32, Logical> {
    Rectangle::new(
        (0, 32).into(),
        (server.area.size.w, server.area.size.h - 32).into(),
    )
}

pub(super) fn reserved(
    server: &Server,
    main: &Window,
    layer: &LayerSurface,
) -> Result<(), Box<dyn Error>> {
    let map = layer_map_for_output(server.state.output.as_ref().ok_or("output disappeared")?);
    if map.layer_geometry(layer) != Some(Rectangle::new((0, 0).into(), (960, 32).into()))
        || map.non_exclusive_zone() != workarea(server)
        || !server.state.layer_is_visible(layer)
        || server.scene_size != 6
    {
        return Err(format!("Waybar must reserve 32px and import one bar surface plus client and four borders: geometry={:?}, zone={:?}, scene={}", map.layer_geometry(layer), map.non_exclusive_zone(), server.scene_size).into());
    }
    drop(map);
    appearance::tiled(server, main, workarea(server))?;
    membership(server, main, 0)
}

pub(super) fn membership(
    server: &Server,
    main: &Window,
    active: usize,
) -> Result<(), Box<dyn Error>> {
    let state = &server.state;
    if state.workspaces.active != active
        || state.workspaces.index_of(main) != Some(0)
        || state.windows.as_slice() != [main.clone()]
        || state.window_is_floating(main)
        || state.workspace_floating_count(0) != 0
        || state.window_tile_geometry(main).is_none()
    {
        return Err("Waybar activation changed original tile membership".into());
    }
    let visible: Vec<_> = state.visible_windows().cloned().collect();
    let expected = if active == 0 {
        vec![main.clone()]
    } else {
        vec![]
    };
    if visible != expected || state.space().elements().count() != expected.len() {
        return Err("Waybar activation changed tile order or visible workspace contents".into());
    }
    Ok(())
}

pub(super) fn fullscreen(
    server: &Server,
    main: &Window,
    layer: &LayerSurface,
) -> Result<(), Box<dyn Error>> {
    if !server.state.layer_is_mapped(layer)
        || server.state.layer_is_visible(layer)
        || server.state.window_frame_geometry(main) != Some(server.area)
        || server.state.window_client_geometry(main) != Some(server.area)
        || server.scene_size != 1
        || server
            .state
            .surface_under((24.0, 16.0).into())
            .map(|hit| hit.0)
            .as_ref()
            != main.toplevel().map(|top| top.wl_surface())
    {
        return Err(
            "fullscreen must hide mapped Top Waybar and import only the gapless, borderless owner"
                .into(),
        );
    }
    Ok(())
}
