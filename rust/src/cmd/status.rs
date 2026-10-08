//! Port of lib/cmd-status.sh: a read-only snapshot of the stack.
//!
//! Same steps, messages and streams as the bash: the report goes to stdout,
//! nothing asks for sudo, and a failing `ip route` pipeline aborts silently
//! with exit 1, which is what `set -euo pipefail` does to the bash.

use std::path::Path;
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cmd::local_ip;
use crate::core::util::awk_number;
use crate::core::{log, run, util};
use crate::t;

/// `RTLSDR_USB_ID` from cmd-status.sh.
const RTLSDR_USB_ID: &str = "0bda:2838";

/// `READSB_JSON` from 20-readsb.sh, defined before the command modules load.
const READSB_JSON: &str = "/run/readsb/aircraft.json";

/// `cmd::status`: prints the whole report and exits 0, whatever is missing.
pub fn run() -> i32 {
    println!(
        "\n{}easy1090{} {} - {}\n",
        log::bold(),
        log::reset(),
        crate::VERSION,
        t!("sts_title")
    );

    println!("{}{}{}", log::bold(), t!("sts_hardware"), log::reset());
    dongle();
    package(&t!("sts_driver"), "rtl-sdr-blog-git");

    section("sts_decoding");
    package("readsb", "readsb-wiedehopf-git");
    service(&t!("sts_service"), "readsb");
    decoding();

    section("sts_web");
    service("lighttpd", "lighttpd");
    service("tar1090", "tar1090");
    web();

    section("sts_optional");
    package("SDR++", "sdrpp-git");
    package("SatDump", "satdump");

    println!();
    0
}

/// `printf "\n${BOLD}%s${RESET}\n"`: a blank line and a bold heading.
fn section(key: &str) {
    println!("\n{}{}{}", log::bold(), t!(key), log::reset());
}

/// `status::line`: two-space indent, the label padded with str::width to 16
/// columns, the state colored and left-padded to 12 bytes, then the detail.
fn line(name: &str, state: &str, detail: &str) {
    let color = if state == t!("sts_running") || state == t!("sts_installed") {
        log::green()
    } else if state == t!("sts_stopped") || state == t!("sts_absent") {
        log::red()
    } else {
        log::yellow()
    };

    let pad = 16usize.saturating_sub(str_width(name)).max(1);
    let spaces = " ".repeat(pad);

    println!(
        "  {}{}{}{}{} {}",
        name,
        spaces,
        color,
        pad_state(state),
        log::reset(),
        detail
    );
}

/// The label column counts glyphs (bytes that are not UTF-8 continuation
/// bytes), exactly like str::width in lib/common.sh.
fn str_width(s: &str) -> usize {
    s.bytes().filter(|b| !(0x80..=0xbf).contains(b)).count()
}

/// bash printf `%-12s` pads by bytes, not characters.
fn pad_state(state: &str) -> String {
    let mut padded = state.to_string();
    while padded.len() < 12 {
        padded.push(' ');
    }
    padded
}

/// `status::dongle`: lsusb probe, `?` state when the tool is missing.
fn dongle() {
    if !util::have_cmd("lsusb") {
        line("RTL-SDR", "?", &t!("sts_lsusb_missing"));
        return;
    }

    // `lsusb | grep -qi`: stderr of lsusb stays visible, like the bash.
    let found = run::capture(["lsusb"], Stdio::inherit())
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .to_lowercase()
                .contains(RTLSDR_USB_ID)
        })
        .unwrap_or(false);

    if found {
        line(
            "RTL-SDR",
            &t!("sts_installed"),
            &t!("sts_dongle_found", RTLSDR_USB_ID),
        );
    } else {
        line("RTL-SDR", &t!("sts_absent"), &t!("sts_dongle_absent"));
    }
}

/// `status::package`: `pkg::is_installed` then the `pacman -Q` line.
fn package(label: &str, name: &str) {
    if is_installed(name) {
        let detail = capture_or(&["pacman", "-Q", name], name);
        line(label, &t!("sts_installed"), &detail);
    } else {
        line(label, &t!("sts_absent"), name);
    }
}

/// `pkg::is_installed`: `pacman -Q <package> &>/dev/null`; a missing pacman
/// (non-Arch machine) reads as not installed, like the bash.
fn is_installed(name: &str) -> bool {
    run::capture(["pacman", "-Q", name], Stdio::null())
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// `status::service`: is the unit listed, then is-active/is-enabled.
fn service(label: &str, unit: &str) {
    let unit_file = format!("{unit}.service");
    let listed = run::capture(
        ["systemctl", "list-unit-files", unit_file.as_str()],
        Stdio::null(),
    )
    .map(|out| out.status.success())
    .unwrap_or(false);

    if !listed {
        line(label, &t!("sts_absent"), &t!("sts_unit_absent"));
        return;
    }

    // `$(systemctl is-active "$unit" 2>/dev/null || true)`: the stdout is
    // kept even when the command fails, since is-active prints the state and
    // exits nonzero for anything but "active".
    let active = capture_trimmed(&["systemctl", "is-active", unit]);
    let enabled = capture_trimmed(&["systemctl", "is-enabled", unit]);

    if active == "active" {
        line(label, &t!("sts_running"), &enabled);
    } else {
        line(label, &t!("sts_stopped"), &enabled);
    }
}

/// `status::decoding`: freshness of the JSON, not the aircraft count.
fn decoding() {
    if !Path::new(READSB_JSON).is_file() {
        line(
            &t!("sts_decode_row"),
            &t!("sts_absent"),
            &t!("sts_json_absent", READSB_JSON),
        );
        return;
    }

    if !util::have_cmd("jq") {
        line(&t!("sts_decode_row"), "?", &t!("sts_jq_missing"));
        return;
    }

    let now = capture_or(&["jq", "-r", ".now // 0", READSB_JSON], "0");
    let aircraft = capture_or(&["jq", "-r", ".aircraft | length", READSB_JSON], "0");
    let age = json_age(&now);

    if age <= 60 {
        line(
            &t!("sts_decode_row"),
            &t!("sts_running"),
            &t!("sts_json_fresh", age, aircraft),
        );
    } else {
        line(
            &t!("sts_decode_row"),
            &t!("sts_stopped"),
            &t!("sts_json_stale", age),
        );
    }
}

/// `status::web`: lighttpd first, then an HTTP probe of the map.
fn web() {
    let active = run::capture(
        ["systemctl", "is-active", "--quiet", "lighttpd"],
        Stdio::null(),
    )
    .map(|out| out.status.success())
    .unwrap_or(false);

    if !active {
        line(&t!("sts_map"), &t!("sts_stopped"), &t!("sts_lighttpd_down"));
        return;
    }

    let code = capture_trimmed(&[
        "curl",
        "-s",
        "-o",
        "/dev/null",
        "-w",
        "%{http_code}",
        "http://localhost/tar1090/",
    ]);

    if code == "200" {
        let ip = local_ip();
        let host = if ip.is_empty() {
            "localhost"
        } else {
            ip.as_str()
        };
        line(
            &t!("sts_map"),
            &t!("sts_running"),
            &format!("http://{host}/tar1090/"),
        );
    } else {
        let shown = if code.is_empty() { "?" } else { code.as_str() };
        line(&t!("sts_map"), &t!("sts_stopped"), &t!("sts_http", shown));
    }
}

/// `awk -v n="$now" 'BEGIN { printf "%d", systime() - n }'`: seconds since
/// the JSON timestamp, `%d` truncating toward zero.
fn json_age(now: &str) -> i64 {
    let timestamp = awk_number(now);
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0) as f64;
    (now_secs - timestamp).trunc() as i64
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

/// `$(cmd ... 2>/dev/null || printf '%s' fallback)`: on failure the fallback
/// is appended to whatever the command printed, like the `||` inside the bash
/// substitution; a command that cannot be spawned lands in the fallback.
fn capture_or(args: &[&str], fallback: &str) -> String {
    match run::capture(args, Stdio::null()) {
        Some(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if out.status.success() {
                stdout.trim_end_matches('\n').to_string()
            } else {
                format!("{stdout}{fallback}")
            }
        }
        None => fallback.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_width_counts_glyphs_like_the_bash_helper() {
        assert_eq!(str_width("serviço"), 7);
        assert_eq!(str_width("decodificação"), 13);
        assert_eq!(str_width("RTL-SDR"), 7);
    }

    #[test]
    fn states_pad_by_bytes() {
        assert_eq!(pad_state("running"), "running     ");
        assert_eq!(pad_state("123456789012"), "123456789012");
        assert_eq!(pad_state("1234567890123"), "1234567890123");
    }
}
