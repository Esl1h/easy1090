//! Integration checks of the read-only `open` command: the listing goes to
//! stderr with exit 0, an unknown target exits 1, and missing components
//! degrade with the same error as the bash instead of panicking.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_easy1090"));
    cmd.env("LC_ALL", "C")
        .env("LC_MESSAGES", "C")
        .env("LANG", "C");
    cmd
}

const LISTING_EN: &str = "\nAvailable targets:\n  viewadsb    live table in the terminal (ncurses)\n  sbs         decoded message stream (CSV)\n  map         web map in the browser\n  sdrpp       SDR++ (graphical)\n  satdump     SatDump (graphical)\n\n";

/// A PATH directory holding nothing but an `nc` stub, so the dry-run
/// preview path never depends on what the machine has installed.
fn nc_stub_dir(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("easy1090-open-test-{}-{name}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let nc = dir.join("nc");
    fs::write(&nc, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&nc, fs::Permissions::from_mode(0o755)).unwrap();
    dir
}

#[test]
fn no_target_lists_the_targets_on_stderr() {
    let out = bin().arg("open").output().unwrap();
    assert!(out.status.success());
    assert!(out.stdout.is_empty());
    assert_eq!(String::from_utf8(out.stderr).unwrap(), LISTING_EN);
}

#[test]
fn no_target_speaks_portuguese() {
    let out = bin().args(["--lang", "pt", "open"]).output().unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Alvos disponíveis:"));
    assert!(stderr.contains("mapa web no navegador"));
}

#[test]
fn unknown_target_errors_and_lists_on_stderr() {
    let out = bin().args(["open", "frobnicate"]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.starts_with("[ERROR] Unknown target: frobnicate\n"));
    assert!(stderr.ends_with(LISTING_EN));
}

#[test]
fn missing_components_error_without_panicking() {
    let out = bin()
        .env("PATH", "/nonexistent-easy1090-test")
        .args(["open", "viewadsb"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Command not found: viewadsb. Is the component installed?"));

    let out = bin()
        .env("PATH", "/nonexistent-easy1090-test")
        .args(["open", "nc"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Command not found: nc. Is the component installed?"));
}

#[test]
fn aliases_resolve_to_the_component_name() {
    let out = bin()
        .env("PATH", "/nonexistent-easy1090-test")
        .args(["open", "view"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Command not found: viewadsb. Is the component installed?"));
}

#[test]
fn gui_without_a_graphical_session_errors() {
    let out = bin()
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .args(["open", "sdrpp"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains(
        "No graphical session ($DISPLAY/$WAYLAND_DISPLAY empty); cannot open sdrpp here."
    ));
}

#[test]
fn sbs_dry_run_previews_the_default_port() {
    let dir = nc_stub_dir("default-port");
    let out = bin()
        .env("PATH", dir.as_os_str())
        .env_remove("NET_SBS_PORT")
        .args(["--dry-run", "open", "sbs"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("[INFO ] Running: nc localhost 30003\n"));
    assert!(stderr.contains("[DRY  ] nc localhost 30003\n"));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn sbs_reads_the_port_from_the_environment_only() {
    // status/open never load install.conf, so the port comes from the
    // environment with a 30003 default, exactly like the bash.
    let dir = nc_stub_dir("env-port");
    let out = bin()
        .env("PATH", dir.as_os_str())
        .env("NET_SBS_PORT", "12345")
        .args(["--dry-run", "open", "sbs"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("[DRY  ] nc localhost 12345\n"));
    fs::remove_dir_all(&dir).ok();
}
