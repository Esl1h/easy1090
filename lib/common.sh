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
declare FILE_CHANGED=false

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
    util::have_cmd "$1" || util::die "$(t cmd_required "$1")"
}

# Asks for confirmation. Always true under --yes or --dry-run.
util::confirm() {
    local prompt="$1"

    [[ "$ASSUME_YES" == true || "$DRY_RUN" == true ]] && return 0

    local answer
    read -r -p "$(printf "${BOLD}%s${RESET} %s" "$prompt" "$(t yes_no)")" answer
    [[ "$answer" =~ ^["$(t yes_chars)"]$ ]]
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
#
# Sets FILE_CHANGED so callers can decide whether a service needs restarting.
# Without that signal it is easy to write a new config and leave the daemon
# running on the old one, since `systemctl enable --now` is a no-op on an
# already running unit.
run::sudo_write() {
    local path="$1"
    local content="$2"

    FILE_CHANGED=false

    # The comparison runs in dry-run too, otherwise the preview would claim a
    # change (and a service restart) on every run.
    if [[ -f "$path" ]] && [[ "$(cat "$path" 2>/dev/null)" == "$content" ]]; then
        log::debug "sem alteração: $path"
        return 0
    fi

    FILE_CHANGED=true

    if [[ "$DRY_RUN" == true ]]; then
        log::dry_run "sudo tee $path <<'EOF'"
        printf '%s\n' "$content" | sed 's/^/        /' >&2
        log::dry_run "EOF"
        return 0
    fi

    log::debug "escrevendo $path"
    FILE_CHANGED=true
    printf '%s\n' "$content" | sudo tee "$path" >/dev/null
}

#===============================================================================
# SERVICE STATE
#===============================================================================

# True when the unit has been running since before the file was last written,
# which means it cannot possibly have loaded it.
#
# This closes the gap that "the config file exists" leaves open: a previous run
# may have written it and skipped the restart, so presence alone never proves
# the daemon is actually using it.
svc::predates_file() {
    local unit="$1"
    local file="$2"
    local started file_mtime

    [[ -f "$file" ]] || return 1
    systemctl is-active --quiet "$unit" 2>/dev/null || return 1

    started="$(systemctl show "$unit" -p ActiveEnterTimestamp --value 2>/dev/null)"
    [[ -n "$started" ]] || return 1

    started="$(date -d "$started" +%s 2>/dev/null)" || return 1
    file_mtime="$(stat -c %Y "$file" 2>/dev/null)" || return 1

    [[ "$started" -lt "$file_mtime" ]]
}

#===============================================================================
# PRIVILEGE HANDLING
#===============================================================================

# Validates sudo once and keeps the timestamp warm in background. Builds here
# can take ~45min (SatDump), and we do not want a password prompt in the middle
# of an unattended compile.
sudo::init() {
    [[ "$DRY_RUN" == true ]] && return 0

    log::info "$(t sudo_validating)"
    sudo -v || util::die "$(t sudo_failed)"

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
        if [[ "$DRY_RUN" == true ]]; then
            log::info "$(t cfg_missing_dry)"
            log::dry_run "cp $example_file $config_file"
            # Nothing is written in dry-run, so read the example instead. Without
            # this the run reports defaults that differ from what a real install
            # would use (the checksum pin, for one).
            # shellcheck source=/dev/null
            source "$example_file"
        else
            # Creating it from the example is harmless and is what every first
            # run needs, so it is not worth a prompt the user can trip over.
            [[ -f "$example_file" ]] || util::die "$(t cfg_example_missing "$example_file")"
            cp "$example_file" "$config_file"
            log::info "$(t cfg_created "$config_file")"
        fi
    fi

    # The language was already resolved (and possibly asked) before this point,
    # and sourcing the file would overwrite it with the empty value the example
    # ships. Keep the resolved one.
    local resolved_language="$UI_LANGUAGE"

    # shellcheck source=/dev/null
    [[ -f "$config_file" ]] && source "$config_file"

    UI_LANGUAGE="$resolved_language"
    cfg::_apply_defaults

    # Persist it so the next run does not ask again. Testing for the key is not
    # enough: the example ships UI_LANGUAGE="", what matters is having a value.
    if [[ -f "$config_file" && "$DRY_RUN" != true && -z "${UI_LANGUAGE_FROM_CONFIG:-}" ]]; then
        cfg::_persist "$config_file" UI_LANGUAGE "$UI_LANGUAGE"
    fi
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
    FEEDER_ASKED="${FEEDER_ASKED:-false}"

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
        log::warn "$(t pos_dry)"
        RECEIVER_LAT="${RECEIVER_LAT:-0.0}"
        RECEIVER_LON="${RECEIVER_LON:-0.0}"
        return 0
    fi

    [[ "$ASSUME_YES" == true ]] &&
        util::die "$(t pos_required_yes "$config_file")"

    log::info "$(t pos_intro)"

    # Nobody knows their coordinates by heart, so point at the tools instead of
    # leaving a bare prompt on screen.
    printf '\n%s\n%s\n%s\n%s\n\n%s\n\n' \
        "$(t pos_help_title)" \
        "$(t pos_help_osm)" \
        "$(t pos_help_gmaps)" \
        "$(t pos_help_latlong)" \
        "$(t pos_help_tip)" >&2

    read -r -p "$(t pos_lat)" RECEIVER_LAT
    read -r -p "$(t pos_lon)" RECEIVER_LON

    [[ -n "$RECEIVER_LAT" && -n "$RECEIVER_LON" ]] || util::die "$(t pos_required)"

    cfg::_persist "$config_file" RECEIVER_LAT "$RECEIVER_LAT"
    cfg::_persist "$config_file" RECEIVER_LON "$RECEIVER_LON"
    log::success "$(t pos_saved "$config_file")"
}

# Feeding a public network is opt-in and asked explicitly, because it publishes
# both the aircraft you receive and where you are.
cfg::require_feeder() {
    local config_file="$1"
    local answer

    # Already decided (config edited by hand or previous run), or no one to ask.
    if [[ "$FEEDER_ADSBEXCHANGE" == true ]]; then
        return 0
    fi
    if [[ "$DRY_RUN" == true || "$ASSUME_YES" == true ]]; then
        return 0
    fi
    [[ -f "$config_file" ]] && grep -qE '^FEEDER_ASKED="true"' "$config_file" && return 0

    printf '\n%s\n%s\n\n%s\n%s\n\n%s\n\n' \
        "$(t feed_title)" \
        "$(t feed_explain)" \
        "$(t feed_opt_none)" \
        "$(t feed_opt_adsbx)" \
        "$(t feed_fa_note)" >&2

    read -r -p "$(t feed_prompt)" answer

    case "$answer" in
    2)
        FEEDER_ADSBEXCHANGE=true
        cfg::_persist "$config_file" FEEDER_ADSBEXCHANGE "true"
        ;;
    *)
        log::info "$(t feed_none)"
        ;;
    esac

    cfg::_persist "$config_file" FEEDER_ASKED "true"
}

cfg::_persist() {
    local file="$1" key="$2" value="$3"

    if grep -qE "^${key}=" "$file"; then
        sed -i "s|^${key}=.*|${key}=\"${value}\"|" "$file"
    else
        printf '%s="%s"\n' "$key" "$value" >>"$file"
    fi
}
