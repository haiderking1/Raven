"""Reap detached regression clients without signalling another session."""
import ctypes
import os
from pathlib import Path
import select
import signal
import time

CLIENTS = {"waybar", "Xwayland", "xwayland-satellite"}


def become_subreaper():
    libc = ctypes.CDLL(None, use_errno=True)
    if libc.prctl(36, 1, 0, 0, 0) != 0:  # PR_SET_CHILD_SUBREAPER
        raise OSError(ctypes.get_errno(), "cannot become child subreaper")


def identity(path, runtime):
    if path.stat().st_uid != os.getuid():
        return None
    executable = Path(os.readlink(path / "exe")).name
    if executable not in CLIENTS:
        return None
    env = (path / "environ").read_bytes().split(b"\0")
    for key in ("XDG_RUNTIME_DIR", "RAVEN_TEST_PRIVATE_RUNTIME"):
        if os.fsencode(f"{key}={runtime}") not in env:
            return None
    # starttime is field 22; comm can contain spaces or parentheses.
    start = (path / "stat").read_text().rsplit(")", 1)[1].split()[19]
    return executable, start


def owned_clients(runtime):
    found = []
    for path in Path("/proc").iterdir():
        if not path.name.isdecimal():
            continue
        fd = None
        try:
            before = identity(path, runtime)
            if before is None:
                continue
            fd = os.pidfd_open(int(path.name))
            if identity(path, runtime) != before:
                os.close(fd)
                continue
            found.append((int(path.name), fd, before[0]))
        except (FileNotFoundError, ProcessLookupError, PermissionError):
            if fd is not None:
                os.close(fd)
    return found


def reap_adopted():
    while True:
        try:
            pid, _ = os.waitpid(-1, os.WNOHANG)
            if pid == 0:
                return
        except ChildProcessError:
            return


def cleanup(runtime, aborted, log):
    # The cargo group has stopped; only setsid clients can remain alive.
    # Repeat to catch Xwayland forked while its satellite was stopping.
    forced = []
    deadline = time.monotonic() + 8
    while True:
        reap_adopted()
        clients = owned_clients(runtime)
        if not clients:
            print(f"detached-client cleanup: forced={forced or 'none'}", file=log, flush=True)
            return forced
        if not aborted:
            print("successful test left owned clients; treating run as failed", file=log, flush=True)
            aborted = True
        try:
            for pid, fd, name in clients:
                forced.append(f"{name}:{pid}")
                print(f"private-runtime cleanup: SIGTERM {name} pid={pid}", file=log, flush=True)
                try:
                    signal.pidfd_send_signal(fd, signal.SIGTERM)
                except ProcessLookupError:
                    pass
            waiting = {fd for _, fd, _ in clients}
            grace = time.monotonic() + 1
            while waiting and time.monotonic() < grace:
                ready, _, _ = select.select(list(waiting), [], [], max(0, grace - time.monotonic()))
                waiting.difference_update(ready)
            for pid, fd, name in clients:
                if fd in waiting:
                    print(f"private-runtime cleanup: SIGKILL {name} pid={pid}", file=log, flush=True)
                    try:
                        signal.pidfd_send_signal(fd, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
            for pid, fd, name in clients:
                if not select.select([fd], [], [], 1)[0]:
                    raise RuntimeError(f"owned {name} pid={pid} survived SIGKILL")
        finally:
            for _, fd, _ in clients:
                os.close(fd)
        if time.monotonic() >= deadline:
            raise TimeoutError("private-runtime cleanup exceeded 8 seconds")
