"""Run the single ignored integration with owned resources and bounded teardown."""
from datetime import datetime
import os
from pathlib import Path
import shutil
import sys
import traceback

from .bus import private_bus
from .command import run
from .isolation import child_environment, create_runtime
from .processes import become_subreaper, cleanup
from .signals import Interrupted, ignore, install
from .sockets import cleanup as cleanup_sockets


def main():
    repo = Path(__file__).resolve().parents[8]
    become_subreaper()
    runtime, logs = create_runtime(repo)
    log_path = logs / f"satellite-waybar-{datetime.now():%Y%m%d-%H%M%S}-{os.getpid()}.log"
    print(f"isolated regression log: {log_path}", flush=True)
    status = 1
    with log_path.open("w") as log:
        print(f"private runtime: {runtime}", file=log, flush=True)
        install()
        try:
            env = child_environment(runtime, logs)
            with private_bus(runtime, env, log):
                status = run(repo, env, log)
        except Interrupted as error:
            status = 128 + error.signum
            print(str(error), file=log, flush=True)
        except Exception:
            traceback.print_exc(file=log)
            status = 1
        finally:
            # Do not let a repeated timeout signal interrupt kill/wait or unlink.
            ignore()
            try:
                forced = cleanup(runtime, status != 0, log)
                sockets_removed = cleanup_sockets(runtime, status != 0 or bool(forced), log)
                if (forced or sockets_removed) and status == 0:
                    status = 1
                shutil.rmtree(runtime)
                print("private runtime removed", file=log, flush=True)
            except Exception:
                traceback.print_exc(file=log)
                print(f"cleanup incomplete; retained private runtime {runtime}", file=log, flush=True)
                status = 1
        print(f"isolated regression exit: {status}", file=log, flush=True)
    with log_path.open() as log:
        shutil.copyfileobj(log, sys.stdout)
    return status


if __name__ == "__main__":
    sys.exit(main())
