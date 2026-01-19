# tests/functional/test_sockcan_replay.py
import os
import subprocess
import time
from pathlib import Path
from typing import Optional

# Linux capability number for CAP_NET_ADMIN
_CAP_NET_ADMIN_BIT = 12


from afb_test import AFBTestCase, configure_afb_binding_tests, run_afb_binding_tests

import libafb

ROOT = Path(__file__).resolve().parents[2]
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))

SOCKCAN_SO = TARGET / "debug" / "libafb_sockcan.so"
MODEL3_SO = TARGET / "debug" / "libafb_model3.so"

MODEL3_LOG = ROOT / "examples" / "samples" / "model3" / "candump" / "model3.log"

bindings = {"sockbcm": str(SOCKCAN_SO), "model3": str(MODEL3_SO)}

config = {
            "libafb_sockcan.so": {
                "sock_api": "sockcan",
                "sock_evt": "sockbcm",
                "uid":"sockbcm"
            },
            "libafb_model3.so": {
                "sock_api": "sockbcm",
                "sock_evt": "sockcan",
                "uid":"model3"
            }
        }

class PrivilegeError(PermissionError):
    pass

def _has_cap_net_admin() -> bool:
    """
    Return True if the current process has CAP_NET_ADMIN in its effective set.
    Works on Linux by reading /proc/self/status (CapEff).
    """
    try:
        with open("/proc/self/status", "r", encoding="utf-8") as f:
            for line in f:
                if line.startswith("CapEff:"):
                    capeff_hex = line.split(":", 1)[1].strip()
                    capeff = int(capeff_hex, 16)
                    return bool(capeff & (1 << _CAP_NET_ADMIN_BIT))
    except (FileNotFoundError, ValueError, OSError):
        pass
    return False

def _is_root() -> bool:
    """Return True if running as root (UID 0)."""
    try:
        return os.geteuid() == 0
    except AttributeError:
        # Non-POSIX platforms
        return False

def _ensure_net_admin_privileges(what: str = "manage network links") -> None:
    """
    Raise if the process lacks privileges typically required for `ip link ...`.
    Accepts either root or CAP_NET_ADMIN.
    """
    if _is_root() or _has_cap_net_admin():
        return
    raise PrivilegeError(
        f"Insufficient privileges to {what}. "
        f"Need root (uid=0) or CAP_NET_ADMIN."
    )

def _setup_vcan() -> None:
    # These commands may require root or CAP_NET_ADMIN in CI.
    _ensure_net_admin_privileges("set up vcan0")

    subprocess.run(["modprobe", "vcan"], check=False)
    subprocess.run(["modprobe", "can-bcm"], check=False)

    # Create vcan0 if missing
    r = subprocess.run(["ip", "link", "show", "vcan0"], capture_output=True, text=True)
    if r.returncode != 0:
        subprocess.run(["ip", "link", "add", "dev", "vcan0", "type", "vcan"], check=True)

    subprocess.run(["ip", "link", "set", "up", "vcan0"], check=True)


def _un_setup_vcan(remove_modules: bool = False) -> None:
    """
    Tear down vcan0 if present.
    Optionally tries to unload kernel modules (best-effort) if remove_modules=True.
    """
    _ensure_net_admin_privileges("tear down vcan0")

    # If the link exists, bring it down and delete it.
    r = subprocess.run(["ip", "link", "show", "vcan0"], capture_output=True, text=True)
    if r.returncode == 0:
        subprocess.run(["ip", "link", "set", "down", "vcan0"], check=False)
        subprocess.run(["ip", "link", "del", "dev", "vcan0"], check=False)

    if remove_modules:
        # Best-effort: modules may be in use by other interfaces/processes.
        subprocess.run(["modprobe", "-r", "can-bcm"], check=False)
        subprocess.run(["modprobe", "-r", "vcan"], check=False)

def setUpModule():
    # Note: afb-test-py passes {"uid": <binding_uid>, "path": <so>} as JSON config.
    # parse_sockcan_config() uses "uid" as api_uid. So api name will be "sockcan".
    configure_afb_binding_tests(bindings=bindings, config=config)


class TestSockcanReplay(AFBTestCase):

    def test_check_ok(self):
        print("___________________________________________________________ test_check_ok")
        r = libafb.callsync(self.binder, "sockbcm", "check")
        assert r.args == ()


    def test_subscribe_receives_event_on_replay1(self):
        print("___________________________________________________________ test_subscribe_receives_event_on_replay1")

        r=libafb.callsync(
            self.binder,
            "model3",
            "DiSpeedCounter",
            {"action": "subscribe"},
        )
        assert r.args == ('Subscribe (canid:599) sig:DiSpeedCounter OK',)

        with self.assertEventEmitted("model3", "DiSpeedCounter", timeout_ms=5000):
            # Replay the log: mapping from recorded interface "elmcan" to vcan0
            p = subprocess.Popen(
                ["canplayer", "vcan0=elmcan", "-l", "i", "-g", "1", "-I", str(MODEL3_LOG)],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            time.sleep(1)
            p.terminate()
            rc = p.wait(timeout=6)

if __name__ == "__main__":
    #_setup_vcan()
    run_afb_binding_tests(bindings)
    #_un_setup_vcan(remove_modules=True)
