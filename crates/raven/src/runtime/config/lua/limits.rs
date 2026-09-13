use mlua::{DebugEvent, HookTriggers, Lua, VmState};
use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

pub(super) fn install(lua: &Lua) -> mlua::Result<Rc<Cell<Option<usize>>>> {
    lua.set_memory_limit(32 * 1024 * 1024)?;
    let line = Rc::new(Cell::new(None));
    let location = line.clone();
    let start = Instant::now();
    let instructions = Cell::new(0usize);
    lua.set_hook(
        HookTriggers::new().every_line().every_nth_instruction(1000),
        move |_, debug| {
            if let Some(line) = debug.current_line() {
                location.set(Some(line));
            }
            if matches!(debug.event(), DebugEvent::Count) {
                instructions.set(instructions.get() + 1000);
            }
            if instructions.get() > 2_000_000 || start.elapsed() > Duration::from_millis(500) {
                return Err(mlua::Error::RuntimeError(
                    "configuration exceeded its execution budget; check for an infinite loop"
                        .into(),
                ));
            }
            Ok(VmState::Continue)
        },
    )?;
    Ok(line)
}
