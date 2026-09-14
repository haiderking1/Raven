#[path = "switcher/keyboard.rs"]
mod input;
#[path = "../src/desktop/switcher/layout.rs"]
mod layout;
#[path = "../src/desktop/switcher/model.rs"]
mod model;
#[path = "switcher/selection.rs"]
mod selection;
#[allow(dead_code)]
#[path = "../src/runtime/startup/mod.rs"]
mod startup;
mod runtime {
    pub mod settings {
        pub use crate::startup::{StartupEntry, StartupPlan};
    }
}
