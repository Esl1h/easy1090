//! Port of lib/cmd-uninstall.sh: the uninstall command.
//!
//! Best effort reversal of what install did, and nothing else. It removes
//! the services, configs and packages easy1090 installed, and deliberately
//! leaves alone anything shared with the rest of the system.
//!
//! Paths come from the install steps, so there is a single source of truth.

use std::path::Path;

use crate::core::util::home;
use crate::core::{confirm, log, run, sudo, svc, util};
use crate::pkg::{self, Backend};
use crate::steps::driver::DVB_BLACKLIST;
use crate::steps::optional;
use crate::steps::readsb::{READSB_DEFAULTS, READSB_PACKAGE, READSB_UDEV_RULE};
use crate::steps::tar1090::{
    MOD_REDIRECT_AVAILABLE, MOD_REDIRECT_ENABLED, TAR1090_DEFAULTS, TAR1090_PATH, TAR1090_UNINSTALL,
};
use crate::t;

/// `cmd::uninstall`: the plan, the confirmation, then the steps in reverse
/// order of the install: web layer first, hardware last. The banner is
/// printed by the entrypoint, like the bash main() does.
pub fn run(args: &[&str], config_file: &Path) -> i32 {
    let keep_packages = match parse_args(args) {
        Some(keep_packages) => keep_packages,
        None => return 1,
    };

    plan();

    if !confirm::confirm(&t!("un_confirm")) {
        util::die(&t!("un_aborted"));
    }

    sudo::init();
    let backend = pkg::arch::Arch;

    uninstall_tar1090();
    uninstall_readsb(&backend, keep_packages);
    uninstall_driver(&backend, keep_packages);
    uninstall_optional(&backend, keep_packages);
    uninstall_local_files(config_file);

    eprintln!();
    log::success(&t!("un_done"));
    log::info(&t!("un_users_note"));

    sudo::cleanup();
    0
}

/// uninstall::parse_args: --keep-packages and -h; unknown options fail the
/// command.
fn parse_args(args: &[&str]) -> Option<bool> {
    let mut keep_packages = false;
    for arg in args {
        match *arg {
            "--keep-packages" => keep_packages = true,
            "-h" | "--help" => {
                print!("{}", t!("un_usage", crate::VERSION));
                std::process::exit(0);
            }
            other => {
                log::error(&t!("cli_unknown_opt", other));
                return None;
            }
        }
    }
    Some(keep_packages)
}

fn plan() {
    eprintln!("\n{}{}{}", log::bold(), t!("un_plan"), log::reset());
    eprint!(
        "{}\n{}\n{}\n{}\n{}\n\n",
        t!("un_plan_driver"),
        t!("un_plan_readsb"),
        t!("un_plan_tar1090"),
        t!("un_plan_optional"),
        t!("un_plan_local"),
    );
    eprint!("{}\n\n", t!("un_keep"));
}

/// uninstall::rm_path: `sudo rm -rf` when the path exists, a skip line
/// otherwise.
fn rm_path(path: &str) {
    if !Path::new(path).exists() {
        log::skip(&t!("un_absent", path));
        return;
    }

    run::sudo(["rm", "-rf", path]);
    if !run::dry_run() {
        log::success(&t!("un_removed", path));
    }
}

fn stop_service(unit: &str) {
    if !svc::unit_exists(unit) {
        return;
    }

    log::info(&t!("un_stopping", unit));
    run::sudo(["systemctl", "disable", "--now", unit]);
}

fn remove_package(backend: &dyn Backend, package: &str, keep_packages: bool) {
    if keep_packages {
        log::skip(&t!("un_pkg_kept"));
        return;
    }

    backend.remove(package);
}

/// Upstream ships its own uninstaller and knows best which files it created,
/// so we call it and only clean up what easy1090 added on top.
fn uninstall_tar1090() {
    log::step(&t!("un_step_tar1090"));

    if Path::new(TAR1090_UNINSTALL).is_file() {
        log::info(&t!("un_upstream", TAR1090_UNINSTALL));
        run::sudo(["bash", TAR1090_UNINSTALL]);
    } else {
        log::warn(&t!("un_upstream_missing"));
        stop_service("tar1090");
        rm_path(TAR1090_PATH);
    }

    // Left behind by upstream on purpose, and ours to clean.
    rm_path(TAR1090_DEFAULTS);

    // easy1090 additions.
    rm_path(MOD_REDIRECT_ENABLED);
    rm_path(MOD_REDIRECT_AVAILABLE);
    lighttpd_include();
}

/// Removes only the line we appended. The Arch default lighttpd.conf never
/// had it, so taking it out restores the shipped state.
fn lighttpd_include() {
    let conf = "/etc/lighttpd/lighttpd.conf";
    if !Path::new(conf).is_file() {
        return;
    }
    let has_include = std::fs::read_to_string(conf)
        .map(|content| content.contains("include_shell \"cat /etc/lighttpd/conf-enabled"))
        .unwrap_or(false);
    if !has_include {
        return;
    }

    run::sudo([
        "sed",
        "-i",
        "\\#include_shell \"cat /etc/lighttpd/conf-enabled#d",
        conf,
    ]);
    if !run::dry_run() {
        log::success(&t!("un_include_removed"));
    }

    if svc::is_active("lighttpd") {
        log::info(&t!("un_lighttpd_restart"));
        run::sudo(["systemctl", "restart", "lighttpd"]);
    }
}

fn uninstall_readsb(backend: &dyn Backend, keep_packages: bool) {
    log::step(&t!("un_step_readsb"));

    stop_service("readsb");
    remove_package(backend, READSB_PACKAGE, keep_packages);
    rm_path(READSB_DEFAULTS);
    rm_path(READSB_UDEV_RULE);

    // The dry-run previews the command; a real run ignores a failure.
    run::sudo(["udevadm", "control", "--reload-rules"]);
}

fn uninstall_driver(backend: &dyn Backend, keep_packages: bool) {
    log::step(&t!("un_step_driver"));

    remove_package(backend, "rtl-sdr-blog-git", keep_packages);
    rm_path(DVB_BLACKLIST);
}

fn uninstall_optional(backend: &dyn Backend, keep_packages: bool) {
    log::step(&t!("un_step_optional"));

    let mut found = false;

    if backend.is_installed(optional::SDRPP_PACKAGE) {
        remove_package(backend, optional::SDRPP_PACKAGE, keep_packages);
        found = true;
    }
    if backend.is_installed(optional::SATDUMP_PACKAGE) {
        remove_package(backend, optional::SATDUMP_PACKAGE, keep_packages);
        found = true;
    }

    if !found {
        log::skip(&t!("un_nothing"));
    }
}

fn uninstall_local_files(config_file: &Path) {
    log::step(&t!("un_step_local"));

    let cache = format!("{}/.cache/easy1090", home());
    if Path::new(&cache).is_dir() {
        run::cmd(["rm", "-rf", &cache]);
        if !run::dry_run() {
            log::success(&t!("un_removed", &cache));
        }
    } else {
        log::skip(&t!("un_absent", &cache));
    }

    // The config holds the user's coordinates, so it is never removed
    // silently.
    if config_file.is_file() && confirm::confirm(&t!("un_config_ask")) {
        run::cmd(["rm", "-f", config_file.to_str().unwrap_or_default()]);
        if !run::dry_run() {
            log::success(&t!("un_removed", config_file.display()));
        }
    }
}
