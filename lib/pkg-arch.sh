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
        log::skip "$(t pkg_installed "$*")"
        return 0
    fi

    log::info "$(t pkg_pacman "${missing[*]}")"
    run::sudo pacman -S --needed --noconfirm "${missing[@]}"
}

# Removal is always an explicit step, never a side effect of --noconfirm.
# pacman answers "N" to the conflict prompt under --noconfirm, which silently
# aborts the install instead of replacing the conflicting package.
pkg::remove() {
    local package="$1"

    if ! pkg::is_installed "$package"; then
        log::skip "$(t pkg_absent "$package")"
        return 0
    fi

    log::warn "$(t pkg_removing "$package")"
    run::sudo pacman -R --noconfirm "$package"
}

pkg::install_aur() {
    local package="$1"

    if pkg::is_installed "$package"; then
        log::skip "$(t pkg_installed "$package")"
        return 0
    fi

    util::require_command yay
    log::info "$(t pkg_aur "$package")"

    # Deliberately without --removemake. AUR packages routinely list the same
    # library in both makedepends and optdepends: needed to build a plugin, and
    # needed again at runtime to load it. yay only sees the make side and
    # removes it, leaving the program installed with broken plugins and no
    # error anywhere. Measured on sdrpp-git: 10 plugins lost their libraries,
    # including the audio sink.
    #
    # The cost is leaving build tooling (cmake and friends) on disk. Reclaim it
    # with `yay -Yc` if that matters more than the plugins working.
    run::cmd yay -S --answerclean All --answerdiff None --noconfirm "$package"
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
        log::info "$(t pkg_clean_build "$build_dir")"
        run::cmd rm -rf "$build_dir"
    fi

    run::cmd mkdir -p "$(dirname "$build_dir")"
    log::info "$(t pkg_cloning "$package")"
    run::cmd git clone --depth 1 "https://aur.archlinux.org/${package}.git" "$build_dir"
}

pkg::makepkg_install() {
    local build_dir="$1"

    log::info "$(t pkg_building "$build_dir")"

    if [[ "$DRY_RUN" == true ]]; then
        log::dry_run "cd $build_dir"
        log::dry_run "makepkg -si --noconfirm --cleanbuild"
        return 0
    fi

    [[ -d "$build_dir" ]] || util::die "$(t pkg_build_missing "$build_dir")"

    (
        cd "$build_dir" || exit 1
        run::cmd makepkg -si --noconfirm --cleanbuild
    )
}
