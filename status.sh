#!/usr/bin/env bash
#===============================================================================
# easy1090 - status de cada componente
#
# Somente leitura: não inicia, não para e não altera nada. Serve para bater o
# olho e saber o que falta ou o que caiu.
#
# Author: Esli
# License: MIT
#===============================================================================

set -euo pipefail

readonly EASY1090_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=lib/common.sh
source "${EASY1090_ROOT}/lib/common.sh"
# shellcheck source=lib/pkg-arch.sh
source "${EASY1090_ROOT}/lib/pkg-arch.sh"

readonly READSB_JSON="/run/readsb/aircraft.json"

status::line() {
    local name="$1" state="$2" detail="${3:-}"
    local color="$RESET"

    case "$state" in
    rodando | instalado) color="$GREEN" ;;
    parado | ausente) color="$RED" ;;
    *) color="$YELLOW" ;;
    esac

    printf "  %-14s ${color}%-12s${RESET} %s\n" "$name" "$state" "$detail"
}

status::dongle() {
    if ! util::have_cmd lsusb; then
        status::line "RTL-SDR" "?" "lsusb não instalado"
        return
    fi

    if lsusb | grep -qi "0bda:2838"; then
        status::line "RTL-SDR" "instalado" "detectada no USB (0bda:2838)"
    else
        status::line "RTL-SDR" "ausente" "nada em lsusb"
    fi
}

status::package() {
    local label="$1" package="$2"

    if pkg::is_installed "$package"; then
        status::line "$label" "instalado" "$(pacman -Q "$package" 2>/dev/null || printf '%s' "$package")"
    else
        status::line "$label" "ausente" "$package"
    fi
}

status::service() {
    local label="$1" unit="$2" detail="${3:-}"

    if ! systemctl list-unit-files "${unit}.service" &>/dev/null; then
        status::line "$label" "ausente" "unidade não instalada"
        return
    fi

    local active enabled
    active="$(systemctl is-active "$unit" 2>/dev/null || true)"
    enabled="$(systemctl is-enabled "$unit" 2>/dev/null || true)"

    if [[ "$active" == "active" ]]; then
        status::line "$label" "rodando" "${enabled}${detail:+ | $detail}"
    else
        status::line "$label" "parado" "$enabled"
    fi
}

# Freshness of the JSON, not aircraft count: an empty sky is not a fault.
status::decoding() {
    [[ -f "$READSB_JSON" ]] || {
        status::line "decodificação" "ausente" "$READSB_JSON não existe"
        return
    }

    util::have_cmd jq || {
        status::line "decodificação" "?" "jq não instalado"
        return
    }

    local now aircraft age
    now="$(jq -r '.now // 0' "$READSB_JSON" 2>/dev/null || printf '0')"
    aircraft="$(jq -r '.aircraft | length' "$READSB_JSON" 2>/dev/null || printf '0')"
    age="$(awk -v n="$now" 'BEGIN { printf "%d", systime() - n }')"

    if [[ "$age" -le 60 ]]; then
        status::line "decodificação" "rodando" "JSON de ${age}s atrás, ${aircraft} aeronave(s)"
    else
        status::line "decodificação" "parado" "JSON parado há ${age}s"
    fi
}

status::web() {
    if ! systemctl is-active --quiet lighttpd 2>/dev/null; then
        status::line "mapa web" "parado" "lighttpd inativo"
        return
    fi

    local code
    code="$(curl -s -o /dev/null -w '%{http_code}' "http://localhost/tar1090/" 2>/dev/null || true)"

    if [[ "$code" == "200" ]]; then
        local ip
        ip="$(ip route get 1.1.1.1 2>/dev/null | grep -o 'src [0-9.]*' | cut -d' ' -f2)"
        status::line "mapa web" "rodando" "http://${ip:-localhost}/tar1090/"
    else
        status::line "mapa web" "parado" "HTTP ${code:-sem resposta}"
    fi
}

main() {
    printf "\n${BOLD}easy1090${RESET} %s - status\n\n" "$EASY1090_VERSION"

    printf "${BOLD}Hardware e driver${RESET}\n"
    status::dongle
    status::package "driver" "rtl-sdr-blog-git"

    printf "\n${BOLD}Decodificação${RESET}\n"
    status::package "readsb" "readsb-wiedehopf-git"
    status::service "serviço" "readsb"
    status::decoding

    printf "\n${BOLD}Web${RESET}\n"
    status::service "lighttpd" "lighttpd"
    status::service "tar1090" "tar1090"
    status::web

    printf "\n${BOLD}Opcionais${RESET}\n"
    status::package "SDR++" "sdrpp-git"
    status::package "SatDump" "satdump"

    printf "\n"
}

main "$@"
