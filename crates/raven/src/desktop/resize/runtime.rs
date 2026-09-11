use crate::state::State;
use smithay::reexports::calloop::{
    LoopHandle,
    ping::make_ping,
    timer::{TimeoutAction, Timer},
};
use std::{error::Error, time::Instant};

impl State {
    /// Install before dispatching clients. Callback pacing timers share this handle
    /// but run independently of transaction readiness and the render scheduler.
    pub(crate) fn install_resize_transactions(
        &mut self,
        handle: LoopHandle<'static, State>,
    ) -> Result<(), Box<dyn Error>> {
        if self.resize.handle.is_some() {
            return Err("resize transaction sources are already installed".into());
        }
        let (ping, source) = make_ping()?;
        handle.insert_source(source, |(), _, state| state.drive_resize_transactions())?;
        self.resize.ping = Some(ping);
        self.resize.handle = Some(handle);
        Ok(())
    }

    pub(crate) fn wake_resize_transactions(&self) {
        if let Some(ping) = &self.resize.ping {
            ping.ping();
        }
    }

    pub(super) fn arm_resize_deadline(&mut self) {
        if self.resize.timer.is_some() {
            return;
        }
        let Some(batch) = &self.resize.batch else {
            return;
        };
        let Some(handle) = &self.resize.handle else {
            // State embedders must install the source. Never leave a gate with
            // no possible deadline if setup was omitted.
            eprintln!("raven: resize transaction event source is not installed");
            self.release_resize_transaction();
            return;
        };
        let gate = batch.released.clone();
        let result =
            handle.insert_source(Timer::from_deadline(batch.deadline), move |_, _, state| {
                if state
                    .resize
                    .batch
                    .as_ref()
                    .is_some_and(|batch| std::sync::Arc::ptr_eq(&batch.released, &gate))
                {
                    state.resize.timer = None;
                    state.release_resize_transaction();
                }
                TimeoutAction::Drop
            });
        match result {
            Ok(token) => self.resize.timer = Some(token),
            Err(error) => {
                eprintln!(
                    "raven: cannot register resize deadline; releasing coordination: {error}"
                );
                self.release_resize_transaction();
            }
        }
    }

    pub(super) fn drive_resize_transactions(&mut self) {
        if self.resize.depth != 0 || self.resize.releasing {
            return;
        }
        self.notify_fullscreen_resize_clients();
        let Some(batch) = &self.resize.batch else {
            return;
        };
        let ready = self.resize.held.values().all(|held| {
            held.configure.is_none()
                || held.applied
                || held
                    .ready
                    .as_ref()
                    .is_some_and(|waits| waits.iter().all(|wait| wait.is_ready()))
        });
        if ready || Instant::now() >= batch.deadline {
            self.release_resize_transaction();
        }
    }
}
