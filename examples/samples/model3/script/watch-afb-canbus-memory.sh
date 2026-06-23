#!/usr/bin/env bash
set -euo pipefail

PROC_NAME="${1:-afb-canbus}"
INTERVAL="${2:-0}"

bytes_to_mib() {
    local bytes="${1:-0}"
    awk -v b="$bytes" 'BEGIN { printf "%.1f MiB", b / 1024 / 1024 }'
}

signed_bytes_to_mib() {
    local bytes="${1:-0}"
    awk -v b="$bytes" 'BEGIN {
        sign = (b >= 0) ? "+" : "-"
        if (b < 0) b = -b
        printf "%s%.1f MiB", sign, b / 1024 / 1024
    }'
}

find_pid() {
    pgrep -x "$PROC_NAME" | head -n 1
}

read_status_value_kb() {
    local pid="$1"
    local key="$2"

    awk -v key="$key" '
        $1 == key ":" {
            print $2 * 1024
            found = 1
            exit
        }
        END {
            if (!found) print 0
        }
    ' "/proc/${pid}/status"
}

read_status_text() {
    local pid="$1"
    local key="$2"

    awk -v key="$key" '
        $1 == key ":" {
            sub("^[^:]+:[ \t]*", "")
            print
            found = 1
            exit
        }
        END {
            if (!found) print "n/a"
        }
    ' "/proc/${pid}/status"
}

count_fds() {
    local pid="$1"

    if [[ -d "/proc/${pid}/fd" ]]; then
        find "/proc/${pid}/fd" -maxdepth 1 -mindepth 1 2>/dev/null | wc -l
    else
        echo "0"
    fi
}

print_mem() {
    local pid="$1"
    local base_rss="${2:-}"
    local base_data="${3:-}"

    local name state rss hwm vmsize data rss_anon rss_file threads fds

    name="$(read_status_text "$pid" "Name")"
    state="$(read_status_text "$pid" "State")"

    rss="$(read_status_value_kb "$pid" "VmRSS")"
    hwm="$(read_status_value_kb "$pid" "VmHWM")"
    vmsize="$(read_status_value_kb "$pid" "VmSize")"
    data="$(read_status_value_kb "$pid" "VmData")"
    rss_anon="$(read_status_value_kb "$pid" "RssAnon")"
    rss_file="$(read_status_value_kb "$pid" "RssFile")"

    threads="$(awk '$1 == "Threads:" { print $2; found = 1; exit } END { if (!found) print 0 }' "/proc/${pid}/status")"
    fds="$(count_fds "$pid")"

    printf "pid=%s name=%s state=%s rss=%s hwm=%s vmsize=%s data=%s rss_anon=%s rss_file=%s threads=%s fds=%s" \
        "$pid" \
        "$name" \
        "$state" \
        "$(bytes_to_mib "$rss")" \
        "$(bytes_to_mib "$hwm")" \
        "$(bytes_to_mib "$vmsize")" \
        "$(bytes_to_mib "$data")" \
        "$(bytes_to_mib "$rss_anon")" \
        "$(bytes_to_mib "$rss_file")" \
        "$threads" \
        "$fds"

    if [[ -n "$base_rss" && -n "$base_data" ]]; then
        printf " rss_delta=%s data_delta=%s" \
            "$(signed_bytes_to_mib "$((rss - base_rss))")" \
            "$(signed_bytes_to_mib "$((data - base_data))")"
    fi

    printf "\n"
}

pid="$(find_pid || true)"

if [[ -z "${pid}" ]]; then
    echo "ERROR: process not found: ${PROC_NAME}" >&2
    exit 1
fi

base_rss="$(read_status_value_kb "$pid" "VmRSS")"
base_data="$(read_status_value_kb "$pid" "VmData")"

echo "# [mem] before: $(print_mem "$pid")"

if [[ "$INTERVAL" == "0" ]]; then
    exit 0
fi

loop=1
while kill -0 "$pid" 2>/dev/null; do
    sleep "$INTERVAL"
    echo "# [mem] sample ${loop}: $(print_mem "$pid" "$base_rss" "$base_data")"
    loop=$((loop + 1))
done

echo "# process exited: pid=${pid}"
