use crate::state::State;
use smithay::{
    delegate_shm,
    reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    wayland::{
        buffer::BufferHandler,
        shm::{ShmHandler, ShmState},
    },
};

impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}
impl BufferHandler for State {
    // Smithay owns imported buffers and releases them when renderer references drop.
    fn buffer_destroyed(&mut self, _buffer: &WlBuffer) {}
}
delegate_shm!(State);
