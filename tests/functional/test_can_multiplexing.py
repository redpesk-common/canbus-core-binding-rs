#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Copyright (C) 2026 Ronan Le Martret
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

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



ROOT = Path(__file__).resolve().parents[2]
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))

# Binding shared objects (built by cargo)
SOCKCAN_SO = TARGET / "debug" / "libafb_sockcan.so"
MULTIPLEX_SO = TARGET / "debug" / "libafb_can_multiplexing.so"

# Two instances of the same binding with different uids/config
bindings = {
    "sockbcm": str(SOCKCAN_SO),
    "can-multiplexing": str(MULTIPLEX_SO),
}

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

def to_upper_camel_case(name: str, prefix_keywords: bool = True) -> str:
    """
    Rough equivalent of heck::ToUpperCamelCase for typical DBC identifiers.
    Example: "MUX_signal" -> "MuxSignal"
             "MUX_TEST"   -> GROUP_NAME
    """

    # split on underscores, dashes, spaces (adapt if you have other separators)
    parts = [p for p in re.split(r"[_\-\s]+", name) if p]
    return "".join(p[:1].upper() + p[1:].lower() for p in parts)

def _get_api_verb_name(msg_name: str, sig_name: str) -> str:
    msg_verb = to_upper_camel_case(msg_name)
    sig_verb = to_upper_camel_case(sig_name)
    return f"{msg_verb}/{sig_verb}"

# AFB API exposed by the multiplexing binding
CAN_API = "can-multiplexing"
GROUP_NAME = "MUX_TEST"

# Known multiplexed message frame id from the reference DBC (322 == 0x142)
FRAME_ID = 0x142

# Default SocketCAN interface
VCAN_IFACE = "vcan0"


def get_event_value_from_data(data: Dict[str, Any]) -> Any:
    return data.get("value")

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

def _subscribe_signal(binder, verb: str) -> None:
    """
    Subscribe to a signal via the binding.
    `signal_path` is expected to be "MessageName/SignalName".
    """

    r = libafb.callsync(
        binder,
        CAN_API,
        verb,
        {"action": "subscribe"},
    )
    # Avoid hardcoding the exact response string; only check it looks successful.
    assert len(r.args) == 1
    assert "Subscribe" in r.args[0]

def setUpModule():
    configure_afb_binding_tests(bindings=bindings, config=config)

class TestCanMultiplexing(AFBTestCase):

    def test_check_ok(self):
        print(f"test_check_ok -----------------------------------------------------------------")
        r1 = libafb.callsync(self.binder, "sockbcm", "check")
        assert r1.args == ()


    def test_mux_signal_event_emitted(self):
        """
        Subscribe to the multiplexer signal and verify at least one event is emitted
        when a multiplexed frame is sent.
        """
        print(f"test_mux_signal_event_emitted -----------------------------------------------------------------")
        _, msg = _load_mux_message()

        signal_path = "MUX_signal"
        api_verb = _get_api_verb_name(GROUP_NAME, signal_path)
        _subscribe_signal(self.binder, api_verb)

        bus = can.interface.Bus(channel=VCAN_IFACE, interface="socketcan")
        try:
            expected = {"U8": 0}
            with self.subTest(event=f"{CAN_API}/{api_verb}_1", expected=expected):
                # Send one frame with a valid page and assert an event is emitted.

                with self.assertEventEmitted(CAN_API, api_verb, timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
                    _send_encoded(bus, msg, {"MUX_signal": 0, "open": 0, "closed": 0, "direction": 0, "mode": 0})

            #/we need to sleep a bit to avoid mixing events
            time.sleep(0.5)

            expected = {"U8": 1}
            with self.subTest(event=f"{CAN_API}/{api_verb}_2", expected=expected):
                # Send one frame with a valid page and assert an event is emitted.
                with self.assertEventEmitted(CAN_API, api_verb, timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
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
        print(f"test_mux_0_signal_event_emitted -----------------------------------------------------------------")
        _, msg = _load_mux_message()
        bus = can.interface.Bus(channel=VCAN_IFACE, interface="socketcan")
        try:
            _send_encoded(bus, msg, {"MUX_signal": 0, "open": 0, "closed": 0, "direction": 0, "mode": 0})

            #/we need to sleep a bit to avoid mixing events
            time.sleep(1)

            expected = {'Bool': True}
            with self.subTest(event=f"test_mux_0_signal_event_emitted", expected=expected):
                signal_path = "open"
                api_verb = _get_api_verb_name(GROUP_NAME, signal_path)
                _subscribe_signal(self.binder, api_verb)

                # Send one frame with a valid page and assert an event is emitted.
                with self.assertEventEmitted(CAN_API, api_verb, timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
                        _send_encoded(bus, msg, {"MUX_signal": 0, "open": 1, "closed": 0, "direction": 0, "mode": 0})

                #/we need to sleep a bit to avoid mixing events
                time.sleep(0.5)
                signal_path = "closed"
                api_verb = _get_api_verb_name(GROUP_NAME, signal_path)
                _subscribe_signal(self.binder, api_verb)

                {'Bool': True}
                with self.subTest(event=f"{CAN_API}/{api_verb}", expected=expected):
                    # Send one frame with a valid page and assert an event is emitted.
                    with self.assertEventEmitted(CAN_API, api_verb, timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
                        _send_encoded(bus, msg, {"MUX_signal": 0, "open": 1, "closed": 1, "direction": 0, "mode": 0})

                #/we need to sleep a bit to avoid mixing events
                time.sleep(0.5)
                signal_path = "direction"
                api_verb = _get_api_verb_name(GROUP_NAME, signal_path)
                _subscribe_signal(self.binder, api_verb)

                expected = {"U8": 2}
                with self.subTest(event=f"{CAN_API}/{api_verb}", expected=expected):
                    # Send one frame with a valid page and assert an event is emitted.
                    with self.assertEventEmitted(CAN_API, api_verb, timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
                        _send_encoded(bus, msg, {"MUX_signal": 0, "open": 1, "closed": 1, "direction": 2, "mode": 0})

                #/we need to sleep a bit to avoid mixing events
                time.sleep(0.5)
                signal_path = "mode"
                api_verb = _get_api_verb_name(GROUP_NAME, signal_path)
                _subscribe_signal(self.binder, api_verb)

                expected = {"U8": 3}
                with self.subTest(event=f"{CAN_API}/{api_verb}", expected=expected):
                    # Send one frame with a valid page and assert an event is emitted.
                    with self.assertEventEmitted(CAN_API, api_verb, timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
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
        print(f"test_mux_1_signal_event_emitted -----------------------------------------------------------------")
        _, msg = _load_mux_message()
        bus = can.interface.Bus(channel=VCAN_IFACE, interface="socketcan")
        try:
            _send_encoded(bus, msg, {"MUX_signal": 1,"speed": 0, "temperature": 0})

            #/we need to sleep a bit to avoid mixing events
            time.sleep(0.5)

            signal_path = "speed"
            api_verb = _get_api_verb_name(GROUP_NAME, signal_path)
            _subscribe_signal(self.binder, api_verb)
            expected = {"F64": 51}
            with self.subTest(event=f"{CAN_API}/{api_verb}", expected=expected):
                # Send one frame with a valid page and assert an event is emitted.
                with self.assertEventEmitted(CAN_API, api_verb, timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
                    _send_encoded(bus, msg, {"MUX_signal": 1,"speed": 51, "temperature": 0})

            #/we need to sleep a bit to avoid mixing events
            time.sleep(0.5)
            signal_path = "temperature"
            api_verb = _get_api_verb_name(GROUP_NAME, signal_path)
            _subscribe_signal(self.binder, api_verb)
            expected = {"I8": 37}
            with self.subTest(event=f"{CAN_API}/{api_verb}", expected=expected):
                # Send one frame with a valid page and assert an event is emitted.
                with self.assertEventEmitted(CAN_API, api_verb, timeout_ms=5000, value=expected, value_lambda=get_event_value_from_data):
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
