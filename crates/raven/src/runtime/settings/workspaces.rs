#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkspaceSettings {
    pub show_all: bool,
    /// One-based workspace numbers. Visibility hints never remove workspaces.
    pub persistent: Vec<usize>,
}
impl WorkspaceSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self
            .persistent
            .iter()
            .any(|number| !(1..=10).contains(number))
        {
            Err("persistent workspace numbers must be between 1 and 10".into())
        } else {
            Ok(())
        }
    }
}
