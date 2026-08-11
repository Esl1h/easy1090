#!/usr/bin/env bash
#===============================================================================
# easy1090 - shared helpers
#
# Logging, command execution (with dry-run), privilege handling and config
# loading. Sourced by install.sh and by every module in lib/.
#
# Author: Esli
# License: MIT
# Repository: https://github.com/Esl1h/easy1090
#===============================================================================

# Guard against double sourcing (readonly vars would abort the run)
[[ -n "${_EASY1090_COMMON_LOADED:-}" ]] && return 0
readonly _EASY1090_COMMON_LOADED=1

#===============================================================================
# CONSTANTS AND GLOBAL STATE
#===============================================================================

readonly EASY1090_VERSION="0.1.0"

# Colors for output (disabled if not a TTY)
if [[ -t 1 ]]; then
    readonly RED='\033[0;31m'
    readonly GREEN='\033[0;32m'
    readonly YELLOW='\033[0;33m'
    readonly BLUE='\033[0;34m'
    readonly MAGENTA='\033[0;35m'
    readonly CYAN='\033[0;36m'
    readonly BOLD='\033[1m'
    readonly RESET='\033[0m'
else
    readonly RED='' GREEN='' YELLOW='' BLUE='' MAGENTA='' CYAN='' BOLD='' RESET=''
fi

declare LOG_LEVEL=2 # 0=error, 1=warn, 2=info, 3=debug
declare DRY_RUN=false
declare ASSUME_YES=false
declare SUDO_KEEPALIVE_PID=""

#===============================================================================
# LOGGING FUNCTIONS
#===============================================================================

log::_output() {
    local level="$1"
    local color="$2"
    local message="$3"

    printf "${color}[%-5s]${RESET} %s\n" "$level" "$message" >&2
}

log::debug() {
    [[ $LOG_LEVEL -ge 3 ]] && log::_output "DEBUG" "$CYAN" "$*"
    return 0
}

log::info() {
    [[ $LOG_LEVEL -ge 2 ]] && log::_output "INFO" "$BLUE" "$*"
    return 0
}

log::warn() {
    [[ $LOG_LEVEL -ge 1 ]] && log::_output "WARN" "$YELLOW" "$*"
    return 0
}

log::error() {
    log::_output "ERROR" "$RED" "$*"
    return 0
}

log::success() {
    [[ $LOG_LEVEL -ge 2 ]] && log::_output "OK" "$GREEN" "$*"
    return 0
}

log::skip() {
    [[ $LOG_LEVEL -ge 2 ]] && log::_output "SKIP" "$CYAN" "$*"
    return 0
}

log::dry_run() {
    log::_output "DRY" "$MAGENTA" "$*"
    return 0
}

log::step() {
    printf "\n${BOLD}==> %s${RESET}\n" "$*" >&2
}

#===============================================================================
# UTILITY FUNCTIONS
#===============================================================================

util::die() {
    log::error "$*"
    exit 1
}

util::have_cmd() {
    command -v "$1" &>/dev/null
}

util::require_command() {
    util::have_cmd "$1" || util::die "Comando obrigatório não encontrado: $1"
}

# Asks for confirmation. Always true under --yes or --dry-run.
util::confirm() {
    local prompt="$1"

    [[ "$ASSUME_YES" == true || "$DRY_RUN" == true ]] && return 0

    local answer
    read -r -p "$(printf "${BOLD}%s${RESET} [s/N] " "$prompt")" answer
    [[ "$answer" =~ ^[SsYy]$ ]]
}

#===============================================================================
# COMMAND EXECUTION
#===============================================================================

# Renders argv as a copy-pasteable command line. This is what --dry-run prints,
# so it has to be the real command, not a description of it.
run::_render() {
    local out="" arg
    for arg in "$@"; do
        if [[ "$arg" =~ ^[A-Za-z0-9_./:=@%+-]+$ ]]; then
            out+="$arg "
        else
            out+="$(printf '%q' "$arg") "
        fi
    done
    printf '%s' "${out% }"
}

run::cmd() {
    local rendered
    rendered="$(run::_render "$@")"

    if [[ "$DRY_RUN" == true ]]; then
        log::dry_run "$rendered"
        return 0
    fi

    log::debug "exec: $rendered"
    "$@"
}

run::sudo() {
    run::cmd sudo "$@"
}

# Writes a file as root. Kept separate from run::sudo because the redirection
# has to happen on the privileged side of the pipe, not in our shell.
run::sudo_write() {
    local path="$1"
    local content="$2"

    if [[ "$DRY_RUN" == true ]]; then
        log::dry_run "sudo tee $path <<'EOF'"
        printf '%s\n' "$content" | sed 's/^/        /' >&2
        log::dry_run "EOF"
        return 0
    fi

    log::debug "escrevendo $path"
    printf '%s\n' "$content" | sudo tee "$path" >/dev/null
}

#===============================================================================
# PRIVILEGE HANDLING
#===============================================================================

# Validates sudo once and keeps the timestamp warm in background. Builds here
# can take ~45min (SatDump), and we do not want a password prompt in the middle
# of an unattended compile.
sudo::init() {
    [[ "$DRY_RUN" == true ]] && return 0

    log::info "Validando sudo (a senha pode ser pedida agora)."
    sudo -v || util::die "Não foi possível validar o sudo."

    (
        while true; do
            sleep 60
            sudo -n true 2>/dev/null || exit 0
        done
    ) &
    SUDO_KEEPALIVE_PID=$!
    log::debug "keepalive de sudo no pid $SUDO_KEEPALIVE_PID"
}

sudo::cleanup() {
    if [[ -n "$SUDO_KEEPALIVE_PID" ]]; then
        kill "$SUDO_KEEPALIVE_PID" 2>/dev/null || true
        SUDO_KEEPALIVE_PID=""
    fi
}

#===============================================================================
# CONFIGURATION
#===============================================================================

# Loads install.conf. It is plain sourceable shell (the /etc/default style the
# Arch crowd already knows), so no TOML parser dependency in the bash version.
cfg::load() {
    local config_file="$1"
    local example_file="$2"

    if [[ ! -f "$config_file" ]]; then
        log::warn "Config não encontrada em $config_file"
        if [[ "$DRY_RUN" == true ]]; then
            log::dry_run "cp $example_file $config_file"
            # Nothing is written in dry-run, so read the example instead. Without
            # this the run reports defaults that differ from what a real install
            # would use (the checksum pin, for one).
            # shellcheck source=/dev/null
            source "$example_file"
        else
            util::confirm "Criar a partir de $(basename "$example_file")?" ||
                util::die "Sem config não dá pra seguir. Copie o exemplo e edite."
            cp "$example_file" "$config_file"
            log::success "Criada $config_file"
        fi
    fi

    # shellcheck source=/dev/null
    [[ -f "$config_file" ]] && source "$config_file"

    cfg::_apply_defaults
}

cfg::_apply_defaults() {
    RECEIVER_GAIN="${RECEIVER_GAIN:-auto}"
    RECEIVER_PPM="${RECEIVER_PPM:-0}"
    DECODER_MAX_RANGE="${DECODER_MAX_RANGE:-450}"
    JSON_LOCATION_ACCURACY="${JSON_LOCATION_ACCURACY:-2}"

    COMPONENT_TAR1090="${COMPONENT_TAR1090:-true}"
    COMPONENT_SDRPP="${COMPONENT_SDRPP:-false}"
    COMPONENT_SATDUMP="${COMPONENT_SATDUMP:-false}"

    NET_RI_PORT="${NET_RI_PORT:-30001}"
    NET_RO_PORT="${NET_RO_PORT:-30002}"
    NET_SBS_PORT="${NET_SBS_PORT:-30003}"
    NET_BI_PORT="${NET_BI_PORT:-30004,30104}"
    NET_BO_PORT="${NET_BO_PORT:-30005}"

    FEEDER_ADSBEXCHANGE="${FEEDER_ADSBEXCHANGE:-false}"
    FEEDER_FLIGHTAWARE="${FEEDER_FLIGHTAWARE:-false}"

    TAR1090_INSTALLER_SHA256="${TAR1090_INSTALLER_SHA256:-}"
    PREFERRED_TERMINAL="${PREFERRED_TERMINAL:-}"
}

# Receiver position is the only thing we cannot guess. Prompts when missing.
cfg::require_position() {
    local config_file="$1"

    if [[ -n "${RECEIVER_LAT:-}" && -n "${RECEIVER_LON:-}" ]]; then
        log::debug "posição: $RECEIVER_LAT, $RECEIVER_LON"
        return 0
    fi

    if [[ "$DRY_RUN" == true ]]; then
        log::warn "RECEIVER_LAT/RECEIVER_LON vazios; usaria valores informados na execução real."
        RECEIVER_LAT="${RECEIVER_LAT:-0.0}"
        RECEIVER_LON="${RECEIVER_LON:-0.0}"
        return 0
    fi

    [[ "$ASSUME_YES" == true ]] &&
        util::die "RECEIVER_LAT/RECEIVER_LON são obrigatórios com --yes. Preencha $config_file."

    log::info "A posição da antena é usada para calcular alcance e distância das aeronaves."
    read -r -p "Latitude (ex: -23.58): " RECEIVER_LAT
    read -r -p "Longitude (ex: -46.55): " RECEIVER_LON

    [[ -n "$RECEIVER_LAT" && -n "$RECEIVER_LON" ]] || util::die "Latitude e longitude são obrigatórias."

    cfg::_persist "$config_file" RECEIVER_LAT "$RECEIVER_LAT"
    cfg::_persist "$config_file" RECEIVER_LON "$RECEIVER_LON"
    log::success "Posição gravada em $config_file"
}

cfg::_persist() {
    local file="$1" key="$2" value="$3"

    if grep -qE "^${key}=" "$file"; then
        sed -i "s|^${key}=.*|${key}=\"${value}\"|" "$file"
    else
        printf '%s="%s"\n' "$key" "$value" >>"$file"
    fi
}
