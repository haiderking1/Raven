use smithay::desktop::Window;
use std::sync::Mutex;
#[derive(Default)]
pub(super) struct Minimized {
    pub requested: bool,
    pub hidden: bool,
    pub tile: Option<usize>,
}
pub(super) fn with<T>(window: &Window, f: impl FnOnce(&mut Minimized) -> T) -> T {
    window
        .user_data()
        .insert_if_missing(|| Mutex::new(Minimized::default()));
    f(&mut window
        .user_data()
        .get::<Mutex<Minimized>>()
        .unwrap()
        .lock()
        .unwrap())
}
pub(crate) fn hidden(window: &Window) -> bool {
    window
        .user_data()
        .get::<Mutex<Minimized>>()
        .is_some_and(|v| v.lock().unwrap().hidden)
}
pub(crate) fn tile(window: &Window) -> Option<usize> {
    window
        .user_data()
        .get::<Mutex<Minimized>>()
        .and_then(|v| v.lock().unwrap().tile)
}
pub(crate) fn remember_tile(window: &Window, slot: Option<usize>) {
    with(window, |v| v.tile = slot);
}
pub(crate) fn clear(window: &Window) {
    if let Some(v) = window.user_data().get::<Mutex<Minimized>>() {
        *v.lock().unwrap() = Minimized::default();
    }
}
