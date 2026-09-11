"""Bound the existing cargo test and stop its owned process group on interruption."""
import os
import signal
import subprocess

from .signals import ignore

TEST = "runtime::client::tests::installed_satellite_maps_and_fullscreens_through_real_scene"


def run(repo, env, log):
    command = ["cargo", "test", "--locked", "--offline", "-p", "raven", TEST,
               "--", "--ignored", "--exact", TEST, "--nocapture"]
    child = subprocess.Popen(command, cwd=repo, env=env, stdout=log, stderr=log,
                             start_new_session=True)
    print(f"owned cargo process group: {child.pid}", file=log, flush=True)
    try:
        try:
            return child.wait(timeout=90)
        except subprocess.TimeoutExpired:
            print("cargo regression exceeded 90 seconds", file=log, flush=True)
            return 124
    finally:
        ignore()
        # A failed test can leave non-setsid children even after cargo exits.
        # This group was created by this Popen, never an ambient session group.
        try:
            os.killpg(child.pid, 0)
            print(f"cargo-group cleanup: SIGTERM pgid={child.pid}", file=log, flush=True)
            os.killpg(child.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            child.wait(timeout=3)
        except subprocess.TimeoutExpired:
            pass
        try:
            os.killpg(child.pid, 0)
            print(f"cargo-group cleanup: SIGKILL pgid={child.pid}", file=log, flush=True)
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        child.wait(timeout=3)
