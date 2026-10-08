//! Service state checks shared by the install steps and the mutating
//! commands: the `svc::predates_file` helper from lib/common.sh plus the
//! `systemctl` probes the bash spells out at each call site.
//!
//! These are all read-only and run in full under `--dry-run`, exactly like
//! the bash, so the previewed restart decision matches the real one.

use std::process::Stdio;

use crate::core::run;

/// `systemctl is-active --quiet <unit> 2>/dev/null`; anything but a clean
/// success reads as inactive, which is what a missing systemd does too.
pub fn is_active(unit: &str) -> bool {
    run::capture(["systemctl", "is-active", "--quiet", unit], Stdio::null())
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// `systemctl is-enabled --quiet <unit> 2>/dev/null`.
pub fn is_enabled(unit: &str) -> bool {
    run::capture(["systemctl", "is-enabled", "--quiet", unit], Stdio::null())
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// `systemctl list-unit-files <unit>.service &>/dev/null`: true when the
/// unit file exists at all, whether or not it is enabled.
pub fn unit_exists(unit: &str) -> bool {
    let unit_file = format!("{unit}.service");
    run::capture(["systemctl", "list-unit-files", &unit_file], Stdio::null())
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// True when the unit has been running since before the file was last
/// written, which means it cannot possibly have loaded it.
///
/// This closes the gap that "the config file exists" leaves open: a previous
/// run may have written it and skipped the restart, so presence alone never
/// proves the daemon is actually using it.
pub fn predates_file(unit: &str, file: &str) -> bool {
    if !std::path::Path::new(file).is_file() {
        return false;
    }
    if !is_active(unit) {
        return false;
    }

    let started = capture_trimmed(&[
        "systemctl",
        "show",
        unit,
        "-p",
        "ActiveEnterTimestamp",
        "--value",
    ]);
    if started.is_empty() {
        return false;
    }

    // The bash hands the timestamp to `date -d ... +%s`; that parse is kept
    // external for the same reason every other check is: the bash is the
    // contract, and `date` is what it runs.
    let started_secs = match capture_number(&["date", "-d", &started, "+%s"]) {
        Some(secs) => secs,
        None => return false,
    };
    let file_mtime = match capture_number(&["stat", "-c", "%Y", file]) {
        Some(mtime) => mtime,
        None => return false,
    };

    started_secs < file_mtime
}

/// `$(cmd ... 2>/dev/null)`: stdout with trailing newlines stripped, empty
/// when the program cannot be spawned.
fn capture_trimmed(args: &[&str]) -> String {
    run::capture(args, Stdio::null())
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .trim_end_matches('\n')
                .to_string()
        })
        .unwrap_or_default()
}

/// The output of a capture parsed as an integer; the bash `2>/dev/null`
/// plus `|| return 1` on the substitution reads as None.
fn capture_number(args: &[&str]) -> Option<i64> {
    let out = run::capture(args, Stdio::null())?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The container and CI shapes: no systemd, so is-active fails and the
    /// predates check stops there without touching the timestamp commands.
    #[test]
    fn predates_without_an_active_unit_is_false() {
        // A file that certainly exists, so the first check passes and the
        // service state is what decides.
        assert!(!predates_file(
            "easy1090-no-such-unit-42",
            "/etc/os-release"
        ));
    }
}
