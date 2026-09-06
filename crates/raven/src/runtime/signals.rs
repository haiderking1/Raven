use crate::state::State;
use calloop::{
    LoopHandle,
    signals::{Signal, Signals},
};
use std::{error::Error, io};

pub(super) fn install(handle: LoopHandle<'static, State>) -> Result<(), Box<dyn Error>> {
    handle.insert_source(
        Signals::new(&[
            Signal::SIGINT,
            Signal::SIGTERM,
            Signal::SIGHUP,
            Signal::SIGCHLD,
        ])?,
        |event, _, state| {
            if event.signal() == Signal::SIGCHLD {
                if let Some(clients) = state.clients.as_mut() {
                    clients.reap();
                }
            } else {
                state.loop_signal.stop();
                state.loop_signal.wakeup();
            }
        },
    )?;
    Ok(())
}

// The child must not inherit calloop's blocked termination or SIGCHLD signals.
pub(super) fn unblock_in_child() -> io::Result<()> {
    // SAFETY: sigemptyset initializes the mask before use. sigprocmask is
    // async-signal-safe, touches only the calling child thread, and receives
    // valid pointers. This function allocates nothing before a failed syscall.
    unsafe {
        let mut mask = std::mem::MaybeUninit::<libc::sigset_t>::uninit();
        if libc::sigemptyset(mask.as_mut_ptr()) != 0 {
            return Err(io::Error::last_os_error());
        }
        if libc::sigprocmask(libc::SIG_SETMASK, mask.as_ptr(), std::ptr::null_mut()) != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}
