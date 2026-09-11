"""Own one foreground daemon with no service discovery or activation helpers."""
import os
import subprocess
import time
from contextlib import contextmanager

from .signals import ignore


@contextmanager
def private_bus(runtime, env, log):
    config = runtime / "bus.conf"
    config.write_text(f"""<busconfig>
  <type>session</type>
  <listen>unix:path={runtime}/bus</listen>
  <auth>EXTERNAL</auth>
  <policy context="default">
    <deny user="*"/>
    <allow user="{os.getuid()}"/>
    <deny own="*"/>
    <deny send_destination="*"/>
    <deny receive_sender="*"/>
  </policy>
  <policy user="{os.getuid()}">
    <allow own="*"/>
    <allow send_destination="*"/>
    <allow receive_sender="*"/>
  </policy>
</busconfig>
""")
    # No standard_session_servicedirs, servicedir, include, includedir,
    # servicehelper or systemd activation. Missing portals stay missing.
    address_file = runtime / "bus.address"
    with address_file.open("w") as address:
        daemon = subprocess.Popen(
            ["dbus-daemon", "--nofork", f"--config-file={config}", "--print-address=1"],
            env=env, stdout=address, stderr=log, start_new_session=True,
        )
    try:
        deadline = time.monotonic() + 5
        while not address_file.read_text().strip():
            if daemon.poll() is not None:
                raise RuntimeError(f"private D-Bus exited with {daemon.returncode}")
            if time.monotonic() >= deadline:
                raise TimeoutError("private D-Bus startup exceeded 5 seconds")
            time.sleep(0.02)
        published = address_file.read_text().strip().split(",guid=", 1)[0]
        if published != env["DBUS_SESSION_BUS_ADDRESS"] or not (runtime / "bus").is_socket():
            raise RuntimeError(f"unexpected private bus address: {published}")
        print(f"private D-Bus: pid={daemon.pid}, address={published}", file=log, flush=True)
        yield daemon
    finally:
        ignore()
        if daemon.poll() is None:
            daemon.terminate()
            try:
                daemon.wait(timeout=3)
            except subprocess.TimeoutExpired:
                print("private D-Bus required SIGKILL", file=log, flush=True)
                daemon.kill()
        daemon.wait(timeout=3)
        print(f"private D-Bus reaped: pid={daemon.pid}, status={daemon.returncode}",
              file=log, flush=True)
