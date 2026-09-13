use super::{Settings, StartupEntry, StartupPlan};
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        self.input.validate()?;
        self.appearance.validate().map_err(|e| e.to_string())?;
        self.workspaces.validate()?;
        self.startup.validate_all()?;
        StartupPlan {
            entries: vec![
                StartupEntry {
                    argv: self.terminal.clone(),
                    ..Default::default()
                },
                StartupEntry {
                    argv: self.launcher.clone(),
                    ..Default::default()
                },
            ],
        }
        .validate_all()
        .map_err(|e| format!("terminal/launcher commands: {e}"))?;
        if self.cursor.theme.trim().is_empty()
            || self.cursor.size == 0
            || self.cursor.size > i32::MAX as u32
        {
            return Err(
                "cursor requires a theme and a positive logical-pixel size fitting i32".into(),
            );
        }
        self.bindings.validate()
    }
}
