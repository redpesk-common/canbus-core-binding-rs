# tests/functional/test_can_multiplexing.py
import os
import random
import subprocess
import time
from pathlib import Path
from typing import Dict, Any

import can
import cantools

from afb_test import AFBTestCase, configure_afb_binding_tests, run_afb_binding_tests, EventAssertionError
import libafb
import re

def to_upper_camel_case(name: str, prefix_keywords: bool = True) -> str:
    """
    Rough equivalent of heck::ToUpperCamelCase for typical DBC identifiers.
    Example: "MUX_signal" -> "MuxSignal"
             "MUX_TEST"   -> "MuxTest"
    """

    # split on underscores, dashes, spaces (adapt if you have other separators)
    parts = [p for p in re.split(r"[_\-\s]+", name) if p]
    return "".join(p[:1].upper() + p[1:].lower() for p in parts)


# Linux capability number for CAP_NET_ADMIN
_CAP_NET_ADMIN_BIT = 12

ROOT = Path(__file__).resolve().parents[2]
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))

# AFB API exposed by the multiplexing binding
CAN_API = "can-multiplexing"

# Binding shared objects (built by cargo)
SOCKCAN_SO = TARGET / "debug" / "libafb_sockcan.so"
MULTIPLEX_SO = TARGET / "debug" / "libafb_can_multiplexing.so"

# Two instances of the same binding with different uids/config
bindings = {
    "sockbcm": str(SOCKCAN_SO),
    "can-multiplexing": str(MULTIPLEX_SO),
}

GROUP_NAME = "MUX_TEST"

# Per-binding instance configuration (keys must match `bindings` keys)
config = {
    "libafb_sockcan.so": {
        "sock_api": "sockcan",
        "sock_evt": "sockbcm",
        "uid": "sockbcm",
    },
    "libafb_can_multiplexing.so": {
        "sock_api": "sockbcm",
        "sock_evt": "sockcan",
        "uid": "can-multiplexing",
    },
}



# Test assets
SCRIPT_DIR = Path(__file__).resolve().parent
DBC_FILE = (SCRIPT_DIR / ".." / ".." / "examples" / "samples" / "dbc_multiplexing" / "dbc" / "multiplexing.dbc").resolve()

# Known multiplexed message frame id from the reference DBC (322 == 0x142)
FRAME_ID = 0x142

# Default SocketCAN interface
VCAN_IFACE = "vcan0"


def get_event_value_from_data(data: Dict[str, Any]) -> Any:
    return data.get("value")

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
        return False


def _ensure_net_admin_privileges(what: str = "manage network links") -> None:
    """
    Raise if the process lacks privileges typically required for `ip link ...`.
    Accepts either root or CAP_NET_ADMIN.
    """
    if _is_root() or _has_cap_net_admin():
        return
    raise PrivilegeError(
        f"Insufficient privileges to {what}. Need root (uid=0) or CAP_NET_ADMIN."
    )


def _setup_vcan() -> None:
    """Create and bring up vcan0 if needed."""
    _ensure_net_admin_privileges("set up vcan0")

    subprocess.run(["modprobe", "vcan"], check=False)
    subprocess.run(["modprobe", "can-bcm"], check=False)

    r = subprocess.run(["/usr/bin/ip", "link", "show", VCAN_IFACE], capture_output=True, text=True)
    if r.returncode != 0:
        subprocess.run(["/usr/bin/ip", "link", "add", "dev", VCAN_IFACE, "type", "vcan"], check=True)

    subprocess.run(["/usr/bin/ip", "link", "set", "up", VCAN_IFACE], check=True)


def _un_setup_vcan(remove_modules: bool = False) -> None:
    """Tear down vcan0 if present (best-effort)."""
    _ensure_net_admin_privileges("tear down vcan0")

    r = subprocess.run(["ip", "link", "show", VCAN_IFACE], capture_output=True, text=True)
    if r.returncode == 0:
        subprocess.run(["ip", "link", "set", "down", VCAN_IFACE], check=False)
        subprocess.run(["ip", "link", "del", "dev", VCAN_IFACE], check=False)

    if remove_modules:
        subprocess.run(["modprobe", "-r", "can-bcm"], check=False)
        subprocess.run(["modprobe", "-r", "vcan"], check=False)


def _load_mux_message():
    """Load multiplexing.dbc and return (db, msg)."""
    if not DBC_FILE.exists():
        raise FileNotFoundError(f"Missing DBC file: {DBC_FILE}")
    db = cantools.database.load_file(DBC_FILE)
    msg = db.get_message_by_frame_id(FRAME_ID)
    return db, msg


def _send_encoded(bus: can.Bus, msg, values: Dict[str, Any]) -> bytes:
    """Encode with cantools and send on the CAN bus."""
    data = msg.encode(values)
    tx = can.Message(arbitration_id=msg.frame_id, data=data, is_extended_id=False)
    bus.send(tx)
    return data


def _subscribe_signal(binder, signal_path: str) -> None:
    """
    Subscribe to a signal via the binding.
    `signal_path` is expected to be "MessageName/SignalName".
    """
    path_verb = to_upper_camel_case(signal_path)
    binding_verb = to_upper_camel_case(GROUP_NAME) + "/" + path_verb
    print(f"Subscribing to signal event: {binding_verb}")

    r = libafb.callsync(
        binder,
        CAN_API,
        binding_verb,
        {"action": "subscribe"},
    )
    print(f"  Subscribe response r.args: {r.args}")
    print(f"  Subscribe path_verb: {path_verb}")
    print(f"  Subscribe response r.args[0]: {r.args[0]}")
    # Avoid hardcoding the exact response string; only check it looks successful.
    assert len(r.args) == 1
    assert "Subscribe" in r.args[0]
    assert path_verb in r.args[0]


def setUpModule():
    configure_afb_binding_tests(bindings=bindings, config=config)

class TestCanMultiplexing(AFBTestCase):

    def test_check_ok(self):
        r1 = libafb.callsync(self.binder, "sockbcm", "check")
        assert r1.args == ()

    def test_mux_signal_event_emitted(self):
        """
        Subscribe to the multiplexer signal and verify at least one event is emitted
        when a multiplexed frame is sent.
        """
        _, msg = _load_mux_message()

        signal_path = "MUX_signal"
        _subscribe_signal(self.binder, signal_path)

        bus = can.interface.Bus(channel=VCAN_IFACE, interface="socketcan")
        try:
            expected = {"U8": 0}
            with self.subTest(event=f"{CAN_API}/{to_upper_camel_case(signal_path)}", expected=expected):
                # Send one frame with a valid page and assert an event is emitted.
                with self.assertEventEmitted(CAN_API, to_upper_camel_case(signal_path), timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
                    _send_encoded(bus, msg, {"MUX_signal": 0, "open": 0, "closed": 0, "direction": 2, "mode": 3})

            #/we need to sleep a bit to avoid mixing events
            time.sleep(0.5)

            expected = {"U8": 1}
            with self.subTest(event=f"{CAN_API}/{to_upper_camel_case(signal_path)}", expected=expected):
                # Send one frame with a valid page and assert an event is emitted.
                with self.assertEventEmitted(CAN_API, to_upper_camel_case(signal_path), timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
                    _send_encoded(bus, msg, {"MUX_signal": 1,"speed": 0, "temperature": 0})


        except EventAssertionError as e:
            print(f"[AFB] {e}")
            raise
        finally:
            bus.shutdown()


    def test_mux_0_signal_event_emitted(self):
        """
        Subscribe to the multiplexer signal and verify at least one event is emitted
        when a multiplexed frame is sent.
        """
        _, msg = _load_mux_message()
        bus = can.interface.Bus(channel=VCAN_IFACE, interface="socketcan")
        try:
            _send_encoded(bus, msg, {"MUX_signal": 0, "open": 0, "closed": 0, "direction": 0, "mode": 0})

            #/we need to sleep a bit to avoid mixing events
            time.sleep(0.5)

            signal_path = "open"
            _subscribe_signal(self.binder, signal_path)
            expected = {'Bool': True}
            with self.subTest(event=f"{CAN_API}/{to_upper_camel_case(signal_path)}", expected=expected):
                # Send one frame with a valid page and assert an event is emitted.
                with self.assertEventEmitted(CAN_API, to_upper_camel_case(signal_path), timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
                    _send_encoded(bus, msg, {"MUX_signal": 0, "open": 1, "closed": 0, "direction": 0, "mode": 0})

            #/we need to sleep a bit to avoid mixing events
            time.sleep(0.5)
            signal_path = "closed"
            _subscribe_signal(self.binder, signal_path)

            {'Bool': True}
            with self.subTest(event=f"{CAN_API}/{to_upper_camel_case(signal_path)}", expected=expected):
                # Send one frame with a valid page and assert an event is emitted.
                with self.assertEventEmitted(CAN_API, to_upper_camel_case(signal_path), timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
                    _send_encoded(bus, msg, {"MUX_signal": 0, "open": 1, "closed": 1, "direction": 0, "mode": 0})

            #/we need to sleep a bit to avoid mixing events
            time.sleep(0.5)
            signal_path = "direction"
            _subscribe_signal(self.binder, signal_path)

            expected = {"U8": 2}
            with self.subTest(event=f"{CAN_API}/{to_upper_camel_case(signal_path)}", expected=expected):
                # Send one frame with a valid page and assert an event is emitted.
                with self.assertEventEmitted(CAN_API, to_upper_camel_case(signal_path), timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
                    _send_encoded(bus, msg, {"MUX_signal": 0, "open": 1, "closed": 1, "direction": 2, "mode": 0})

            #/we need to sleep a bit to avoid mixing events
            time.sleep(0.5)
            signal_path = "mode"
            _subscribe_signal(self.binder, signal_path)

            expected = {"U8": 3}
            with self.subTest(event=f"{CAN_API}/{to_upper_camel_case(signal_path)}", expected=expected):
                # Send one frame with a valid page and assert an event is emitted.
                with self.assertEventEmitted(CAN_API, to_upper_camel_case(signal_path), timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
                    _send_encoded(bus, msg, {"MUX_signal": 0, "open": 1, "closed": 1, "direction": 2, "mode": 3})


        except EventAssertionError as e:
            print(f"[AFB] {e}")
            raise
        finally:
            bus.shutdown()


    def test_mux_1_signal_event_emitted(self):
        """
        Subscribe to the multiplexer signal and verify at least one event is emitted
        when a multiplexed frame is sent.
        """
        _, msg = _load_mux_message()
        bus = can.interface.Bus(channel=VCAN_IFACE, interface="socketcan")
        try:
            _send_encoded(bus, msg, {"MUX_signal": 1,"speed": 0, "temperature": 0})

            #/we need to sleep a bit to avoid mixing events
            time.sleep(0.5)

            signal_path = "speed"
            _subscribe_signal(self.binder, signal_path)
            expected = {"F64": 51}
            with self.subTest(event=f"{CAN_API}/{to_upper_camel_case(signal_path)}", expected=expected):
                # Send one frame with a valid page and assert an event is emitted.
                with self.assertEventEmitted(CAN_API, to_upper_camel_case(signal_path), timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
                    _send_encoded(bus, msg, {"MUX_signal": 1,"speed": 51, "temperature": 0})

            #/we need to sleep a bit to avoid mixing events
            time.sleep(0.5)
            signal_path = "temperature"
            _subscribe_signal(self.binder, signal_path)

            expected = {"I8": 37}
            with self.subTest(event=f"{CAN_API}/{to_upper_camel_case(signal_path)}", expected=expected):
                # Send one frame with a valid page and assert an event is emitted.
                with self.assertEventEmitted(CAN_API, to_upper_camel_case(signal_path), timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
                    _send_encoded(bus, msg, {"MUX_signal": 1,"speed": 0, "temperature": 37})

        except EventAssertionError as e:
            print(f"[AFB] {e}")
            raise
        finally:
            bus.shutdown()

if __name__ == "__main__":
    #_setup_vcan()
    try:
        run_afb_binding_tests(bindings)
    finally:
        pass
        #_un_setup_vcan(remove_modules=True)
