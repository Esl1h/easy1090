#!/usr/bin/env bash
#===============================================================================
# easy1090 - open a component
#
# Runs things in the current terminal, not in a spawned window. This stack
# usually lives on a headless box reached over SSH, where "open a new terminal"
# has no meaning. Only the map and the two GUIs need a graphical session, and
# those check for one before trying.
#
# No sudo anywhere in here.
#
# Author: Esli
# License: MIT
#===============================================================================

cmd::open() {
    local target="${1:-}"

    [[ -z "$target" ]] && {
        open::list
        return 0
    }

    case "$target" in
    viewadsb | view) open::exec viewadsb ;;
    sbs | raw | nc) open::sbs ;;
    map | tar1090 | web) open::map ;;
    sdrpp | sdr) open::gui sdrpp ;;
    satdump) open::gui satdump-ui ;;
    *)
        log::error "$(t open_unknown "$target")"
        open::list
        return 1
        ;;
    esac
}

open::list() {
    printf "\n${BOLD}%s${RESET}\n" "$(t open_targets)" >&2
    printf '%s\n%s\n%s\n%s\n%s\n\n' \
        "$(t open_t_viewadsb)" \
        "$(t open_t_sbs)" \
        "$(t open_t_map)" \
        "$(t open_t_sdrpp)" \
        "$(t open_t_satdump)" >&2
}

# exec replaces this shell, so Ctrl+C behaves exactly as if you had typed the
# command yourself.
open::exec() {
    local cmd="$1"
    shift

    util::have_cmd "$cmd" || {
        log::error "$(t open_missing "$cmd")"
        return 1
    }

    log::info "$(t open_running "$cmd $*")"

    if [[ "$DRY_RUN" == true ]]; then
        log::dry_run "$(run::_render "$cmd" "$@")"
        return 0
    fi

    exec "$cmd" "$@"
}

open::sbs() {
    local port="${NET_SBS_PORT:-30003}"

    util::have_cmd nc || {
        log::error "$(t open_missing nc)"
        return 1
    }

    open::exec nc localhost "$port"
}

open::map() {
    local ip url
    ip="$(ip route get 1.1.1.1 2>/dev/null | grep -o 'src [0-9.]*' | cut -d' ' -f2)"
    url="http://${ip:-localhost}/tar1090/"

    log::info "$(t open_url "$url")"

    if ! open::has_display; then
        # On a headless box printing the URL is the useful outcome, not an error.
        return 0
    fi

    util::have_cmd xdg-open || return 0
    run::cmd xdg-open "$url"
}

open::gui() {
    local cmd="$1"

    if ! open::has_display; then
        log::error "$(t open_no_display "$cmd")"
        return 1
    fi

    open::exec "$cmd"
}

open::has_display() {
    [[ -n "${DISPLAY:-}" || -n "${WAYLAND_DISPLAY:-}" ]]
}
