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

from __future__ import annotations

import argparse
import csv
from contextlib import contextmanager, nullcontext
from datetime import datetime, timezone
import enum
import gc
import json
import logging
import os
import queue
import random
import shutil
import subprocess
import threading
from pathlib import Path
import sys
import time
import unittest
from typing import Any, Dict, Iterable, List, Optional, Set, Tuple

import can
import cantools
import libafb
from afb_test import (
    AFBTestCase,
    EventAssertionError,
    configure_afb_binding_tests,
    run_afb_binding_tests,
    serialize_test_case_result,
    unserialize_in_result,
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

# When enabled, the test harness creates only a lightweight Python binder and
# imports APIs from an already running external binder instead of loading the
# binding shared objects locally.
EXTERNAL_BINDER_MODE = False
EXTERNAL_API_IMPORTS: Dict[str, str] = {}
EXTERNAL_BINDER_SOCKSPEC: Optional[str] = None
EXTERNAL_CLIENT_BIN = os.environ.get("AFB_CLIENT_BIN", "afb-client")


def _env_int(name: str, default: int, *, min_value: Optional[int] = None) -> int:
    raw = os.environ.get(name)
    if raw is None or not raw.strip():
        return default
    try:
        value = int(raw, 0)
    except ValueError:
        LOG.warning("Invalid integer value for %s=%r, using %d", name, raw, default)
        return default
    if min_value is not None and value < min_value:
        LOG.warning("Invalid value for %s=%r, using %d", name, raw, default)
        return default
    return value


def _env_float(name: str, default: Optional[float] = None) -> Optional[float]:
    raw = os.environ.get(name)
    if raw is None or not raw.strip():
        return default
    try:
        return float(raw)
    except ValueError:
        LOG.warning("Invalid float value for %s=%r, using %s", name, raw, default)
        return default


def _env_bool(name: str, default: bool = False) -> bool:
    raw = os.environ.get(name)
    if raw is None or not raw.strip():
        return default
    return raw.strip().lower() in {"1", "yes", "true", "on", "y"}


def _env_path(name: str) -> Optional[Path]:
    raw = os.environ.get(name)
    if raw is None or not raw.strip():
        return None
    return Path(raw).expanduser()


def _mib_to_bytes(value_mib: Optional[float]) -> Optional[int]:
    if value_mib is None:
        return None
    return int(value_mib * 1024 * 1024)


# -----------------------------------------------------------------------------
# Test selection / parameters
# -----------------------------------------------------------------------------

_canids: Optional[Iterable[int]] = None
# _canids = (526,)  # Uncomment to focus locally.

RANDOM_SEED = _env_int("TEST_SEED", int(time.time()))
ALLOWED_CANIDS: Optional[Set[int]] = None

SCENARIO_LOOPS = _env_int("TEST_SCENARIO_LOOPS", 1, min_value=1)
MEMORY_REPORT_EVERY = _env_int("TEST_MEMORY_REPORT_EVERY", 1, min_value=0)
MEMORY_PID: Optional[int] = (
    _env_int("TEST_MEMORY_PID", os.getpid(), min_value=1)
    if os.environ.get("TEST_MEMORY_PID")
    else None
)
FAIL_ON_RSS_GROWTH_BYTES = _mib_to_bytes(_env_float("TEST_FAIL_ON_RSS_GROWTH_MIB"))
GC_BETWEEN_LOOPS = _env_bool("TEST_GC_BETWEEN_LOOPS", False)
LOOP_DELAY_MS = _env_int("TEST_LOOP_DELAY_MS", 0, min_value=0)
VARY_SEED_PER_LOOP = _env_bool("TEST_VARY_SEED_PER_LOOP", False)

METRICS_LOG_FILE = _env_path("TEST_METRICS_LOG_FILE")
METRICS_LOG_FORMAT = os.environ.get("TEST_METRICS_LOG_FORMAT", "csv").strip().lower()
METRICS_LOG_APPEND = _env_bool("TEST_METRICS_LOG_APPEND", False)
METRICS_LOG_FLUSH = _env_bool("TEST_METRICS_LOG_FLUSH", True)
METRICS_LOG_EVERY = _env_int("TEST_METRICS_LOG_EVERY", 1, min_value=0)


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


def _verbose_output_enabled() -> bool:
    return LOG.isEnabledFor(logging.INFO)


def _verbose_print(*args: Any, **kwargs: Any) -> None:
    if _verbose_output_enabled():
        print(*args, **kwargs)


def _read_json(path: Path) -> Dict[str, Any]:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


def _parse_api_uri_mapping(value: str) -> Tuple[str, str]:
    if "=" not in value:
        raise ValueError(
            f"Invalid API URI mapping. Expected '<api>=<uri>', got: {value!r}"
        )
    api, uri = value.split("=", 1)
    api = api.strip()
    uri = uri.strip()
    if not api:
        raise ValueError(f"Invalid API URI mapping without API name: {value!r}")
    if not uri:
        raise ValueError(f"Invalid API URI mapping without URI: {value!r}")
    return api, uri


def _join_external_api_uri(base_uri: str, api: str) -> str:
    base = base_uri.rstrip("/")
    if base.endswith("/api"):
        return f"{base}/{api}"
    return f"{base}/api/{api}"


def _normalize_external_client_sockspec(uri: str) -> str:
    """
    Return the SOCKSPEC expected by afb-client for the binder HTTP endpoint.

    afb-client without --direct talks to the binder WS/HTTP endpoint. For a
    binder served at http://host:port, the SOCKSPEC is http://host:port/api.
    A user-facing tcp:host:port value is accepted for convenience and converted
    to http://host:port/api.
    """
    base = uri.rstrip("/")

    if base.startswith("tcp:"):
        base = "http://" + base[len("tcp:") :]

    if base.startswith(("http://", "https://")):
        if base.endswith("/api"):
            return base
        return f"{base}/api"

    if base.endswith("/api"):
        return base
    return f"{base}/api"


def _join_external_binder_sockspec(base_uri: str) -> str:
    return _normalize_external_client_sockspec(base_uri)


def _infer_external_binder_sockspec(
    *,
    base_uri: Optional[str],
    imports: Dict[str, str],
) -> Optional[str]:
    if base_uri:
        return _join_external_binder_sockspec(base_uri)

    for api, uri in sorted(imports.items()):
        suffix = f"/{api}"
        if uri.rstrip("/").endswith(suffix):
            return _normalize_external_client_sockspec(uri.rstrip("/")[: -len(suffix)])

    return None


def _build_external_api_imports(
    *,
    base_uri: Optional[str],
    explicit_api_uris: Optional[List[str]],
) -> Dict[str, str]:
    imports: Dict[str, str] = {}

    if base_uri:
        for api in sorted(bindings.keys()):
            imports[api] = _join_external_api_uri(base_uri, api)

    for item in explicit_api_uris or []:
        api, uri = _parse_api_uri_mapping(item)
        imports[api] = uri

    return imports


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

        so_name = item.get("path")
        if not so_name:
            raise ValueError(
                "Invalid binder config: each 'binding' entry must contain 'path'"
            )

        # uid is now in set[so_name]
        entry = set_cfg.get(so_name)
        if entry is None:
            raise ValueError(
                f"Invalid binder config: missing 'set[\"{so_name}\"]' entry for binding .so file {so_name}"
            )

        uid = entry.get("uid")
        if not isinstance(uid, str) or not uid.strip():
            raise ValueError(
                f"Invalid binder config: set['{so_name}'].uid must be a non-empty string"
            )

        if uid in new_bindings and Path(new_bindings[uid]).name != so_name:
            raise ValueError(
                f"Invalid binder config: duplicate uid '{uid}' used for multiple bindings "
                f"({new_bindings[uid]} and {so_name})"
            )

        new_bindings[uid] = str(so_name)

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
# Runtime diagnostics
# -----------------------------------------------------------------------------

_MEMORY_BYTE_KEYS = {
    "VmPeak",
    "VmSize",
    "VmLck",
    "VmPin",
    "VmHWM",
    "VmRSS",
    "RssAnon",
    "RssFile",
    "RssShmem",
    "VmData",
    "VmStk",
    "VmExe",
    "VmLib",
    "VmPTE",
    "VmSwap",
}

_MEMORY_INT_KEYS = {"Threads", "FDSize"}


def _format_bytes(value: Optional[int]) -> str:
    if value is None:
        return "n/a"
    return f"{value / (1024 * 1024):.1f} MiB"


def _format_signed_bytes(value: Optional[int]) -> str:
    if value is None:
        return "n/a"
    sign = "+" if value >= 0 else "-"
    return f"{sign}{abs(value) / (1024 * 1024):.1f} MiB"


def _read_proc_status(pid: int) -> Dict[str, Any]:
    status_path = Path("/proc") / str(pid) / "status"
    stats: Dict[str, Any] = {}
    try:
        for line in status_path.read_text(encoding="utf-8").splitlines():
            if ":" not in line:
                continue
            key, raw_value = line.split(":", 1)
            value = raw_value.strip()
            if key in _MEMORY_BYTE_KEYS:
                parts = value.split()
                if parts and parts[0].isdigit():
                    stats[key] = int(parts[0]) * 1024
            elif key in _MEMORY_INT_KEYS:
                parts = value.split()
                if parts and parts[0].isdigit():
                    stats[key] = int(parts[0])
            elif key in {"Name", "State"}:
                stats[key] = value
    except OSError as exc:
        stats["error"] = str(exc)
    return stats


def _count_open_fds(pid: int) -> Optional[int]:
    fd_dir = Path("/proc") / str(pid) / "fd"
    try:
        return sum(1 for _ in fd_dir.iterdir())
    except OSError:
        return None


def _memory_snapshot(pid: int) -> Dict[str, Any]:
    snapshot = _read_proc_status(pid)
    snapshot["pid"] = pid
    snapshot["fds"] = _count_open_fds(pid)
    snapshot["timestamp"] = time.time()
    return snapshot


def _format_memory_snapshot(
    snapshot: Dict[str, Any], baseline: Optional[Dict[str, Any]] = None
) -> str:
    parts = [f"pid={snapshot.get('pid')}"]
    name = snapshot.get("Name")
    if name:
        parts.append(f"name={name}")
    state = snapshot.get("State")
    if state:
        parts.append(f"state={state}")

    parts.extend(
        [
            f"rss={_format_bytes(snapshot.get('VmRSS'))}",
            f"hwm={_format_bytes(snapshot.get('VmHWM'))}",
            f"vmsize={_format_bytes(snapshot.get('VmSize'))}",
            f"data={_format_bytes(snapshot.get('VmData'))}",
            f"rss_anon={_format_bytes(snapshot.get('RssAnon'))}",
            f"rss_file={_format_bytes(snapshot.get('RssFile'))}",
        ]
    )

    threads = snapshot.get("Threads")
    if threads is not None:
        parts.append(f"threads={threads}")

    fds = snapshot.get("fds")
    if fds is not None:
        parts.append(f"fds={fds}")

    if baseline is not None:
        rss = snapshot.get("VmRSS")
        base_rss = baseline.get("VmRSS")
        data = snapshot.get("VmData")
        base_data = baseline.get("VmData")
        if isinstance(rss, int) and isinstance(base_rss, int):
            parts.append(f"rss_delta={_format_signed_bytes(rss - base_rss)}")
        if isinstance(data, int) and isinstance(base_data, int):
            parts.append(f"data_delta={_format_signed_bytes(data - base_data)}")

    error = snapshot.get("error")
    if error:
        parts.append(f"error={error}")

    return " ".join(parts)


_METRICS_FIELDS = [
    "timestamp_iso",
    "timestamp_unix",
    "event",
    "loop_index",
    "loop_total",
    "elapsed_s",
    "seed",
    "pid",
    "proc_name",
    "proc_state",
    "can_api",
    "vcan_iface",
    "dbc_file",
    "canids",
    "vmrss_bytes",
    "vmhwm_bytes",
    "vmsize_bytes",
    "vmdata_bytes",
    "rssanon_bytes",
    "rssfile_bytes",
    "threads",
    "fds",
    "vmrss_delta_bytes",
    "vmdata_delta_bytes",
    "vmrss_mib",
    "vmhwm_mib",
    "vmsize_mib",
    "vmdata_mib",
    "rssanon_mib",
    "rssfile_mib",
    "vmrss_delta_mib",
    "vmdata_delta_mib",
    "messages",
    "signals",
    "subscribed",
    "events",
    "encode_errors",
    "overflow_errors",
    "unknown_verbs",
    "skipped",
]


def _bytes_to_mib(value: Optional[int]) -> Optional[float]:
    if value is None:
        return None
    return round(value / (1024 * 1024), 3)


def _metric_bytes(snapshot: Dict[str, Any], key: str) -> Optional[int]:
    value = snapshot.get(key)
    return value if isinstance(value, int) else None


def _build_metrics_record(
    *,
    event: str,
    snapshot: Dict[str, Any],
    baseline: Optional[Dict[str, Any]] = None,
    stats: Optional[Dict[str, int]] = None,
    elapsed_s: Optional[float] = None,
    loop_index: Optional[int] = None,
    loop_total: Optional[int] = None,
    seed: Optional[int] = None,
) -> Dict[str, Any]:
    timestamp = float(snapshot.get("timestamp", time.time()))
    vmrss = _metric_bytes(snapshot, "VmRSS")
    vmhwm = _metric_bytes(snapshot, "VmHWM")
    vmsize = _metric_bytes(snapshot, "VmSize")
    vmdata = _metric_bytes(snapshot, "VmData")
    rssanon = _metric_bytes(snapshot, "RssAnon")
    rssfile = _metric_bytes(snapshot, "RssFile")

    vmrss_delta: Optional[int] = None
    vmdata_delta: Optional[int] = None
    if baseline is not None:
        base_rss = _metric_bytes(baseline, "VmRSS")
        base_data = _metric_bytes(baseline, "VmData")
        if vmrss is not None and base_rss is not None:
            vmrss_delta = vmrss - base_rss
        if vmdata is not None and base_data is not None:
            vmdata_delta = vmdata - base_data

    record: Dict[str, Any] = {
        "timestamp_iso": datetime.fromtimestamp(timestamp, tz=timezone.utc).isoformat(),
        "timestamp_unix": round(timestamp, 6),
        "event": event,
        "loop_index": loop_index,
        "loop_total": loop_total,
        "elapsed_s": round(elapsed_s, 6) if elapsed_s is not None else None,
        "seed": seed,
        "pid": snapshot.get("pid"),
        "proc_name": snapshot.get("Name"),
        "proc_state": snapshot.get("State"),
        "can_api": CAN_API,
        "vcan_iface": VCAN_IFACE,
        "dbc_file": str(DBC_FILE),
        "canids": _format_canids_filter(),
        "vmrss_bytes": vmrss,
        "vmhwm_bytes": vmhwm,
        "vmsize_bytes": vmsize,
        "vmdata_bytes": vmdata,
        "rssanon_bytes": rssanon,
        "rssfile_bytes": rssfile,
        "threads": snapshot.get("Threads"),
        "fds": snapshot.get("fds"),
        "vmrss_delta_bytes": vmrss_delta,
        "vmdata_delta_bytes": vmdata_delta,
        "vmrss_mib": _bytes_to_mib(vmrss),
        "vmhwm_mib": _bytes_to_mib(vmhwm),
        "vmsize_mib": _bytes_to_mib(vmsize),
        "vmdata_mib": _bytes_to_mib(vmdata),
        "rssanon_mib": _bytes_to_mib(rssanon),
        "rssfile_mib": _bytes_to_mib(rssfile),
        "vmrss_delta_mib": _bytes_to_mib(vmrss_delta),
        "vmdata_delta_mib": _bytes_to_mib(vmdata_delta),
    }

    if stats is not None:
        record.update(stats)

    return record


class _MetricsLogger:
    def __init__(
        self,
        path: Path,
        *,
        log_format: str,
        append: bool,
        flush: bool,
    ) -> None:
        self.path = path
        self.log_format = log_format
        self.append = append
        self.flush = flush
        self._file: Optional[Any] = None
        self._writer: Optional[csv.DictWriter] = None

    def __enter__(self) -> "_MetricsLogger":
        if self.log_format not in {"csv", "jsonl"}:
            raise ValueError(
                f"Unsupported metrics log format: {self.log_format}. "
                "Expected 'csv' or 'jsonl'."
            )

        self.path.parent.mkdir(parents=True, exist_ok=True)
        mode = "a" if self.append else "w"
        newline = "" if self.log_format == "csv" else None
        self._file = self.path.open(mode, encoding="utf-8", newline=newline)

        if self.log_format == "csv":
            self._writer = csv.DictWriter(
                self._file, fieldnames=_METRICS_FIELDS, extrasaction="ignore"
            )
            should_write_header = (
                not self.append
                or not self.path.exists()
                or self.path.stat().st_size == 0
            )
            if should_write_header:
                self._writer.writeheader()

        return self

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        if self._file is not None:
            self._file.close()
        self._file = None
        self._writer = None

    def write(self, record: Dict[str, Any]) -> None:
        if self._file is None:
            return

        if self.log_format == "jsonl":
            self._file.write(json.dumps(record, sort_keys=True) + "\n")
        else:
            assert self._writer is not None
            self._writer.writerow(
                {field: record.get(field) for field in _METRICS_FIELDS}
            )

        if self.flush:
            self._file.flush()


def _should_write_metrics_loop(loop_index: int, loop_total: int) -> bool:
    if METRICS_LOG_FILE is None or METRICS_LOG_EVERY <= 0:
        return False
    return (
        loop_index == 1
        or loop_index == loop_total
        or loop_index % METRICS_LOG_EVERY == 0
    )


def _format_canids_filter() -> str:
    if ALLOWED_CANIDS is None:
        return "all"
    return ",".join(f"0x{x:X}" for x in sorted(ALLOWED_CANIDS))


def _new_scenario_stats() -> Dict[str, int]:
    return {
        "messages": 0,
        "signals": 0,
        "subscribed": 0,
        "events": 0,
        "encode_errors": 0,
        "overflow_errors": 0,
        "unknown_verbs": 0,
        "skipped": 0,
    }


def _merge_scenario_stats(dst: Dict[str, int], src: Dict[str, int]) -> None:
    for key, value in src.items():
        dst[key] = dst.get(key, 0) + value


def _format_scenario_stats(stats: Dict[str, int]) -> str:
    return " ".join(f"{key}={value}" for key, value in sorted(stats.items()))


@contextmanager
def _assert_can_event_emitted(
    binder: Any,
    api: str,
    event_name: str,
    *,
    timeout_ms: int,
    value: Any,
    value_lambda: Any,
):
    """
    Wait for a decoded CAN event.

    With locally loaded bindings the event prefix is stable and matches the API
    name. With imported APIs, the event may be forwarded through the local
    binder with a different API prefix depending on the import/export mode.
    Register a wildcard handler in external-binder mode so the test still checks
    the event label and payload without depending on that prefix.
    """
    import time as _time

    exact_pattern = f"{api}/{event_name}"
    patterns = [exact_pattern]
    if EXTERNAL_BINDER_MODE:
        patterns.append(f"*/{event_name}")

    state: Dict[str, Any] = {"received": False, "data": [], "event": None}

    def on_evt(*args: Any) -> None:
        state["received"] = True
        if len(args) > 1:
            state["event"] = args[1]
        data = state["data"]
        data.clear()
        if len(args) > 3:
            data.extend(args[3:])

    for pattern in patterns:
        libafb.evthandler(
            binder,
            {"pattern": pattern, "callback": on_evt},
            None,
        )

    try:
        yield

        start = _time.monotonic()
        while (_time.monotonic() - start) * 1000.0 <= timeout_ms and not state[
            "received"
        ]:
            _time.sleep(0.01)

        if not state["received"]:
            raise EventAssertionError(
                f"No event received within {timeout_ms} ms for pattern '{exact_pattern}'."
            )

        list_data = state["data"]
        if len(list_data) > 0:
            got_value = (
                value_lambda(list_data[0]) if value_lambda is not None else list_data[0]
            )
            if got_value != value:
                raise EventAssertionError(
                    f"Event received but payload value mismatch for pattern '{exact_pattern}'.\n"
                    f"Received event: {state['event']!r}\n"
                    f"Expected value: {value!r}\n"
                    f"Got value:      {got_value!r}\n"
                    f"Full payload:   {list_data!r}"
                )
    finally:
        state["data"].clear()
        state["event"] = None


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
# External binder client
# -----------------------------------------------------------------------------


class _ExternalAfbClientError(RuntimeError):
    pass


def _external_client_available() -> str:
    client = shutil.which(EXTERNAL_CLIENT_BIN)
    if client is None:
        raise _ExternalAfbClientError(
            f"Missing external client '{EXTERNAL_CLIENT_BIN}'. Install afb-client "
            "or set AFB_CLIENT_BIN to a compatible client binary."
        )
    return client


def _line_has_afb_error(output: str) -> bool:
    """
    Return True only for real afb-client errors.

    afb-client may return multi-line output, for example:

      ON-REPLY 1:sockbcm/check: OK
      {"jtype":"afb-reply","request":{"status":"success","code":0}}

    Evaluate each line independently so a successful ON-REPLY line followed by a
    JSON reply is not misclassified as an error.
    """
    for raw_line in output.splitlines() or [output]:
        stripped = raw_line.strip()
        if not stripped:
            continue

        lower = stripped.lower()

        # afb-client prefixes replies with a human-readable status line such as:
        #   ON-REPLY 1:model3/SomeSignalContainingError: OK
        # Do not classify the line as an error only because the API/verb name
        # contains the word "Error". Only the reply status matters here.
        if lower.startswith("on-reply "):
            if not lower.endswith(": ok"):
                return True
            continue

        for candidate in _iter_json_candidates(stripped):
            for item in _walk_json_values(candidate):
                if not isinstance(item, dict):
                    continue

                request = item.get("request")
                if isinstance(request, dict):
                    status = str(request.get("status", "")).strip().lower()
                    if status:
                        if status not in {"success", "ok"}:
                            return True
                        continue

                    code = request.get("code")
                    if isinstance(code, int):
                        if code != 0:
                            return True
                        continue

                status = str(item.get("status", "")).strip().lower()
                if status in {"error", "failed", "failure"}:
                    return True

        if any(
            token in lower
            for token in (
                "disconnected",
                "server hung up",
                "unknown-verb",
                "permission-denied",
                "invalid",
            )
        ):
            return True

    return False


def _iter_json_candidates(line: str) -> Iterable[Any]:
    stripped = line.strip()
    if not stripped:
        return

    starts = [idx for idx, char in enumerate(stripped) if char in "[{"]
    if not starts and stripped:
        starts = [0]

    for idx in starts:
        try:
            yield json.loads(stripped[idx:])
        except Exception:
            continue


def _walk_json_values(value: Any) -> Iterable[Any]:
    yield value
    if isinstance(value, dict):
        for child in value.values():
            yield from _walk_json_values(child)
    elif isinstance(value, list):
        for child in value:
            yield from _walk_json_values(child)


def _extract_external_event_payload(
    line: str, event_name: str
) -> Optional[Dict[str, Any]]:
    if event_name not in line:
        return None

    for candidate in _iter_json_candidates(line):
        dicts = [
            item for item in _walk_json_values(candidate) if isinstance(item, dict)
        ]
        for item in dicts:
            if "value" in item:
                return item
        if dicts:
            return dicts[-1]

    return None


class _ExternalAfbSubscription:
    def __init__(
        self,
        *,
        sockspec: str,
        api: str,
        verb: str,
        request: Dict[str, Any],
        timeout_ms: int,
        expected_value: Any,
        value_lambda: Any,
    ) -> None:
        self.sockspec = sockspec
        self.api = api
        self.verb = verb
        self.request = request
        self.timeout_ms = timeout_ms
        self.expected_value = expected_value
        self.value_lambda = value_lambda
        self.event_name = f"{api}/{verb}"
        self.process: Optional[subprocess.Popen[str]] = None
        self.lines: "queue.Queue[str]" = queue.Queue()
        self.reader: Optional[threading.Thread] = None

    def __enter__(self) -> "_ExternalAfbSubscription":
        client = _external_client_available()
        cmd = [
            client,
            "-r",
            "-s",
            "-k",
            self.sockspec,
            self.api,
            self.verb,
            json.dumps(self.request, separators=(",", ":")),
        ]

        LOG.info("[external-client] start cmd=%r", cmd)
        self.process = subprocess.Popen(
            cmd,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )

        assert self.process.stdout is not None

        def _read_stdout() -> None:
            assert self.process is not None
            stdout = self.process.stdout
            assert stdout is not None
            try:
                for raw_line in stdout:
                    line = raw_line.rstrip("\n")
                    if line:
                        self.lines.put(line)
            finally:
                if not stdout.closed:
                    stdout.close()

        self.reader = threading.Thread(target=_read_stdout, daemon=True)
        self.reader.start()
        try:
            self._wait_subscribe_reply()
        except Exception:
            self.close()
            raise
        return self

    def __exit__(self, exc_type: Any, exc: Any, tb: Any) -> None:
        try:
            if exc_type is None:
                self.wait_event()
        finally:
            self.close()

    def _wait_subscribe_reply(self) -> None:
        deadline = time.monotonic() + 5.0
        seen: List[str] = []

        while time.monotonic() < deadline:
            line = self._get_line(timeout=0.25)
            if line is None:
                self._check_process_alive(seen)
                continue

            seen.append(line)
            LOG.info("[external-client] line %s", line)
            if _line_has_afb_error(line):
                raise _ExternalAfbClientError(
                    f"Subscribe failed for {self.api}/{self.verb}: {line}"
                )

            # afb-client emits one reply line for the initial subscribe call.
            # Keep the process alive after that line to receive future events.
            return

        raise _ExternalAfbClientError(
            "No subscribe reply received from external binder for "
            f"{self.api}/{self.verb}. Seen lines: {seen!r}"
        )

    def wait_event(self) -> None:
        deadline = time.monotonic() + (self.timeout_ms / 1000.0)
        seen: List[str] = []

        while time.monotonic() < deadline:
            line = self._get_line(timeout=0.05)
            if line is None:
                self._check_process_alive(seen)
                continue

            seen.append(line)
            LOG.info("[external-client] line %s", line)

            payload = _extract_external_event_payload(line, self.event_name)
            if payload is None:
                continue

            got_value = (
                self.value_lambda(payload) if self.value_lambda is not None else payload
            )
            if got_value != self.expected_value:
                raise EventAssertionError(
                    f"Event received but payload value mismatch for pattern "
                    f"'{self.event_name}'.\n"
                    f"Expected value: {self.expected_value!r}\n"
                    f"Got value:      {got_value!r}\n"
                    f"Full payload:   {payload!r}"
                )
            return

        raise EventAssertionError(
            f"No event received within {self.timeout_ms} ms for pattern "
            f"'{self.event_name}'. Seen lines: {seen!r}"
        )

    def close(self) -> None:
        process = self.process
        if process is None:
            return

        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=2)

        stdout = process.stdout
        if stdout is not None and not stdout.closed:
            stdout.close()

        if self.reader is not None and self.reader.is_alive():
            self.reader.join(timeout=1)

        self.reader = None
        self.process = None

    def _get_line(self, *, timeout: float) -> Optional[str]:
        try:
            return self.lines.get(timeout=timeout)
        except queue.Empty:
            return None

    def _check_process_alive(self, seen: List[str]) -> None:
        if self.process is None:
            return
        rc = self.process.poll()
        if rc is not None:
            raise _ExternalAfbClientError(
                f"afb-client exited unexpectedly with status {rc} while waiting "
                f"for {self.api}/{self.verb}. Seen lines: {seen!r}"
            )


def _external_call(
    api: str, verb: str, request: Optional[Dict[str, Any]] = None
) -> str:
    if EXTERNAL_BINDER_SOCKSPEC is None:
        raise _ExternalAfbClientError("External binder sockspec is not configured")

    client = _external_client_available()
    cmd = [
        client,
        "-r",
        "-s",
        EXTERNAL_BINDER_SOCKSPEC,
        api,
        verb,
        json.dumps(request or {}, separators=(",", ":")),
    ]
    completed = subprocess.run(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    output = completed.stdout.strip()
    LOG.info(
        "[external-client] call cmd=%r rc=%s output=%r",
        cmd,
        completed.returncode,
        output,
    )
    if completed.returncode != 0 or _line_has_afb_error(output):
        raise _ExternalAfbClientError(
            f"External call failed for {api}/{verb}: rc={completed.returncode} output={output!r}"
        )
    return output


@contextmanager
def _external_event_subscription(
    api: str,
    verb: str,
    *,
    timeout_ms: int,
    value: Any,
    value_lambda: Any,
):
    if EXTERNAL_BINDER_SOCKSPEC is None:
        raise _ExternalAfbClientError("External binder sockspec is not configured")

    request = {"action": "subscribe", "flag": "all", "rate": 0, "watchdog": 0}
    with _ExternalAfbSubscription(
        sockspec=EXTERNAL_BINDER_SOCKSPEC,
        api=api,
        verb=verb,
        request=request,
        timeout_ms=timeout_ms,
        expected_value=value,
        value_lambda=value_lambda,
    ):
        yield


# -----------------------------------------------------------------------------
# AFB verbs
# -----------------------------------------------------------------------------


def _is_unknown_verb_error(e: Exception) -> bool:
    return "unknown-verb" in str(e).lower()


def _verb_for(msg_name: str, sig_name: str) -> str:
    return f"{to_upper_camel_case(msg_name)}/{to_upper_camel_case(sig_name)}"


def _subscribe_signal(binder: Any, msg_name: str, sig_name: str) -> str:
    verb = _verb_for(msg_name, sig_name)
    request = {"action": "subscribe", "flag": "all", "rate": 0, "watchdog": 0}

    try:
        if EXTERNAL_BINDER_MODE:
            LOG.info(
                "[external-subscribe] call api=%s verb=%s request=%r",
                CAN_API,
                verb,
                request,
            )

        reply = libafb.callsync(binder, CAN_API, verb, request)

        if EXTERNAL_BINDER_MODE:
            LOG.info(
                "[external-subscribe] reply api=%s verb=%s reply=%r status=%r args=%r",
                CAN_API,
                verb,
                reply,
                getattr(reply, "status", None),
                getattr(reply, "args", None),
            )

        return verb
    except Exception as e:
        if EXTERNAL_BINDER_MODE:
            LOG.info(
                "[external-subscribe] error api=%s verb=%s error=%r",
                CAN_API,
                verb,
                e,
            )
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
    LOG.info("Using bindings: %s", bindings)
    LOG.info("Using config: %s", config)
    if EXTERNAL_BINDER_MODE:
        LOG.info("Using external API imports: %s", EXTERNAL_API_IMPORTS)
        LOG.info("Using external binder sockspec: %s", EXTERNAL_BINDER_SOCKSPEC)
        if not EXTERNAL_API_IMPORTS:
            raise RuntimeError(
                "External binder mode requested but no API import URI was configured."
            )
        configure_afb_binding_tests(bindings={}, config={})
        return

    if not bindings or not config:
        raise RuntimeError(
            "Binder configuration not initialized. "
            "Run the script directly with --config/--can-api/--dbc-file, "
            "or export BINDER_CONFIG before running the tests."
        )
    configure_afb_binding_tests(bindings=bindings, config=config)


class TestBindingDbcGenerated(AFBTestCase):
    def run(self, result: Optional[unittest.TestResult] = None) -> None:
        if not EXTERNAL_BINDER_MODE:
            return super().run(result)

        # External mode intentionally does not create a local libafb binder.
        # The test uses a real external client socket for calls/subscriptions,
        # which matches the devtools behavior and keeps events tied to the same
        # client session that issued the subscribe request.
        return unittest.TestCase.run(self, result)

    def test_check_ok(self) -> None:
        if EXTERNAL_BINDER_MODE:
            _external_call("sockbcm", "check")
            return

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

    def _run_dbc_random_signal_scenario(
        self, db: Any, bus: can.Bus, rng: random.Random
    ) -> Dict[str, int]:
        stats = _new_scenario_stats()

        for msg in db.messages:
            canid = msg.frame_id
            if ALLOWED_CANIDS is not None and canid not in ALLOWED_CANIDS:
                continue

            stats["messages"] += 1
            msg_name = msg.name
            is_extended = bool(msg.is_extended_frame)

            for sig in msg.signals:
                stats["signals"] += 1
                sig_name = sig.name

                if sig.is_multiplexer:
                    valid = _valid_mux_ids_for_message(msg)
                    candidate = rng.choice(valid) if valid else _safe_default_value(sig)
                else:
                    candidate = _pick_random_value(sig, rng)

                values = _build_values_for_message(msg, sig, candidate, rng)

                try:
                    data = _encode_message(msg, values)
                except cantools.database.errors.EncodeError as e:
                    stats["encode_errors"] += 1
                    stats["skipped"] += 1
                    LOG.warning(
                        f"[AFB] Encode error canid={canid} msg={msg_name} sig={sig_name}: {e}"
                    )
                    continue
                except OverflowError as e:
                    stats["overflow_errors"] += 1
                    stats["skipped"] += 1
                    LOG.info(
                        f"[AFB] Encode overflow canid={canid} msg={msg_name} sig={sig_name}: {e}"
                    )
                    continue

                decoded = _decode_message(msg, data)
                expected = decoded.get(sig_name, candidate)
                if _is_boolean_signal(sig):
                    expected = bool(expected)
                expected_norm = _normalize_scalar(expected)

                case_label = (
                    f"canid={canid} msg={msg_name} "
                    f"sig={sig_name} expected={expected_norm}"
                )

                verb = _verb_for(msg_name, sig_name)

                subscribed = False
                try:
                    event_ctx = (
                        _external_event_subscription(
                            CAN_API,
                            verb,
                            timeout_ms=5000,
                            value=expected_norm,
                            value_lambda=lambda payload: _normalize_scalar(
                                _extract_event_value(payload)
                            ),
                        )
                        if EXTERNAL_BINDER_MODE
                        else self.assertEventEmitted(
                            CAN_API,
                            verb,
                            timeout_ms=5000,
                            value=expected_norm,
                            value_lambda=lambda payload: _normalize_scalar(
                                _extract_event_value(payload)
                            ),
                        )
                    )

                    with event_ctx:
                        if not EXTERNAL_BINDER_MODE:
                            try:
                                _subscribe_signal(self.binder, msg_name, sig_name)
                            except Exception as e:
                                if _is_unknown_verb_error(e):
                                    stats["unknown_verbs"] += 1
                                    stats["skipped"] += 1
                                    cc_msg = to_upper_camel_case(msg_name)
                                    cc_sig = to_upper_camel_case(sig_name)
                                    LOG.debug(
                                        f"[AFB] Skip unknown verb for canid={canid} "
                                        f"msg={msg_name}({cc_msg}) sig={sig_name}({cc_sig})"
                                    )
                                    continue
                                LOG.error("[AFB] Subscribe failed: %s", case_label)
                                raise

                        subscribed = True
                        stats["subscribed"] += 1
                        _send_frame(bus, canid, data, is_extended)
                    stats["events"] += 1
                except EventAssertionError as e:
                    LOG.error("[AFB] EventAssertionError %s", case_label)
                    LOG.error(f"[AFB] Values used for encode: {values}")
                    LOG.error(f"[AFB] Encoded bytes: {data.hex()}")
                    raise EventAssertionError(f"{case_label}: {e}") from e
                finally:
                    if subscribed and not EXTERNAL_BINDER_MODE:
                        _unsubscribe_signal(self.binder, msg_name, sig_name)

        return stats

    def test_dbc_random_signal_events(self) -> None:
        if not DBC_FILE.exists():
            raise FileNotFoundError(f"Missing DBC file: {DBC_FILE}")

        db = cantools.database.load_file(str(DBC_FILE))
        bus = can.interface.Bus(channel=VCAN_IFACE, interface="socketcan")
        memory_pid = MEMORY_PID or os.getpid()
        overall_stats = _new_scenario_stats()
        overall_start = time.monotonic()
        overall_before = _memory_snapshot(memory_pid)

        print(
            "# [scenario] "
            f"pid={memory_pid} loops={SCENARIO_LOOPS} "
            f"seed={RANDOM_SEED} vary_seed={VARY_SEED_PER_LOOP} "
            f"can_api={CAN_API} vcan={VCAN_IFACE} "
            f"dbc={DBC_FILE} canids={_format_canids_filter()}"
        )
        print(f"# [mem] before: {_format_memory_snapshot(overall_before)}")
        metrics_logger: Optional[_MetricsLogger] = None
        if METRICS_LOG_FILE is not None:
            metrics_logger = _MetricsLogger(
                METRICS_LOG_FILE,
                log_format=METRICS_LOG_FORMAT,
                append=METRICS_LOG_APPEND,
                flush=METRICS_LOG_FLUSH,
            )
            print(
                "# [metrics] "
                f"file={METRICS_LOG_FILE} format={METRICS_LOG_FORMAT} "
                f"append={METRICS_LOG_APPEND} every={METRICS_LOG_EVERY}"
            )

        total_elapsed = 0.0
        overall_after: Dict[str, Any] = overall_before

        try:
            with metrics_logger if metrics_logger is not None else nullcontext():
                if metrics_logger is not None:
                    metrics_logger.write(
                        _build_metrics_record(
                            event="start",
                            snapshot=overall_before,
                            loop_total=SCENARIO_LOOPS,
                            seed=RANDOM_SEED,
                            elapsed_s=0.0,
                        )
                    )

                for loop_index in range(1, SCENARIO_LOOPS + 1):
                    if GC_BETWEEN_LOOPS:
                        gc.collect()

                    loop_seed = (
                        RANDOM_SEED + loop_index - 1
                        if VARY_SEED_PER_LOOP
                        else RANDOM_SEED
                    )
                    rng = random.Random(loop_seed)
                    loop_before = _memory_snapshot(memory_pid)
                    loop_start = time.monotonic()

                    stats = self._run_dbc_random_signal_scenario(db, bus, rng)
                    _merge_scenario_stats(overall_stats, stats)

                    elapsed = time.monotonic() - loop_start
                    loop_after = _memory_snapshot(memory_pid)

                    if MEMORY_REPORT_EVERY > 0 and (
                        loop_index == 1
                        or loop_index == SCENARIO_LOOPS
                        or loop_index % MEMORY_REPORT_EVERY == 0
                    ):
                        print(
                            f"# [loop {loop_index}/{SCENARIO_LOOPS}] "
                            f"elapsed={elapsed:.3f}s seed={loop_seed} "
                            f"{_format_scenario_stats(stats)}"
                        )
                        print(
                            f"# [mem] loop {loop_index}: "
                            f"{_format_memory_snapshot(loop_after, loop_before)}"
                        )

                    if metrics_logger is not None and _should_write_metrics_loop(
                        loop_index, SCENARIO_LOOPS
                    ):
                        metrics_logger.write(
                            _build_metrics_record(
                                event="loop",
                                snapshot=loop_after,
                                baseline=loop_before,
                                stats=stats,
                                elapsed_s=elapsed,
                                loop_index=loop_index,
                                loop_total=SCENARIO_LOOPS,
                                seed=loop_seed,
                            )
                        )

                    if LOOP_DELAY_MS > 0 and loop_index < SCENARIO_LOOPS:
                        time.sleep(LOOP_DELAY_MS / 1000.0)

                total_elapsed = time.monotonic() - overall_start
                overall_after = _memory_snapshot(memory_pid)
                if metrics_logger is not None:
                    metrics_logger.write(
                        _build_metrics_record(
                            event="summary",
                            snapshot=overall_after,
                            baseline=overall_before,
                            stats=overall_stats,
                            elapsed_s=total_elapsed,
                            loop_total=SCENARIO_LOOPS,
                            seed=RANDOM_SEED,
                        )
                    )
        finally:
            bus.shutdown()

        print(
            f"# [summary] elapsed={total_elapsed:.3f}s "
            f"{_format_scenario_stats(overall_stats)}"
        )
        print(
            f"# [mem] after: {_format_memory_snapshot(overall_after, overall_before)}"
        )

        if FAIL_ON_RSS_GROWTH_BYTES is not None:
            before_rss = overall_before.get("VmRSS")
            after_rss = overall_after.get("VmRSS")
            if isinstance(before_rss, int) and isinstance(after_rss, int):
                growth = after_rss - before_rss
                if growth > FAIL_ON_RSS_GROWTH_BYTES:
                    raise AssertionError(
                        "RSS memory growth exceeded threshold: "
                        f"growth={_format_signed_bytes(growth)} "
                        f"threshold={_format_bytes(FAIL_ON_RSS_GROWTH_BYTES)}"
                    )


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
        "--external-binder-uri",
        dest="external_binder_uri",
        default=None,
        help=(
            "Import tested APIs from an already running binder instead of loading "
            "local binding .so files. The API name is appended below the AFB "
            "root API path for imported API URIs. For example "
            "tcp:localhost:1234 becomes tcp:localhost:1234/api/model3. "
            "The external afb-client backend converts this to "
            "http://localhost:1234/api when calling afb-client."
        ),
    )
    p.add_argument(
        "--external-api-uri",
        dest="external_api_uris",
        action="append",
        default=[],
        metavar="API=URI",
        help=(
            "Import one API from an already running binder. Can be repeated. "
            "Example: --external-api-uri model3=tcp:localhost:1234/api/model3."
        ),
    )
    p.add_argument(
        "--external-client-bin",
        dest="external_client_bin",
        default=EXTERNAL_CLIENT_BIN,
        help=(
            "Client binary used by --external-binder-uri mode. "
            "Default: AFB_CLIENT_BIN or afb-client."
        ),
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
    p.add_argument(
        "--loops",
        dest="loops",
        type=int,
        default=SCENARIO_LOOPS,
        help=(
            "Replay the DBC signal scenario N times in the same binder process. "
            "Default: TEST_SCENARIO_LOOPS or 1."
        ),
    )
    p.add_argument(
        "--memory-report-every",
        dest="memory_report_every",
        type=int,
        default=MEMORY_REPORT_EVERY,
        help=(
            "Print memory diagnostics every N loops. Use 0 to keep only before/after. "
            "Default: TEST_MEMORY_REPORT_EVERY or 1."
        ),
    )
    p.add_argument(
        "--memory-pid",
        dest="memory_pid",
        type=int,
        default=MEMORY_PID,
        help=(
            "PID to monitor for memory diagnostics. Default: current Python/binder process. "
            "Can also be set with TEST_MEMORY_PID."
        ),
    )
    p.add_argument(
        "--fail-on-rss-growth-mib",
        dest="fail_on_rss_growth_mib",
        type=float,
        default=(
            FAIL_ON_RSS_GROWTH_BYTES / (1024 * 1024)
            if FAIL_ON_RSS_GROWTH_BYTES is not None
            else None
        ),
        help=(
            "Fail the test when RSS growth after all loops exceeds this MiB threshold. "
            "Default: TEST_FAIL_ON_RSS_GROWTH_MIB or disabled."
        ),
    )
    p.add_argument(
        "--gc-between-loops",
        dest="gc_between_loops",
        action="store_true",
        default=GC_BETWEEN_LOOPS,
        help="Run Python gc.collect() before each loop. Default: TEST_GC_BETWEEN_LOOPS.",
    )
    p.add_argument(
        "--loop-delay-ms",
        dest="loop_delay_ms",
        type=int,
        default=LOOP_DELAY_MS,
        help="Sleep this many milliseconds between loops. Default: TEST_LOOP_DELAY_MS or 0.",
    )
    p.add_argument(
        "--vary-seed-per-loop",
        dest="vary_seed_per_loop",
        action="store_true",
        default=VARY_SEED_PER_LOOP,
        help=(
            "Use TEST_SEED + loop index instead of replaying the exact same random values "
            "on every loop. Default: TEST_VARY_SEED_PER_LOOP."
        ),
    )
    p.add_argument(
        "--metrics-log",
        dest="metrics_log",
        type=Path,
        default=METRICS_LOG_FILE,
        help=(
            "Write timing, memory and scenario counters to this file. "
            "Default: TEST_METRICS_LOG_FILE or disabled."
        ),
    )
    p.add_argument(
        "--metrics-log-format",
        dest="metrics_log_format",
        choices=("csv", "jsonl"),
        default=METRICS_LOG_FORMAT if METRICS_LOG_FORMAT in {"csv", "jsonl"} else "csv",
        help="Metrics log format. Default: TEST_METRICS_LOG_FORMAT or csv.",
    )
    p.add_argument(
        "--metrics-log-every",
        dest="metrics_log_every",
        type=int,
        default=METRICS_LOG_EVERY,
        help=(
            "Write one metrics row every N loops when --metrics-log is enabled. "
            "Use 0 to write only start and summary. "
            "Default: TEST_METRICS_LOG_EVERY or 1."
        ),
    )
    p.add_argument(
        "--metrics-log-append",
        dest="metrics_log_append",
        action="store_true",
        default=METRICS_LOG_APPEND,
        help="Append to the metrics log instead of overwriting it. Default: TEST_METRICS_LOG_APPEND.",
    )
    p.add_argument(
        "--no-metrics-log-flush",
        dest="metrics_log_flush",
        action="store_false",
        default=METRICS_LOG_FLUSH,
        help="Do not flush the metrics log after each row.",
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
    LOG.info("Using binder config: %s", cfg_path)
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

    if args.loops < 1:
        p.error("--loops must be >= 1")
    SCENARIO_LOOPS = args.loops

    if args.memory_report_every < 0:
        p.error("--memory-report-every must be >= 0")
    MEMORY_REPORT_EVERY = args.memory_report_every

    if args.memory_pid is not None and args.memory_pid < 1:
        p.error("--memory-pid must be >= 1")
    MEMORY_PID = args.memory_pid

    if args.fail_on_rss_growth_mib is not None and args.fail_on_rss_growth_mib < 0:
        p.error("--fail-on-rss-growth-mib must be >= 0")
    FAIL_ON_RSS_GROWTH_BYTES = _mib_to_bytes(args.fail_on_rss_growth_mib)

    if args.loop_delay_ms < 0:
        p.error("--loop-delay-ms must be >= 0")
    LOOP_DELAY_MS = args.loop_delay_ms

    GC_BETWEEN_LOOPS = bool(args.gc_between_loops)
    VARY_SEED_PER_LOOP = bool(args.vary_seed_per_loop)

    METRICS_LOG_FILE = (
        args.metrics_log.expanduser() if args.metrics_log is not None else None
    )
    METRICS_LOG_FORMAT = str(args.metrics_log_format).strip().lower()
    METRICS_LOG_APPEND = bool(args.metrics_log_append)
    METRICS_LOG_FLUSH = bool(args.metrics_log_flush)

    if args.metrics_log_every < 0:
        p.error("--metrics-log-every must be >= 0")
    METRICS_LOG_EVERY = args.metrics_log_every

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

    EXTERNAL_CLIENT_BIN = str(args.external_client_bin)

    EXTERNAL_API_IMPORTS = _build_external_api_imports(
        base_uri=args.external_binder_uri,
        explicit_api_uris=args.external_api_uris,
    )
    EXTERNAL_BINDER_SOCKSPEC = _infer_external_binder_sockspec(
        base_uri=args.external_binder_uri,
        imports=EXTERNAL_API_IMPORTS,
    )
    EXTERNAL_BINDER_MODE = bool(EXTERNAL_API_IMPORTS)

    if EXTERNAL_BINDER_MODE:
        missing_apis = [
            api for api in bindings.keys() if api not in EXTERNAL_API_IMPORTS
        ]
        if missing_apis:
            p.error(
                "External binder mode requires an import URI for every configured API. "
                f"Missing: {', '.join(sorted(missing_apis))}"
            )

    # unittest parses sys.argv internally; remove our custom CLI args
    # so it doesn't choke on --config/--verbose/--debug/--quiet.
    sys.argv = [sys.argv[0]] + remaining

    run_afb_binding_tests(bindings)
