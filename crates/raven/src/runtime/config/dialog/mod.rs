mod connection;
use std::{
    ffi::CString,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    os::{fd::FromRawFd, unix::process::CommandExt},
    process::{Child, Command, Stdio},
};

unsafe extern "C" {
    fn raven_config_dialog(message: *const std::ffi::c_char) -> std::ffi::c_int;
}

pub(crate) fn run_if_requested() -> Option<Result<(), Box<dyn std::error::Error>>> {
    if std::env::args_os().nth(1).as_deref() != Some(std::ffi::OsStr::new("--config-error-ui")) {
        return None;
    }
    Some((|| {
        let mut text = String::new();
        std::io::stdin()
            .take(1024 * 1024)
            .read_to_string(&mut text)?;
        let text = CString::new(text.replace(char::from(0), "[NUL]"))?;
        // SAFETY: the message stays alive until GTK's main loop returns. GTK is
        // initialized only in this separate process, never in the compositor.
        let result = unsafe { raven_config_dialog(text.as_ptr()) };
        if result != 0 {
            return Err(format!("configuration window exited with {result}").into());
        }
        Ok(())
    })())
}

#[derive(Default)]
pub(super) struct Dialog {
    child: Option<Child>,
    retired: Vec<Child>,
    message: Option<String>,
}
impl Dialog {
    pub fn reap(&mut self) {
        if self
            .child
            .as_mut()
            .is_some_and(|child| child.try_wait().ok().flatten().is_some())
        {
            self.child = None;
        }
        self.retired
            .retain_mut(|child| child.try_wait().ok().flatten().is_none());
    }
    pub fn close(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            self.retired.push(child);
        }
        self.message = None;
    }
    pub fn show(
        &mut self,
        message: String,
        socket: &std::ffi::OsStr,
        display: &mut smithay::reexports::wayland_server::DisplayHandle,
    ) -> std::io::Result<()> {
        self.reap();
        if self.child.is_some() && self.message.as_ref() == Some(&message) {
            return Ok(());
        }
        self.close();
        // An anonymous in-memory file avoids disk files and blocking pipe writes.
        // SAFETY: the NUL-terminated name is static; a successful fd is owned once.
        let fd = unsafe { libc::memfd_create(c"raven-config-error".as_ptr(), libc::MFD_CLOEXEC) };
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        let mut report = unsafe { File::from_raw_fd(fd) };
        report.write_all(message.as_bytes())?;
        report.seek(SeekFrom::Start(0))?;
        let mut command = Command::new(std::env::current_exe()?);
        command
            .arg("--config-error-ui")
            .stdin(Stdio::from(report))
            .env("WAYLAND_DISPLAY", socket)
            .env("GDK_BACKEND", "wayland")
            .env("XDG_CURRENT_DESKTOP", "Raven")
            .env_remove("WAYLAND_SOCKET");
        // SAFETY: only async-signal-safe signal-mask operations run after fork.
        unsafe {
            command.pre_exec(crate::runtime::signals::unblock_in_child);
        }
        let connection = connection::attach(display, &mut command)?;
        self.child = Some(command.spawn()?);
        drop(connection);
        self.message = Some(message);
        Ok(())
    }
}
impl Drop for Dialog {
    fn drop(&mut self) {
        self.close();
        for child in &mut self.retired {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
