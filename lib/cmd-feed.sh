#!/usr/bin/env bash
#===============================================================================
# easy1090 - ADSBExchange feed
#
# Two separate things, which the ADSBExchange documentation tends to blur:
#
#   Feeding is just a --net-connector in readsb. That is all it takes for your
#   data to reach them, and easy1090 already handles it through the config.
#
#   The stats package is optional and separate. It generates a feeder UUID and
#   pushes receiver statistics, which is what makes your feeder appear on their
#   site and lets you claim it under an account.
#
# Their installer does not run on Arch. It calls `adduser` with no fallback,
# and with `set -e` the script dies on that line before installing anything.
# We create the system user first, so their `id -u` check passes and the
# adduser branch is skipped, then hand the rest to their script unmodified.
# Same principle as the tar1090 module: work around it, never patch it.
#
# Author: Esli
# License: MIT
#===============================================================================

# -g: sourced from inside load_modules().
declare -g FEED_ACTION="enable"

readonly FEED_HOST="feed.adsbexchange.com"
readonly FEED_STATS_REPO="https://github.com/adsbexchange/adsbexchange-stats"
readonly FEED_STATS_CLONE="${HOME}/.cache/easy1090/adsbexchange-stats"
readonly FEED_STATS_UNIT="adsbexchange-stats"
readonly FEED_STATS_USER="adsbexchange"
readonly FEED_STATS_PATH="/usr/local/share/adsbexchange-stats"
readonly FEED_STATS_DEFAULTS="/etc/default/adsbexchange-stats"
# create-uuid.sh writes to one of these, depending on how the host was set up.
readonly FEED_UUID_FILES=(
    /boot/adsbx-uuid
    /usr/local/share/adsbexchange/adsbx-uuid
)

cmd::feed() {
    feed::parse_args "$@" || return 1

    cfg::load "$CONFIG_FILE" "$CONFIG_EXAMPLE"

    case "$FEED_ACTION" in
    status)
        feed::show_status
        return 0
        ;;
    stats)
        trap 'sudo::cleanup' EXIT
        sudo::init
        feed::install_stats
        feed::show_info
        ;;
    disable)
        trap 'sudo::cleanup' EXIT
        sudo::init
        feed::set_enabled false
        ;;
    enable)
        trap 'sudo::cleanup' EXIT
        sudo::init
        feed::set_enabled true
        feed::install_stats
        feed::show_info
        ;;
    esac
}

feed::parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
        --status) FEED_ACTION="status" ;;
        --disable) FEED_ACTION="disable" ;;
        --stats) FEED_ACTION="stats" ;;
        -h | --help)
            t feed_usage "$EASY1090_VERSION"
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

#===============================================================================
# FEEDING
#===============================================================================

# Flips the config key and reconverges readsb, which is what actually puts the
# --net-connector on the running process.
feed::set_enabled() {
    local wanted="$1"

    log::step "$(t feed_step_cfg)"

    if [[ "$FEEDER_ADSBEXCHANGE" == "$wanted" ]]; then
        [[ "$wanted" == true ]] && log::skip "$(t feed_already_on)" || log::skip "$(t feed_already_off)"
    else
        [[ "$wanted" == true ]] && log::warn "$(t feed_enabling)" || log::info "$(t feed_disabling)"
        FEEDER_ADSBEXCHANGE="$wanted"
        [[ "$DRY_RUN" == true ]] ||
            cfg::_persist "$CONFIG_FILE" FEEDER_ADSBEXCHANGE "$wanted"
        cfg::_persist_dry "FEEDER_ADSBEXCHANGE" "$wanted"
    fi

    # Rewrites /etc/default/readsb and restarts only if the content changed.
    readsb::configure
    readsb::enable
}

# Keeps the dry-run honest about the config file it would touch.
cfg::_persist_dry() {
    [[ "$DRY_RUN" == true ]] || return 0
    log::dry_run "sed -i 's|^$1=.*|$1=\"$2\"|' $CONFIG_FILE"
}

#===============================================================================
# STATS PACKAGE
#===============================================================================

feed::stats_installed() {
    systemctl list-unit-files "${FEED_STATS_UNIT}.service" &>/dev/null
}

feed::install_stats() {
    log::step "$(t feed_step_stats)"

    if feed::stats_installed && [[ "$FEED_ACTION" != "stats" ]]; then
        log::skip "$(t feed_stats_present)"
        return 0
    fi

    printf '\n%s\n%s\n\n%s\n\n' \
        "$(t feed_stats_intro)" \
        "$(t feed_stats_repo "$FEED_STATS_REPO")" \
        "$(t feed_stats_note)" >&2

    util::confirm "$(t feed_stats_confirm)" || {
        log::info "$(t feed_stats_skipped)"
        return 0
    }

    feed::stats_dependencies
    feed::stats_user
    feed::stats_run
}

# Their script installs these with apt or yum only, so on Arch it silently
# installs nothing and the service misbehaves later. `host` comes from bind.
feed::stats_dependencies() {
    local -a needed=(curl jq gzip perl bind)

    log::info "$(t feed_stats_deps "${needed[*]}")"
    pkg::install "${needed[@]}"
}

feed::stats_user() {
    if id -u "$FEED_STATS_USER" &>/dev/null; then
        log::skip "$(t feed_stats_user_ok)"
        return 0
    fi

    log::info "$(t feed_stats_user)"
    run::sudo useradd --system --home-dir "$FEED_STATS_PATH" --no-create-home \
        --shell /usr/bin/nologin "$FEED_STATS_USER"
}

# Skips their stats.sh bootstrap, which only exists to apt-get install git and
# then clone this repository. Cloning it ourselves keeps apt out of the picture
# and makes the code visible before it runs.
feed::stats_run() {
    [[ -d "$FEED_STATS_CLONE" ]] && run::cmd rm -rf "$FEED_STATS_CLONE"
    run::cmd mkdir -p "$(dirname "$FEED_STATS_CLONE")"

    log::info "$(t feed_stats_cloning)"
    run::cmd git clone --depth 1 "$FEED_STATS_REPO" "$FEED_STATS_CLONE"

    log::info "$(t feed_stats_running)"
    if [[ "$DRY_RUN" == true ]]; then
        log::dry_run "cd $FEED_STATS_CLONE && sudo bash install.sh"
    else
        (cd "$FEED_STATS_CLONE" && sudo bash install.sh) || {
            log::error "$(t feed_stats_failed)"
            return 1
        }

        systemctl is-active --quiet "$FEED_STATS_UNIT" &&
            log::success "$(t feed_stats_ok)"
    fi

    # Runs in dry-run too, so the preview shows every step the real run takes.
    feed::stats_datasource
    return 0
}

# Their json-status only looks at /run/adsbexchange-feed, which is created by
# the ADSBExchange feed package. We feed straight from readsb, so our JSON is
# in /run/readsb and the service loops on "No valid data source directory".
#
# The escape hatch is theirs: USE_OLD_PATH=1 makes it try /run/readsb first.
# Their installer only writes it when it detects a Raspberry Pi image, which
# is why it never lands on a normal machine.
feed::stats_datasource() {
    local content="# easy1090: look at /run/readsb, where our readsb writes its JSON.
# Without this, json-status only checks /run/adsbexchange-feed and reports
# \"No valid data source directory found\" forever.
USE_OLD_PATH=1"

    if [[ -f "$FEED_STATS_DEFAULTS" ]] && grep -q "^USE_OLD_PATH=1" "$FEED_STATS_DEFAULTS"; then
        log::skip "$(t feed_datasource_ok)"
        return 0
    fi

    log::info "$(t feed_datasource_fix)"
    run::sudo_write "$FEED_STATS_DEFAULTS" "$content"

    log::info "$(t feed_datasource_restart)"
    run::sudo systemctl restart "$FEED_STATS_UNIT"

    [[ "$DRY_RUN" == true ]] && return 0

    run::cmd sleep 5
    if journalctl -u "$FEED_STATS_UNIT" --since "-1min" --no-pager 2>/dev/null |
        grep -q "Using JSON directory"; then
        log::success "$(t feed_datasource_working)"
    else
        log::warn "$(t feed_datasource_wait)"
    fi
    return 0
}

#===============================================================================
# INFORMATION
#===============================================================================

feed::uuid() {
    local f
    for f in "${FEED_UUID_FILES[@]}"; do
        [[ -r "$f" ]] && {
            tr -d '[:space:]' <"$f"
            return 0
        }
    done
    return 1
}

feed::show_info() {
    log::step "$(t feed_step_info)"

    local uuid
    if uuid="$(feed::uuid)" && [[ -n "$uuid" ]]; then
        log::success "$(t feed_uuid "$uuid")"
        printf '\n%s\n%s\n%s\n\n' \
            "$(t feed_url_stats "https://www.adsbexchange.com/api/feeders/?feed=${uuid}")" \
            "$(t feed_url_myip)" \
            "$(t feed_url_account)" >&2
    else
        log::warn "$(t feed_uuid_missing)"
        printf '\n%s\n\n' "$(t feed_url_myip)" >&2
    fi

    feed::privacy_note
}

feed::privacy_note() {
    local human
    case "${JSON_LOCATION_ACCURACY:-2}" in
    0) human="$(t feed_privacy_none)" ;;
    1) human="$(t feed_privacy_approx)" ;;
    *) human="$(t feed_privacy_exact)" ;;
    esac

    log::info "$(t feed_privacy "${JSON_LOCATION_ACCURACY:-2}" "$human")"
}

# Read-only: no sudo, no changes.
feed::show_status() {
    log::step "$(t feed_step_cfg)"

    if [[ "$FEEDER_ADSBEXCHANGE" == true ]]; then
        log::success "$(t feed_already_on)"
    else
        log::info "$(t feed_already_off)"
    fi

    local peer
    # Column 4 is the peer, so this only matches connections we opened out to
    # someone else's 30005. A LAN client attached to our own beast port shows
    # up with an ephemeral peer port and is correctly ignored.
    peer="$(ss -tn state established 2>/dev/null | awk '$4 ~ /:30005$/ {print $4}' | head -1)"
    if [[ -n "$peer" ]]; then
        log::success "$(t feed_connected "$peer")"
    else
        log::warn "$(t feed_not_connected)"
    fi

    log::step "$(t feed_step_stats)"
    if feed::stats_installed; then
        log::success "$(t feed_stats_present)"
    else
        log::warn "$(t feed_stats_skipped)"
    fi

    feed::show_info
}
