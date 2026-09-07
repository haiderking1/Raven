use calloop::{
    Dispatcher, LoopHandle, RegistrationToken,
    timer::{TimeoutAction, Timer},
    transient::TransientSource,
};
use std::time::Instant;

/// A reusable registration, dormant after firing until explicitly rearmed.
pub(super) struct Wake<Data: 'static> {
    handle: LoopHandle<'static, Data>,
    source: Dispatcher<'static, TransientSource<Timer>, Data>,
    token: RegistrationToken,
}

impl<Data: 'static> Wake<Data> {
    pub fn new(
        handle: LoopHandle<'static, Data>,
        mut fired: impl FnMut(Instant, &mut Data) + 'static,
    ) -> calloop::Result<Self> {
        let source = Dispatcher::new(
            TransientSource::from(Timer::immediate()),
            move |deadline, _, data| {
                fired(deadline, data);
                TimeoutAction::Drop
            },
        );
        let token = handle.register_dispatcher(source.clone())?;
        Ok(Self {
            handle,
            source,
            token,
        })
    }

    /// Called after source dispatch, never while the timer itself is borrowed.
    pub fn arm(&mut self, deadline: Option<Instant>) -> calloop::Result<()> {
        let mut source = self.source.as_source_mut();
        let current = source.map(|timer| timer.current_deadline()).flatten();
        if current == deadline {
            return Ok(());
        }
        match deadline {
            Some(deadline) if source.is_none() => {
                *source = Timer::from_deadline(deadline).into();
            }
            Some(deadline) => source.replace(Timer::from_deadline(deadline)),
            None => source.remove(),
        }
        drop(source);
        self.handle.update(&self.token)
    }
}

impl<Data: 'static> Drop for Wake<Data> {
    fn drop(&mut self) {
        self.handle.remove(self.token);
    }
}

#[cfg(test)]
mod tests;
