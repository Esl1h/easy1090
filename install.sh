#!/usr/bin/env bash
#===============================================================================
# easy1090 - one command ADS-B stack installer
#
# On a clean Arch box with an RTL-SDR plugged in, leaves this running:
#   driver (RTL-SDR Blog fork) -> readsb (wiedehopf fork, JSON) -> tar1090
#
# Idempotent: re-running is how you update. Each module detects what is already
# in place and skips it.
#
# Author: Esli
# License: MIT
# Repository: https://github.com/Esl1h/easy1090
#===============================================================================

set -euo pipefail

readonly SCRIPT_NAME="${0##*/}"
readonly EASY1090_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly CONFIG_FILE="${EASY1090_ROOT}/install.conf"
readonly CONFIG_EXAMPLE="${EASY1090_ROOT}/install.conf.example"

# shellcheck source=lib/common.sh
source "${EASY1090_ROOT}/lib/common.sh"
# shellcheck source=lib/i18n.sh
source "${EASY1090_ROOT}/lib/i18n.sh"

#===============================================================================
# ARGUMENT PARSING
#===============================================================================

declare CLI_LAT="" CLI_LON="" CLI_LANG=""
declare CLI_SKIP_TAR1090=false CLI_FULL=false
declare CLI_SKIP_SDRPP=false CLI_SKIP_SATDUMP=false

usage() {
    t cli_usage "$EASY1090_VERSION"
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
        --full) CLI_FULL=true ;;
        --lang)
            CLI_LANG="${2:-}"
            shift
            ;;
        --lat)
            CLI_LAT="${2:-}"
            shift
            ;;
        --lon)
            CLI_LON="${2:-}"
            shift
            ;;
        --skip-tar1090) CLI_SKIP_TAR1090=true ;;
        --skip-sdrpp) CLI_SKIP_SDRPP=true ;;
        --skip-satdump) CLI_SKIP_SATDUMP=true ;;
        --dry-run) DRY_RUN=true ;;
        --yes | -y) ASSUME_YES=true ;;
        --verbose) LOG_LEVEL=3 ;;
        --version)
            printf 'easy1090 %s\n' "$EASY1090_VERSION"
            exit 0
            ;;
        -h | --help)
            HELP_REQUESTED=true
            ;;
        *)
            UNKNOWN_OPT="$1"
            ;;
        esac
        shift
    done
}

# CLI wins over install.conf.
apply_overrides() {
    [[ -n "$CLI_LAT" ]] && RECEIVER_LAT="$CLI_LAT"
    [[ -n "$CLI_LON" ]] && RECEIVER_LON="$CLI_LON"

    if [[ "$CLI_FULL" == true ]]; then
        COMPONENT_TAR1090=true
        COMPONENT_SDRPP=true
        COMPONENT_SATDUMP=true
    fi

    [[ "$CLI_SKIP_TAR1090" == true ]] && COMPONENT_TAR1090=false
    [[ "$CLI_SKIP_SDRPP" == true ]] && COMPONENT_SDRPP=false
    [[ "$CLI_SKIP_SATDUMP" == true ]] && COMPONENT_SATDUMP=false

    return 0
}

#===============================================================================
# MAIN
#===============================================================================

load_modules() {
    local module
    for module in \
        "${EASY1090_ROOT}/lib/pkg-${PKG_TARGET:-arch}.sh" \
        "${EASY1090_ROOT}"/lib/[0-9][0-9]-*.sh; do
        # shellcheck source=/dev/null
        source "$module"
    done
}

banner() {
    printf "\n${BOLD}easy1090${RESET} %s\n" "$EASY1090_VERSION" >&2
    [[ "$DRY_RUN" == true ]] && log::warn "$(t cli_dry_warning)"
    return 0
}

main() {
    declare HELP_REQUESTED=false UNKNOWN_OPT=""

    parse_args "$@"

    # Language first: every message below, including errors, goes through it.
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

    load_modules
    banner

    preflight::run

    cfg::load "$CONFIG_FILE" "$CONFIG_EXAMPLE"
    apply_overrides
    cfg::require_position "$CONFIG_FILE"
    cfg::require_feeder "$CONFIG_FILE"

    trap 'sudo::cleanup' EXIT
    sudo::init

    driver::run
    readsb::run
    [[ "$COMPONENT_TAR1090" == true ]] && tar1090::run
    optional::run

    validate::run
}

main "$@"
