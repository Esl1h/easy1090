#!/usr/bin/env bash
#===============================================================================
# easy1090 - update command
#
# Versions only. Configuration and services are `install`'s job, and the split
# is deliberate: updating should never silently start a 45 minute rebuild as a
# side effect of converging a config file.
#
# There are two update paths because there are two install paths:
#
#   yay packages (driver, SDR++, SatDump) need `yay -Syu --devel`. A plain
#   `yay -Syu` never updates a -git package, because it compares against the
#   version declared in the AUR, which does not move when upstream commits.
#
#   readsb is built with makepkg outside yay, so that a prepare() patch stays
#   possible when a new GCC breaks the build. The price is that yay never
#   records it in its VCS database and will not offer an update for it, even
#   with --devel. So we compare commits ourselves.
#
# Author: Esli
# License: MIT
#===============================================================================

# -g because these files are sourced from inside load_modules(): a plain
# declare would create a function-local that vanishes when it returns.
declare -g UPDATE_SKIP_AUR=false UPDATE_SKIP_READSB=false
declare -g UPDATE_READSB_CHANGED=false

cmd::update() {
    update::parse_args "$@" || return 1

    trap 'sudo::cleanup' EXIT
    sudo::init

    [[ "$UPDATE_SKIP_AUR" == false ]] && update::aur
    [[ "$UPDATE_SKIP_READSB" == false ]] && update::readsb
    update::services

    printf '\n' >&2
    log::success "$(t upd_done)"
    log::info "$(t upd_hint_install)"
}

update::parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
        --skip-aur) UPDATE_SKIP_AUR=true ;;
        --skip-readsb) UPDATE_SKIP_READSB=true ;;
        -h | --help)
            t upd_usage "$EASY1090_VERSION"
            exit 0
            ;;
        *)
            log::error "$(t cli_unknown_opt "$1")"
            return 1
            ;;
        esac
        shift
    done
    return 0
}

update::aur() {
    log::step "$(t upd_step_aur)"
    log::info "$(t upd_yay_note)"

    # Runs as the normal user; yay escalates on its own when it reaches pacman.
    run::cmd yay -Syu --devel --noconfirm
}

#===============================================================================
# readsb
#===============================================================================

# The pkgver of a VCS package ends in .g<short-sha>, which is the commit it was
# built from. That is the only reliable record of what is actually installed.
update::readsb_installed_commit() {
    pacman -Q "$READSB_PACKAGE" 2>/dev/null |
        awk '{print $2}' |
        sed -n 's/.*\.g\([0-9a-f]\{7,\}\)-.*/\1/p'
}

update::readsb_upstream_commit() {
    local sha
    sha="$(timeout 30 git ls-remote "$READSB_UPSTREAM" HEAD 2>/dev/null | cut -f1)"
    [[ -n "$sha" ]] || return 1
    printf '%s' "${sha:0:7}"
}

update::readsb() {
    log::step "$(t upd_step_readsb)"

    if ! pkg::is_installed "$READSB_PACKAGE"; then
        log::warn "$(t upd_not_installed "$READSB_PACKAGE")"
        return 0
    fi

    local installed upstream
    installed="$(update::readsb_installed_commit)"
    upstream="$(update::readsb_upstream_commit)" || upstream=""

    if [[ -z "$installed" || -z "$upstream" ]]; then
        log::warn "$(t upd_readsb_unknown)"
        return 0
    fi

    log::info "$(t upd_readsb_installed "$installed")"
    log::info "$(t upd_readsb_upstream "$upstream")"

    # Compared by prefix: git describe may use more than seven characters when
    # a short sha would be ambiguous.
    if [[ "$installed" == "$upstream"* || "$upstream" == "$installed"* ]]; then
        log::skip "$(t upd_readsb_current)"
        return 0
    fi

    log::info "$(t upd_readsb_behind)"
    util::confirm "$(t upd_readsb_confirm)" || {
        log::info "$(t upd_readsb_skipped)"
        return 0
    }

    pkg::build_aur_isolated "$READSB_PACKAGE" "$READSB_BUILD_DIR"
    pkg::makepkg_install "$READSB_BUILD_DIR"
    UPDATE_READSB_CHANGED=true

    [[ "$DRY_RUN" == true ]] || log::success "$(t upd_readsb_rebuilt "$(update::readsb_installed_commit)")"
}

#===============================================================================
# Services
#===============================================================================

# A new binary is only in use after a restart. Config is untouched here, so the
# only reason to restart is a package that actually changed underneath.
update::services() {
    log::step "$(t upd_step_services)"

    local -a units=()
    [[ "$UPDATE_READSB_CHANGED" == true ]] && units+=(readsb)

    # An AUR update can replace the lighttpd or tar1090 units and their files.
    # svc::predates_file answers the only question that matters: is the running
    # process older than what is on disk?
    svc::predates_file lighttpd /usr/lib/systemd/system/lighttpd.service && units+=(lighttpd)
    svc::predates_file tar1090 /usr/lib/systemd/system/tar1090.service && units+=(tar1090)

    if [[ ${#units[@]} -eq 0 ]]; then
        log::skip "$(t upd_services_ok)"
        return 0
    fi

    run::sudo systemctl daemon-reload

    local unit
    for unit in "${units[@]}"; do
        log::info "$(t upd_services_restart "$unit")"
        run::sudo systemctl restart "$unit" || true
    done
}
