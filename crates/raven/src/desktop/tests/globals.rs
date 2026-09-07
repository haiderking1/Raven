use super::{
    fixture::Fixture,
    wire::{string, word},
};

pub(super) fn bind(f: &mut Fixture, name: &str, version: u32) -> u32 {
    let registry = f.id();
    f.wire.request(1, 1, &[registry]);
    let events = f.dispatch();
    let global = events
        .iter()
        .find(|event| {
            event.object == registry && event.opcode == 0 && {
                let len = word(&event.args[4..]) as usize;
                &event.args[8..8 + len - 1] == name.as_bytes()
            }
        })
        .expect("requested test global is advertised");
    let object = f.id();
    let mut args = word(&global.args).to_ne_bytes().to_vec();
    args.extend(string(name));
    args.extend(version.to_ne_bytes());
    args.extend(object.to_ne_bytes());
    f.wire.bytes(registry, 0, &args, None);
    f.dispatch();
    object
}
