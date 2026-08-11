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
    log::step "Driver RTL-SDR"

    driver::install_fork
    driver::blacklist_dvb
    driver::validate
}

driver::is_done() {
    pkg::is_installed "$DRIVER_PACKAGE"
}

driver::install_fork() {
    if driver::is_done; then
        log::skip "$DRIVER_PACKAGE já instalado."
        return 0
    fi

    # The AUR package declares a conflict with the generic rtl-sdr. Under
    # --noconfirm pacman answers "N" to the replace prompt and the install
    # aborts, so the removal has to be its own explicit step.
    if pkg::is_installed "$DRIVER_CONFLICTS"; then
        log::warn "O pacote genérico $DRIVER_CONFLICTS conflita com o fork da RTL-SDR Blog."
        util::confirm "Remover $DRIVER_CONFLICTS agora?" ||
            util::die "Sem remover o conflito, a instalação do fork falha."
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
# easy1090: mantém o driver de TV digital longe do dongle RTL-SDR.
# blacklist impede o autoload no boot; install impede a carga por alias
# quando o udev pede o módulo no hotplug.
blacklist $DVB_MODULE
install $DVB_MODULE /bin/false
EOF
    )"

    if [[ -f "$DVB_BLACKLIST" ]] && grep -q "install $DVB_MODULE /bin/false" "$DVB_BLACKLIST"; then
        log::skip "Blacklist do $DVB_MODULE já configurada."
    else
        log::info "Configurando blacklist de $DVB_MODULE"
        run::sudo_write "$DVB_BLACKLIST" "$content"
    fi

    if lsmod 2>/dev/null | grep -q "^${DVB_MODULE}"; then
        log::info "Descarregando $DVB_MODULE (carregado agora)."
        run::sudo modprobe -r "$DVB_MODULE" ||
            log::warn "Não consegui descarregar $DVB_MODULE; pode ser necessário reiniciar."
    else
        log::skip "$DVB_MODULE não está carregado."
    fi
}

driver::validate() {
    if [[ "$DRY_RUN" == true ]]; then
        log::dry_run "rtl_test -t"
        return 0
    fi

    util::have_cmd rtl_test || {
        log::warn "rtl_test não encontrado no PATH; pulando validação do driver."
        return 0
    }

    log::info "Validando o hardware com rtl_test."

    # rtl_test -t always ends with "No E4000 tuner found, aborting." on this
    # hardware. That is the E4000 specific gain test, not a failure, so we look
    # at the detection lines instead of the exit code.
    local output
    output="$(timeout 15 rtl_test -t 2>&1 || true)"
    log::debug "$output"

    if grep -q "No supported devices found" <<<"$output"; then
        log::error "Nenhum dispositivo suportado encontrado."
        log::error "Cheque o cabo e a porta USB (prefira as traseiras, ligadas direto à placa-mãe)."
        return 1
    fi

    if grep -q "R828D" <<<"$output"; then
        log::success "Tuner R828D detectado (RTL-SDR Blog V4)."
    elif grep -q "R820T" <<<"$output"; then
        log::success "Tuner R820T/R820T2 detectado (v3 ou clone)."
    else
        log::warn "Dongle respondeu, mas não identifiquei o tuner. Saída completa em --log-level debug."
    fi

    return 0
}
