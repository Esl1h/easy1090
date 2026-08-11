#!/usr/bin/env bash
#===============================================================================
# easy1090 - preflight checks
#
# Everything here is read-only, so it runs in full even under --dry-run. The
# point is to fail early and loudly, instead of letting the user discover a
# broken assumption halfway through a 45 minute compile.
#
# Author: Esli
# License: MIT
#===============================================================================

readonly RTLSDR_USB_VENDOR="0bda"
readonly RTLSDR_USB_PRODUCT="2838"

preflight::run() {
    log::step "Preflight"

    preflight::not_root
    preflight::interactive_tty
    preflight::distro
    preflight::tooling
    preflight::dongle

    log::success "Preflight concluído."
}

# makepkg and yay refuse to run as root, but udev/systemd/pacman -U need it.
# The design is: run as a normal user, escalate per step.
preflight::not_root() {
    if [[ $EUID -eq 0 ]]; then
        util::die "Não rode como root. Use seu usuário normal; o script pede sudo quando precisa (makepkg e yay se recusam a rodar como root)."
    fi
    log::success "Usuário normal (uid $EUID)."
}

# sudo needs a real tty for the password. Without it an automation pipeline
# either hangs or fails silently, and it always happens at the worst moment.
preflight::interactive_tty() {
    if [[ "$DRY_RUN" == true ]]; then
        log::info "TTY: verificação relaxada em --dry-run."
        return 0
    fi

    if [[ ! -t 0 ]]; then
        util::die "Sem terminal interativo. O sudo precisa de um tty real; rode direto num terminal ou numa sessão SSH de verdade."
    fi
    log::success "Terminal interativo disponível."
}

preflight::distro() {
    [[ -r /etc/os-release ]] || util::die "/etc/os-release não encontrado; distro não identificada."

    local id id_like pretty
    id="$(. /etc/os-release && printf '%s' "${ID:-}")"
    id_like="$(. /etc/os-release && printf '%s' "${ID_LIKE:-}")"
    pretty="$(. /etc/os-release && printf '%s' "${PRETTY_NAME:-desconhecida}")"

    if [[ "$id" == "arch" || "$id_like" == *arch* ]]; then
        log::success "Distro compatível: $pretty"
        return 0
    fi

    util::die "A v1 suporta apenas Arch e derivados (detectado: $pretty). Outras distros estão no roadmap; veja o README."
}

preflight::tooling() {
    local -a required=(pacman git curl)
    local -a missing=()
    local tool

    for tool in "${required[@]}"; do
        util::have_cmd "$tool" || missing+=("$tool")
    done

    [[ ${#missing[@]} -gt 0 ]] && util::die "Faltam comandos essenciais: ${missing[*]}"

    if ! util::have_cmd yay; then
        util::die "yay não encontrado. Instale um helper de AUR antes de continuar (o easy1090 depende dele para readsb e demais pacotes do AUR)."
    fi

    log::success "Ferramentas presentes: ${required[*]} yay"
}

preflight::dongle() {
    if ! util::have_cmd lsusb; then
        log::warn "lsusb não encontrado (pacote usbutils); pulando a checagem do dongle."
        return 0
    fi

    if lsusb | grep -qi "${RTLSDR_USB_VENDOR}:${RTLSDR_USB_PRODUCT}"; then
        log::success "RTL-SDR detectada no barramento USB (${RTLSDR_USB_VENDOR}:${RTLSDR_USB_PRODUCT})."
        return 0
    fi

    log::warn "Nenhuma RTL-SDR encontrada em lsusb (${RTLSDR_USB_VENDOR}:${RTLSDR_USB_PRODUCT})."
    log::warn "A instalação continua, mas nada vai decodificar sem o dongle conectado."

    util::confirm "Seguir mesmo assim?" || util::die "Interrompido a pedido do usuário."
}
