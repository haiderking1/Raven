mod apply;
mod check;
mod commands;
pub(super) use check::run_if_requested as check_if_requested;
mod diagnostic;
pub(crate) mod dialog;
mod files;
mod lua;
mod watch;

use crate::{backend::tty::PreparedCursor, runtime::settings::Settings};
pub(crate) use apply::dispatch;
use std::{ffi::OsString, path::PathBuf, sync::mpsc::Sender};
pub(super) use watch::Service;

pub(crate) struct Prepared {
    pub settings: Settings,
    pub cursor: PreparedCursor,
}

pub(crate) struct Runtime {
    pub(crate) settings: Settings,
    pending: Option<Result<Prepared, String>>,
    pub(crate) initial_cursor: Option<PreparedCursor>,
    socket: Option<OsString>,
    dialog: dialog::Dialog,
    control: Option<Sender<watch::Control>>,
}
impl Runtime {
    pub(crate) fn startup_error(&mut self, error: String) {
        self.pending = Some(Err(format!(
            "Startup configuration failed; built-in defaults are active.\n\n{error}"
        )));
    }
    pub(crate) fn request_reload(&self) {
        if let Some(control) = &self.control {
            let _ = control.send(watch::Control::Reload);
        }
    }
}

pub(super) fn initial(
    defaults: Settings,
    use_file: bool,
) -> Result<(Prepared, Option<PathBuf>, Option<String>), Box<dyn std::error::Error>> {
    if !use_file {
        defaults.validate()?;
        let cursor = PreparedCursor::load(defaults.cursor.clone())?;
        return Ok((
            Prepared {
                settings: defaults,
                cursor,
            },
            None,
            None,
        ));
    }
    let path = files::path()?;
    let result = files::ensure(&path)
        .map_err(|e| format!("{}: {e}", path.display()))
        .and_then(|()| files::read(&path))
        .and_then(|source| lua::parse(&source, &path, defaults.clone()));
    match result {
        Ok(prepared) => Ok((prepared, Some(path), None)),
        Err(error) => {
            let cursor = PreparedCursor::load(defaults.cursor.clone())?;
            Ok((
                Prepared {
                    settings: defaults,
                    cursor,
                },
                Some(path),
                Some(error),
            ))
        }
    }
}
