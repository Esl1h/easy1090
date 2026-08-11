#!/usr/bin/env bash
#===============================================================================
# easy1090 - RTL-SDR driver
#
# Installs the RTL-SDR Blog fork of librtlsdr and gets the kernel DVB driver
# out of the way.
#
# The fork is installed unconditionally, without probing the model: it is
# mandatory for the V4 (R828D tuner plus the internal HF upconverter, which the
# osmocom driver mishandles) and harmless for v3 and clones.
#
# Author: Esli
# License: MIT
#===============================================================================

readonly DRIVER_PACKAGE="rtl-sdr-blog-git"
readonly DRIVER_CONFLICTS="rtl-sdr"
readonly DVB_MODULE="dvb_usb_rtl28xxu"
readonly DVB_BLACKLIST="/etc/modprobe.d/blacklist-rtlsdr.conf"

driver::run() {
    log::step "$(t drv_step)"

    driver::install_fork
    driver::blacklist_dvb
    driver::validate
}

driver::is_done() {
    pkg::is_installed "$DRIVER_PACKAGE"
}

driver::install_fork() {
    if driver::is_done; then
        log::skip "$(t drv_installed "$DRIVER_PACKAGE")"
        return 0
    fi

    # The AUR package declares a conflict with the generic rtl-sdr. Under
    # --noconfirm pacman answers "N" to the replace prompt and the install
    # aborts, so the removal has to be its own explicit step.
    if pkg::is_installed "$DRIVER_CONFLICTS"; then
        log::warn "$(t drv_conflict "$DRIVER_CONFLICTS")"
        util::confirm "$(t drv_conflict_confirm "$DRIVER_CONFLICTS")" ||
            util::die "$(t drv_conflict_abort)"
        pkg::remove "$DRIVER_CONFLICTS"
    fi

    pkg::install_aur "$DRIVER_PACKAGE"
}

# blacklist alone only stops autoload at boot. udev still pulls the module by
# alias on hotplug, so the dongle gets claimed as a DVB tuner the moment it is
# replugged. The install line is what actually closes that door.
driver::blacklist_dvb() {
    local content
    content="$(
        cat <<EOF
# easy1090: keeps the digital TV driver away from the RTL-SDR dongle.
# blacklist stops autoload at boot; install stops the alias-triggered load
# when udev asks for the module on hotplug.
blacklist $DVB_MODULE
install $DVB_MODULE /bin/false
EOF
    )"

    if [[ -f "$DVB_BLACKLIST" ]] && grep -q "install $DVB_MODULE /bin/false" "$DVB_BLACKLIST"; then
        log::skip "$(t drv_blacklist_ok "$DVB_MODULE")"
    else
        log::info "$(t drv_blacklist_set "$DVB_MODULE")"
        run::sudo_write "$DVB_BLACKLIST" "$content"
    fi

    if lsmod 2>/dev/null | grep -q "^${DVB_MODULE}"; then
        log::info "$(t drv_module_unload "$DVB_MODULE")"
        run::sudo modprobe -r "$DVB_MODULE" ||
            log::warn "$(t drv_module_unload_fail "$DVB_MODULE")"
    else
        log::skip "$(t drv_module_absent "$DVB_MODULE")"
    fi
}

driver::validate() {
    if [[ "$DRY_RUN" == true ]]; then
        log::dry_run "rtl_test -t"
        return 0
    fi

    util::have_cmd rtl_test || {
        log::warn "$(t drv_test_missing)"
        return 0
    }

    log::info "$(t drv_test_running)"

    # rtl_test -t always ends with "No E4000 tuner found, aborting." on this
    # hardware. That is the E4000 specific gain test, not a failure, so we look
    # at the detection lines instead of the exit code.
    local output
    output="$(timeout 15 rtl_test -t 2>&1 || true)"
    log::debug "$output"

    if grep -q "No supported devices found" <<<"$output"; then
        log::error "$(t drv_no_device)"
        log::error "$(t drv_no_device_hint)"
        return 1
    fi

    if grep -q "R828D" <<<"$output"; then
        log::success "$(t drv_tuner_v4)"
    elif grep -q "R820T" <<<"$output"; then
        log::success "$(t drv_tuner_v3)"
    else
        log::warn "$(t drv_tuner_unknown)"
    fi

    return 0
}
