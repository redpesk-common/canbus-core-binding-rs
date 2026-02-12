# tests/functional/test_model3_replay.py
from __future__ import annotations

import enum
import os
import random
from pathlib import Path
from typing import Any, Dict, Iterable, Optional, Set, Tuple, List

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

ROOT = Path(__file__).resolve().parents[2]
TARGET = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target"))

CAN_API = "model3"

SOCKCAN_SO = TARGET / "debug" / "libafb_sockcan.so"
MODEL3_SO = TARGET / "debug" / "libafb_model3.so"

bindings = {
    "sockbcm": str(SOCKCAN_SO),
    "model3": str(MODEL3_SO),
}

config = {
    "libafb_sockcan.so": {
        "sock_api": "sockcan",
        "sock_evt": "sockbcm",
        "uid": "sockbcm",
    },
    "libafb_model3.so": {
        "sock_api": "sockbcm",
        "sock_evt": "sockcan",
        "uid": "model3",
    },
}

SCRIPT_DIR = Path(__file__).resolve().parent
DBC_FILE = (SCRIPT_DIR / ".." / ".." / "examples" / "samples" / "model3" / "dbc" / "model3can.dbc").resolve()

VCAN_IFACE = os.environ.get("MODEL3_VCAN_IFACE", "vcan0")

# -----------------------------------------------------------------------------
# Test selection / parameters
# -----------------------------------------------------------------------------

_DEFAULT_CANIDS: Optional[Iterable[int]] = None
# _DEFAULT_CANIDS = (526,)  # Uncomment to focus locally.

RANDOM_SEED = int(os.environ.get("MODEL3_TEST_SEED", "12345"))
ITERS_PER_SIGNAL = int(os.environ.get("MODEL3_TEST_ITERS", "1"))
MAX_SIGNALS = int(os.environ.get("MODEL3_TEST_MAX_SIGNALS", "0"))  # 0 => no limit


def _parse_canids_env() -> Optional[Set[int]]:
    raw = os.environ.get("MODEL3_TEST_CANIDS", "").strip()
    if not raw:
        return None if _DEFAULT_CANIDS is None else {int(x) for x in _DEFAULT_CANIDS}
    if raw.lower() == "all":
        return None

    canids: Set[int] = set()
    for token in raw.split(","):
        t = token.strip().lower()
        if not t:
            continue
        base = 16 if t.startswith("0x") else 10
        canids.add(int(t, base))
    return canids


ALLOWED_CANIDS = _parse_canids_env()


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
                    inside_hex = (prev is not None and prev.isdigit() and c in "ABCDEF")
                    if not inside_hex:
                        seg = word[init : i + 1]
                        prev_is_lower = init > 0 and word[init - 1].islower()
                        out.append(_capitalize_segment(seg, is_last_segment=(i + 1 == n), prev_is_lower=prev_is_lower))
                        init = i + 1
                        mode = _WordMode.Boundary
                        i += 1
                        continue

                # digit->UpperLower boundary only midword (fix "...Veh2Heading", "...Sensor1Raw...").
                if c.isdigit() and nxt.isupper() and (nxt2 is not None and nxt2.islower()) and init != 0:
                    seg = word[init : i + 1]
                    prev_is_lower = init > 0 and word[init - 1].islower()
                    out.append(_capitalize_segment(seg, is_last_segment=(i + 1 == n), prev_is_lower=prev_is_lower))
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
                        out.append(_capitalize_segment(seg, is_last_segment=(i + 1 == n), prev_is_lower=prev_is_lower))
                        init = i + 1
                        mode = _WordMode.Boundary
                        i += 1
                        continue

                # heck rule #2: acronym boundary before current
                if mode == _WordMode.Uppercase and c.isupper() and nxt.islower():
                    seg = word[init:i]
                    prev_is_lower = init > 0 and word[init - 1].islower()
                    out.append(_capitalize_segment(seg, is_last_segment=(i == n), prev_is_lower=prev_is_lower))
                    init = i
                    mode = _WordMode.Boundary
                    i += 1
                    continue

            mode = next_mode
            i += 1

        if init < n:
            seg = word[init:]
            prev_is_lower = init > 0 and word[init - 1].islower()
            out.append(_capitalize_segment(seg, is_last_segment=True, prev_is_lower=prev_is_lower))

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
    length = getattr(sig, "length", None)
    is_signed = getattr(sig, "is_signed", False)
    scale = getattr(sig, "scale", None)
    offset = getattr(sig, "offset", None)
    mn = getattr(sig, "minimum", None)
    mx = getattr(sig, "maximum", None)

    return (
        length == 1
        and not is_signed
        and scale in (None, 1, 1.0)
        and offset in (None, 0, 0.0)
        and mn in (None, 0, 0.0)
        and mx in (None, 1, 1.0)
    )


def _raw_range_from_bitlen(sig: Any) -> Tuple[int, int]:
    length = int(getattr(sig, "length", 8))
    signed = bool(getattr(sig, "is_signed", False))
    if signed:
        return (-(1 << (length - 1)), (1 << (length - 1)) - 1)
    return (0, (1 << length) - 1)


def _encodable_physical_range(sig: Any) -> Tuple[float, float, float]:
    dbc_mn = getattr(sig, "minimum", None)
    dbc_mx = getattr(sig, "maximum", None)
    scale = float(getattr(sig, "scale", 1.0) or 1.0)
    offset = float(getattr(sig, "offset", 0.0) or 0.0)

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
    choices = getattr(sig, "choices", None)
    length = int(getattr(sig, "length", 8))

    if isinstance(choices, dict) and choices:
        scale = float(getattr(sig, "scale", 1.0) or 1.0)
        offset = float(getattr(sig, "offset", 0.0) or 0.0)
        mn, mx, _ = _encodable_physical_range(sig)
        signed = bool(getattr(sig, "is_signed", False))

        valid_phys: List[Any] = []
        for raw, label in choices.items():
            if isinstance(label, str) and label.strip().lower() in {"sna", "na", "n/a"}:
                continue
            raw_i = int(raw)
            if signed:
                raw_i = _twos_complement(raw_i, length)
            phys = (float(raw_i) * scale) + offset
            if mn - 1e-9 <= phys <= mx + 1e-9:
                valid_phys.append(phys if (scale != 1.0 or offset != 0.0) else int(raw_i))

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
        return msg.decode(data)


def _encode_message(msg: Any, values: Dict[str, Any]) -> bytes:
    try:
        return msg.encode(values, strict=False)
    except TypeError:
        return msg.encode(values)
    except Exception:
        return msg.encode(values)


# -----------------------------------------------------------------------------
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
    mux_ids = getattr(sig, "multiplexer_ids", None)
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
    for s in getattr(msg, "signals", []):
        mids = _mux_ids_as_int_set(s)
        if not mids:
            continue
        for x in mids:
            xi = _to_int_mux(x)
            if xi is not None:
                ids.add(int(xi))
    return sorted(ids)


def _find_multiplexer_signal(msg: Any) -> Optional[Any]:
    for s in getattr(msg, "signals", []):
        if getattr(s, "is_multiplexer", False):
            return s
    return None


def _pick_mux_selector_value(msg: Any, mux_sig: Any, target_sig: Any, rng: random.Random) -> int:
    valid = _valid_mux_ids_for_message(msg)
    target_ids = _mux_ids_as_int_set(target_sig) or set()
    target_ids_i = sorted(int(_to_int_mux(x)) for x in target_ids if _to_int_mux(x) is not None)

    if target_ids_i:
        for v in target_ids_i:
            if not valid or v in valid:
                return v
        return target_ids_i[0]

    if valid:
        return int(rng.choice(valid))

    choices = getattr(mux_sig, "choices", None)
    if isinstance(choices, dict) and choices:
        keys = [k for k in (_to_int_mux(x) for x in choices.keys()) if k is not None]
        if keys:
            return int(rng.choice(sorted(keys)))

    return 0


def _build_values_for_message(msg: Any, target_sig: Any, target_value: Any, rng: random.Random) -> Dict[str, Any]:
    mux_sig = _find_multiplexer_signal(msg)
    mux_value: Optional[int] = None
    values: Dict[str, Any] = {}

    if mux_sig is not None:
        mux_value = _to_int_mux(_pick_mux_selector_value(msg, mux_sig, target_sig, rng))
        values[mux_sig.name] = mux_value
        if target_sig.name == mux_sig.name:
            target_value = mux_value

    for s in getattr(msg, "signals", []):
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

    for s in getattr(msg, "signals", []):
        values.setdefault(s.name, _safe_default_value(s))

    return values


def _send_frame(bus: can.Bus, can_id: int, data: bytes, is_extended: bool) -> None:
    bus.send(can.Message(arbitration_id=can_id, data=data, is_extended_id=bool(is_extended)))


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
            raise RuntimeError(f"_subscribe_signal: unknown-verb:{verb} for msg={msg_name}, sig={sig_name}") from e
        raise


def _unsubscribe_signal(binder: Any, msg_name: str, sig_name: str) -> str:
    verb = _verb_for(msg_name, sig_name)
    try:
        libafb.callsync(binder, CAN_API, verb, {"action": "unsubscribe"})
        return verb
    except Exception as e:
        if _is_unknown_verb_error(e):
            print(f"[AFB] Skip unsubscribe unknown verb for msg={msg_name} sig={sig_name} verb={verb}")
            return verb
        raise


# -----------------------------------------------------------------------------
# Test harness
# -----------------------------------------------------------------------------

def setUpModule() -> None:
    configure_afb_binding_tests(bindings=bindings, config=config)


class TestModel3DbcGenerated(AFBTestCase):
    def test_check_ok(self) -> None:
        r = libafb.callsync(self.binder, "sockbcm", "check")
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
                canid = int(getattr(msg, "frame_id", 0))
                if ALLOWED_CANIDS is not None and canid not in ALLOWED_CANIDS:
                    continue

                msg_name = getattr(msg, "name", f"canid_{canid}")
                is_extended = bool(getattr(msg, "is_extended_frame", False))

                for sig in getattr(msg, "signals", []):
                    if MAX_SIGNALS > 0 and tested >= MAX_SIGNALS:
                        return

                    sig_name = getattr(sig, "name", "unknown_signal")

                    for _ in range(max(1, ITERS_PER_SIGNAL)):
                        if getattr(sig, "is_multiplexer", False):
                            valid = _valid_mux_ids_for_message(msg)
                            candidate = rng.choice(valid) if valid else _safe_default_value(sig)
                        else:
                            candidate = _pick_random_value(sig, rng)

                        values = _build_values_for_message(msg, sig, candidate, rng)

                        try:
                            data = _encode_message(msg, values)
                        except cantools.database.errors.EncodeError as e:
                            print(f"[AFB] Encode error canid={canid} msg={msg_name} sig={sig_name}: {e}")
                            continue
                        except OverflowError as e:
                            print(f"[AFB] Encode overflow canid={canid} msg={msg_name} sig={sig_name}: {e}")
                            continue

                        decoded = _decode_message(msg, data)
                        expected = decoded.get(sig_name, candidate)
                        if _is_boolean_signal(sig):
                            expected = bool(expected)
                        expected_norm = _normalize_scalar(expected)

                        with self.subTest(canid=canid, msg=msg_name, sig=sig_name, expected=expected_norm):
                            try:
                                verb = _subscribe_signal(self.binder, msg_name, sig_name)
                            except Exception as e:
                                if _is_unknown_verb_error(e):
                                    cc_msg = to_upper_camel_case(msg_name)
                                    cc_sig = to_upper_camel_case(sig_name)
                                    print(
                                        f"[AFB] Skip unknown verb for canid={canid} "
                                        f"msg={msg_name}({cc_msg}) sig={sig_name}({cc_sig})"
                                    )
                                    continue
                                raise

                            try:
                                with self.assertEventEmitted(
                                    CAN_API,
                                    verb,
                                    timeout_ms=5000,
                                    value=expected_norm,
                                    value_lambda=lambda payload: _normalize_scalar(_extract_event_value(payload)),
                                ):
                                    _send_frame(bus, canid, data, is_extended)
                            except EventAssertionError:
                                print(f"[AFB] EventAssertionError canid={canid} msg={msg_name} sig={sig_name} expected={expected_norm}")
                                print(f"[AFB] Values used for encode: {values}")
                                print(f"[AFB] Encoded bytes: {data.hex()}")
                                raise
                            finally:
                                _unsubscribe_signal(self.binder, msg_name, sig_name)

                    tested += 1
        finally:
            bus.shutdown()


if __name__ == "__main__":
    run_afb_binding_tests(bindings)
