#!/usr/bin/env bash
#===============================================================================
# easy1090 - status command
#
# Read-only: does not start, stop or change anything, and never asks for sudo.
# Meant for a quick glance at what is missing or what fell over.
#
# Author: Esli
# License: MIT
#===============================================================================

readonly RTLSDR_USB_ID="0bda:2838"

cmd::status() {
    printf "\n${BOLD}easy1090${RESET} %s - %s\n\n" "$EASY1090_VERSION" "$(t sts_title)"

    printf "${BOLD}%s${RESET}\n" "$(t sts_hardware)"
    status::dongle
    status::package "$(t sts_driver)" "rtl-sdr-blog-git"

    printf "\n${BOLD}%s${RESET}\n" "$(t sts_decoding)"
    status::package "readsb" "readsb-wiedehopf-git"
    status::service "$(t sts_service)" "readsb"
    status::decoding

    printf "\n${BOLD}%s${RESET}\n" "$(t sts_web)"
    status::service "lighttpd" "lighttpd"
    status::service "tar1090" "tar1090"
    status::web

    printf "\n${BOLD}%s${RESET}\n" "$(t sts_optional)"
    status::package "SDR++" "sdrpp-git"
    status::package "SatDump" "satdump"

    printf "\n"
}

status::line() {
    local name="$1" state="$2" detail="${3:-}"
    local color="$RESET"

    case "$state" in
    "$(t sts_running)" | "$(t sts_installed)") color="$GREEN" ;;
    "$(t sts_stopped)" | "$(t sts_absent)") color="$RED" ;;
    *) color="$YELLOW" ;;
    esac

    # printf pads by bytes, and accented labels ("serviço", "decodificação")
    # would come out ragged. ${#var} counts characters under a UTF-8 locale, so
    # the padding is computed by hand.
    local pad=$((16 - $(str::width "$name")))
    [[ $pad -lt 1 ]] && pad=1

    printf "  %s%*s${color}%-12s${RESET} %s\n" "$name" "$pad" "" "$state" "$detail"
}

status::dongle() {
    if ! util::have_cmd lsusb; then
        status::line "RTL-SDR" "?" "$(t sts_lsusb_missing)"
        return
    fi

    if lsusb | grep -qi "$RTLSDR_USB_ID"; then
        status::line "RTL-SDR" "$(t sts_installed)" "$(t sts_dongle_found "$RTLSDR_USB_ID")"
    else
        status::line "RTL-SDR" "$(t sts_absent)" "$(t sts_dongle_absent)"
    fi
}

status::package() {
    local label="$1" package="$2"

    if pkg::is_installed "$package"; then
        status::line "$label" "$(t sts_installed)" "$(pacman -Q "$package" 2>/dev/null || printf '%s' "$package")"
    else
        status::line "$label" "$(t sts_absent)" "$package"
    fi
}

status::service() {
    local label="$1" unit="$2"

    if ! systemctl list-unit-files "${unit}.service" &>/dev/null; then
        status::line "$label" "$(t sts_absent)" "$(t sts_unit_absent)"
        return
    fi

    local active enabled
    active="$(systemctl is-active "$unit" 2>/dev/null || true)"
    enabled="$(systemctl is-enabled "$unit" 2>/dev/null || true)"

    if [[ "$active" == "active" ]]; then
        status::line "$label" "$(t sts_running)" "$enabled"
    else
        status::line "$label" "$(t sts_stopped)" "$enabled"
    fi
}

# Freshness of the JSON, not aircraft count: an empty sky is not a fault.
status::decoding() {
    [[ -f "$READSB_JSON" ]] || {
        status::line "$(t sts_decode_row)" "$(t sts_absent)" "$(t sts_json_absent "$READSB_JSON")"
        return
    }

    util::have_cmd jq || {
        status::line "$(t sts_decode_row)" "?" "$(t sts_jq_missing)"
        return
    }

    local now aircraft age
    now="$(jq -r '.now // 0' "$READSB_JSON" 2>/dev/null || printf '0')"
    aircraft="$(jq -r '.aircraft | length' "$READSB_JSON" 2>/dev/null || printf '0')"
    age="$(awk -v n="$now" 'BEGIN { printf "%d", systime() - n }')"

    if [[ "$age" -le 60 ]]; then
        status::line "$(t sts_decode_row)" "$(t sts_running)" "$(t sts_json_fresh "$age" "$aircraft")"
    else
        status::line "$(t sts_decode_row)" "$(t sts_stopped)" "$(t sts_json_stale "$age")"
    fi
}

status::web() {
    if ! systemctl is-active --quiet lighttpd 2>/dev/null; then
        status::line "$(t sts_map)" "$(t sts_stopped)" "$(t sts_lighttpd_down)"
        return
    fi

    local code
    code="$(curl -s -o /dev/null -w '%{http_code}' "http://localhost/tar1090/" 2>/dev/null || true)"

    if [[ "$code" == "200" ]]; then
        local ip
        ip="$(ip route get 1.1.1.1 2>/dev/null | grep -o 'src [0-9.]*' | cut -d' ' -f2)"
        status::line "$(t sts_map)" "$(t sts_running)" "http://${ip:-localhost}/tar1090/"
    else
        status::line "$(t sts_map)" "$(t sts_stopped)" "$(t sts_http "${code:-?}")"
    fi
}
