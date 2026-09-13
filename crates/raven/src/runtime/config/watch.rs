use super::{Prepared, files, lua};
use crate::{runtime::settings::Settings, state::State};
use calloop::{LoopHandle, RegistrationToken, channel};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

pub(super) enum Control {
    Changed,
    Reload,
    Stop,
}

pub(in crate::runtime) struct Service {
    control: Sender<Control>,
    worker: Option<JoinHandle<()>>,
    handle: LoopHandle<'static, State>,
    token: RegistrationToken,
}
impl Service {
    pub fn install(
        path: PathBuf,
        defaults: Settings,
        state: &mut State,
        handle: LoopHandle<'static, State>,
        socket: std::ffi::OsString,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (control, requests) = mpsc::channel();
        let (results, receiver) = channel::channel::<Result<Prepared, String>>();
        let token = handle
            .insert_source(receiver, |event, _, state| {
                if let channel::Event::Msg(result) = event {
                    state.config.pending = Some(result);
                }
            })
            .map_err(|e| e.error)?;
        let notifications = control.clone();
        let worker = match thread::Builder::new()
            .name("raven-config".into())
            .spawn(move || {
                run(&path, defaults, requests, notifications, results);
            }) {
            Ok(worker) => worker,
            Err(error) => {
                handle.remove(token);
                return Err(error.into());
            }
        };
        state.config.socket = Some(socket);
        state.config.control = Some(control.clone());
        Ok(Self {
            control,
            worker: Some(worker),
            handle,
            token,
        })
    }
}
impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.control.send(Control::Stop);
        self.handle.remove(self.token);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn watcher(path: &Path, control: Sender<Control>) -> notify::Result<RecommendedWatcher> {
    let target = path.to_owned();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if event.as_ref().map_or(true, |event| {
            !matches!(event.kind, notify::EventKind::Access(_))
                && event
                    .paths
                    .iter()
                    .any(|path| path == &target || target.starts_with(path))
        }) {
            let _ = control.send(Control::Changed);
        }
    })?;
    // Watch the directory for atomic renames and its parent for directory replacement.
    let directory = path.parent().expect("configuration has a parent");
    let mut parent = directory.parent().unwrap_or(directory);
    while !parent.is_dir() {
        parent = parent
            .parent()
            .ok_or_else(|| notify::Error::generic("no existing config ancestor"))?;
    }
    watcher.watch(parent, RecursiveMode::NonRecursive)?;
    if directory.is_dir() && directory != parent {
        watcher.watch(directory, RecursiveMode::NonRecursive)?;
    }
    Ok(watcher)
}

fn run(
    path: &Path,
    defaults: Settings,
    requests: Receiver<Control>,
    notifications: Sender<Control>,
    results: channel::Sender<Result<Prepared, String>>,
) {
    let mut watch = watcher(path, notifications.clone()).ok();
    let mut previous: Option<Result<Vec<u8>, String>> = None;
    let mut force = true;
    loop {
        let source = files::read(path);
        if force || previous.as_ref() != Some(&source) {
            let result = match &source {
                Ok(source) => lua::parse(source, path, defaults.clone()),
                Err(error) => Err(error.clone()),
            };
            if files::read(path) != source {
                previous = None;
                force = true;
                continue;
            }
            previous = Some(source);
            if results.send(result).is_err() {
                break;
            }
        }
        force = false;
        // A low-rate fallback detects missed native events and recovers replaced
        // directories. All file access stays on this worker, never in rendering.
        match requests.recv_timeout(Duration::from_secs(1)) {
            Ok(Control::Stop) | Err(RecvTimeoutError::Disconnected) => break,
            Ok(Control::Reload) => force = true,
            Ok(Control::Changed) => {
                loop {
                    match requests.recv_timeout(Duration::from_millis(120)) {
                        Ok(Control::Stop) | Err(RecvTimeoutError::Disconnected) => return,
                        Ok(Control::Reload) => {
                            force = true;
                            break;
                        }
                        Ok(Control::Changed) => {}
                        Err(RecvTimeoutError::Timeout) => break,
                    }
                }
                watch = watcher(path, notifications.clone()).ok();
            }
            Err(RecvTimeoutError::Timeout) => {
                if watch.is_none() {
                    watch = watcher(path, notifications.clone()).ok();
                }
            }
        }
    }
}
