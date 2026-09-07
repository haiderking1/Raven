use super::super::{
    fixture::Fixture,
    wire::{string, word},
};

pub(super) fn bind_pointer(f: &mut Fixture) -> u32 {
    let registry = f.id();
    f.wire.request(1, 1, &[registry]);
    let events = f.dispatch();
    let global = events
        .iter()
        .find(|event| {
            event.object == registry && event.opcode == 0 && {
                let len = word(&event.args[4..]) as usize;
                &event.args[8..8 + len - 1] == b"wl_seat"
            }
        })
        .expect("seat global");
    let seat = f.id();
    let mut args = word(&global.args).to_ne_bytes().to_vec();
    args.extend(string("wl_seat"));
    args.extend(5u32.to_ne_bytes());
    args.extend(seat.to_ne_bytes());
    f.wire.bytes(registry, 0, &args, None);
    let pointer = f.id();
    f.wire.request(seat, 0, &[pointer]);
    f.dispatch();
    pointer
}
