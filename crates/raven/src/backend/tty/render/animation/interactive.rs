use super::direct::{self, Content};
use crate::{
    backend::tty::{TtyBackend, render::SceneElement},
    state::State,
};
use smithay::{
    backend::renderer::{gles::GlesRenderer, utils::CommitCounter},
    desktop::Window,
    utils::{Logical, Rectangle, Size},
};
use std::sync::Mutex;

#[derive(Default)]
struct LiveContent {
    content: Content,
    target: Option<Rectangle<i32, Logical>>,
    commit: CommitCounter,
}

impl TtyBackend {
    pub(crate) fn prepare_live_resize(window: &Window) {
        window
            .user_data()
            .insert_if_missing(|| Mutex::new(LiveContent::default()));
        let mut cache = window
            .user_data()
            .get::<Mutex<LiveContent>>()
            .unwrap()
            .lock()
            .unwrap();
        cache.content = Content::capture(window, Some(&cache.content));
    }

    pub(crate) fn live_resize_content_settled(window: &Window, target: Size<i32, Logical>) -> bool {
        window
            .user_data()
            .get::<Mutex<LiveContent>>()
            .is_some_and(|cache| {
                let cache = cache.lock().unwrap();
                cache.target.is_some_and(|frame| frame.size == target)
                    && cache.content.settled(window)
            })
    }
}

pub(in crate::backend::tty::render) fn append(
    renderer: &mut GlesRenderer,
    state: &State,
    window: &Window,
    area: Rectangle<i32, Logical>,
    scale: f64,
    elements: &mut Vec<SceneElement>,
) -> bool {
    if !state.window_has_live_resize(window) {
        return false;
    }
    let Some(target) = state.window_target_client_geometry(window) else {
        return false;
    };
    TtyBackend::prepare_live_resize(window);
    let (content, commit) = {
        let mut cache = window
            .user_data()
            .get::<Mutex<LiveContent>>()
            .unwrap()
            .lock()
            .unwrap();
        cache.target = Some(target);
        // The wrapper owns its damage counter, including newly committed client
        // pixels when pointer geometry has not changed. This never requests a frame.
        cache.commit.increment();
        (cache.content.clone(), cache.commit)
    };
    elements.extend(direct::append(
        renderer,
        state,
        window,
        area,
        scale,
        target,
        commit,
        &content,
        state.window_is_floating(window),
    ));
    true
}
