//! Integration checks of the read-only `status` command. On a machine where
//! the stack, systemd or pacman are missing, the report degrades with the
//! same "absent" rows as the bash and still exits 0.

use std::process::Command;

fn bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_easy1090"));
    cmd.env("LC_ALL", "C")
        .env("LC_MESSAGES", "C")
        .env("LANG", "C");
    cmd
}

fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[test]
fn report_goes_to_stdout_and_exits_zero() {
    let out = bin().arg("status").output().unwrap();
    assert!(out.status.success());
    assert!(
        out.stderr.is_empty(),
        "status must not write to stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with(&format!("\neasy1090 {} - status\n\n", version())));
    for section in ["Hardware and driver", "Decoding", "Web", "Optional"] {
        assert!(stdout.contains(section), "missing section: {section}");
    }
    assert!(stdout.ends_with("\n\n"), "final blank line missing");
}

#[test]
fn status_speaks_portuguese() {
    let out = bin().args(["--lang", "pt", "status"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    for section in ["Hardware e driver", "Decodificação", "Opcionais"] {
        assert!(stdout.contains(section), "missing section: {section}");
    }
    // The accented label is padded by measured width, not by byte length.
    assert!(stdout.contains("  decodificação   "));
}

#[test]
fn status_is_read_only_and_ignores_options() {
    // status ignores --dry-run and extra arguments: same header, no stderr,
    // no DRY previews.
    let out = bin()
        .args(["status", "--dry-run", "extra"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with(&format!("\neasy1090 {} - status\n\n", version())));
}

#[test]
fn degrades_gracefully_without_the_tools() {
    // A search path that cannot contain lsusb, systemctl, jq or pacman: the
    // bash prints the "absent" rows and exits 0, and so must this.
    let out = bin()
        .env("PATH", "/nonexistent-easy1090-test")
        .arg("status")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("lsusb not installed"));
    assert!(stdout.contains("unit not installed"));
}
