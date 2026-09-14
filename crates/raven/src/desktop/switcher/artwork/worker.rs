use super::{Request, raster};
use smithay::reexports::calloop::LoopSignal;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
type ResultSlot = Option<(Request, Result<Vec<u8>, String>)>;
#[derive(Default)]
struct Shared {
    pending: Option<Request>,
    result: ResultSlot,
    stop: bool,
}
pub(super) struct Worker {
    shared: Arc<(Mutex<Shared>, Condvar)>,
    thread: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn new(signal: LoopSignal) -> Self {
        let shared = Arc::new((Mutex::new(Shared::default()), Condvar::new()));
        let work = shared.clone();
        let thread = thread::spawn(move || {
            let mut raster = raster::Raster::new();
            loop {
                let request = {
                    let mut guard = work.0.lock().unwrap();
                    while guard.pending.is_none() && !guard.stop {
                        guard = work.1.wait(guard).unwrap();
                    }
                    if guard.stop {
                        break;
                    }
                    guard.pending.take().unwrap()
                };
                let pixels = raster.paint(&request);
                work.0.lock().unwrap().result = Some((request, pixels));
                signal.wakeup();
            }
        });
        Self {
            shared,
            thread: Some(thread),
        }
    }
    pub fn request(&self, request: Request) {
        self.shared.0.lock().unwrap().pending = Some(request);
        self.shared.1.notify_one();
    }
    pub fn take(&self) -> ResultSlot {
        self.shared.0.lock().unwrap().result.take()
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.shared.0.lock().unwrap().stop = true;
        self.shared.1.notify_one();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
