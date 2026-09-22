#!/usr/bin/env bash
#===============================================================================
# easy1090 - install command
#
# Idempotent: re-running is how you update. Each module detects what is already
# in place and skips it.
#
# Author: Esli
# License: MIT
#===============================================================================

cmd::install() {
    local CLI_LAT="" CLI_LON=""
    local CLI_FULL=false CLI_SKIP_TAR1090=false
    local CLI_SKIP_SDRPP=false CLI_SKIP_SATDUMP=false

    install::parse_args "$@" || return 1

    preflight::run

    cfg::load "$CONFIG_FILE" "$CONFIG_EXAMPLE"
    install::apply_overrides
    cfg::require_position "$CONFIG_FILE"
    # Remember CLI coordinates so the next run does not ask again.
    [[ -n "$CLI_LAT" ]] && cfg::_persist "$CONFIG_FILE" RECEIVER_LAT "$CLI_LAT"
    [[ -n "$CLI_LON" ]] && cfg::_persist "$CONFIG_FILE" RECEIVER_LON "$CLI_LON"
    cfg::require_feeder "$CONFIG_FILE"

    trap 'sudo::cleanup' EXIT
    sudo::init

    driver::run
    readsb::run
    [[ "$COMPONENT_TAR1090" == true ]] && tar1090::run
    optional::run

    validate::run
}

install::parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
        --full) CLI_FULL=true ;;
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
        -h | --help)
            t cli_usage "$EASY1090_VERSION"
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

# CLI wins over install.conf, and coordinates given here are remembered in
# install.conf, exactly like the ones asked on the first run.
install::apply_overrides() {
    if [[ -n "$CLI_LAT" ]]; then
        cfg::validate_position "$CLI_LAT" "${CLI_LON:-0.0}"
        RECEIVER_LAT="$CLI_LAT"
    fi
    if [[ -n "$CLI_LON" ]]; then
        cfg::validate_position "${CLI_LAT:-0.0}" "$CLI_LON"
        RECEIVER_LON="$CLI_LON"
    fi

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
