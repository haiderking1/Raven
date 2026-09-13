use crate::{server::Server, wire::word};
use std::io::{ErrorKind, Read};

#[test]
fn invalid_axes_and_sources_report_the_specified_protocol_errors() {
    for (opcode, args, error) in [
        (3, vec![1, 99, 0], 0),
        (6, vec![1, 99], 0),
        (7, vec![1, 99, 0, 1], 0),
        (5, vec![99], 1),
    ] {
        let mut s = Server::new(2);
        s.create();
        s.wire.request(6, opcode, &args);
        s.pump();
        let mut bytes = Vec::new();
        let mut buffer = [0u8; 512];
        loop {
            match s.wire.socket.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => bytes.extend_from_slice(&buffer[..n]),
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) => panic!("{e}"),
            }
        }
        assert!(bytes.len() >= 20, "missing protocol error");
        assert_eq!(word(&bytes), 1); // wl_display.error
        assert_eq!(word(&bytes[4..]) & 0xffff, 0);
        assert_eq!(word(&bytes[8..]), 6); // offending pointer
        assert_eq!(word(&bytes[12..]), error);
        assert!(s.state.events.is_empty());
    }
}
