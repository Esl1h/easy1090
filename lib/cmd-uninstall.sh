#!/usr/bin/env bash
#===============================================================================
# easy1090 - uninstall command
#
# Best effort reversal of what install did, and nothing else. It removes the
# services, configs and packages easy1090 installed, and deliberately leaves
# alone anything shared with the rest of the system.
#
# Paths come from the install modules, so there is a single source of truth.
#
# Author: Esli
# License: MIT
#===============================================================================

# -g because these files are sourced from inside load_modules(): a plain
# declare would create a function-local that vanishes when it returns.
declare -g KEEP_PACKAGES=false

cmd::uninstall() {
    uninstall::parse_args "$@" || return 1

    uninstall::plan
    util::confirm "$(t un_confirm)" || util::die "$(t un_aborted)"

    trap 'sudo::cleanup' EXIT
    sudo::init

    # Reverse order of the install: web layer first, hardware last.
    uninstall::tar1090
    uninstall::readsb
    uninstall::driver
    uninstall::optional
    uninstall::local_files

    printf '\n' >&2
    log::success "$(t un_done)"
    log::info "$(t un_users_note)"
}

uninstall::parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
        --keep-packages) KEEP_PACKAGES=true ;;
        -h | --help)
            t un_usage "$EASY1090_VERSION"
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

uninstall::plan() {
    printf "\n${BOLD}%s${RESET}\n" "$(t un_plan)" >&2
    printf '%s\n%s\n%s\n%s\n%s\n\n' \
        "$(t un_plan_driver)" \
        "$(t un_plan_readsb)" \
        "$(t un_plan_tar1090)" \
        "$(t un_plan_optional)" \
        "$(t un_plan_local)" >&2
    printf '%s\n\n' "$(t un_keep)" >&2
}

#===============================================================================
# HELPERS
#===============================================================================

uninstall::rm_path() {
    local path="$1"

    if [[ ! -e "$path" ]]; then
        log::skip "$(t un_absent "$path")"
        return 0
    fi

    run::sudo rm -rf "$path"
    [[ "$DRY_RUN" == true ]] || log::success "$(t un_removed "$path")"
}

uninstall::stop_service() {
    local unit="$1"

    systemctl list-unit-files "${unit}.service" &>/dev/null || return 0

    log::info "$(t un_stopping "$unit")"
    run::sudo systemctl disable --now "$unit" || true
}

uninstall::remove_package() {
    local package="$1"

    if [[ "$KEEP_PACKAGES" == true ]]; then
        log::skip "$(t un_pkg_kept)"
        return 0
    fi

    pkg::remove "$package"
}

#===============================================================================
# STEPS
#===============================================================================

# Upstream ships its own uninstaller and knows best which files it created, so
# we call it and only clean up what easy1090 added on top.
uninstall::tar1090() {
    log::step "$(t un_step_tar1090)"

    if [[ -f "$TAR1090_UNINSTALL" ]]; then
        log::info "$(t un_upstream "$TAR1090_UNINSTALL")"
        run::sudo bash "$TAR1090_UNINSTALL" || true
    else
        log::warn "$(t un_upstream_missing)"
        uninstall::stop_service tar1090
        uninstall::rm_path "$TAR1090_PATH"
    fi

    # Left behind by upstream on purpose, and ours to clean.
    uninstall::rm_path "$TAR1090_DEFAULTS"

    # easy1090 additions.
    uninstall::rm_path "$MOD_REDIRECT_ENABLED"
    uninstall::rm_path "$MOD_REDIRECT_AVAILABLE"
    uninstall::lighttpd_include
}

# Removes only the line we appended. The Arch default lighttpd.conf never had
# it, so taking it out restores the shipped state.
uninstall::lighttpd_include() {
    [[ -f "$LIGHTTPD_CONF" ]] || return 0
    grep -q 'include_shell "cat /etc/lighttpd/conf-enabled' "$LIGHTTPD_CONF" || return 0

    run::sudo sed -i '\#include_shell "cat /etc/lighttpd/conf-enabled#d' "$LIGHTTPD_CONF"
    [[ "$DRY_RUN" == true ]] || log::success "$(t un_include_removed)"

    if systemctl is-active --quiet lighttpd 2>/dev/null; then
        log::info "$(t un_lighttpd_restart)"
        run::sudo systemctl restart lighttpd || true
    fi
}

uninstall::readsb() {
    log::step "$(t un_step_readsb)"

    uninstall::stop_service readsb
    uninstall::remove_package readsb-wiedehopf-git
    uninstall::rm_path "$READSB_DEFAULTS"
    uninstall::rm_path "$READSB_UDEV_RULE"

    if [[ "$DRY_RUN" != true ]]; then
        run::sudo udevadm control --reload-rules || true
    else
        run::sudo udevadm control --reload-rules
    fi
}

uninstall::driver() {
    log::step "$(t un_step_driver)"

    uninstall::remove_package rtl-sdr-blog-git
    uninstall::rm_path "$DVB_BLACKLIST"
}

uninstall::optional() {
    log::step "$(t un_step_optional)"

    local found=false

    if pkg::is_installed sdrpp-git; then
        uninstall::remove_package sdrpp-git
        found=true
    fi
    if pkg::is_installed satdump; then
        uninstall::remove_package satdump
        found=true
    fi

    [[ "$found" == false ]] && log::skip "$(t un_nothing)"
    return 0
}

uninstall::local_files() {
    log::step "$(t un_step_local)"

    if [[ -d "$(dirname "$READSB_BUILD_DIR")" ]]; then
        run::cmd rm -rf "$(dirname "$READSB_BUILD_DIR")"
        [[ "$DRY_RUN" == true ]] || log::success "$(t un_removed "$(dirname "$READSB_BUILD_DIR")")"
    else
        log::skip "$(t un_absent "$(dirname "$READSB_BUILD_DIR")")"
    fi

    # The config holds the user's coordinates, so it is never removed silently.
    if [[ -f "$CONFIG_FILE" ]] && util::confirm "$(t un_config_ask)"; then
        run::cmd rm -f "$CONFIG_FILE"
        [[ "$DRY_RUN" == true ]] || log::success "$(t un_removed "$CONFIG_FILE")"
    fi
}
