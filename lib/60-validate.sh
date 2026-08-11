#!/usr/bin/env bash
#===============================================================================
# easy1090 - final validation
#
# Confirms the pipeline end to end and prints where to look. Read-only.
#
# Author: Esli
# License: MIT
#===============================================================================

validate::run() {
    log::step "$(t val_step)"

    if [[ "$DRY_RUN" == true ]]; then
        log::dry_run "systemctl is-active readsb lighttpd tar1090"
        log::dry_run "curl -s http://localhost/tar1090/data/aircraft.json"
        return 0
    fi

    # Note: failures=$((...)) instead of ((failures++)). Post-increment from
    # zero evaluates to 0, which is a non-zero exit status, and set -e would
    # abort the run on the very first failure it is trying to count.
    local failures=0

    validate::service readsb || failures=$((failures + 1))
    [[ "$COMPONENT_TAR1090" == true ]] && { validate::service lighttpd || failures=$((failures + 1)); }
    [[ "$COMPONENT_TAR1090" == true ]] && { validate::service tar1090 || failures=$((failures + 1)); }

    validate::decoding || failures=$((failures + 1))
    [[ "$COMPONENT_TAR1090" == true ]] && { validate::web || failures=$((failures + 1)); }

    validate::summary "$failures"
    return "$failures"
}

validate::service() {
    local unit="$1"

    if ! systemctl list-unit-files "${unit}.service" &>/dev/null; then
        log::error "$(t val_unit_missing "$unit")"
        return 1
    fi

    local active enabled
    active="$(systemctl is-active "$unit" 2>/dev/null || true)"
    enabled="$(systemctl is-enabled "$unit" 2>/dev/null || true)"

    if [[ "$active" == "active" && "$enabled" == "enabled" ]]; then
        log::success "$(t val_service_ok "$unit")"
        return 0
    fi

    log::error "$(t val_service_bad "$unit" "${active:-?}" "${enabled:-?}")"
    return 1
}

# An empty sky is not a failure. What proves the decoder is alive is the JSON
# being refreshed, so we check the file's own clock instead of aircraft count.
validate::decoding() {
    [[ -f "$READSB_JSON" ]] || {
        log::error "$(t val_json_missing "$READSB_JSON")"
        return 1
    }

    local now aircraft age
    now="$(jq -r '.now // 0' "$READSB_JSON" 2>/dev/null || printf '0')"
    aircraft="$(jq -r '.aircraft | length' "$READSB_JSON" 2>/dev/null || printf '0')"
    age="$(awk -v n="$now" 'BEGIN { printf "%d", systime() - n }')"

    if [[ "$age" -gt 60 ]]; then
        log::error "$(t val_json_stale "$age")"
        return 1
    fi

    log::success "$(t val_decoding_ok "$age" "$aircraft")"

    [[ "$aircraft" -eq 0 ]] && log::info "$(t val_zero_aircraft)"

    return 0
}

validate::web() {
    local code
    code="$(curl -s -o /dev/null -w '%{http_code}' "http://localhost/tar1090/" || true)"

    if [[ "$code" != "200" ]]; then
        log::error "$(t val_web_bad "${code:-?}")"
        return 1
    fi

    if curl -sf "http://localhost/tar1090/data/aircraft.json" -o /dev/null; then
        log::success "$(t val_web_ok)"
        return 0
    fi

    log::error "$(t val_web_data_bad)"
    return 1
}

validate::summary() {
    local failures="$1"
    local ip

    ip="$(ip route get 1.1.1.1 2>/dev/null | grep -o 'src [0-9.]*' | cut -d' ' -f2)"

    printf "\n"
    if [[ "$failures" -eq 0 ]]; then
        log::success "$(t val_all_ok)"
    else
        log::error "$(t val_failures "$failures")"
    fi

    printf "\n${BOLD}%s${RESET}\n" "$(t val_howto)" >&2
    printf "  %-32s %s\n" "viewadsb" "$(t val_howto_viewadsb)" >&2
    printf "  %-32s %s\n" "nc localhost $NET_SBS_PORT" "$(t val_howto_nc)" >&2
    printf "  %-32s\n" "jq . $READSB_JSON" >&2
    [[ "$COMPONENT_TAR1090" == true ]] &&
        printf "  %-32s %s\n" "http://${ip:-localhost}/tar1090/" "$(t val_howto_map)" >&2
    printf "\n" >&2
}
