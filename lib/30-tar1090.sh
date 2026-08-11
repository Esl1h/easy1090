#!/usr/bin/env bash
#===============================================================================
# easy1090 - tar1090 (live web map)
#
# There is no tar1090 package in the AUR. Upstream ships an install.sh written
# for Debian and Raspberry Pi OS, which we vendor under vendor/ instead of
# piping it from the network on every run.
#
# Two Arch specific gaps this module closes, both silent failures:
#   - lighttpd.conf on Arch is minimal and never includes conf-enabled, so the
#     config the tar1090 installer drops there is simply never read.
#   - the upstream installer only restarts lighttpd if it was already running;
#     a freshly installed one stays dead and disabled.
#
# Author: Esli
# License: MIT
#===============================================================================

readonly TAR1090_INSTALLER="${EASY1090_ROOT}/vendor/tar1090-install.sh"
readonly LIGHTTPD_CONF="/etc/lighttpd/lighttpd.conf"
readonly LIGHTTPD_CONF_D="/etc/lighttpd/conf.d"
readonly LIGHTTPD_CONF_AVAILABLE="/etc/lighttpd/conf-available"
readonly LIGHTTPD_CONF_ENABLED="/etc/lighttpd/conf-enabled"
readonly TAR1090_URL_PATH="/tar1090/"

tar1090::run() {
    log::step "tar1090 (mapa web ao vivo)"

    tar1090::dependencies
    tar1090::verify_vendored
    tar1090::install
    tar1090::fix_lighttpd_include
    tar1090::fix_mod_redirect
    tar1090::enable_lighttpd
}

tar1090::is_done() {
    systemctl list-unit-files tar1090.service &>/dev/null &&
        systemctl is-enabled --quiet tar1090 2>/dev/null
}

tar1090::dependencies() {
    pkg::install jq lighttpd

    # The upstream installer only switches to its automatic lighttpd mode when
    # this directory exists (a Debian convention the Arch package does not use).
    if [[ -d "$LIGHTTPD_CONF_D" ]]; then
        log::skip "$LIGHTTPD_CONF_D já existe."
    else
        log::info "Criando $LIGHTTPD_CONF_D (o instalador do tar1090 procura por ele)."
        run::sudo mkdir -p "$LIGHTTPD_CONF_D"
    fi
}

# The vendored copy is pinned by checksum. Updating it is a deliberate act:
# refresh the file, then update TAR1090_INSTALLER_SHA256 in install.conf.
tar1090::verify_vendored() {
    [[ -f "$TAR1090_INSTALLER" ]] ||
        util::die "Instalador do tar1090 não encontrado em $TAR1090_INSTALLER"

    local actual
    actual="$(sha256sum "$TAR1090_INSTALLER" | cut -d' ' -f1)"

    if [[ -z "$TAR1090_INSTALLER_SHA256" ]]; then
        log::warn "TAR1090_INSTALLER_SHA256 não definido na config."
        log::warn "Checksum atual do arquivo vendorizado: $actual"
        log::warn "Fixe esse valor na config para detectar alterações futuras."
        return 0
    fi

    if [[ "$actual" != "$TAR1090_INSTALLER_SHA256" ]]; then
        log::error "Checksum do instalador do tar1090 não confere."
        log::error "  esperado: $TAR1090_INSTALLER_SHA256"
        log::error "  obtido:   $actual"
        util::die "Recuse-se a rodar script de root alterado. Revise o arquivo antes de atualizar o pin."
    fi

    log::success "Instalador vendorizado confere com o pin da config."
}

tar1090::install() {
    if tar1090::is_done; then
        log::skip "Serviço tar1090 já habilitado."
        return 0
    fi

    log::info "Rodando o instalador oficial do tar1090 (vendorizado)."
    run::sudo bash "$TAR1090_INSTALLER"
}

# Without this include, everything the tar1090 installer wrote is dead config.
tar1090::fix_lighttpd_include() {
    [[ -f "$LIGHTTPD_CONF" ]] || {
        log::warn "$LIGHTTPD_CONF não encontrado; pulando o ajuste do include."
        return 0
    }

    if grep -q "conf-enabled" "$LIGHTTPD_CONF"; then
        log::skip "lighttpd.conf já inclui conf-enabled."
        return 0
    fi

    log::info "Adicionando o include de conf-enabled ao lighttpd.conf (o padrão do Arch não tem)."
    if [[ "$DRY_RUN" == true ]]; then
        log::dry_run "echo 'include_shell \"cat ${LIGHTTPD_CONF_ENABLED}/*.conf\"' | sudo tee -a $LIGHTTPD_CONF"
    else
        printf 'include_shell "cat %s/*.conf"\n' "$LIGHTTPD_CONF_ENABLED" |
            sudo tee -a "$LIGHTTPD_CONF" >/dev/null
    fi

    log::info "Validando a configuração do lighttpd antes de subir."
    run::sudo lighttpd -tt -f "$LIGHTTPD_CONF" ||
        util::die "Configuração do lighttpd inválida. Revise $LIGHTTPD_CONF antes de continuar."
}

# The tar1090 config uses url.redirect to send /tar1090 to /tar1090/, but only
# ships module loaders for mod_alias and mod_setenv. On Debian mod_redirect is
# already on in the base config; on Arch it is not, so lighttpd parses the
# directive, warns "unknown config-key: url.redirect (ignored)" and moves on.
# The result is a 404 on exactly the slash-less URL the installer prints at the
# end of its own run.
tar1090::fix_mod_redirect() {
    local available="${LIGHTTPD_CONF_AVAILABLE}/06-mod_redirect.conf"
    local enabled="${LIGHTTPD_CONF_ENABLED}/06-mod_redirect.conf"

    [[ -d "$LIGHTTPD_CONF_AVAILABLE" ]] || {
        log::debug "Sem $LIGHTTPD_CONF_AVAILABLE; nada a fazer."
        return 0
    }

    if [[ -f "$available" && -e "$enabled" ]]; then
        log::skip "mod_redirect já habilitado."
        return 0
    fi

    log::info "Habilitando mod_redirect (o tar1090 usa url.redirect, e o Arch não carrega esse módulo por padrão)."
    run::sudo_write "$available" '# easy1090: o tar1090 usa url.redirect para a URL sem barra final.
server.modules += ( "mod_redirect" )'
    run::sudo ln -sf "$available" "$enabled"
}

tar1090::enable_lighttpd() {
    log::info "Habilitando e iniciando o lighttpd."
    run::sudo systemctl enable --now lighttpd

    [[ "$DRY_RUN" == true ]] && return 0

    run::cmd sleep 2

    local code slashless
    code="$(curl -s -o /dev/null -w '%{http_code}' "http://localhost${TAR1090_URL_PATH}" || true)"

    if [[ "$code" == "200" ]]; then
        log::success "Mapa web respondendo em http://localhost${TAR1090_URL_PATH}"

        # The installer advertises the URL without the trailing slash, so it is
        # worth confirming that the redirect really works.
        slashless="$(curl -s -o /dev/null -w '%{http_code}' "http://localhost${TAR1090_URL_PATH%/}" || true)"
        [[ "$slashless" =~ ^(200|301|302)$ ]] ||
            log::warn "http://localhost${TAR1090_URL_PATH%/} (sem barra final) devolveu $slashless; o redirect não está ativo."
    else
        log::error "Mapa web devolveu HTTP ${code:-sem resposta}."
        log::error "Cheque: systemctl status lighttpd tar1090"
        return 1
    fi
}
