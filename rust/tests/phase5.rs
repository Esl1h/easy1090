//! Integration checks of the phase 5 mutating commands, all through
//! `--dry-run` so nothing on the machine is ever touched: no sudo, no
//! pacman, no file writes. The expectations are loose (message markers, not
//! byte-exact output) so they hold on any distro, the CI runner included;
//! the byte-for-byte parity against the bash is proven by the container diff
//! matrix, which needs an Arch image and cannot run here.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_easy1090"));
    cmd.env("LC_ALL", "C")
        .env("LC_MESSAGES", "C")
        .env("LANG", "C");
    cmd
}

fn stderr(args: &[&str]) -> String {
    let out = bin().args(args).output().unwrap();
    String::from_utf8(out.stderr).unwrap()
}

/// The binary resolves its config next to its working directory (the rust/
/// crate root under cargo test). These tests create one so the config-driven
/// paths are exercised with the real files, then remove it again.
struct SeededConfig {
    config: PathBuf,
    example: PathBuf,
}

impl SeededConfig {
    fn new() -> SeededConfig {
        let config = PathBuf::from("install.conf");
        let example = PathBuf::from("install.conf.example");
        let content = "UI_LANGUAGE=\"en\"\nRECEIVER_LAT=\"-23.58\"\nRECEIVER_LON=\"-46.55\"\n\
                       FEEDER_ASKED=\"true\"\nFEEDER_ADSBEXCHANGE=\"false\"\n\
                       FEEDER_AIRPLANESLIVE=\"false\"\n";
        fs::write(&config, content).unwrap();
        fs::write(&example, content).unwrap();
        SeededConfig { config, example }
    }
}

impl Drop for SeededConfig {
    fn drop(&mut self) {
        fs::remove_file(&self.config).ok();
        fs::remove_file(&self.example).ok();
    }
}

#[test]
fn update_dry_run_previews_yay_and_skips_readsb_when_absent() {
    // No sudo under --dry-run, and a machine without pacman (or without the
    // package) degrades to the "not installed" warning exactly like the
    // bash: the readsb step never runs git ls-remote.
    let err = stderr(&["--lang", "en", "update", "--dry-run"]);

    assert!(err.contains("==> AUR packages tracked by yay\n"), "{err}");
    assert!(err.contains("[DRY  ] yay -Syu --devel --noconfirm\n"));
    assert!(err.contains("==> readsb (built outside yay)\n"));
    assert!(err.contains("==> Services\n"));
    assert!(err.contains("Update complete."));
}

#[test]
fn feed_status_dry_run_lists_the_networks_without_sudo() {
    let err = stderr(&["--lang", "en", "feed", "--status", "--dry-run"]);

    assert!(err.contains("==> Feeds\n"), "{err}");
    assert!(err.contains("Available networks:"));
    assert!(err.contains("adsbexchange"));
    assert!(err.contains("airplaneslive"));
    assert!(err.contains("==> Stats package\n"));
    assert!(err.contains("==> Your feeder\n"));
}

#[test]
fn feed_enable_dry_run_flips_the_config_and_reconverges() {
    let _seeded = SeededConfig::new();
    let err = stderr(&["--lang", "en", "feed", "adsbexchange", "--dry-run"]);

    assert!(err.contains("Enabling the feed."), "{err}");
    // The persist preview shows the sed the real run would issue.
    assert!(err.contains("sed -i 's|^FEEDER_ADSBEXCHANGE=.*|FEEDER_ADSBEXCHANGE=\"true\"|'"));
    // readsb is not installed on this machine, so the guard message shows.
    assert!(err.contains("readsb is not installed yet"));
    // The stats package flow follows, all previewed.
    assert!(err.contains("==> Stats package\n"));
    assert!(err.contains("git clone --depth 1 https://github.com/adsbexchange/adsbexchange-stats"));
}

#[test]
fn feed_disable_dry_run_skips_when_already_off() {
    let _seeded = SeededConfig::new();
    let err = stderr(&["--lang", "en", "feed", "--disable", "--dry-run"]);

    assert!(err.contains("Feed is already disabled."), "{err}");
}

#[test]
fn service_commands_dry_run_skip_missing_units() {
    for action in ["start", "stop", "restart"] {
        let err = stderr(&["--lang", "en", action, "--dry-run"]);
        assert!(err.contains("==> Services\n"), "{action}: {err}");
        assert!(
            err.contains("readsb is not installed; skipping."),
            "{action}: {err}"
        );
        assert!(err.contains("Done."), "{action}: {err}");
    }
}

#[test]
fn stop_dry_run_reverses_the_unit_order() {
    // The stop order is top-down: tar1090, lighttpd, readsb.
    let err = stderr(&["--lang", "en", "stop", "--dry-run"]);
    let tar = err.find("tar1090 is not installed").unwrap_or(usize::MAX);
    let readsb = err.find("readsb is not installed").unwrap_or(usize::MAX);
    assert!(
        tar < readsb,
        "stop must mention tar1090 before readsb: {err}"
    );
}

#[test]
fn uninstall_dry_run_walks_the_plan_without_touching_anything() {
    let err = stderr(&["--lang", "en", "uninstall", "--dry-run"]);

    assert!(err.contains("What will be removed:"));
    assert!(err.contains("==> tar1090\n"));
    assert!(err.contains("==> readsb\n"));
    assert!(err.contains("==> RTL-SDR driver\n"));
    assert!(err.contains("==> optional\n"));
    assert!(err.contains("==> local files\n"));
    assert!(err.contains("Uninstall complete."));
}

#[test]
fn per_command_help_speaks_portuguese_too() {
    let err_pt = stderr(&["--lang", "pt", "install", "-h", "--dry-run"]);
    assert!(err_pt.contains("Modo --dry-run"), "{err_pt}");

    let out = bin()
        .args(["--lang", "pt", "install", "-h", "--dry-run"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("USO\n"), "{stdout}");
    assert!(
        stdout.ends_with("As flags acima sobrescrevem o que estiver lá.\n"),
        "{stdout}"
    );
}
