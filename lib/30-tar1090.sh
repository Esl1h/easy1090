#!/usr/bin/env bash
#===============================================================================
# easy1090 - tar1090 (live web map)
#
# There is no tar1090 package in the AUR. Upstream ships an install.sh written
# for Debian and Raspberry Pi OS, which we vendor under vendor/ instead of
# piping it from the network on every run.
#
# Three Arch specific gaps this module closes, all silent failures:
#   - lighttpd.conf on Arch is minimal and never includes conf-enabled, so the
#     config the tar1090 installer drops there is simply never read.
#   - mod_redirect is not loaded, so url.redirect is ignored and the slash-less
#     URL 404s (the very URL upstream prints when it finishes).
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
    log::step "$(t tar_step)"

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
        log::skip "$(t tar_confd_ok "$LIGHTTPD_CONF_D")"
    else
        log::info "$(t tar_confd_create "$LIGHTTPD_CONF_D")"
        run::sudo mkdir -p "$LIGHTTPD_CONF_D"
    fi
}

# The vendored copy is pinned by checksum. Updating it is a deliberate act:
# refresh the file, then update TAR1090_INSTALLER_SHA256 in install.conf.
tar1090::verify_vendored() {
    [[ -f "$TAR1090_INSTALLER" ]] || util::die "$(t tar_vendor_missing "$TAR1090_INSTALLER")"

    local actual
    actual="$(sha256sum "$TAR1090_INSTALLER" | cut -d' ' -f1)"

    if [[ -z "$TAR1090_INSTALLER_SHA256" ]]; then
        log::warn "$(t tar_pin_missing)"
        log::warn "$(t tar_pin_current "$actual")"
        log::warn "$(t tar_pin_hint)"
        return 0
    fi

    if [[ "$actual" != "$TAR1090_INSTALLER_SHA256" ]]; then
        log::error "$(t tar_pin_mismatch)"
        log::error "$(t tar_pin_expected "$TAR1090_INSTALLER_SHA256")"
        log::error "$(t tar_pin_got "$actual")"
        util::die "$(t tar_pin_abort)"
    fi

    log::success "$(t tar_pin_ok)"
}

tar1090::install() {
    if tar1090::is_done; then
        log::skip "$(t tar_service_ok)"
        return 0
    fi

    log::info "$(t tar_running)"
    run::sudo bash "$TAR1090_INSTALLER"
}

# Without this include, everything the tar1090 installer wrote is dead config.
tar1090::fix_lighttpd_include() {
    [[ -f "$LIGHTTPD_CONF" ]] || {
        log::warn "$(t tar_conf_missing "$LIGHTTPD_CONF")"
        return 0
    }

    if grep -q "conf-enabled" "$LIGHTTPD_CONF"; then
        log::skip "$(t tar_include_ok)"
        return 0
    fi

    log::info "$(t tar_include_add)"
    if [[ "$DRY_RUN" == true ]]; then
        log::dry_run "echo 'include_shell \"cat ${LIGHTTPD_CONF_ENABLED}/*.conf\"' | sudo tee -a $LIGHTTPD_CONF"
    else
        printf 'include_shell "cat %s/*.conf"\n' "$LIGHTTPD_CONF_ENABLED" |
            sudo tee -a "$LIGHTTPD_CONF" >/dev/null
    fi

    log::info "$(t tar_lighttpd_check)"
    run::sudo lighttpd -tt -f "$LIGHTTPD_CONF" ||
        util::die "$(t tar_lighttpd_invalid "$LIGHTTPD_CONF")"
}

# The tar1090 config uses url.redirect to send /tar1090 to /tar1090/, but only
# ships module loaders for mod_alias and mod_setenv. On Debian mod_redirect is
# already on in the base config; on Arch it is not, so lighttpd parses the
# directive, warns "unknown config-key: url.redirect (ignored)" and moves on.
tar1090::fix_mod_redirect() {
    local available="${LIGHTTPD_CONF_AVAILABLE}/06-mod_redirect.conf"
    local enabled="${LIGHTTPD_CONF_ENABLED}/06-mod_redirect.conf"

    [[ -d "$LIGHTTPD_CONF_AVAILABLE" ]] || return 0

    if [[ -f "$available" && -e "$enabled" ]]; then
        log::skip "$(t tar_redirect_ok)"
        return 0
    fi

    log::info "$(t tar_redirect_add)"
    run::sudo_write "$available" "$(printf '%s\nserver.modules += ( "mod_redirect" )' "$(t tar_redirect_comment)")"
    run::sudo ln -sf "$available" "$enabled"
}

tar1090::enable_lighttpd() {
    log::info "$(t tar_lighttpd_enable)"
    run::sudo systemctl enable --now lighttpd

    [[ "$DRY_RUN" == true ]] && return 0

    run::cmd sleep 2

    local code slashless url="http://localhost${TAR1090_URL_PATH}"
    code="$(curl -s -o /dev/null -w '%{http_code}' "$url" || true)"

    if [[ "$code" == "200" ]]; then
        log::success "$(t tar_web_ok "$url")"

        # The installer advertises the URL without the trailing slash, so it is
        # worth confirming that the redirect really works.
        slashless="$(curl -s -o /dev/null -w '%{http_code}' "${url%/}" || true)"
        [[ "$slashless" =~ ^(200|301|302)$ ]] ||
            log::warn "$(t tar_web_slashless "${url%/}" "$slashless")"
    else
        log::error "$(t tar_web_fail "${code:-?}")"
        log::error "$(t tar_web_hint)"
        return 1
    fi
}
