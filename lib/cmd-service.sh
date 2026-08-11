#!/usr/bin/env bash
#===============================================================================
# easy1090 - start / stop / restart
#
# Drives the three units as a group, in dependency order, so you do not have to
# remember that tar1090 exists on top of readsb and lighttpd. Individual units
# are still accepted for the odd case.
#
# Author: Esli
# License: MIT
#===============================================================================

readonly SERVICE_UNITS=(readsb lighttpd tar1090)

cmd::start() { service::act start "$@"; }
cmd::stop() { service::act stop "$@"; }
cmd::restart() { service::act restart "$@"; }

service::act() {
    local action="$1"
    shift

    local -a units=()
    if [[ $# -gt 0 ]]; then
        units=("$@")
    else
        units=("${SERVICE_UNITS[@]}")
        # Bring the stack down from the top, and up from the bottom.
        [[ "$action" == "stop" ]] && units=(tar1090 lighttpd readsb)
    fi

    log::step "$(t svc_step)"

    trap 'sudo::cleanup' EXIT
    sudo::init

    local unit
    for unit in "${units[@]}"; do
        if ! systemctl list-unit-files "${unit}.service" &>/dev/null; then
            log::skip "$(t svc_not_installed "$unit")"
            continue
        fi

        log::info "$(t svc_acting "$unit" "$action")"
        run::sudo systemctl "$action" "$unit" || true
    done

    log::success "$(t svc_done)"
}
