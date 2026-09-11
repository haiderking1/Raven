use crate::runtime::{
    client::Clients,
    startup::{StartupEntry, StartupPlan},
};
use std::{error::Error, ffi::OsString, fs, path::PathBuf};

pub(in crate::runtime::client::tests) struct Fixture {
    root: PathBuf,
    pub log: PathBuf,
}

impl Fixture {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // Check isolation before any fixture starts clients or library threads.
        let runtime = super::isolation::runtime()?;
        let root = runtime.join("waybar-fixture");
        fs::create_dir(&root)?;
        let log_dir = PathBuf::from(
            std::env::var_os("RAVEN_TEST_LOG_DIR").ok_or("missing verification log directory")?,
        );
        let log = log_dir.join(format!("waybar-{}.log", std::process::id()));
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&log)?;
        let fixture = Self { root, log };
        fs::write(
            fixture.root.join("config.json"),
            include_str!("config.json"),
        )?;
        fs::write(fixture.root.join("style.css"), include_str!("style.css"))?;
        Ok(fixture)
    }

    pub fn start(&self, clients: &mut Clients) {
        // exec preserves the startup-owned PID. Only this child's output is redirected.
        let entry = StartupEntry {
            argv: vec!["/bin/sh".into(), "-c".into(),
                "echo WAYBAR_PID=$$ > \"$1\"; exec waybar -l debug -c \"$2\" -s \"$3\" >> \"$1\" 2>&1".into(),
                "raven-private-waybar".into(), self.log.clone().into_os_string(),
                self.root.join("config.json").into_os_string(), self.root.join("style.css").into_os_string()],
            env: [(OsString::from("WAYLAND_DEBUG"), OsString::from("client"))].into(),
            ..StartupEntry::default()
        };
        clients.start_startup(
            &StartupPlan {
                entries: vec![entry],
            }
            .validate(),
        );
    }

    pub fn text(&self) -> Result<String, Box<dyn Error>> {
        Ok(fs::read_to_string(&self.log)?)
    }

    pub fn pid(&self) -> Result<u32, Box<dyn Error>> {
        self.text()?
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("WAYBAR_PID="))
            .ok_or("Waybar launcher did not record its owned PID")?
            .parse()
            .map_err(Into::into)
    }

    pub fn check_exit(&self) -> Result<(), Box<dyn Error>> {
        let text = self.text()?;
        if text.is_empty() {
            return Ok(());
        }
        let pid = self.pid()?;
        let status = fs::read_to_string(format!("/proc/{pid}/status"))?;
        if status
            .lines()
            .any(|line| line.starts_with("State:") && line.contains("Z (zombie)"))
        {
            let detail = text
                .lines()
                .rev()
                .find(|line| line.contains("[error]"))
                .unwrap_or("no Waybar error message");
            return Err(format!("installed Waybar exited before the integration completed with the owned private session D-Bus: {detail}; log {}", self.log.display()).into());
        }
        Ok(())
    }

    pub fn assert_running(&self) -> Result<(), Box<dyn Error>> {
        self.check_exit()?;
        let pid = self.pid()?;
        if fs::read_to_string(format!("/proc/{pid}/comm"))?.trim() != "waybar" {
            return Err("startup child is not the real Waybar process".into());
        }
        Ok(())
    }

    pub fn cleanup(&self) -> Result<(), Box<dyn Error>> {
        fs::remove_dir_all(&self.root)?;
        if self.root.exists() {
            return Err("generated Waybar files survived cleanup".into());
        }
        Ok(())
    }

    pub fn assert_stopped(&self) -> Result<(), Box<dyn Error>> {
        if PathBuf::from(format!("/proc/{}", self.pid()?)).exists() {
            return Err("Clients::drop did not reap owned Waybar".into());
        }
        Ok(())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
