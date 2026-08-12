#!/usr/bin/env bash
#===============================================================================
# easy1090 - readsb (ADS-B decoder)
#
# Installs the wiedehopf fork, not the Mictronics one. The Mictronics fork
# writes aircraft.pb (binary protobuf) and has no flag for JSON, which every
# web frontend expects. The wiedehopf fork writes aircraft.json natively, is
# actively maintained, and is from the same author as tar1090.
#
# Author: Esli
# License: MIT
#===============================================================================

readonly READSB_PACKAGE="readsb-wiedehopf-git"
readonly READSB_CONFLICTS="readsb-git"
# Used by the update command to compare the installed commit with HEAD. The
# package name above already pins which fork this is.
readonly READSB_UPSTREAM="https://github.com/wiedehopf/readsb"
readonly READSB_BUILD_DIR="${HOME}/.cache/easy1090/readsb-wiedehopf-git"
readonly READSB_DEFAULTS="/etc/default/readsb"
readonly READSB_UDEV_RULE="/etc/udev/rules.d/99-readsb-rtlsdr.rules"
readonly READSB_LEGACY_OVERRIDE="/etc/systemd/system/readsb.service.d/override.conf"
readonly READSB_JSON="/run/readsb/aircraft.json"

readsb::run() {
    log::step "$(t rsb_step)"

    readsb::install
    readsb::drop_legacy_override
    readsb::udev_rule
    readsb::configure
    readsb::enable
}

readsb::is_done() {
    pkg::is_installed "$READSB_PACKAGE"
}

readsb::install() {
    if readsb::is_done; then
        log::skip "$(t rsb_installed "$READSB_PACKAGE")"
        return 0
    fi

    if pkg::is_installed "$READSB_CONFLICTS"; then
        log::warn "$(t rsb_conflict "$READSB_CONFLICTS" "$READSB_PACKAGE")"
        util::confirm "$(t rsb_conflict_confirm "$READSB_CONFLICTS")" ||
            util::die "$(t rsb_conflict_abort)"
        pkg::remove "$READSB_CONFLICTS"
    fi

    pkg::build_aur_isolated "$READSB_PACKAGE" "$READSB_BUILD_DIR"
    pkg::makepkg_install "$READSB_BUILD_DIR"
}

# Part 1 of the blog series created a systemd override referencing
# $USER_OPTIONS, a variable this package does not define. Left behind it breaks
# the new service at startup.
readsb::drop_legacy_override() {
    [[ -f "$READSB_LEGACY_OVERRIDE" ]] || return 0

    if grep -q 'USER_OPTIONS' "$READSB_LEGACY_OVERRIDE" 2>/dev/null; then
        log::warn "$(t rsb_legacy_override)"
        run::sudo rm -f "$READSB_LEGACY_OVERRIDE"
        run::sudo rmdir "$(dirname "$READSB_LEGACY_OVERRIDE")" 2>/dev/null || true
        run::sudo systemctl daemon-reload
        log::success "$(t rsb_legacy_removed)"
    fi
}

# The stock rtl-sdr rule grants the device to the plugdev group. That is enough
# for an interactive user (systemd-logind adds a session ACL on top), which is
# exactly why the problem hides: the readsb service user has no session and no
# ACL, so it still gets EACCES. Hence a dedicated rule for the service group.
readsb::udev_rule() {
    local rule="SUBSYSTEM==\"usb\", ATTRS{idVendor}==\"${RTLSDR_USB_VENDOR}\", ATTRS{idProduct}==\"${RTLSDR_USB_PRODUCT}\", GROUP=\"readsb\", MODE=\"0660\""

    if [[ -f "$READSB_UDEV_RULE" ]] && grep -q 'GROUP="readsb"' "$READSB_UDEV_RULE"; then
        log::skip "$(t rsb_udev_ok)"
        return 0
    fi

    log::info "$(t rsb_udev_create)"
    run::sudo_write "$READSB_UDEV_RULE" "$(printf '%b\n%s' "$(t rsb_udev_comment)" "$rule")"

    run::sudo udevadm control --reload-rules
    # Reprocesses already connected devices, so no need to unplug the dongle.
    run::sudo udevadm trigger
}

readsb::configure() {
    local receiver_options decoder_options net_options json_options net_connectors=""

    receiver_options="--device 0 --device-type rtlsdr --gain ${RECEIVER_GAIN} --ppm ${RECEIVER_PPM}"
    receiver_options+=" --lat ${RECEIVER_LAT} --lon ${RECEIVER_LON}"

    decoder_options="--max-range ${DECODER_MAX_RANGE} --write-json-every 1"

    net_options="--net --net-ri-port ${NET_RI_PORT} --net-ro-port ${NET_RO_PORT}"
    net_options+=" --net-sbs-port ${NET_SBS_PORT} --net-bi-port ${NET_BI_PORT}"
    net_options+=" --net-bo-port ${NET_BO_PORT}"

    # Feeders are opt-in. Sharing your receiver position with a third party is
    # a decision, not a default.
    if [[ "$FEEDER_ADSBEXCHANGE" == true ]]; then
        log::warn "$(t feed_enabled "ADSBExchange")"
        net_connectors+=" --net-connector feed.adsbexchange.com,30005,beast_reduce_out"
    fi
    if [[ "$FEEDER_AIRPLANESLIVE" == true ]]; then
        log::warn "$(t feed_enabled "airplanes.live")"
        # Same connector string their own installer writes, including the
        # failover endpoint on 64004.
        net_connectors+=" --net-connector feed.airplanes.live,30004,beast_reduce_plus_out,feed.airplanes.live,64004"
    fi
    [[ -n "$net_connectors" ]] && net_options+="$net_connectors"

    json_options="--json-location-accuracy ${JSON_LOCATION_ACCURACY} --range-outline-hours 24"

    log::info "$(t rsb_defaults_write "$READSB_DEFAULTS")"
    run::sudo_write "$READSB_DEFAULTS" "$(printf '%b\nRECEIVER_OPTIONS="%s"\nDECODER_OPTIONS="%s"\nNET_OPTIONS="%s"\nJSON_OPTIONS="%s"' \
        "$(t rsb_defaults_header)" \
        "$receiver_options" "$decoder_options" "$net_options" "$json_options")"

    READSB_NEEDS_RESTART="$FILE_CHANGED"

    # Covers the case where a previous run wrote the config but never restarted:
    # the file is unchanged now, yet the daemon predates it.
    svc::predates_file readsb "$READSB_DEFAULTS" && READSB_NEEDS_RESTART=true

    return 0
}

readsb::enable() {
    log::info "$(t rsb_enabling)"
    run::sudo systemctl daemon-reload
    run::sudo systemctl enable readsb

    # `enable --now` does nothing to a unit that is already running, which
    # would leave the daemon on the previous configuration. Restart explicitly
    # whenever the config we just wrote actually changed.
    #
    # is-active is read-only, so it is queried in dry-run too: that way the
    # printed command is the one a real run would issue.
    if systemctl is-active --quiet readsb 2>/dev/null; then
        if [[ "${READSB_NEEDS_RESTART:-false}" == true ]]; then
            log::info "$(t rsb_restarting)"
            run::sudo systemctl restart readsb
        fi
    else
        run::sudo systemctl start readsb
    fi

    [[ "$DRY_RUN" == true ]] && return 0

    run::cmd sleep 3

    if systemctl is-active --quiet readsb; then
        log::success "$(t rsb_active)"
    else
        log::error "$(t rsb_failed)"
        return 1
    fi

    if [[ -f "$READSB_JSON" ]]; then
        log::success "$(t rsb_json_ok "$READSB_JSON")"
    else
        log::warn "$(t rsb_json_wait "$READSB_JSON")"
    fi
}
