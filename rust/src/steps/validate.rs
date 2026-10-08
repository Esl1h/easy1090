//! Port of lib/60-validate.sh: the final validation.
//!
//! Confirms the pipeline end to end and prints where to look. Read-only: the
//! only command executions are the state probes, and under `--dry-run` the
//! two informational probes are previewed instead of run.

use std::path::Path;
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::cfg::Config;
use crate::core::util::awk_number;
use crate::core::{log, run, svc};
use crate::steps::readsb::READSB_JSON;
use crate::t;

/// `validate::run`: services, decoding and the web map; the exit code is the
/// failure count, which the install command propagates.
pub fn run(config: &Config) -> i32 {
    log::step(&t!("val_step"));

    if run::dry_run() {
        log::dry_run("systemctl is-active readsb lighttpd tar1090");
        log::dry_run("curl -s http://localhost/tar1090/data/aircraft.json");
        return 0;
    }

    let tar1090 = config.get("COMPONENT_TAR1090") == Some("true");
    let mut failures = 0;

    if !service("readsb") {
        failures += 1;
    }
    if tar1090 && !service("lighttpd") {
        failures += 1;
    }
    if tar1090 && !service("tar1090") {
        failures += 1;
    }

    if !decoding() {
        failures += 1;
    }
    if tar1090 && !web() {
        failures += 1;
    }

    summary(failures, config);
    failures
}

fn service(unit: &str) -> bool {
    if !svc::unit_exists(unit) {
        log::error(&t!("val_unit_missing", unit));
        return false;
    }

    let active = capture_trimmed(&["systemctl", "is-active", unit]);
    let enabled = capture_trimmed(&["systemctl", "is-enabled", unit]);

    if active == "active" && enabled == "enabled" {
        log::success(&t!("val_service_ok", unit));
        return true;
    }

    let active = if active.is_empty() {
        "?"
    } else {
        active.as_str()
    };
    let enabled = if enabled.is_empty() {
        "?"
    } else {
        enabled.as_str()
    };
    log::error(&t!("val_service_bad", unit, active, enabled));
    false
}

/// An empty sky is not a failure. What proves the decoder is alive is the
/// JSON being refreshed, so we check the file's own clock instead of
/// aircraft count.
fn decoding() -> bool {
    if !Path::new(READSB_JSON).is_file() {
        log::error(&t!("val_json_missing", READSB_JSON));
        return false;
    }

    let now = capture_or(&["jq", "-r", ".now // 0", READSB_JSON], "0");
    let aircraft = capture_or(&["jq", "-r", ".aircraft | length", READSB_JSON], "0");
    let age = json_age(&now);

    if age > 60 {
        log::error(&t!("val_json_stale", age));
        return false;
    }

    log::success(&t!("val_decoding_ok", age, aircraft));

    if aircraft == "0" {
        log::info(&t!("val_zero_aircraft"));
    }

    true
}

fn web() -> bool {
    let code = capture_trimmed(&[
        "curl",
        "-s",
        "-o",
        "/dev/null",
        "-w",
        "%{http_code}",
        "http://localhost/tar1090/",
    ]);

    if code != "200" {
        log::error(&t!(
            "val_web_bad",
            if code.is_empty() { "?" } else { code.as_str() }
        ));
        return false;
    }

    // `curl -sf ... -o /dev/null`, with the failure message left to curl.
    let data_ok = run::capture(
        [
            "curl",
            "-sf",
            "http://localhost/tar1090/data/aircraft.json",
            "-o",
            "/dev/null",
        ],
        Stdio::inherit(),
    )
    .map(|out| out.status.success())
    .unwrap_or(false);

    if data_ok {
        log::success(&t!("val_web_ok"));
        true
    } else {
        log::error(&t!("val_web_data_bad"));
        false
    }
}

fn summary(failures: i32, config: &Config) {
    let ip = crate::cmd::local_ip();

    eprintln!();
    if failures == 0 {
        log::success(&t!("val_all_ok"));
    } else {
        log::error(&t!("val_failures", failures));
    }

    eprintln!("\n{}{}{}", log::bold(), t!("val_howto"), log::reset());
    eprintln!("  {:<32} {}", "viewadsb", t!("val_howto_viewadsb"));
    eprintln!(
        "  {:<32} {}",
        format!(
            "nc localhost {}",
            config.get("NET_SBS_PORT").unwrap_or("30003")
        ),
        t!("val_howto_nc"),
    );
    eprintln!("  {:<32}", format!("jq . {READSB_JSON}"));
    if config.get("COMPONENT_TAR1090") == Some("true") {
        let host = if ip.is_empty() {
            "localhost"
        } else {
            ip.as_str()
        };
        eprintln!(
            "  {:<32} {}",
            format!("http://{host}/tar1090/"),
            t!("val_howto_map"),
        );
    }
    eprintln!();
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

/// `$(cmd ... 2>/dev/null || true)`: stdout with trailing newlines
/// stripped, empty when the program cannot be spawned.
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
/// is appended to whatever the command printed; a command that cannot be
/// spawned lands in the fallback.
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
