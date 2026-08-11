#!/usr/bin/env bash
#===============================================================================
# easy1090 - uninstaller
#
# Best effort reversal of what install.sh did, and nothing else. It removes the
# services, configs and packages easy1090 installed, and deliberately leaves
# alone anything shared with the rest of the system.
#
# Author: Esli
# License: MIT
# Repository: https://github.com/Esl1h/easy1090
#===============================================================================

set -euo pipefail

readonly EASY1090_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly CONFIG_FILE="${EASY1090_ROOT}/install.conf"

# shellcheck source=lib/common.sh
source "${EASY1090_ROOT}/lib/common.sh"
# shellcheck source=lib/i18n.sh
source "${EASY1090_ROOT}/lib/i18n.sh"
# shellcheck source=lib/pkg-arch.sh
source "${EASY1090_ROOT}/lib/pkg-arch.sh"

readonly TAR1090_PATH="/usr/local/share/tar1090"
readonly TAR1090_UNINSTALL="${TAR1090_PATH}/uninstall.sh"
readonly TAR1090_DEFAULTS="/etc/default/tar1090"
readonly LIGHTTPD_CONF="/etc/lighttpd/lighttpd.conf"
readonly MOD_REDIRECT_AVAILABLE="/etc/lighttpd/conf-available/06-mod_redirect.conf"
readonly MOD_REDIRECT_ENABLED="/etc/lighttpd/conf-enabled/06-mod_redirect.conf"
readonly READSB_DEFAULTS="/etc/default/readsb"
readonly READSB_UDEV_RULE="/etc/udev/rules.d/99-readsb-rtlsdr.rules"
readonly DVB_BLACKLIST="/etc/modprobe.d/blacklist-rtlsdr.conf"
readonly BUILD_CACHE="${HOME}/.cache/easy1090"

declare KEEP_PACKAGES=false CLI_LANG=""
declare HELP_REQUESTED=false UNKNOWN_OPT=""

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

    if [[ -d "$BUILD_CACHE" ]]; then
        run::cmd rm -rf "$BUILD_CACHE"
        [[ "$DRY_RUN" == true ]] || log::success "$(t un_removed "$BUILD_CACHE")"
    else
        log::skip "$(t un_absent "$BUILD_CACHE")"
    fi

    # The config holds the user's coordinates, so it is never removed silently.
    if [[ -f "$CONFIG_FILE" ]] && util::confirm "$(t un_config_ask)"; then
        run::cmd rm -f "$CONFIG_FILE"
        [[ "$DRY_RUN" == true ]] || log::success "$(t un_removed "$CONFIG_FILE")"
    fi
}

#===============================================================================
# MAIN
#===============================================================================

usage() {
    t un_usage "$EASY1090_VERSION"
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
        --keep-packages) KEEP_PACKAGES=true ;;
        --lang)
            CLI_LANG="${2:-}"
            shift
            ;;
        --dry-run) DRY_RUN=true ;;
        --yes | -y) ASSUME_YES=true ;;
        --verbose) LOG_LEVEL=3 ;;
        -h | --help) HELP_REQUESTED=true ;;
        *) UNKNOWN_OPT="$1" ;;
        esac
        shift
    done
}

plan() {
    printf "\n${BOLD}%s${RESET}\n" "$(t un_plan)" >&2
    printf '%s\n%s\n%s\n%s\n%s\n\n' \
        "$(t un_plan_driver)" \
        "$(t un_plan_readsb)" \
        "$(t un_plan_tar1090)" \
        "$(t un_plan_optional)" \
        "$(t un_plan_local)" >&2
    printf '%s\n\n' "$(t un_keep)" >&2
}

main() {
    parse_args "$@"
    i18n::init "$CONFIG_FILE" "$CLI_LANG"

    if [[ -n "$UNKNOWN_OPT" ]]; then
        log::error "$(t cli_unknown_opt "$UNKNOWN_OPT")"
        usage
        exit 1
    fi

    if [[ "$HELP_REQUESTED" == true ]]; then
        usage
        exit 0
    fi

    printf "\n${BOLD}easy1090${RESET} %s - %s\n" "$EASY1090_VERSION" "$(t un_title)" >&2
    [[ "$DRY_RUN" == true ]] && log::warn "$(t cli_dry_warning)"

    plan
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

main "$@"
