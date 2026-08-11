#!/usr/bin/env bash
#===============================================================================
# easy1090 - package manager layer for Arch and derivatives
#
# Every call to pacman/yay/makepkg lives here. Supporting another distro means
# writing a pkg-debian.sh with the same pkg::* interface, without touching the
# modules in lib/.
#
# Interface expected from any pkg-*.sh:
#   pkg::is_installed <package>
#   pkg::install <package>...
#   pkg::remove <package>
#   pkg::install_aur <package>
#   pkg::build_aur_isolated <package> <build-dir>
#
# Author: Esli
# License: MIT
#===============================================================================

[[ -n "${_EASY1090_PKG_LOADED:-}" ]] && return 0
readonly _EASY1090_PKG_LOADED=1

readonly PKG_FLAVOR="arch"

pkg::is_installed() {
    pacman -Q "$1" &>/dev/null
}

pkg::install() {
    local -a missing=()
    local package

    for package in "$@"; do
        pkg::is_installed "$package" || missing+=("$package")
    done

    if [[ ${#missing[@]} -eq 0 ]]; then
        log::skip "Já instalado: $*"
        return 0
    fi

    log::info "Instalando via pacman: ${missing[*]}"
    run::sudo pacman -S --needed --noconfirm "${missing[@]}"
}

# Removal is always an explicit step, never a side effect of --noconfirm.
# pacman answers "N" to the conflict prompt under --noconfirm, which silently
# aborts the install instead of replacing the conflicting package.
pkg::remove() {
    local package="$1"

    if ! pkg::is_installed "$package"; then
        log::skip "Não instalado, nada a remover: $package"
        return 0
    fi

    log::warn "Removendo pacote conflitante: $package"
    run::sudo pacman -R --noconfirm "$package"
}

pkg::install_aur() {
    local package="$1"

    if pkg::is_installed "$package"; then
        log::skip "Já instalado: $package"
        return 0
    fi

    util::require_command yay
    log::info "Instalando via AUR: $package"
    run::cmd yay -S --answerclean All --answerdiff None --removemake --noconfirm "$package"
}

# Builds an AUR package outside ~/.cache/yay.
#
# yay resets the PKGBUILD to the upstream state on every run (correct anti
# tampering behavior), so any patch applied inside the cache is lost. Building
# from a copy is the only way to keep a prepare() patch, and we keep the habit
# even for packages that currently build clean, because the next GCC release
# tends to break exactly these unmaintained C projects.
pkg::build_aur_isolated() {
    local package="$1"
    local build_dir="$2"

    # Cloning straight from the AUR instead of `yay -G`: it lands exactly where
    # we ask, with no dependency on the helper's current working directory.
    if [[ -d "$build_dir" ]]; then
        log::info "Limpando build anterior: $build_dir"
        run::cmd rm -rf "$build_dir"
    fi

    run::cmd mkdir -p "$(dirname "$build_dir")"
    log::info "Clonando PKGBUILD de $package"
    run::cmd git clone --depth 1 "https://aur.archlinux.org/${package}.git" "$build_dir"
}

pkg::makepkg_install() {
    local build_dir="$1"

    log::info "Compilando e instalando ($build_dir)"

    if [[ "$DRY_RUN" == true ]]; then
        log::dry_run "cd $build_dir"
        log::dry_run "makepkg -si --noconfirm --cleanbuild"
        return 0
    fi

    [[ -d "$build_dir" ]] || util::die "Diretório de build não encontrado: $build_dir"

    (
        cd "$build_dir" || exit 1
        run::cmd makepkg -si --noconfirm --cleanbuild
    )
}
