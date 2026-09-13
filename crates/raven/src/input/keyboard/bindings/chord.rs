use smithay::input::keyboard::{Keysym, ModifiersState};
use xkbcommon::xkb;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Chord {
    modifiers: u8,
    symbol: u32,
}

fn normalize(symbol: Keysym) -> u32 {
    if let Some(character) = symbol.key_char() {
        let mut lower = character.to_lowercase();
        if let Some(character) = lower.next()
            && lower.next().is_none()
        {
            return Keysym::from_char(character).raw();
        }
    }
    symbol.raw()
}

impl Chord {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut parts: Vec<_> = text.split('+').collect();
        let key = parts
            .pop()
            .filter(|key| !key.is_empty() && !key.contains('\0'))
            .ok_or("binding requires a keysym name")?;
        let mut modifiers = 0;
        for part in parts {
            let bit = match part.to_ascii_lowercase().as_str() {
                "super" | "logo" => 1,
                "shift" => 2,
                "ctrl" | "control" => 4,
                "alt" => 8,
                _ => return Err(format!("unknown modifier {part:?} in {text:?}")),
            };
            if modifiers & bit != 0 {
                return Err(format!("duplicate modifier in {text:?}"));
            }
            modifiers |= bit;
        }
        let symbol = xkb::keysym_from_name(key, xkb::KEYSYM_CASE_INSENSITIVE);
        if symbol.raw() == 0 {
            return Err(format!("unknown keysym {key:?}"));
        }
        Ok(Self {
            modifiers,
            symbol: normalize(symbol),
        })
    }

    pub fn matches(&self, modifiers: &ModifiersState, symbol: Keysym) -> bool {
        let bits = u8::from(modifiers.logo)
            | (u8::from(modifiers.shift) << 1)
            | (u8::from(modifiers.ctrl) << 2)
            | (u8::from(modifiers.alt) << 3);
        self.modifiers == bits && self.symbol == normalize(symbol)
    }
}
