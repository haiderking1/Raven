use smithay::reexports::calloop::LoopSignal;
use std::sync::{
    Arc,
    mpsc::{self, Receiver},
};
pub(crate) struct Pixels {
    pub rgba: Vec<u8>,
    pub width: i32,
    pub height: i32,
}
pub(super) struct Completed {
    pub png: Option<Arc<[u8]>>,
    pub message: String,
    pub hud: Option<Vec<u8>>,
}
pub(super) struct Job {
    receiver: Receiver<Completed>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Job {
    pub fn try_recv(&self) -> Result<Completed, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
pub(super) fn start(image: Pixels, signal: LoopSignal) -> Result<Job, String> {
    let (tx, rx) = mpsc::sync_channel(1);
    let thread = std::thread::Builder::new()
        .name("raven-screenshot-png".into())
        .spawn(move || {
            let (png, message) = match super::native::encode(&image.rgba, image.width, image.height)
            {
                Ok(png) => {
                    let saved =
                        super::files::destination().and_then(|dir| super::files::save(&dir, &png));
                    let message = match saved {
                        Ok(path) => format!("Copied screenshot\nSaved to {}", path.display()),
                        Err(e) => format!("Copied screenshot, but saving failed\n{e}"),
                    };
                    (Some(Arc::from(png)), message)
                }
                Err(e) => (None, format!("Screenshot failed\n{e}")),
            };
            let hud = super::native::hud_pixels(&message);
            let _ = tx.send(Completed { png, message, hud });
            signal.wakeup();
        })
        .map_err(|e| format!("cannot start PNG encoder: {e}"))?;
    Ok(Job {
        receiver: rx,
        thread: Some(thread),
    })
}
