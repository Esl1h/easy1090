//! Package manager layer, mirroring the contract of lib/pkg-arch.sh.
//!
//! Every call to pacman/yay/makepkg lives behind the `Backend` trait, so a
//! Debian backend remains possible without touching the steps or commands:
//! an implementation of the same interface, selected at one place, and the
//! rest of the tree never names pacman.
//!
//! `is_installed` is a state read: it runs in full even under `--dry-run`
//! and never previews, exactly like the bash probes the package database
//! directly. The mutating calls go through the run layer, so the preview
//! shows the real commands.

pub mod arch;

use crate::core::{log, run, util};
use crate::t;

/// The interface expected from any package backend, the `pkg::*` functions
/// of the bash.
pub trait Backend {
    /// `pkg::is_installed <package>`: a database probe, not a mutation.
    fn is_installed(&self, package: &str) -> bool;

    /// `pkg::install <package>...`: skip the already installed ones, install
    /// the rest with pacman. The skip message lists every requested package,
    /// `${*}` in the bash.
    fn install(&self, packages: &[&str]) {
        let missing: Vec<&str> = packages
            .iter()
            .filter(|package| !self.is_installed(package))
            .copied()
            .collect();

        if missing.is_empty() {
            log::skip(&t!("pkg_installed", packages.join(" ")));
            return;
        }

        log::info(&t!("pkg_pacman", missing.join(" ")));
        let argv: Vec<&str> = ["pacman", "-S", "--needed", "--noconfirm"]
            .into_iter()
            .chain(missing)
            .collect();
        run::sudo(argv);
    }

    /// `pkg::remove <package>`: an explicit step, never a side effect of
    /// --noconfirm, because pacman answers "N" to the conflict prompt under
    /// it and silently aborts instead of replacing the conflicting package.
    fn remove(&self, package: &str) {
        if !self.is_installed(package) {
            log::skip(&t!("pkg_absent", package));
            return;
        }

        log::warn(&t!("pkg_removing", package));
        run::sudo(["pacman", "-R", "--noconfirm", package]);
    }

    /// `pkg::install_aur <package>`. Deliberately without --removemake: AUR
    /// packages routinely list the same library in makedepends and
    /// optdepends, and yay only sees the make side; removing it leaves the
    /// program installed with broken plugins. The cost is leaving build
    /// tooling on disk, reclaimable with `yay -Yc`.
    fn install_aur(&self, package: &str) {
        if self.is_installed(package) {
            log::skip(&t!("pkg_installed", package));
            return;
        }

        if !util::have_cmd("yay") {
            util::die(&t!("cmd_required", "yay"));
        }
        log::info(&t!("pkg_aur", package));

        run::cmd([
            "yay",
            "-S",
            "--answerclean",
            "All",
            "--answerdiff",
            "None",
            "--noconfirm",
            package,
        ]);
    }

    /// `pkg::build_aur_isolated <package> <build-dir>`: a build outside
    /// ~/.cache/yay, so a prepare() patch survives the helper's PKGBUILD
    /// reset. Cloning straight from the AUR lands exactly where we ask.
    fn build_aur_isolated(&self, package: &str, build_dir: &str) {
        if std::path::Path::new(build_dir).is_dir() {
            log::info(&t!("pkg_clean_build", build_dir));
            run::cmd(["rm", "-rf", build_dir]);
        }

        let parent = parent_of(build_dir);
        run::cmd(["mkdir", "-p", &parent]);
        log::info(&t!("pkg_cloning", package));
        let url = format!("https://aur.archlinux.org/{package}.git");
        run::cmd(["git", "clone", "--depth", "1", &url, build_dir]);
    }

    /// `pkg::makepkg_install <build-dir>`: makepkg inside the build
    /// directory. The dry-run preview prints the `cd` and the command as two
    /// lines, exactly like the bash.
    fn makepkg_install(&self, build_dir: &str) {
        log::info(&t!("pkg_building", build_dir));

        if run::dry_run() {
            log::dry_run(&format!("cd {build_dir}"));
            log::dry_run("makepkg -si --noconfirm --cleanbuild");
            return;
        }

        if !std::path::Path::new(build_dir).is_dir() {
            util::die(&t!("pkg_build_missing", build_dir));
        }

        run::cmd_in(build_dir, ["makepkg", "-si", "--noconfirm", "--cleanbuild"]);
    }
}

/// `$(dirname "$build_dir")` for the mkdir step; the build dir is always an
/// absolute path ending in the package name, so a slash split is exact.
pub(crate) fn parent_of(build_dir: &str) -> String {
    match build_dir.rfind('/') {
        Some(index) if index > 0 => build_dir[..index].to_string(),
        Some(_) => "/".to_string(),
        None => ".".to_string(),
    }
}
