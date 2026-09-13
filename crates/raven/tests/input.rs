mod runtime {
    pub use raven::runtime::settings;
}
#[path = "../src/input/devices/keyboard.rs"]
mod keyboard;
#[path = "input/profiles.rs"]
mod profiles;
#[path = "input/repeat.rs"]
mod repeat;
#[path = "../src/input/devices/transaction.rs"]
mod transaction;
#[path = "../src/desktop/tests/wire.rs"]
mod wire;
