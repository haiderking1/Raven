use std::process::Child;

// The regression waits on private owned handles, without polling or a compositor.
#[cfg(test)]
#[path = "../startup/tests/mod.rs"]
mod startup_tests;

/// Own only processes Raven started, never arbitrary connected Wayland clients.
#[derive(Default)]
pub(super) struct Children(Vec<Child>);

impl Children {
    pub(super) fn track(&mut self, child: Child) {
        self.0.push(child);
    }

    pub(super) fn reap(&mut self) {
        // SIGCHLD can coalesce, so check every owned child without blocking.
        self.0.retain_mut(|child| match child.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(error) => {
                eprintln!("raven: cannot reap child {}: {error}", child.id());
                true
            }
        });
    }
}

impl Drop for Children {
    fn drop(&mut self) {
        for child in &mut self.0 {
            if matches!(child.try_wait(), Ok(None)) {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }
}
