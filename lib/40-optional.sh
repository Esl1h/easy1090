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
    log::step "SDR++ (visualizador de espectro)"

    if pkg::is_installed "$SDRPP_PACKAGE"; then
        log::skip "$SDRPP_PACKAGE já instalado."
        return 0
    fi

    log::info "SDR++ não decodifica ADS-B; serve para conferir visualmente a energia RF em 1090 MHz."
    pkg::install_aur "$SDRPP_PACKAGE"
}

#===============================================================================
# SatDump
#===============================================================================

satdump::run() {
    log::step "SatDump (decodificador de satélites)"

    if pkg::is_installed "$SATDUMP_PACKAGE"; then
        log::skip "$SATDUMP_PACKAGE já instalado."
        return 0
    fi

    # Stable release on purpose, not the -git: it is a large C++/CMake project
    # with many plugins, and the tagged version is far less likely to break
    # against whatever GCC the system is on.
    log::warn "O build do SatDump é longo (cerca de 45 minutos no hardware de referência)."
    util::confirm "Continuar com a instalação do SatDump?" || {
        log::info "SatDump pulado."
        return 0
    }

    pkg::install_aur "$SATDUMP_PACKAGE"
}
