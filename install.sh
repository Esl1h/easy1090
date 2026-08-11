#!/usr/bin/env bash
#===============================================================================
# easy1090 - instalador do stack ADS-B em um comando
#
# Deixa rodando, num Arch limpo com uma RTL-SDR conectada:
#   driver (fork RTL-SDR Blog) -> readsb (fork wiedehopf, JSON) -> tar1090
#
# É idempotente: rodar de novo é a forma de atualizar. Cada módulo detecta o
# que já está pronto e pula.
#
# Author: Esli
# License: MIT
# Repository: https://github.com/Esl1h/easy1090
#===============================================================================

set -euo pipefail

readonly SCRIPT_NAME="${0##*/}"
readonly EASY1090_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly CONFIG_FILE="${EASY1090_ROOT}/install.conf"
readonly CONFIG_EXAMPLE="${EASY1090_ROOT}/install.conf.example"

# shellcheck source=lib/common.sh
source "${EASY1090_ROOT}/lib/common.sh"

#===============================================================================
# USAGE
#===============================================================================

usage() {
    cat <<EOF
${BOLD}easy1090${RESET} ${EASY1090_VERSION} - instalador do stack ADS-B (Arch e derivados)

USO
    ./${SCRIPT_NAME} [opções]

OPÇÕES
    --full              instala tudo, inclusive SDR++ e SatDump
    --lat <graus>       latitude da antena (ex: -23.58)
    --lon <graus>       longitude da antena (ex: -46.55)
    --skip-tar1090      não instala o mapa web
    --skip-sdrpp        não instala o SDR++
    --skip-satdump      não instala o SatDump
    --dry-run           roda o preflight e imprime os comandos exatos, sem executar
    --yes               não pergunta nada (exceto a senha do sudo)
    --verbose           log em nível debug
    --version           mostra a versão
    -h, --help          esta ajuda

EXEMPLOS
    ./${SCRIPT_NAME}                                  interativo, pergunta lat/lon
    ./${SCRIPT_NAME} --lat -23.58 --lon -46.55 --yes  sem interação
    ./${SCRIPT_NAME} --dry-run                        mostra o que faria

A configuração vive em install.conf (gerada a partir do .example na primeira
execução). As flags acima sobrescrevem o que estiver lá.
EOF
}

#===============================================================================
# ARGUMENT PARSING
#===============================================================================

declare CLI_LAT="" CLI_LON=""
declare CLI_SKIP_TAR1090=false CLI_FULL=false
declare CLI_SKIP_SDRPP=false CLI_SKIP_SATDUMP=false

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
        --full) CLI_FULL=true ;;
        --lat)
            CLI_LAT="${2:-}"
            shift
            ;;
        --lon)
            CLI_LON="${2:-}"
            shift
            ;;
        --skip-tar1090) CLI_SKIP_TAR1090=true ;;
        --skip-sdrpp) CLI_SKIP_SDRPP=true ;;
        --skip-satdump) CLI_SKIP_SATDUMP=true ;;
        --dry-run) DRY_RUN=true ;;
        --yes | -y) ASSUME_YES=true ;;
        --verbose) LOG_LEVEL=3 ;;
        --version)
            printf 'easy1090 %s\n' "$EASY1090_VERSION"
            exit 0
            ;;
        -h | --help)
            usage
            exit 0
            ;;
        *)
            log::error "Opção desconhecida: $1"
            usage
            exit 1
            ;;
        esac
        shift
    done
}

# CLI wins over install.conf.
apply_overrides() {
    [[ -n "$CLI_LAT" ]] && RECEIVER_LAT="$CLI_LAT"
    [[ -n "$CLI_LON" ]] && RECEIVER_LON="$CLI_LON"

    if [[ "$CLI_FULL" == true ]]; then
        COMPONENT_TAR1090=true
        COMPONENT_SDRPP=true
        COMPONENT_SATDUMP=true
    fi

    [[ "$CLI_SKIP_TAR1090" == true ]] && COMPONENT_TAR1090=false
    [[ "$CLI_SKIP_SDRPP" == true ]] && COMPONENT_SDRPP=false
    [[ "$CLI_SKIP_SATDUMP" == true ]] && COMPONENT_SATDUMP=false

    return 0
}

#===============================================================================
# MAIN
#===============================================================================

load_modules() {
    local module
    for module in \
        "${EASY1090_ROOT}/lib/pkg-${PKG_TARGET:-arch}.sh" \
        "${EASY1090_ROOT}"/lib/[0-9][0-9]-*.sh; do
        # shellcheck source=/dev/null
        source "$module"
    done
}

banner() {
    printf "\n${BOLD}easy1090${RESET} %s\n" "$EASY1090_VERSION" >&2
    [[ "$DRY_RUN" == true ]] &&
        log::warn "Modo --dry-run: nada será alterado; os comandos abaixo são os reais."
    return 0
}

main() {
    parse_args "$@"

    load_modules
    banner

    preflight::run

    cfg::load "$CONFIG_FILE" "$CONFIG_EXAMPLE"
    apply_overrides
    cfg::require_position "$CONFIG_FILE"

    trap 'sudo::cleanup' EXIT
    sudo::init

    driver::run
    readsb::run
    [[ "$COMPONENT_TAR1090" == true ]] && tar1090::run
    optional::run

    validate::run
}

main "$@"
