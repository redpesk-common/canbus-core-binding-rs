#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
#
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

from __future__ import annotations

import argparse
import enum
import json
import logging
import os
import random
from pathlib import Path
import sys
import time
from typing import Any, Dict, Iterable, List, Optional, Set, Tuple

import can
import cantools
import libafb
from afb_test import (
    AFBTestCase,
    EventAssertionError,
    configure_afb_binding_tests,
    run_afb_binding_tests,
)

# -----------------------------------------------------------------------------
# Paths / bindings
# -----------------------------------------------------------------------------
LOG = logging.getLogger("canbus-binding-functional-test")

ROOT = Path(__file__).resolve().parents[2]
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))

CAN_API = ""

# These globals can be initialized from the binder config JSON and/or CLI.
DBC_FILE = (
    ROOT / "examples" / "samples" / "model3" / "dbc" / "model3can.dbc"
).resolve()
VCAN_IFACE = os.environ.get("VCAN_IFACE", "vcan0")

bindings: Dict[str, str] = {}
config: Dict[str, Dict[str, Any]] = {}

# -----------------------------------------------------------------------------
# Test selection / parameters
# -----------------------------------------------------------------------------

_canids: Optional[Iterable[int]] = None
# _canids = (526,)  # Uncomment to focus locally.

RANDOM_SEED = int(os.environ.get("TEST_SEED", time.time()))
ALLOWED_CANIDS: Optional[Set[int]] = None


def _setup_logging(*, quiet: bool, verbose: bool, debug: bool) -> None:
    level = logging.WARNING
    if quiet:
        level = logging.ERROR
    elif debug:
        level = logging.DEBUG
    elif verbose:
        level = logging.INFO

    logging.basicConfig(
        level=level,
        format="%(levelname)s: %(message)s",
    )


def _read_json(path: Path) -> Dict[str, Any]:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def _expand_vars(s: str) -> str:
    return os.path.expandvars(s)


def _parse_int_token(token: str) -> int:
    """
    Parse a CAN ID token:
      - decimal: 123
      - hex: 0x7B
    Underscores are allowed (e.g. 0x1A_2B).
    """
    t = token.strip().lower().replace("_", "")
    if not t:
        raise ValueError("empty integer token")
    base = 16 if t.startswith("0x") else 10
    return int(t, base)


def _resolve_path(value: str, *, base_dir: Path) -> Path:
    """
    Expand env vars + user, then resolve relative paths against the config directory.
    This makes binder-config.json relocatable.
    """
    raw = _expand_vars(str(value))
    p = Path(raw).expanduser()
    if not p.is_absolute():
        p = base_dir / p
    return p.resolve()


def _as_int_list(v: Any) -> Optional[List[int]]:
    if v is None:
        return None
    if isinstance(v, str):
        s = v.strip()
        if not s:
            return []
        # Support both single token ("0x221") and CSV ("0x221,0x222").
        tokens = [t.strip() for t in s.split(",") if t.strip()]
        return [_parse_int_token(t) for t in tokens]
    if isinstance(v, list):
        out: List[int] = []
        for x in v:
            if isinstance(x, int):
                out.append(x)
                continue
            if isinstance(x, str):
                t = x.strip().lower()
                base = 16 if t.startswith("0x") else 10
                out.append(int(t, base))
                continue
            out.append(int(x))
        return out
    if isinstance(v, int):
        return [v]
    return [_parse_int_token(str(v))]


def _init_from_binder_config(path: Path) -> None:
    global CAN_API, DBC_FILE, VCAN_IFACE, bindings, config, _canids, ALLOWED_CANIDS

    if not path.exists():
        LOG.warning("Binder config not found: %s (fallback to defaults)", path)
        return

    base_dir = path.parent

    # Ensure ${CARGO_TARGET_DIR} expands even if the user didn't export it.
    os.environ.setdefault("CARGO_TARGET_DIR", str(TARGET))

    doc = _read_json(path)
    if not isinstance(doc, dict):
        raise ValueError("Invalid binder config: root JSON value must be an object")

    # Optional fields.
    CAN_API = str(doc.get("can_api", CAN_API))

    dbc_file = doc.get("dbc_file")
    if dbc_file:
        DBC_FILE = _resolve_path(str(dbc_file), base_dir=base_dir)

    vcan_iface = doc.get("vcan_iface")
    if vcan_iface:
        VCAN_IFACE = str(vcan_iface)
    # Keep env override behavior.
    VCAN_IFACE = os.environ.get("VCAN_IFACE", VCAN_IFACE)

    canids = doc.get("canids")
    parsed_canids = _as_int_list(canids)
    if parsed_canids is not None:
        _canids = tuple(parsed_canids)

    # "set" maps library basename -> per-binding config (and now includes "uid").
    set_cfg = doc.get("set", {})
    if set_cfg is None:
        set_cfg = {}
    if not isinstance(set_cfg, dict):
        raise ValueError(
            f"Invalid binder config: 'set' must be an object (got {type(set_cfg)})"
        )

    # Validate set entries early (industrial-grade: fail fast).
    for so_name, entry in set_cfg.items():
        if not isinstance(entry, dict):
            raise ValueError(
                f"Invalid binder config: set['{so_name}'] must be an object (got {type(entry)})"
            )

        uid = entry.get("uid")
        if not isinstance(uid, str) or not uid.strip():
            raise ValueError(
                f"Invalid binder config: set['{so_name}'].uid must be a non-empty string"
            )

        # These are required for your runtime behavior; enforce hard.
        if "sock_api" not in entry or "sock_evt" not in entry:
            raise ValueError(
                f"Invalid binder config: set['{so_name}'] must contain 'sock_api' and 'sock_evt'"
            )

    # Required structure: "binding": [{"path": "..."}]
    bind_list = doc.get("binding", [])
    if not isinstance(bind_list, list) or not bind_list:
        raise ValueError("Invalid binder config: 'binding' must be a non-empty list")

    new_bindings: Dict[str, str] = {}

    for item in bind_list:
        if not isinstance(item, dict):
            raise ValueError(
                "Invalid binder config: each 'binding' entry must be an object"
            )

        so_path = item.get("path")
        if not so_path:
            raise ValueError(
                "Invalid binder config: each 'binding' entry must contain 'path'"
            )

        resolved = _resolve_path(str(so_path), base_dir=base_dir)
        if not resolved.exists():
            raise FileNotFoundError(f"Binding shared object not found: {resolved}")

        so_name = resolved.name

        # uid is now in set[so_name]
        entry = set_cfg.get(so_name)
        if entry is None:
            raise ValueError(
                f"Invalid binder config: missing 'set[\"{so_name}\"]' entry for binding path {resolved}"
            )

        uid = entry.get("uid")
        if not isinstance(uid, str) or not uid.strip():
            raise ValueError(
                f"Invalid binder config: set['{so_name}'].uid must be a non-empty string"
            )

        if uid in new_bindings and Path(new_bindings[uid]).name != so_name:
            raise ValueError(
                f"Invalid binder config: duplicate uid '{uid}' used for multiple bindings "
                f"({new_bindings[uid]} and {resolved})"
            )

        new_bindings[uid] = str(resolved)

    # Build config entries keyed by the shared-object basename (libafb_*.so).
    new_config: Dict[str, Dict[str, Any]] = {}

    for uid, so_path in new_bindings.items():
        so_name = Path(so_path).name
        entry = set_cfg.get(so_name, {})
        if not isinstance(entry, dict):
            raise ValueError(
                f"Invalid binder config: set['{so_name}'] must be an object"
            )

        # Sanity: ensure uid matches what we used.
        entry_uid = entry.get("uid")
        if entry_uid != uid:
            raise ValueError(
                f"Invalid binder config: uid mismatch for {so_name}: binding uses '{uid}' "
                f"but set['{so_name}'].uid is '{entry_uid}'"
            )

        # Keep only what the test harness needs (plus optional info/uid for diagnostics).
        filtered: Dict[str, Any] = {
            "sock_api": entry["sock_api"],
            "sock_evt": entry["sock_evt"],
        }
        if "info" in entry:
            filtered["info"] = entry["info"]
        # Keeping uid can help debugging; harmless if unused downstream.
        filtered["uid"] = uid

        new_config[so_name] = filtered

    # Apply.
    bindings = new_bindings
    config = new_config

    # Recompute canid filter now that _canids may have changed.
    ALLOWED_CANIDS = _parse_canids_env()


def _parse_canids_env() -> Optional[Set[int]]:
    raw = os.environ.get("TEST_CANIDS", "").strip()
    if not raw:
        return None if _canids is None else {int(x) for x in _canids}
    if raw.lower() == "all":
        return None

    canids: Set[int] = set()
    for token in raw.split(","):
        t = token.strip().lower()
        if not t:
            continue
        canids.add(_parse_int_token(t))
    return canids


# -----------------------------------------------------------------------------
# CAN-ish casing (ported / adapted from heck)
# -----------------------------------------------------------------------------


class _WordMode(enum.Enum):
    Boundary = 0
    Lowercase = 1
    Uppercase = 2


def _split_alnum_runs(s: str) -> Iterable[str]:
    # Equivalent to Rust `split(|c: char| !c.is_alphanumeric())`.
    buf: List[str] = []
    for ch in s:
        if ch.isalnum():
            buf.append(ch)
            continue
        if buf:
            yield "".join(buf)
            buf.clear()
    if buf:
        yield "".join(buf)


def _lowercase_like_heck(s: str) -> str:
    # Port of heck::lowercase(), including final-sigma handling.
    if not s:
        return ""
    chars = list(s)
    out: List[str] = []
    last = len(chars) - 1
    for i, c in enumerate(chars):
        if i == last and c == "Σ":
            out.append("ς")
        else:
            out.append(c.lower())
    return "".join(out)


def _capitalize_like_heck(s: str) -> str:
    # Port of heck::capitalize().
    if not s:
        return ""
    if len(s) == 1:
        return s[0].upper()
    return s[0].upper() + _lowercase_like_heck(s[1:])


def _capitalize_segment(seg: str, *, is_last_segment: bool, prev_is_lower: bool) -> str:
    """
    Like heck::capitalize(), with deterministic CAN/DBC tweaks.

    prev_is_lower=True means the character just before this segment (in the same alnum-run)
    is a lowercase letter. This is used to decide 12V vs 12v behavior.
    """
    if not seg:
        return ""

    # Count leading digits.
    i = 0
    n = len(seg)
    while i < n and seg[i].isdigit():
        i += 1

    if i == 0:
        return _capitalize_like_heck(seg)

    digits = seg[:i]
    rest = seg[i:]
    if not rest:
        return digits

    # digits + single letter
    if len(rest) == 1 and rest.isalpha():
        # Keep 'V' only when preceded by a lowercase letter (e.g. ota12VSupport).
        if rest in ("V", "v"):
            return digits + ("V" if (rest == "V" and prev_is_lower) else "v")

        # Preserve single-letter suffix at end (e.g. 2I, 31F).
        if is_last_segment and rest.isupper():
            return digits + rest

        return digits + _lowercase_like_heck(rest)

    # Preserve hex-ish suffixes only at end: [0-9A-F]+ with at least one A-F and one digit.
    if is_last_segment:
        is_hex_charset = all(c.isdigit() or ("A" <= c <= "F") for c in rest)
        has_hex_letter = any("A" <= c <= "F" for c in rest)
        has_digit = any(c.isdigit() for c in rest)
        if is_hex_charset and has_hex_letter and has_digit:
            return digits + rest

    return digits + _lowercase_like_heck(rest)


def to_upper_camel_case(s: str) -> str:
    out: List[str] = []

    for word in _split_alnum_runs(s):
        chars = list(word)
        n = len(chars)
        init = 0
        mode = _WordMode.Boundary

        i = 0
        while i < n:
            c = chars[i]

            if c.islower():
                next_mode = _WordMode.Lowercase
            elif c.isupper():
                next_mode = _WordMode.Uppercase
            else:
                next_mode = mode

            if i + 1 < n:
                nxt = chars[i + 1]
                nxt2 = chars[i + 2] if i + 2 < n else None
                nxt3 = chars[i + 3] if i + 3 < n else None
                prev = chars[i - 1] if i - 1 >= 0 else None

                # alpha->digit boundary (but don't split inside hex like "1A5").
                if c.isalpha() and nxt.isdigit():
                    inside_hex = prev is not None and prev.isdigit() and c in "ABCDEF"
                    if not inside_hex:
                        seg = word[init : i + 1]
                        prev_is_lower = init > 0 and word[init - 1].islower()
                        out.append(
                            _capitalize_segment(
                                seg,
                                is_last_segment=(i + 1 == n),
                                prev_is_lower=prev_is_lower,
                            )
                        )
                        init = i + 1
                        mode = _WordMode.Boundary
                        i += 1
                        continue

                # digit->UpperLower boundary only midword (fix "...Veh2Heading", "...Sensor1Raw...").
                if (
                    c.isdigit()
                    and nxt.isupper()
                    and (nxt2 is not None and nxt2.islower())
                    and init != 0
                ):
                    seg = word[init : i + 1]
                    prev_is_lower = init > 0 and word[init - 1].islower()
                    out.append(
                        _capitalize_segment(
                            seg,
                            is_last_segment=(i + 1 == n),
                            prev_is_lower=prev_is_lower,
                        )
                    )
                    init = i + 1
                    mode = _WordMode.Boundary
                    i += 1
                    continue

                # heck rule #1: lower->upper boundary
                # suppress for digit + Upper + Upper + lower (e.g. 12VSupport)
                if next_mode == _WordMode.Lowercase and nxt.isupper():
                    suppress = (
                        c.isdigit()
                        and (nxt2 is not None and nxt2.isupper())
                        and (nxt3 is not None and nxt3.islower())
                    )
                    if not suppress:
                        seg = word[init : i + 1]
                        prev_is_lower = init > 0 and word[init - 1].islower()
                        out.append(
                            _capitalize_segment(
                                seg,
                                is_last_segment=(i + 1 == n),
                                prev_is_lower=prev_is_lower,
                            )
                        )
                        init = i + 1
                        mode = _WordMode.Boundary
                        i += 1
                        continue

                # heck rule #2: acronym boundary before current
                if mode == _WordMode.Uppercase and c.isupper() and nxt.islower():
                    seg = word[init:i]
                    prev_is_lower = init > 0 and word[init - 1].islower()
                    out.append(
                        _capitalize_segment(
                            seg, is_last_segment=(i == n), prev_is_lower=prev_is_lower
                        )
                    )
                    init = i
                    mode = _WordMode.Boundary
                    i += 1
                    continue

            mode = next_mode
            i += 1

        if init < n:
            seg = word[init:]
            prev_is_lower = init > 0 and word[init - 1].islower()
            out.append(
                _capitalize_segment(
                    seg, is_last_segment=True, prev_is_lower=prev_is_lower
                )
            )

    return "".join(out)


# -----------------------------------------------------------------------------
# Encoding helpers
# -----------------------------------------------------------------------------


def _twos_complement(raw: int, bits: int) -> int:
    # Convert unsigned integer on `bits` to signed.
    if bits <= 0:
        return int(raw)
    mask = (1 << bits) - 1
    raw &= mask
    sign_bit = 1 << (bits - 1)
    return raw - (1 << bits) if (raw & sign_bit) else raw


def _extract_event_value(payload: Dict[str, Any]) -> Any:
    v = payload.get("value")
    if isinstance(v, dict) and v:
        return next(iter(v.values()))
    return v


def _normalize_scalar(v: Any, ndigits: int = 6) -> Any:
    return round(v, ndigits) if isinstance(v, float) else v


def _is_boolean_signal(sig: Any) -> bool:
    length = sig.length
    is_signed = sig.is_signed
    scale = float(sig.scale or 1.0)
    offset = float(sig.offset or 0.0)
    mn = sig.minimum
    mx = sig.maximum

    return (
        length == 1
        and not is_signed
        and scale in (None, 1, 1.0)
        and offset in (None, 0, 0.0)
        and mn in (None, 0, 0.0)
        and mx in (None, 1, 1.0)
    )


def _raw_range_from_bitlen(sig: Any) -> Tuple[int, int]:
    length = sig.length
    signed = sig.is_signed
    if signed:
        return (-(1 << (length - 1)), (1 << (length - 1)) - 1)
    return (0, (1 << length) - 1)


def _encodable_physical_range(sig: Any) -> Tuple[float, float, float]:
    dbc_mn = sig.minimum
    dbc_mx = sig.maximum
    scale = float(sig.scale or 1.0)
    offset = float(sig.offset or 0.0)

    rmn, rmx = _raw_range_from_bitlen(sig)
    pmn = rmn * scale + offset
    pmx = rmx * scale + offset
    if pmn > pmx:
        pmn, pmx = pmx, pmn

    mn = pmn if dbc_mn is None else float(dbc_mn)
    mx = pmx if dbc_mx is None else float(dbc_mx)
    if mn > mx:
        mn, mx = mx, mn

    mn = max(mn, pmn)
    mx = min(mx, pmx)
    if mn > mx:
        mn, mx = pmn, pmx

    step = abs(scale) if scale != 0.0 else 1.0
    span = mx - mn
    if span > 0 and step < span / 10000.0:
        step = span / 1000.0
    return float(mn), float(mx), float(step)


def _safe_default_value(sig: Any) -> Any:
    if _is_boolean_signal(sig):
        return False
    try:
        mn, _, step = _encodable_physical_range(sig)
        if step >= 1.0 and abs(step - round(step)) < 1e-12:
            return int(round(mn))
        return mn
    except Exception:
        return 0


def _pick_random_value(sig: Any, rng: random.Random) -> Any:
    choices = sig.choices
    length = sig.length

    if isinstance(choices, dict) and choices:
        scale = float(sig.scale or 1.0)
        offset = float(sig.offset or 0.0)
        mn, mx, _ = _encodable_physical_range(sig)
        signed = bool(sig.is_signed)

        valid_phys: List[Any] = []
        for raw, label in choices.items():
            if isinstance(label, str) and label.strip().lower() in {"sna", "na", "n/a"}:
                continue
            raw_i = int(raw)
            if signed:
                raw_i = _twos_complement(raw_i, length)
            phys = (float(raw_i) * scale) + offset
            if mn - 1e-9 <= phys <= mx + 1e-9:
                valid_phys.append(
                    phys if (scale != 1.0 or offset != 0.0) else int(raw_i)
                )

        if valid_phys:
            return rng.choice(valid_phys)

    if _is_boolean_signal(sig):
        return rng.choice([False, True])

    mn, mx, step = _encodable_physical_range(sig)
    if mn == mx:
        return mn

    looks_integer = step >= 1.0 and abs(step - round(step)) < 1e-12
    if looks_integer:
        lo, hi = int(round(mn)), int(round(mx))
        if lo > hi:
            lo, hi = hi, lo
        return rng.randint(lo, hi)

    span = mx - mn
    n = int(span / step) if step > 0 else 100
    n = max(1, min(n, 2000))
    k = rng.randint(0, n)
    return round(mn + k * step, 6)


def _decode_message(msg: Any, data: bytes) -> Dict[str, Any]:
    try:
        return msg.decode(data, decode_choices=False)
    except TypeError:
        # Older cantools versions don't support decode_choices
        return msg.decode(data)


def _encode_message(msg: Any, values: Dict[str, Any]) -> bytes:
    try:
        return msg.encode(values, strict=False)
    except TypeError:
        # Older cantools versions don't support strict
        return msg.encode(values)
    except Exception:
        return msg.encode(values)


# ----------------------------------------------------------------------------
# Mux handling
# -----------------------------------------------------------------------------


def _to_int_mux(v: Any) -> Optional[int]:
    if v is None:
        return None
    if isinstance(v, bool):
        return int(v)
    if isinstance(v, int):
        return v
    if hasattr(v, "value"):
        try:
            return int(v.value)
        except Exception:
            pass
    return int(v)


def _mux_ids_as_int_set(sig: Any) -> Optional[Set[int]]:
    mux_ids = sig.multiplexer_ids
    if mux_ids is None:
        return None
    if isinstance(mux_ids, dict):
        mux_ids = mux_ids.keys()
    try:
        return {_to_int_mux(x) for x in mux_ids}  # type: ignore[misc]
    except TypeError:
        return {_to_int_mux(mux_ids)}  # type: ignore[misc]


def _valid_mux_ids_for_message(msg: Any) -> List[int]:
    ids: Set[int] = set()
    for s in msg.signals:
        mids = _mux_ids_as_int_set(s)
        if not mids:
            continue
        for x in mids:
            xi = _to_int_mux(x)
            if xi is not None:
                ids.add(int(xi))
    return sorted(ids)


def _find_multiplexer_signal(msg: Any) -> Optional[Any]:
    for s in msg.signals:
        if s.is_multiplexer:
            return s
    return None


def _pick_mux_selector_value(
    msg: Any, mux_sig: Any, target_sig: Any, rng: random.Random
) -> int:
    valid = _valid_mux_ids_for_message(msg)
    target_ids = _mux_ids_as_int_set(target_sig) or set()
    target_ids_i = sorted(
        int(_to_int_mux(x)) for x in target_ids if _to_int_mux(x) is not None
    )

    if target_ids_i:
        for v in target_ids_i:
            if not valid or v in valid:
                return v
        return target_ids_i[0]

    if valid:
        return int(rng.choice(valid))

    choices = mux_sig.choices
    if isinstance(choices, dict) and choices:
        keys = [k for k in (_to_int_mux(x) for x in choices.keys()) if k is not None]
        if keys:
            return int(rng.choice(sorted(keys)))

    return 0


def _build_values_for_message(
    msg: Any, target_sig: Any, target_value: Any, rng: random.Random
) -> Dict[str, Any]:
    mux_sig = _find_multiplexer_signal(msg)
    mux_value: Optional[int] = None
    values: Dict[str, Any] = {}

    if mux_sig is not None:
        mux_value = _to_int_mux(_pick_mux_selector_value(msg, mux_sig, target_sig, rng))
        values[mux_sig.name] = mux_value
        if target_sig.name == mux_sig.name:
            target_value = mux_value

    for s in msg.signals:
        if s.name == target_sig.name:
            values[s.name] = target_value
            continue
        if mux_sig is not None and s.name == mux_sig.name:
            continue

        mux_ids = _mux_ids_as_int_set(s)
        if mux_sig is not None and mux_value is not None and mux_ids is not None:
            if _to_int_mux(mux_value) not in mux_ids:
                continue

        values[s.name] = _pick_random_value(s, rng)

    for s in msg.signals:
        values.setdefault(s.name, _safe_default_value(s))

    return values


def _send_frame(bus: can.Bus, can_id: int, data: bytes, is_extended: bool) -> None:
    bus.send(
        can.Message(arbitration_id=can_id, data=data, is_extended_id=bool(is_extended))
    )


# -----------------------------------------------------------------------------
# AFB verbs
# -----------------------------------------------------------------------------


def _is_unknown_verb_error(e: Exception) -> bool:
    return "unknown-verb" in str(e).lower()


def _verb_for(msg_name: str, sig_name: str) -> str:
    return f"{to_upper_camel_case(msg_name)}/{to_upper_camel_case(sig_name)}"


def _subscribe_signal(binder: Any, msg_name: str, sig_name: str) -> str:
    verb = _verb_for(msg_name, sig_name)
    try:
        libafb.callsync(
            binder,
            CAN_API,
            verb,
            {"action": "subscribe", "flag": "all", "rate": 0, "watchdog": 0},
        )
        return verb
    except Exception as e:
        if _is_unknown_verb_error(e):
            raise RuntimeError(
                f"_subscribe_signal: unknown-verb:{verb} for msg={msg_name}, sig={sig_name}"
            ) from e
        raise


def _unsubscribe_signal(binder: Any, msg_name: str, sig_name: str) -> str:
    verb = _verb_for(msg_name, sig_name)
    try:
        libafb.callsync(binder, CAN_API, verb, {"action": "unsubscribe"})
        return verb
    except Exception as e:
        if _is_unknown_verb_error(e):
            LOG.info(
                f"[AFB] Skip unsubscribe unknown verb for msg={msg_name} sig={sig_name} verb={verb}"
            )
            return verb
        raise


# -----------------------------------------------------------------------------
# Test harness
# -----------------------------------------------------------------------------


def setUpModule() -> None:
    global bindings, config
    print(f"Using bindings : {bindings}")
    print(f"Using config : {config}")
    if not bindings or not config:
        raise RuntimeError(
            "Binder configuration not initialized. "
            "Run the script directly with --config/--can-api/--dbc-file, "
            "or export BINDER_CONFIG before running the tests."
        )
    configure_afb_binding_tests(bindings=bindings, config=config)


class TestBindingDbcGenerated(AFBTestCase):
    def test_check_ok(self) -> None:
        try:
            r = libafb.callsync(self.binder, "sockbcm", "check")
        except Exception as e:
            if _is_unknown_verb_error(e):
                raise RuntimeError("test_check_ok Failed") from e
            raise
        LOG.info(
            f'[AFB] check the sockbcm libafb.callsync(self.binder, "sockbcm", "check"): {r}'
        )
        assert r.args == ()

    def test_dbc_random_signal_events(self) -> None:
        if not DBC_FILE.exists():
            raise FileNotFoundError(f"Missing DBC file: {DBC_FILE}")

        rng = random.Random(RANDOM_SEED)
        db = cantools.database.load_file(str(DBC_FILE))
        bus = can.interface.Bus(channel=VCAN_IFACE, interface="socketcan")

        tested = 0
        try:
            for msg in db.messages:
                canid = msg.frame_id
                if ALLOWED_CANIDS is not None and canid not in ALLOWED_CANIDS:
                    continue

                msg_name = msg.name
                is_extended = bool(msg.is_extended_frame)

                for sig in msg.signals:
                    sig_name = sig.name

                    if sig.is_multiplexer:
                        valid = _valid_mux_ids_for_message(msg)
                        candidate = (
                            rng.choice(valid) if valid else _safe_default_value(sig)
                        )
                    else:
                        candidate = _pick_random_value(sig, rng)

                    values = _build_values_for_message(msg, sig, candidate, rng)

                    try:
                        data = _encode_message(msg, values)
                    except cantools.database.errors.EncodeError as e:
                        LOG.warning(
                            f"[AFB] Encode error canid={canid} msg={msg_name} sig={sig_name}: {e}"
                        )
                        continue
                    except OverflowError as e:
                        LOG.info(
                            f"[AFB] Encode overflow canid={canid} msg={msg_name} sig={sig_name}: {e}"
                        )
                        continue

                    decoded = _decode_message(msg, data)
                    expected = decoded.get(sig_name, candidate)
                    if _is_boolean_signal(sig):
                        expected = bool(expected)
                    expected_norm = _normalize_scalar(expected)

                    with self.subTest(
                        canid=canid, msg=msg_name, sig=sig_name, expected=expected_norm
                    ):
                        try:
                            verb = _subscribe_signal(self.binder, msg_name, sig_name)
                        except Exception as e:
                            if _is_unknown_verb_error(e):
                                cc_msg = to_upper_camel_case(msg_name)
                                cc_sig = to_upper_camel_case(sig_name)
                                LOG.debug(
                                    f"[AFB] Skip unknown verb for canid={canid} "
                                    f"msg={msg_name}({cc_msg}) sig={sig_name}({cc_sig})"
                                )
                                continue
                            LOG.error("[AFB] unknown verb -------------------")
                            raise

                        try:
                            with self.assertEventEmitted(
                                CAN_API,
                                verb,
                                timeout_ms=5000,
                                value=expected_norm,
                                value_lambda=lambda payload: _normalize_scalar(
                                    _extract_event_value(payload)
                                ),
                            ):
                                _send_frame(bus, canid, data, is_extended)
                        except EventAssertionError:
                            LOG.error(
                                f"[AFB] EventAssertionError canid={canid} msg={msg_name} sig={sig_name} expected={expected_norm}"
                            )
                            LOG.error(f"[AFB] Values used for encode: {values}")
                            LOG.error(f"[AFB] Encoded bytes: {data.hex()}")
                            raise
                        finally:
                            _unsubscribe_signal(self.binder, msg_name, sig_name)

                tested += 1
        finally:
            bus.shutdown()


if __name__ == "__main__":
    p = argparse.ArgumentParser(
        description="Run canbus-binding functional tests (AFB binder + SocketCAN).",
        allow_abbrev=False,
    )
    p.add_argument(
        "--config",
        required=True,
        type=Path,
        help="Path to binder-config.json.",
    )

    # Optional overrides (take precedence over config JSON and env).
    p.add_argument(
        "--can-api",
        dest="can_api",
        required=True,
        type=str,
        help="Override AFB API name used by tests (e.g. model3).",
    )
    p.add_argument(
        "--dbc-file",
        dest="dbc_file",
        required=True,
        type=Path,
        help="Override DBC file path used by tests.",
    )
    p.add_argument(
        "--vcan-iface",
        dest="vcan_iface",
        default="vcan0",
        help="Override SocketCAN interface (e.g. vcan0).",
    )
    p.add_argument(
        "--canids",
        dest="canids",
        default=None,
        help=(
            "Comma-separated list of CAN IDs (dec or 0x..). "
            "Used only when TEST_CANIDS is unset. "
            "Use 'none' to disable defaults."
        ),
    )

    g = p.add_mutually_exclusive_group()
    g.add_argument("--quiet", action="store_true", help="Only show errors.")
    g.add_argument("--verbose", action="store_true", help="More logs (INFO).")
    g.add_argument("--debug", action="store_true", help="Debug logs (DEBUG).")

    # Keep unknown args for unittest (e.g. -k, -v, tests selectors, etc.)
    args, remaining = p.parse_known_args()
    _setup_logging(quiet=args.quiet, verbose=args.verbose, debug=args.debug)

    # Apply config early when running as a script.
    cfg_path = args.config.expanduser()
    if not cfg_path.exists():
        p.error(f"--config file not found: {cfg_path}")

    _init_from_binder_config(cfg_path)

    # CLI overrides (highest priority).
    CAN_API = str(args.can_api)
    if not CAN_API.strip():
        p.error("--can-api must be a non-empty string")

    dbc_file = args.dbc_file.expanduser().resolve()
    if not dbc_file.exists():
        p.error(f"--dbc-file file not found: {dbc_file}")
    DBC_FILE = dbc_file

    if args.vcan_iface is not None:
        VCAN_IFACE = str(args.vcan_iface)
        os.environ["VCAN_IFACE"] = VCAN_IFACE

    if args.canids is not None:
        t = str(args.canids).strip().lower()
        if t in {"none", ""}:
            _canids = None
        else:
            tokens = [x.strip() for x in str(args.canids).split(",") if x.strip()]
            parsed = _as_int_list(tokens) or []
            _canids = tuple(parsed)

        # Recompute the effective filter because it depends on _canids.
        ALLOWED_CANIDS = _parse_canids_env()

    # unittest parses sys.argv internally; remove our custom CLI args
    # so it doesn't choke on --config/--verbose/--debug/--quiet.
    sys.argv = [sys.argv[0]] + remaining

    run_afb_binding_tests(bindings)
