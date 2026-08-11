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
    log::step "$(t pre_step)"

    preflight::not_root
    preflight::interactive_tty
    preflight::distro
    preflight::tooling
    preflight::dongle

    log::success "$(t pre_done)"
}

# makepkg and yay refuse to run as root, but udev/systemd/pacman -U need it.
# The design is: run as a normal user, escalate per step.
preflight::not_root() {
    [[ $EUID -eq 0 ]] && util::die "$(t pre_root)"
    log::success "$(t pre_user_ok "$EUID")"
}

# sudo needs a real tty for the password. Without it an automation pipeline
# either hangs or fails silently, and it always happens at the worst moment.
preflight::interactive_tty() {
    if [[ "$DRY_RUN" == true ]]; then
        log::info "$(t pre_tty_dry)"
        return 0
    fi

    [[ -t 0 ]] || util::die "$(t pre_tty_missing)"
    log::success "$(t pre_tty_ok)"
}

preflight::distro() {
    [[ -r /etc/os-release ]] || util::die "$(t pre_osrelease)"

    local id id_like pretty
    id="$(. /etc/os-release && printf '%s' "${ID:-}")"
    id_like="$(. /etc/os-release && printf '%s' "${ID_LIKE:-}")"
    pretty="$(. /etc/os-release && printf '%s' "${PRETTY_NAME:-?}")"

    if [[ "$id" == "arch" || "$id_like" == *arch* ]]; then
        log::success "$(t pre_distro_ok "$pretty")"
        return 0
    fi

    util::die "$(t pre_distro_bad "$pretty")"
}

preflight::tooling() {
    local -a required=(pacman git curl)
    local -a missing=()
    local tool

    for tool in "${required[@]}"; do
        util::have_cmd "$tool" || missing+=("$tool")
    done

    [[ ${#missing[@]} -gt 0 ]] && util::die "$(t pre_tools_missing "${missing[*]}")"

    util::have_cmd yay || util::die "$(t pre_yay_missing)"

    log::success "$(t pre_tools_ok "${required[*]} yay")"
}

preflight::dongle() {
    if ! util::have_cmd lsusb; then
        log::warn "$(t pre_lsusb_missing)"
        return 0
    fi

    local usb_id="${RTLSDR_USB_VENDOR}:${RTLSDR_USB_PRODUCT}"

    if lsusb | grep -qi "$usb_id"; then
        log::success "$(t pre_dongle_ok "$usb_id")"
        return 0
    fi

    log::warn "$(t pre_dongle_missing "$usb_id")"
    log::warn "$(t pre_dongle_warn)"

    util::confirm "$(t pre_dongle_confirm)" || util::die "$(t pre_aborted)"
}
