#!/usr/bin/env bash
#===============================================================================
# easy1090 - final validation
#
# Confirms the pipeline end to end and prints where to look. Read-only.
#
# Author: Esli
# License: MIT
#===============================================================================

validate::run() {
    log::step "Validação final"

    if [[ "$DRY_RUN" == true ]]; then
        log::dry_run "systemctl is-active readsb lighttpd tar1090"
        log::dry_run "curl -s http://localhost/tar1090/data/aircraft.json"
        return 0
    fi

    # Note: failures=$((...)) instead of ((failures++)). Post-increment from
    # zero evaluates to 0, which is a non-zero exit status, and set -e would
    # abort the run on the very first failure it is trying to count.
    local failures=0

    validate::service readsb || failures=$((failures + 1))
    [[ "$COMPONENT_TAR1090" == true ]] && { validate::service lighttpd || failures=$((failures + 1)); }
    [[ "$COMPONENT_TAR1090" == true ]] && { validate::service tar1090 || failures=$((failures + 1)); }

    validate::decoding || failures=$((failures + 1))
    [[ "$COMPONENT_TAR1090" == true ]] && { validate::web || failures=$((failures + 1)); }

    validate::summary "$failures"
    return "$failures"
}

validate::service() {
    local unit="$1"

    if ! systemctl list-unit-files "${unit}.service" &>/dev/null; then
        log::error "Unidade não encontrada: ${unit}.service"
        return 1
    fi

    local active enabled
    active="$(systemctl is-active "$unit" 2>/dev/null || true)"
    enabled="$(systemctl is-enabled "$unit" 2>/dev/null || true)"

    if [[ "$active" == "active" && "$enabled" == "enabled" ]]; then
        log::success "${unit}: ativo e habilitado no boot."
        return 0
    fi

    log::error "${unit}: active=${active:-?} enabled=${enabled:-?}"
    return 1
}

# An empty sky is not a failure. What proves the decoder is alive is the JSON
# being refreshed, so we check the file's own clock instead of aircraft count.
validate::decoding() {
    [[ -f "$READSB_JSON" ]] || {
        log::error "$READSB_JSON não existe. O readsb está gravando JSON?"
        return 1
    }

    local now aircraft age
    now="$(jq -r '.now // 0' "$READSB_JSON" 2>/dev/null || printf '0')"
    aircraft="$(jq -r '.aircraft | length' "$READSB_JSON" 2>/dev/null || printf '0')"
    age="$(awk -v n="$now" 'BEGIN { printf "%d", systime() - n }')"

    if [[ "$age" -gt 60 ]]; then
        log::error "aircraft.json parado há ${age}s; o readsb pode ter travado."
        return 1
    fi

    log::success "readsb decodificando (JSON atualizado há ${age}s, ${aircraft} aeronave(s) na tela)."

    [[ "$aircraft" -eq 0 ]] &&
        log::info "Zero aeronaves agora é normal: depende de tráfego, antena e linha de visada."

    return 0
}

validate::web() {
    local code
    code="$(curl -s -o /dev/null -w '%{http_code}' "http://localhost/tar1090/" || true)"

    if [[ "$code" != "200" ]]; then
        log::error "http://localhost/tar1090/ devolveu ${code:-sem resposta}."
        return 1
    fi

    if curl -sf "http://localhost/tar1090/data/aircraft.json" -o /dev/null; then
        log::success "tar1090 servindo mapa e dados."
        return 0
    fi

    log::error "O mapa responde, mas /tar1090/data/aircraft.json não. Cheque o serviço tar1090."
    return 1
}

validate::summary() {
    local failures="$1"
    local ip

    ip="$(ip route get 1.1.1.1 2>/dev/null | grep -o 'src [0-9.]*' | cut -d' ' -f2)"

    printf "\n"
    if [[ "$failures" -eq 0 ]]; then
        log::success "Tudo no ar."
    else
        log::error "${failures} verificação(ões) falharam."
    fi

    printf "\n${BOLD}Como acompanhar o tráfego:${RESET}\n" >&2
    printf "  viewadsb                         tabela ao vivo no terminal\n" >&2
    printf "  nc localhost %-5s               mensagens decodificadas (SBS/CSV)\n" "$NET_SBS_PORT" >&2
    printf "  jq . %s\n" "$READSB_JSON" >&2
    [[ "$COMPONENT_TAR1090" == true ]] &&
        printf "  http://%s/tar1090/          mapa web ao vivo\n" "${ip:-localhost}" >&2
    printf "\n" >&2
}
