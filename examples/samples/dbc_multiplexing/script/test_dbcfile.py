#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import argparse
import random
import time

import can
import cantools

from pathlib import Path

def clamp(x, lo, hi):
    # English comments only
    return lo if x < lo else hi if x > hi else x


def main():
    ap = argparse.ArgumentParser(description="Random mux test sender/decoder for a multiplexed DBC message.")
    ap.add_argument("--channel", default="vcan0", help="SocketCAN channel (default: vcan0).")
    ap.add_argument("--bustype", default="socketcan", help="python-can bustype (default: socketcan).")
    ap.add_argument("--period-ms", type=float, default=100.0, help="Period between frames (default: 100ms).")
    ap.add_argument("--count", type=int, default=50, help="Number of frames to send (default: 50).")
    ap.add_argument("--seed", type=int, default=None, help="Random seed (default: None).")
    args = ap.parse_args()

    rng = random.Random(args.seed)

    SCRIPT_DIR = Path(__file__).resolve().parent
    dbc_file = (SCRIPT_DIR / ".." / "dbc" / "multiplexing.dbc").resolve()
    frame_id=0x142
    db = cantools.database.load_file(dbc_file)
    msg = db.get_message_by_frame_id(frame_id)

    bus = can.interface.Bus(channel=args.channel, interface=args.bustype)

    period_s = max(0.0, args.period_ms / 1000.0)

    print(f"DBC loaded: {dbc_file}")
    print(f"Message: name={msg.name} frame_id=0x{frame_id:X} dlc={msg.length}")
    print(f"Sending on {args.bustype}:{args.channel} every {args.period_ms} ms, count={args.count}\n")

    try:
        for i in range(args.count):
            mux = rng.choice([0, 1, 2])  # 2 = invalid page

            if mux == 0:
                values = {
                    "MUX_signal": 0,
                    "open": rng.randint(0, 1),
                    "closed": rng.randint(0, 1),
                    "direction": rng.randint(0, 3),
                    "mode": rng.randint(0, 7),
                }
            elif mux == 1:
                # speed: [-51.2..51.1] (factor 0.1)
                speed = round(rng.uniform(-51.2, 51.1), 1)
                # temperature: [-128..127]
                temp = int(round(rng.uniform(-128, 127)))

                values = {
                    "MUX_signal": 1,
                    "speed": speed,
                    "temperature": temp,
                }
            else:
                values = {"MUX_signal": 2}

            # Encode, send
            data = msg.encode(values)
            tx = can.Message(arbitration_id=msg.frame_id, data=data, is_extended_id=False)
            bus.send(tx)

            # Decode back locally (reference) and print
            decoded = msg.decode(data, decode_choices=True)

            # Print a compact line
            hexdata = data.hex().upper()
            print(f"[{i:04d}] id=0x{frame_id:X} mux={mux} data={hexdata} values={values} decoded={decoded}")

            if period_s:
                time.sleep(period_s)

    finally:
        bus.shutdown()


if __name__ == "__main__":
    main()
