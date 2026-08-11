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
readonly READSB_BUILD_DIR="${HOME}/.cache/easy1090/readsb-wiedehopf-git"
readonly READSB_DEFAULTS="/etc/default/readsb"
readonly READSB_UDEV_RULE="/etc/udev/rules.d/99-readsb-rtlsdr.rules"
readonly READSB_LEGACY_OVERRIDE="/etc/systemd/system/readsb.service.d/override.conf"
readonly READSB_JSON="/run/readsb/aircraft.json"

readsb::run() {
    log::step "readsb (decodificador ADS-B)"

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
        log::skip "$READSB_PACKAGE já instalado."
        return 0
    fi

    if pkg::is_installed "$READSB_CONFLICTS"; then
        log::warn "$READSB_CONFLICTS (fork Mictronics) conflita com $READSB_PACKAGE e grava protobuf em vez de JSON."
        util::confirm "Remover $READSB_CONFLICTS agora?" ||
            util::die "Os dois pacotes não convivem; sem remover não dá pra seguir."
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
        log::warn "Override antigo do systemd encontrado (referencia \$USER_OPTIONS, que não existe neste pacote)."
        run::sudo rm -f "$READSB_LEGACY_OVERRIDE"
        run::sudo rmdir "$(dirname "$READSB_LEGACY_OVERRIDE")" 2>/dev/null || true
        run::sudo systemctl daemon-reload
        log::success "Override legado removido."
    fi
}

# The stock rtl-sdr rule grants the device to the plugdev group. That is enough
# for an interactive user (systemd-logind adds a session ACL on top), which is
# exactly why the problem hides: the readsb service user has no session and no
# ACL, so it still gets EACCES. Hence a dedicated rule for the service group.
readsb::udev_rule() {
    local rule="SUBSYSTEM==\"usb\", ATTRS{idVendor}==\"${RTLSDR_USB_VENDOR}\", ATTRS{idProduct}==\"${RTLSDR_USB_PRODUCT}\", GROUP=\"readsb\", MODE=\"0660\""

    if [[ -f "$READSB_UDEV_RULE" ]] && grep -q 'GROUP="readsb"' "$READSB_UDEV_RULE"; then
        log::skip "Regra udev do readsb já existe."
        return 0
    fi

    log::info "Criando regra udev para o usuário de serviço readsb."
    run::sudo_write "$READSB_UDEV_RULE" "# easy1090: entrega o dongle ao grupo do usuário de serviço readsb.
# A regra padrão usa GROUP=\"plugdev\", que não cobre um usuário sem sessão.
$rule"

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
        log::warn "Feeder ADSBExchange habilitado: sua posição e seus dados serão compartilhados."
        net_connectors+=" --net-connector feed.adsbexchange.com,30005,beast_reduce_out"
    fi
    if [[ "$FEEDER_FLIGHTAWARE" == true ]]; then
        log::warn "Feeder FlightAware habilitado: sua posição e seus dados serão compartilhados."
        net_connectors+=" --net-connector piaware.flightaware.com,1200,beast_reduce_out"
    fi
    [[ -n "$net_connectors" ]] && net_options+="$net_connectors"

    json_options="--json-location-accuracy ${JSON_LOCATION_ACCURACY} --range-outline-hours 24"

    log::info "Escrevendo $READSB_DEFAULTS"
    run::sudo_write "$READSB_DEFAULTS" "# Gerado pelo easy1090. Editável à vontade: o instalador só reescreve
# este arquivo quando você roda install.sh de novo.
RECEIVER_OPTIONS=\"${receiver_options}\"
DECODER_OPTIONS=\"${decoder_options}\"
NET_OPTIONS=\"${net_options}\"
JSON_OPTIONS=\"${json_options}\""
}

readsb::enable() {
    log::info "Habilitando e iniciando o serviço readsb."
    run::sudo systemctl daemon-reload
    run::sudo systemctl enable --now readsb

    [[ "$DRY_RUN" == true ]] && return 0

    run::cmd sleep 3

    if systemctl is-active --quiet readsb; then
        log::success "readsb ativo."
    else
        log::error "readsb não subiu. Veja: journalctl -u readsb -n 40 --no-pager"
        return 1
    fi

    if [[ -f "$READSB_JSON" ]]; then
        log::success "JSON sendo gravado em $READSB_JSON"
    else
        log::warn "$READSB_JSON ainda não existe; pode levar alguns segundos."
    fi
}
