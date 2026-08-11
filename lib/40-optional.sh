#!/usr/bin/env bash
#===============================================================================
# easy1090 - optional GUI companions (SDR++ and SatDump)
#
# Neither decodes ADS-B. They are here because they complete the SDR bench:
# SDR++ to eyeball RF energy at 1090 MHz on the waterfall, SatDump for the
# 137 MHz satellite side. Both are off by default in install.conf.
#
# Author: Esli
# License: MIT
#===============================================================================

readonly SDRPP_PACKAGE="sdrpp-git"
readonly SATDUMP_PACKAGE="satdump"

optional::run() {
    [[ "$COMPONENT_SDRPP" == true ]] && sdrpp::run
    [[ "$COMPONENT_SATDUMP" == true ]] && satdump::run
    return 0
}

#===============================================================================
# SDR++
#===============================================================================

sdrpp::run() {
    log::step "$(t opt_sdrpp_step)"

    if pkg::is_installed "$SDRPP_PACKAGE"; then
        log::skip "$(t drv_installed "$SDRPP_PACKAGE")"
        return 0
    fi

    log::info "$(t opt_sdrpp_note)"
    pkg::install_aur "$SDRPP_PACKAGE"
}

#===============================================================================
# SatDump
#===============================================================================

satdump::run() {
    log::step "$(t opt_satdump_step)"

    if pkg::is_installed "$SATDUMP_PACKAGE"; then
        log::skip "$(t drv_installed "$SATDUMP_PACKAGE")"
        return 0
    fi

    # Stable release on purpose, not the -git: it is a large C++/CMake project
    # with many plugins, and the tagged version is far less likely to break
    # against whatever GCC the system is on.
    log::warn "$(t opt_satdump_slow)"
    util::confirm "$(t opt_satdump_confirm)" || {
        log::info "$(t opt_satdump_skipped)"
        return 0
    }

    pkg::install_aur "$SATDUMP_PACKAGE"
}
