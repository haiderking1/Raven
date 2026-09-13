use super::{Action, Bindings};

impl Default for Bindings {
    fn default() -> Self {
        let mut bindings = Self {
            entries: Vec::new(),
        };
        for (chord, action) in [
            ("Super+Shift+Q", Action::Quit),
            ("Super+Q", Action::LaunchTerminal),
            ("Super+D", Action::LaunchFuzzel),
            ("Super+C", Action::CloseWindow),
            ("Super+F", Action::ToggleFullscreen),
            ("Super+V", Action::ToggleFloating),
            ("Super+Shift+R", Action::ReloadConfig),
        ] {
            bindings
                .bind(chord, action)
                .expect("valid built-in binding");
        }
        for index in 0..10 {
            let key = (index + 1) % 10;
            bindings
                .bind(&format!("Super+{key}"), Action::SwitchWorkspace(index))
                .unwrap();
            bindings
                .bind(
                    &format!("Super+Shift+{key}"),
                    Action::MoveToWorkspace(index),
                )
                .unwrap();
        }
        for vt in 1..=12 {
            bindings
                .bind(&format!("Ctrl+Alt+F{vt}"), Action::SwitchVt(vt))
                .unwrap();
        }
        bindings
    }
}
