use super::{Clients, spawn};
use crate::runtime::startup::ValidatedStartupPlan;

impl Clients {
    /// Attempt every entry once for this owner, including failures. Reloading or
    /// passing another plan cannot restart startup children within this session.
    pub(crate) fn start_startup(&mut self, plan: &ValidatedStartupPlan) {
        if std::mem::replace(&mut self.startup_started, true) {
            return;
        }
        let display = self
            .satellite
            .as_ref()
            .and_then(super::satellite::Satellite::display);
        for (index, entry) in plan.entries().iter().enumerate() {
            let result = match entry {
                Ok(entry) => spawn::client(
                    &entry.argv,
                    entry.cwd.as_deref(),
                    &entry.env,
                    &self.socket,
                    display,
                ),
                Err(error) => Err(error.clone().into()),
            };
            match result {
                Ok(Some(child)) => self.children.track(child),
                Ok(None) => unreachable!("startup argv was validated"),
                Err(error) => eprintln!("raven: startup entry {} failed: {error}", index + 1),
            }
        }
    }
}
