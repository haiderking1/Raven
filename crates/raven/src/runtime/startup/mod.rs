//! Typed session startup settings, independent of any configuration parser.
mod validation;

use std::{collections::BTreeMap, ffi::OsString, path::PathBuf};

/// An executable and its literal arguments. No shell parsing or expansion occurs.
#[derive(Clone, Debug, Default)]
pub struct StartupEntry {
    pub argv: Vec<OsString>,
    pub cwd: Option<PathBuf>,
    /// Override inherited variables for this child only. Raven's display settings win.
    pub env: BTreeMap<OsString, OsString>,
}

/// Settings for one session. Constructing or replacing a plan never launches it.
#[derive(Clone, Debug)]
pub struct StartupPlan {
    pub entries: Vec<StartupEntry>,
}

impl Default for StartupPlan {
    fn default() -> Self {
        Self {
            entries: vec![StartupEntry {
                argv: vec![OsString::from("waybar")],
                ..StartupEntry::default()
            }],
        }
    }
}

impl StartupPlan {
    /// Validate before acquiring the backend. Keep errors beside valid entries so
    /// one invalid entry cannot prevent the others from launching.
    /// Executable lookup and filesystem access remain fallible at spawn time.
    pub(crate) fn validate(self) -> ValidatedStartupPlan {
        ValidatedStartupPlan {
            entries: self
                .entries
                .into_iter()
                .map(|entry| validation::entry(&entry).map(|()| entry))
                .collect(),
        }
    }
}

/// Only StartupPlan::validate can construct a launchable plan.
#[derive(Debug)]
pub(crate) struct ValidatedStartupPlan {
    entries: Vec<Result<StartupEntry, String>>,
}

impl ValidatedStartupPlan {
    pub(crate) fn entries(&self) -> &[Result<StartupEntry, String>] {
        &self.entries
    }
}
