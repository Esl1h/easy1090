//! Port of lib/cmd-update.sh: the update command.
//!
//! Versions only. Configuration and services are `install`'s job, and the
//! split is deliberate: updating should never silently start a 45 minute
//! rebuild as a side effect of converging a config file.
//!
//! There are two update paths because there are two install paths:
//!
//!   yay packages (driver, SDR++, SatDump) need `yay -Syu --devel`. A plain
//!   `yay -Syu` never updates a -git package, because it compares against
//!   the version declared in the AUR, which does not move when upstream
//!   commits.
//!
//!   readsb is built with makepkg outside yay, so that a prepare() patch
//!   stays possible when a new GCC breaks the build. The price is that yay
//!   never records it in its VCS database and will not offer an update for
//!   it, even with --devel. So we compare commits ourselves.

use std::process::Stdio;

use crate::core::{confirm, log, run, sudo, svc};
use crate::pkg::{self, Backend};
use crate::steps::readsb::{build_dir, READSB_PACKAGE, READSB_UPSTREAM};
use crate::t;

/// `cmd::update`: parse, then AUR, readsb and the services that need a
/// restart. The banner is printed by the entrypoint, like the bash main()
/// does.
pub fn run(args: &[&str]) -> i32 {
    let mut skip_aur = false;
    let mut skip_readsb = false;
    if !parse_args(args, &mut skip_aur, &mut skip_readsb) {
        return 1;
    }

    sudo::init();
    let backend = pkg::arch::Arch;

    if !skip_aur {
        aur();
    }
    let readsb_changed = if skip_readsb { false } else { readsb(&backend) };
    services(readsb_changed);

    eprintln!();
    log::success(&t!("upd_done"));
    log::info(&t!("upd_hint_install"));

    sudo::cleanup();
    0
}

/// update::parse_args: --skip-aur, --skip-readsb and -h.
fn parse_args(args: &[&str], skip_aur: &mut bool, skip_readsb: &mut bool) -> bool {
    for arg in args {
        match *arg {
            "--skip-aur" => *skip_aur = true,
            "--skip-readsb" => *skip_readsb = true,
            "-h" | "--help" => {
                print!("{}", t!("upd_usage", crate::VERSION));
                std::process::exit(0);
            }
            other => {
                log::error(&t!("cli_unknown_opt", other));
                return false;
            }
        }
    }
    true
}

fn aur() {
    log::step(&t!("upd_step_aur"));
    log::info(&t!("upd_yay_note"));

    // Runs as the normal user; yay escalates on its own when it reaches
    // pacman.
    run::cmd(["yay", "-Syu", "--devel", "--noconfirm"]);
}

/// The pkgver of a VCS package ends in .g<short-sha>, which is the commit it
/// was built from. That is the only reliable record of what is actually
/// installed: `pacman -Q | awk '{print $2}' | sed -n 's/.*\.g\([0-9a-f]\{7,\}\)-.*/\1/p'`.
fn installed_commit() -> Option<String> {
    let out = run::capture(["pacman", "-Q", READSB_PACKAGE], Stdio::null())?;
    let version = String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .nth(1)?
        .to_string();
    installed_commit_from(&version)
}

/// The sed pattern over one version string: the last ".g", then seven or
/// more lowercase hex digits up to a hyphen.
fn installed_commit_from(version: &str) -> Option<String> {
    let hex = version.rfind(".g")? + 2;
    let rest = &version[hex..];
    let digits: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || ('a'..='f').contains(c))
        .collect();
    if digits.len() >= 7 && rest.as_bytes().get(digits.len()) == Some(&b'-') {
        Some(digits)
    } else {
        None
    }
}

/// `timeout 30 git ls-remote <upstream> HEAD 2>/dev/null | cut -f1`, then the
/// first seven characters.
fn upstream_commit() -> Option<String> {
    let out = run::capture(
        ["timeout", "30", "git", "ls-remote", READSB_UPSTREAM, "HEAD"],
        Stdio::null(),
    )?;
    let sha = String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()?
        .to_string();
    if sha.is_empty() {
        None
    } else {
        Some(sha.chars().take(7).collect())
    }
}

/// `update::readsb`: compare the installed commit with upstream HEAD and
/// rebuild when behind. True when the package changed.
fn readsb(backend: &dyn Backend) -> bool {
    log::step(&t!("upd_step_readsb"));

    if !backend.is_installed(READSB_PACKAGE) {
        log::warn(&t!("upd_not_installed", READSB_PACKAGE));
        return false;
    }

    let installed = installed_commit();
    let upstream = upstream_commit();

    let (Some(installed), Some(upstream)) = (installed, upstream) else {
        log::warn(&t!("upd_readsb_unknown"));
        return false;
    };

    log::info(&t!("upd_readsb_installed", &installed));
    log::info(&t!("upd_readsb_upstream", &upstream));

    // Compared by prefix: git describe may use more than seven characters
    // when a short sha would be ambiguous.
    if installed.starts_with(&upstream) || upstream.starts_with(&installed) {
        log::skip(&t!("upd_readsb_current"));
        return false;
    }

    log::info(&t!("upd_readsb_behind"));
    if !confirm::confirm(&t!("upd_readsb_confirm")) {
        log::info(&t!("upd_readsb_skipped"));
        return false;
    }

    let dir = build_dir();
    backend.build_aur_isolated(READSB_PACKAGE, &dir);
    backend.makepkg_install(&dir);

    if !run::dry_run() {
        log::success(&t!(
            "upd_readsb_rebuilt",
            installed_commit().unwrap_or_default()
        ));
    }

    true
}

/// A new binary is only in use after a restart. Config is untouched here, so
/// the only reason to restart is a package that actually changed underneath.
fn services(readsb_changed: bool) {
    log::step(&t!("upd_step_services"));

    let mut units: Vec<&str> = Vec::new();
    if readsb_changed {
        units.push("readsb");
    }

    // An AUR update can replace the lighttpd or tar1090 units and their
    // files. svc::predates_file answers the only question that matters: is
    // the running process older than what is on disk?
    if svc::predates_file("lighttpd", "/usr/lib/systemd/system/lighttpd.service") {
        units.push("lighttpd");
    }
    if svc::predates_file("tar1090", "/usr/lib/systemd/system/tar1090.service") {
        units.push("tar1090");
    }

    if units.is_empty() {
        log::skip(&t!("upd_services_ok"));
        return;
    }

    run::sudo(["systemctl", "daemon-reload"]);

    for unit in units {
        log::info(&t!("upd_services_restart", unit));
        run::sudo_ok(["systemctl", "restart", unit]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_commit_extracts_the_short_sha_from_a_vcs_pkgver() {
        // The real shape: version, then .g<sha>-<pkgrel>.
        assert_eq!(
            installed_commit_from("3.14.0.106.g98765ab-1"),
            Some("98765ab".to_string())
        );
        assert_eq!(
            installed_commit_from("1.0.r1234.gdeadbee-1"),
            Some("deadbee".to_string())
        );
        // The greedy match takes the last .g, like the sed.
        assert_eq!(
            installed_commit_from("1.0.gabc1234.gdeadbee-1"),
            Some("deadbee".to_string())
        );
        // No hyphen after the sha, short sha, no .g: all unparseable.
        assert_eq!(installed_commit_from("1.0.gabcdef-1"), None);
        assert_eq!(installed_commit_from("1.0.0-1"), None);
        assert_eq!(installed_commit_from("1.0.gxyz1234-1"), None);
    }
}
