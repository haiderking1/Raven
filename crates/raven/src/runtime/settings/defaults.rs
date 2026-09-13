use super::*;
impl Default for Settings {
    fn default() -> Self {
        Self {
            input: InputSettings::default(),
            appearance: Appearance::default(),
            resize_animations: ResizeAnimations::default(),
            startup: StartupPlan::default(),
            bindings: Bindings::default(),
            terminal: vec!["foot".into()],
            launcher: vec!["fuzzel".into()],
            cursor: CursorSettings {
                theme: "default".into(),
                size: 24,
            },
            workspaces: WorkspaceSettings::default(),
        }
    }
}
