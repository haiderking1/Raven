use crate::{desktop::tests::fixture::Fixture, state::State};
use smithay::{
    backend::input::ButtonState,
    input::pointer::ButtonEvent,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    utils::SERIAL_COUNTER,
};

// The desktop fixture owns the socket transport. Copy only decoded event fields
// here because its wire module is private to desktop tests.
#[derive(Debug, PartialEq)]
pub(super) struct Event {
    pub object: u32,
    pub opcode: u16,
    pub args: Vec<u8>,
}

pub(super) fn dispatch(f: &mut Fixture) -> Vec<Event> {
    f.dispatch()
        .into_iter()
        .map(|event| Event {
            object: event.object,
            opcode: event.opcode,
            args: event.args,
        })
        .collect()
}

pub(super) fn word(bytes: &[u8]) -> u32 {
    u32::from_ne_bytes(bytes[..4].try_into().unwrap())
}

pub(super) fn text(bytes: &[u8]) -> &str {
    let length = word(bytes) as usize;
    assert!(length > 0);
    assert_eq!(bytes[4 + length - 1], 0);
    std::str::from_utf8(&bytes[4..4 + length - 1]).unwrap()
}

pub(super) fn string(value: &str) -> Vec<u8> {
    let mut bytes = ((value.len() + 1) as u32).to_ne_bytes().to_vec();
    bytes.extend_from_slice(value.as_bytes());
    bytes.push(0);
    bytes.resize(bytes.len().next_multiple_of(4), 0);
    bytes
}

pub(super) fn global(f: &mut Fixture, interface: &str) -> u32 {
    let registry = f.id();
    f.wire.request(1, 1, &[registry]);
    let events = dispatch(f);
    let event = events
        .iter()
        .find(|event| {
            event.object == registry && event.opcode == 0 && {
                let len = word(&event.args[4..]) as usize;
                &event.args[8..8 + len - 1] == interface.as_bytes()
            }
        })
        .expect("advertised global");
    word(&event.args)
}

pub(super) fn bind(
    f: &mut Fixture,
    global: u32,
    interface: &str,
    version: u32,
) -> (u32, Vec<Event>) {
    let id = f.id();
    let mut args = global.to_ne_bytes().to_vec();
    args.extend(string(interface));
    args.extend(version.to_ne_bytes());
    args.extend(id.to_ne_bytes());
    f.wire.bytes(2, 0, &args, None);
    (id, dispatch(f))
}

pub(super) struct Manager {
    pub id: u32,
    pub group: u32,
    pub workspaces: Vec<u32>,
}

impl Manager {
    pub fn bind(f: &mut Fixture, global: u32) -> (Self, Vec<Event>) {
        let (id, events) = bind(f, global, "ext_workspace_manager_v1", 1);
        let group = word(
            &events
                .iter()
                .find(|e| e.object == id && e.opcode == 0)
                .unwrap()
                .args,
        );
        let workspaces = events
            .iter()
            .filter(|e| e.object == id && e.opcode == 1)
            .map(|e| word(&e.args))
            .collect();
        (
            Self {
                id,
                group,
                workspaces,
            },
            events,
        )
    }

    pub fn owns(&self, e: &Event) -> bool {
        e.object == self.id || e.object == self.group || self.workspaces.contains(&e.object)
    }

    pub fn states(&self, events: &[Event]) -> Vec<(usize, u32)> {
        events
            .iter()
            .filter(|e| e.opcode == 3)
            .filter_map(|e| {
                self.workspaces
                    .iter()
                    .position(|id| *id == e.object)
                    .map(|index| (index, word(&e.args)))
            })
            .collect()
    }

    pub fn done(&self, events: &[Event]) {
        let batch: Vec<_> = events.iter().filter(|e| self.owns(e)).collect();
        assert_eq!(
            batch
                .iter()
                .filter(|e| e.object == self.id && e.opcode == 2)
                .count(),
            1
        );
        let last = batch.last().unwrap();
        assert_eq!((last.object, last.opcode), (self.id, 2));
    }
}

pub(super) fn output(f: &mut Fixture) -> Output {
    let output = Output::new(
        "workspace-wire-test".into(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "test".into(),
            model: "test".into(),
        },
    );
    output.change_current_state(
        Some(Mode {
            size: (800, 600).into(),
            refresh: 60_000,
        }),
        None,
        None,
        Some((0, 0).into()),
    );
    output.create_global::<State>(&f.state.display_handle);
    f.state.space_mut().map_output(&output, (0, 0));
    f.state.output = Some(output.clone());
    output
}

pub(super) fn button(f: &mut Fixture, state: ButtonState) {
    let pointer = f.state.seat.get_pointer().unwrap();
    pointer.button(
        &mut f.state,
        &ButtonEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: 1,
            button: 0x110,
            state,
        },
    );
    pointer.frame(&mut f.state);
}
