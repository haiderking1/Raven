use crate::runtime::startup::StartupEntry;
use std::{
    ffi::OsString,
    fs,
    os::unix::ffi::OsStringExt,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

pub(super) struct Fixture {
    pub(super) directory: PathBuf,
}

impl Fixture {
    pub(super) fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "raven-startup-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self { directory }
    }

    pub(super) fn entry(&self, name: &str) -> StartupEntry {
        StartupEntry {
            argv: vec![
                "/bin/sh".into(),
                "-c".into(),
                // The shell is an explicit test executable, not an argv parser.
                r#"printf '%s\000' "$1" "$2" "$PWD" "$WAYLAND_DISPLAY" "${DISPLAY-unset}" "${WAYLAND_SOCKET-unset}" "$XDG_SESSION_TYPE" "$XDG_CURRENT_DESKTOP" "$RAVEN_STARTUP_TEST" "${XAUTHORITY-unset}" "${XDG_RUNTIME_DIR-unset}" >> "$1.out""#.into(),
                "raven-startup-test".into(),
                name.into(),
                OsString::from_vec(b"literal argument ; $HOME * \xff".to_vec()),
            ],
            cwd: Some(self.directory.clone()),
            env: [
                ("WAYLAND_DISPLAY", "wrong-wayland"),
                ("XDG_RUNTIME_DIR", "wrong-runtime-directory"),
                ("DISPLAY", "wrong-x11"),
                ("WAYLAND_SOCKET", "999"),
                ("XDG_SESSION_TYPE", "wrong-session"),
                ("XDG_CURRENT_DESKTOP", "wrong-desktop"),
                ("RAVEN_STARTUP_TEST", "entry override"),
                ("XAUTHORITY", "wrong-authority"),
            ].into_iter().map(|(key, value)| (key.into(), value.into())).collect(),
        }
    }

    pub(super) fn assert_output(&self, name: &str, display: &str, authority: &str) {
        use std::os::unix::ffi::OsStrExt;
        let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR").unwrap_or_else(|| "unset".into());
        let fields: &[&[u8]] = &[
            name.as_bytes(),
            b"literal argument ; $HOME * \xff",
            self.directory.as_os_str().as_bytes(),
            b"raven-startup-test-socket",
            display.as_bytes(),
            b"unset",
            b"wayland",
            b"Raven",
            b"entry override",
            authority.as_bytes(),
            runtime_dir.as_bytes(),
        ];
        let expected: Vec<u8> = fields
            .iter()
            .flat_map(|field| field.iter().copied().chain(std::iter::once(0)))
            .collect();
        assert_eq!(
            fs::read(self.directory.join(format!("{name}.out"))).unwrap(),
            expected
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}
