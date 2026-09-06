use smithay::reexports::rustix::{
    self,
    net::{SendAncillaryBuffer, SendAncillaryMessage, SendFlags, sendmsg},
};
use std::{
    io::{ErrorKind, IoSlice, Read, Write},
    mem::MaybeUninit,
    os::{
        fd::{AsFd, BorrowedFd},
        unix::net::UnixStream,
    },
};

pub struct Wire {
    pub socket: UnixStream,
    pending: Vec<u8>,
}

#[derive(Debug)]
pub struct Event {
    pub object: u32,
    pub opcode: u16,
    pub args: Vec<u8>,
}

impl Wire {
    pub fn new(socket: UnixStream) -> Self {
        socket.set_nonblocking(true).unwrap();
        Self {
            socket,
            pending: Vec::new(),
        }
    }

    pub fn request(&mut self, object: u32, opcode: u16, args: &[u32]) {
        self.bytes(
            object,
            opcode,
            &args
                .iter()
                .flat_map(|v| v.to_ne_bytes())
                .collect::<Vec<_>>(),
            None,
        );
    }

    pub fn bytes(&mut self, object: u32, opcode: u16, args: &[u8], fd: Option<BorrowedFd<'_>>) {
        let mut message = object.to_ne_bytes().to_vec();
        message.extend_from_slice(&(((args.len() as u32 + 8) << 16) | opcode as u32).to_ne_bytes());
        message.extend_from_slice(args);
        if let Some(fd) = fd {
            let fds = [fd];
            let mut storage = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
            let mut ancillary = SendAncillaryBuffer::new(&mut storage);
            assert!(ancillary.push(SendAncillaryMessage::ScmRights(&fds)));
            let sent = sendmsg(
                self.socket.as_fd(),
                &[IoSlice::new(&message)],
                &mut ancillary,
                SendFlags::empty(),
            )
            .unwrap();
            self.socket.write_all(&message[sent..]).unwrap();
        } else {
            self.socket.write_all(&message).unwrap();
        }
    }

    pub fn events(&mut self) -> Vec<Event> {
        let mut buffer = [0u8; 4096];
        loop {
            match self.socket.read(&mut buffer) {
                Ok(0) => panic!("Wayland client disconnected"),
                Ok(n) => self.pending.extend_from_slice(&buffer[..n]),
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) => panic!("Wayland read: {error}"),
            }
        }
        let mut events = Vec::new();
        while self.pending.len() >= 8 {
            let header = word(&self.pending[4..]);
            let size = (header >> 16) as usize;
            assert!(size >= 8);
            if self.pending.len() < size {
                break;
            }
            let event = Event {
                object: word(&self.pending),
                opcode: header as u16,
                args: self.pending[8..size].to_vec(),
            };
            assert!(
                !(event.object == 1 && event.opcode == 0),
                "Wayland protocol error: {event:?}"
            );
            events.push(event);
            self.pending.drain(..size);
        }
        events
    }
}

pub fn word(bytes: &[u8]) -> u32 {
    u32::from_ne_bytes(bytes[..4].try_into().unwrap())
}

pub fn string(value: &str) -> Vec<u8> {
    let mut bytes = ((value.len() + 1) as u32).to_ne_bytes().to_vec();
    bytes.extend_from_slice(value.as_bytes());
    bytes.push(0);
    while bytes.len() % 4 != 0 {
        bytes.push(0);
    }
    bytes
}
