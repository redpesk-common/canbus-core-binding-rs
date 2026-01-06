# tests/functional/test_sockcan_replay.py
import os
import subprocess
import time
from pathlib import Path

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

def _setup_vcan():
    # These commands may require root or CAP_NET_ADMIN in CI.
    subprocess.run(["modprobe", "vcan"], check=False)
    subprocess.run(["modprobe", "can-bcm"], check=False)

    # Create vcan0 if missing
    r = subprocess.run(["ip", "link", "show", "vcan0"], capture_output=True)
    if r.returncode != 0:
        subprocess.run(["ip", "link", "add", "dev", "vcan0", "type", "vcan"], check=True)

    subprocess.run(["ip", "link", "set", "up", "vcan0"], check=True)


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
        #_setup_vcan()

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
    run_afb_binding_tests(bindings)
