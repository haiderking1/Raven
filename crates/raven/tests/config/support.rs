use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

pub(super) struct ConfigFile(pub PathBuf);
impl Drop for ConfigFile {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl ConfigFile {
    pub(super) fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "raven-config-diagnostic-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("create isolated configuration directory");
        Self(path)
    }
    pub(super) fn command(&self, source: &str) -> Command {
        let file = self.0.join("raven.lua");
        fs::write(&file, source).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_raven"));
        command.arg("--check-config").arg(file).current_dir(&self.0);
        command
    }
}

pub(super) fn rejected(source: &str) -> String {
    let directory = ConfigFile::new();
    let output = directory.command(source).output().unwrap();
    assert!(!output.status.success(), "invalid config was accepted");
    String::from_utf8(output.stderr).unwrap()
}
