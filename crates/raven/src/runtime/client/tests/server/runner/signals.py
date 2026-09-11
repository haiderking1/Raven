"""Keep repeated outer timeout signals from interrupting owned-resource teardown."""
import signal

SIGNALS = (signal.SIGTERM, signal.SIGINT, signal.SIGHUP)


class Interrupted(Exception):
    def __init__(self, signum):
        self.signum = signum
        super().__init__(f"runner interrupted by signal {signum}")


def ignore():
    for sig in SIGNALS:
        signal.signal(sig, signal.SIG_IGN)


def interrupt(signum, _frame):
    ignore()
    raise Interrupted(signum)


def install():
    for sig in SIGNALS:
        signal.signal(sig, interrupt)
