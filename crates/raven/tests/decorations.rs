// Run the production protocol handlers in a minimal Wayland server, independent
// of renderer test fixtures. No decoration behavior is replaced by test code.
#[path = "../src/protocols/decorations/kde/mod.rs"]
mod kde;
#[path = "decorations/negotiation.rs"]
mod negotiation;
#[path = "decorations/server.rs"]
mod state;
#[path = "../src/desktop/tests/wire.rs"]
mod wire;
