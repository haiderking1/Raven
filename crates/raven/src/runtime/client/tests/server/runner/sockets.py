"""Remove only the recorded reservation's unchanged filesystem sockets on abort."""
import os
from pathlib import Path
import socket
import stat


def identity(info):
    seconds, nanos = divmod(info.st_ctime_ns, 1_000_000_000)
    return info.st_dev, info.st_ino, info.st_uid, seconds, nanos


def cleanup(runtime, aborted, log):
    manifest = runtime / "x11-ownership"
    if not manifest.exists():
        return False
    header, socket_id, lock_id = manifest.read_text().splitlines()
    number, owner = map(int, header.split())
    if not 0 <= number <= 65535:
        raise RuntimeError("invalid owned X11 display number")
    paths = (Path(f"/tmp/.X11-unix/X{number}"), Path(f"/tmp/.X{number}-lock"))
    recorded = [tuple(map(int, line.split())) for line in (socket_id, lock_id)]
    stale = []
    for index, (path, expected) in enumerate(zip(paths, recorded)):
        try:
            info = path.lstat()
        except FileNotFoundError:
            continue
        if identity(info) != expected or info.st_uid != os.getuid():
            raise RuntimeError(f"refusing changed X11 path: {path}")
        valid_type = stat.S_ISSOCK(info.st_mode) if index == 0 else stat.S_ISREG(info.st_mode)
        if not valid_type or index == 1 and path.read_text().strip() != str(owner):
            raise RuntimeError(f"refusing changed X11 reservation: {path}")
        stale.append((path, expected))
    if not stale:
        print("owned X11 paths removed by test", file=log, flush=True)
        return False
    if Path(f"/proc/{owner}").exists():
        raise RuntimeError(f"X11 reservation owner {owner} still exists")
    if not aborted:
        print("successful test left owned X11 paths; treating run as failed", file=log, flush=True)
    # Holding the abstract address rules out a live listener or a new reservation
    # while the recorded filesystem entries are being removed.
    with socket.socket(socket.AF_UNIX) as probe:
        probe.bind(bytes([0]) + os.fsencode(paths[0]))
        for path, expected in stale:
            if identity(path.lstat()) != expected:
                raise RuntimeError(f"X11 path changed during cleanup: {path}")
            path.unlink()
            print(f"private-runtime cleanup: removed owned {path}", file=log, flush=True)
    return True
