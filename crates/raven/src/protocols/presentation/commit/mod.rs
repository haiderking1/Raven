mod state;
use self::state::{Commit, CommitState, Tracking};
use crate::state::State;
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::{
        compositor::{add_post_commit_hook, add_pre_commit_hook, with_states},
        presentation::PresentationFeedbackCachedState,
    },
};

/// Smithay 0.7 retains old feedback when a replacement commit requests none.
/// Carry a marker through the same cached transaction, including acquire waits
/// and synchronized subsurfaces, rather than discarding during pre-commit.
pub(crate) fn install(surface: &WlSurface) {
    add_pre_commit_hook::<State, _>(surface, |_, _, surface| {
        with_states(surface, |states| {
            if !states.cached_state.has::<PresentationFeedbackCachedState>() {
                return;
            }
            let has_feedback = !states
                .cached_state
                .get::<PresentationFeedbackCachedState>()
                .pending()
                .callbacks
                .is_empty();
            let tracking = states.data_map.get_or_insert(Tracking::default);
            let mut serials = tracking
                .0
                .lock()
                .expect("presentation commit state poisoned");
            serials.next = serials.next.wrapping_add(1);
            states.cached_state.get::<CommitState>().pending().0 = Some(Commit {
                serial: serials.next,
                has_feedback,
            });
        });
    });
    add_post_commit_hook::<State, _>(surface, |_, _, surface| {
        with_states(surface, |states| {
            let Some(tracking) = states.data_map.get::<Tracking>() else {
                return;
            };
            let Some(commit) = states.cached_state.get::<CommitState>().current().0 else {
                return;
            };
            {
                let mut serials = tracking
                    .0
                    .lock()
                    .expect("presentation commit state poisoned");
                if serials.applied == Some(commit.serial) {
                    return;
                }
                serials.applied = Some(commit.serial);
            }
            if !commit.has_feedback {
                for feedback in states
                    .cached_state
                    .get::<PresentationFeedbackCachedState>()
                    .current()
                    .callbacks
                    .drain(..)
                {
                    feedback.discarded();
                }
            }
        });
    });
}
