use std::{
    error::Error,
    fs,
    os::unix::fs::{FileTypeExt, MetadataExt},
    path::PathBuf,
};

pub(super) fn runtime() -> Result<PathBuf, Box<dyn Error>> {
    let runtime = PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").ok_or("use server/run.sh")?);
    let metadata = fs::metadata(&runtime)?;
    let uid = fs::metadata("/proc/self")?.uid();
    if !runtime.is_absolute()
        || fs::canonicalize(&runtime)? != runtime
        || metadata.uid() != uid
        || metadata.mode() & 0o077 != 0
        || std::env::var_os("RAVEN_TEST_PRIVATE_RUNTIME").as_deref() != Some(runtime.as_os_str())
        || [
            "DISPLAY",
            "WAYLAND_DISPLAY",
            "WAYLAND_SOCKET",
            "XAUTHORITY",
            "RAVEN_XWAYLAND_SATELLITE",
            "DBUS_STARTER_ADDRESS",
            "DBUS_STARTER_BUS_TYPE",
            "AT_SPI_BUS_ADDRESS",
            "NOTIFY_SOCKET",
        ]
        .iter()
        .any(|key| std::env::var_os(key).is_some())
        || std::env::var("DBUS_SESSION_BUS_ADDRESS")?
            != format!("unix:path={}/bus", runtime.display())
        || std::env::var("DBUS_SYSTEM_BUS_ADDRESS")?
            != format!("unix:path={}/no-system-bus", runtime.display())
        || std::env::var("GDK_BACKEND").as_deref() != Ok("wayland")
        || std::env::var("NO_AT_BRIDGE").as_deref() != Ok("1")
    {
        return Err("isolated environment required; run runtime/client/tests/server/run.sh".into());
    }
    let bus = fs::symlink_metadata(runtime.join("bus"))?;
    if !bus.file_type().is_socket() || bus.uid() != uid {
        return Err("session bus must be the owned private runtime socket".into());
    }
    for (key, directory) in [
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_CACHE_HOME", "cache"),
        ("XDG_DATA_HOME", "data"),
        ("XDG_CONFIG_DIRS", "config-dirs"),
        ("XDG_DATA_DIRS", "data-dirs"),
        ("HOME", "home"),
    ] {
        let path = PathBuf::from(std::env::var_os(key).ok_or("missing private directory")?);
        if path != runtime.join(directory) || fs::canonicalize(&path)? != path {
            return Err(format!("{key} is not the exact private directory").into());
        }
    }
    Ok(runtime)
}
