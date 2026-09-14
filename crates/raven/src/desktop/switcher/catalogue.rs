use super::model::Candidate;
use crate::state::State;
use smithay::{
    desktop::Window,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::IsAlive,
    wayland::{compositor::with_states, shell::xdg::XdgToplevelSurfaceData},
};

struct AnonymousApp(u64);
static NEXT_ANONYMOUS_APP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

impl State {
    pub(crate) fn remember_app_focus(&mut self, surface: Option<&WlSurface>) {
        if self.switcher.activating || self.switcher.pending.is_some() {
            return;
        }
        let Some(surface) = surface else {
            return;
        };
        let window = self
            .windows
            .iter()
            .find(|window| {
                let mut owns = false;
                window.with_surfaces(|candidate, _| owns |= candidate == surface);
                owns
            })
            .cloned();
        if let Some(window) = window.filter(|w| !self.is_configuration_error(w)) {
            super::model::remember(&mut self.switcher.history, window);
        }
    }
    pub(super) fn switcher_candidates(&self) -> Vec<Candidate<Window>> {
        self.windows
            .iter()
            .filter_map(|window| {
                if !window.alive() || self.is_configuration_error(window) {
                    return None;
                }
                let workspace = self.workspaces.index_of(window)?;
                self.workspaces.entries[workspace]
                    .space
                    .element_location(window)?;
                let surface = window.toplevel()?.wl_surface();
                let (app, title) = with_states(surface, |states| {
                    let role = states
                        .data_map
                        .get::<XdgToplevelSurfaceData>()?
                        .lock()
                        .ok()?;
                    Some((
                        role.app_id.clone().unwrap_or_default(),
                        role.title.clone().unwrap_or_default(),
                    ))
                })?;
                let app = app.trim().to_lowercase();
                let app = app.strip_suffix(".desktop").unwrap_or(&app).to_owned();
                // Missing app IDs must never collapse unrelated clients into one app.
                let app = if app.is_empty() {
                    window.user_data().insert_if_missing(|| {
                        AnonymousApp(
                            NEXT_ANONYMOUS_APP.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                        )
                    });
                    format!(
                        "raven:window:{}",
                        window.user_data().get::<AnonymousApp>().unwrap().0
                    )
                } else {
                    app
                };
                Some(Candidate {
                    window: window.clone(),
                    app,
                    title: if title.is_empty() {
                        "Application".into()
                    } else {
                        title.chars().take(512).collect()
                    },
                })
            })
            .collect()
    }
}
