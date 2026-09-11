"""Private paths and child-only environment for the installed-client regression."""
import os
from pathlib import Path
import subprocess
import tempfile


def create_runtime(repo):
    logs = repo / "prefromance-inputlag" / "verification"
    subprocess.run(["git", "check-ignore", "-q", str(logs / "probe.log")],
                   cwd=repo, check=True)
    logs.mkdir(parents=True, exist_ok=True)
    runtime = Path(tempfile.mkdtemp(prefix="raven-satellite-waybar.", dir="/tmp"))
    return runtime, logs


def child_environment(runtime, logs):
    env = os.environ.copy()
    for key in (
        "DISPLAY", "WAYLAND_DISPLAY", "WAYLAND_SOCKET", "XAUTHORITY",
        "RAVEN_XWAYLAND_SATELLITE", "DBUS_STARTER_ADDRESS", "DBUS_STARTER_BUS_TYPE",
        "AT_SPI_BUS_ADDRESS", "NOTIFY_SOCKET", "DBUS_SESSION_BUS_PID",
        "DBUS_SESSION_BUS_WINDOWID",
    ):
        env.pop(key, None)
    for key, directory in {
        "XDG_CONFIG_HOME": "config", "XDG_CACHE_HOME": "cache",
        "XDG_DATA_HOME": "data", "XDG_CONFIG_DIRS": "config-dirs",
        "XDG_DATA_DIRS": "data-dirs", "HOME": "home",
    }.items():
        path = runtime / directory
        path.mkdir(mode=0o700)
        env[key] = str(path)
    # Keep cargo's existing offline cache while GUI libraries get a private HOME.
    env["CARGO_HOME"] = os.environ.get("CARGO_HOME", str(Path.home() / ".cargo"))
    env["RUSTUP_HOME"] = os.environ.get("RUSTUP_HOME", str(Path.home() / ".rustup"))
    env.update(
        XDG_RUNTIME_DIR=str(runtime), RAVEN_TEST_PRIVATE_RUNTIME=str(runtime),
        RAVEN_TEST_LOG_DIR=str(logs), GDK_BACKEND="wayland", NO_AT_BRIDGE="1",
        DBUS_SESSION_BUS_ADDRESS=f"unix:path={runtime}/bus",
        # A system-bus fallback must not reach the host either.
        DBUS_SYSTEM_BUS_ADDRESS=f"unix:path={runtime}/no-system-bus",
    )
    return env
