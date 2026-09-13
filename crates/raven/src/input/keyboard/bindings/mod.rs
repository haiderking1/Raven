mod chord;
mod defaults;
use super::actions::Action;
use chord::Chord;
use smithay::input::keyboard::{Keysym, ModifiersState};

#[derive(Clone, Debug)]
pub struct Bindings {
    entries: Vec<(Chord, Action)>,
}

impl Bindings {
    pub fn bind(&mut self, chord: &str, action: Action) -> Result<(), String> {
        validate_action(&action)?;
        let chord = Chord::parse(chord)?;
        if let Some((_, previous)) = self.entries.iter_mut().find(|(key, _)| *key == chord) {
            *previous = action;
        } else {
            self.entries.push((chord, action));
        }
        Ok(())
    }
    pub fn unbind(&mut self, chord: &str) -> Result<(), String> {
        let chord = Chord::parse(chord)?;
        self.entries.retain(|(key, _)| *key != chord);
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        for (_, action) in &self.entries {
            validate_action(action)?;
        }
        Ok(())
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub(super) fn action(&self, modifiers: &ModifiersState, symbols: &[Keysym]) -> Option<Action> {
        self.entries
            .iter()
            .find(|(chord, _)| {
                symbols
                    .iter()
                    .any(|symbol| chord.matches(modifiers, *symbol))
            })
            .map(|(_, action)| action.clone())
    }
}

fn validate_action(action: &Action) -> Result<(), String> {
    match action {
        Action::SwitchWorkspace(index) | Action::MoveToWorkspace(index) if *index >= 10 => {
            Err("workspace index is outside 0..10".into())
        }
        Action::SwitchVt(vt) if !(1..=12).contains(vt) => Err("VT must be between 1 and 12".into()),
        Action::Spawn(argv) => crate::runtime::settings::StartupPlan {
            entries: vec![crate::runtime::settings::StartupEntry {
                argv: argv.clone(),
                ..Default::default()
            }],
        }
        .validate_all(),
        _ => Ok(()),
    }
}
