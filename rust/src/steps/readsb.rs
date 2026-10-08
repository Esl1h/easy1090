//! Port of lib/20-readsb.sh: readsb, the ADS-B decoder.
//!
//! Installs the wiedehopf fork, not the Mictronics one. The Mictronics fork
//! writes aircraft.pb (binary protobuf) and has no flag for JSON, which
//! every web frontend expects. The wiedehopf fork writes aircraft.json
//! natively, is actively maintained, and is from the same author as tar1090.

use std::path::Path;

use crate::core::cfg::Config;
use crate::core::{confirm, log, run, svc, util};
use crate::pkg::Backend;
use crate::preflight::{RTLSDR_USB_PRODUCT, RTLSDR_USB_VENDOR};
use crate::t;

pub const READSB_PACKAGE: &str = "readsb-wiedehopf-git";
const READSB_CONFLICTS: &str = "readsb-git";
/// Used by the update command to compare the installed commit with HEAD. The
/// package name above already pins which fork this is.
pub const READSB_UPSTREAM: &str = "https://github.com/wiedehopf/readsb";
/// `${HOME}/.cache/easy1090/readsb-wiedehopf-git`, the `$(dirname ...)` of
/// which is also the uninstall command's local cache path.
pub fn build_dir() -> String {
    format!("{}/.cache/easy1090/readsb-wiedehopf-git", util::home())
}
pub const READSB_DEFAULTS: &str = "/etc/default/readsb";
pub const READSB_UDEV_RULE: &str = "/etc/udev/rules.d/99-readsb-rtlsdr.rules";
const READSB_LEGACY_OVERRIDE: &str = "/etc/systemd/system/readsb.service.d/override.conf";
/// Shared with the validate step and the status command.
pub const READSB_JSON: &str = "/run/readsb/aircraft.json";

/// `readsb::run`: install, legacy override cleanup, udev rule, config and
/// enablement. False when the service does not come up, which aborts the
/// install like `set -e` does in the bash.
pub fn run(pkg: &dyn Backend, config: &Config) -> bool {
    log::step(&t!("rsb_step"));

    install(pkg);
    drop_legacy_override();
    udev_rule();
    let needs_restart = configure(config);
    enable(needs_restart)
}

fn install(pkg: &dyn Backend) {
    if pkg.is_installed(READSB_PACKAGE) {
        log::skip(&t!("rsb_installed", READSB_PACKAGE));
        return;
    }

    if pkg.is_installed(READSB_CONFLICTS) {
        log::warn(&t!("rsb_conflict", READSB_CONFLICTS, READSB_PACKAGE));
        if !confirm::confirm(&t!("rsb_conflict_confirm", READSB_CONFLICTS)) {
            util::die(&t!("rsb_conflict_abort"));
        }
        pkg.remove(READSB_CONFLICTS);
    }

    let dir = build_dir();
    pkg.build_aur_isolated(READSB_PACKAGE, &dir);
    pkg.makepkg_install(&dir);
}

/// Part 1 of the blog series created a systemd override referencing
/// $USER_OPTIONS, a variable this package does not define. Left behind it
/// breaks the new service at startup.
fn drop_legacy_override() {
    if !Path::new(READSB_LEGACY_OVERRIDE).is_file() {
        return;
    }
    let mentions_user_options = std::fs::read_to_string(READSB_LEGACY_OVERRIDE)
        .map(|content| content.contains("USER_OPTIONS"))
        .unwrap_or(false);
    if !mentions_user_options {
        return;
    }

    log::warn(&t!("rsb_legacy_override"));
    run::sudo(["rm", "-f", READSB_LEGACY_OVERRIDE]);
    // `run::sudo rmdir "$(dirname ...)" 2>/dev/null || true`: the directory
    // stays when it is not empty, silently either way.
    run::sudo_quiet(["rmdir", "/etc/systemd/system/readsb.service.d"]);
    run::sudo(["systemctl", "daemon-reload"]);
    log::success(&t!("rsb_legacy_removed"));
}

/// The stock rtl-sdr rule grants the device to the plugdev group. That is
/// enough for an interactive user (systemd-logind adds a session ACL on
/// top), which is exactly why the problem hides: the readsb service user has
/// no session and no ACL, so it still gets EACCES. Hence a dedicated rule
/// for the service group.
fn udev_rule() {
    let rule = format!(
        "SUBSYSTEM==\"usb\", ATTRS{{idVendor}}==\"{RTLSDR_USB_VENDOR}\", \
         ATTRS{{idProduct}}==\"{RTLSDR_USB_PRODUCT}\", GROUP=\"readsb\", MODE=\"0660\""
    );

    let present = std::fs::read_to_string(READSB_UDEV_RULE)
        .map(|content| content.contains("GROUP=\"readsb\""))
        .unwrap_or(false);
    if present {
        log::skip(&t!("rsb_udev_ok"));
        return;
    }

    log::info(&t!("rsb_udev_create"));
    let content = format!("{}\n{rule}", t!("rsb_udev_comment"));
    run::sudo_write(READSB_UDEV_RULE, &content);

    run::sudo(["udevadm", "control", "--reload-rules"]);
    // Reprocesses already connected devices, so no need to unplug the
    // dongle.
    run::sudo(["udevadm", "trigger"]);
}

/// Writes /etc/default/readsb from the config and returns whether the
/// service needs a restart: the file changed on this run, or the running
/// daemon predates the current file.
pub fn configure(config: &Config) -> bool {
    let [receiver_options, decoder_options, net_options, json_options] = option_strings(config);

    log::info(&t!("rsb_defaults_write", READSB_DEFAULTS));
    let content = format!(
        "{}\nRECEIVER_OPTIONS=\"{receiver_options}\"\nDECODER_OPTIONS=\"{decoder_options}\"\
         \nNET_OPTIONS=\"{net_options}\"\nJSON_OPTIONS=\"{json_options}\"",
        t!("rsb_defaults_header"),
    );
    let changed = run::sudo_write(READSB_DEFAULTS, &content);

    let mut needs_restart = changed;
    // Covers the case where a previous run wrote the config but never
    // restarted: the file is unchanged now, yet the daemon predates it.
    if svc::predates_file("readsb", READSB_DEFAULTS) {
        needs_restart = true;
    }

    needs_restart
}

/// The four option strings the config turns into, exactly as readsb::configure
/// builds them. Feeders are opt-in: sharing the receiver position with a
/// third party is a decision, not a default. The connector appends happen
/// here so the unit tests can pin every branch.
pub fn option_strings(config: &Config) -> [String; 4] {
    let value = |key, default: &str| config.get(key).unwrap_or(default).to_string();

    let receiver_options = format!(
        "--device 0 --device-type rtlsdr --gain {} --ppm {} --lat {} --lon {}",
        value("RECEIVER_GAIN", "auto"),
        value("RECEIVER_PPM", "0"),
        value("RECEIVER_LAT", ""),
        value("RECEIVER_LON", ""),
    );

    let decoder_options = format!(
        "--max-range {} --write-json-every 1",
        value("DECODER_MAX_RANGE", "450"),
    );

    let mut net_options = format!(
        "--net --net-ri-port {} --net-ro-port {} --net-sbs-port {} --net-bi-port {} \
         --net-bo-port {}",
        value("NET_RI_PORT", "30001"),
        value("NET_RO_PORT", "30002"),
        value("NET_SBS_PORT", "30003"),
        value("NET_BI_PORT", "30004,30104"),
        value("NET_BO_PORT", "30005"),
    );

    if config.get("FEEDER_ADSBEXCHANGE") == Some("true") {
        log::warn(&t!("feed_enabled", "ADSBExchange"));
        net_options.push_str(" --net-connector feed.adsbexchange.com,30005,beast_reduce_out");
    }
    if config.get("FEEDER_AIRPLANESLIVE") == Some("true") {
        log::warn(&t!("feed_enabled", "airplanes.live"));
        // Same connector string their own installer writes, including the
        // failover endpoint on 64004.
        net_options.push_str(
            " --net-connector feed.airplanes.live,30004,beast_reduce_plus_out,\
             feed.airplanes.live,64004",
        );
    }

    let json_options = format!(
        "--json-location-accuracy {} --range-outline-hours 24",
        value("JSON_LOCATION_ACCURACY", "2"),
    );

    [receiver_options, decoder_options, net_options, json_options]
}

/// `readsb::enable`: daemon-reload, enable, then an explicit restart
/// whenever the config we just wrote actually changed, because
/// `enable --now` does nothing to a unit that is already running. False
/// when the service is not active after the start, the rsb_failed error.
pub fn enable(needs_restart: bool) -> bool {
    log::info(&t!("rsb_enabling"));
    run::sudo(["systemctl", "daemon-reload"]);
    run::sudo(["systemctl", "enable", "readsb"]);

    // is-active is read-only, so it is queried in dry-run too: that way the
    // printed command is the one a real run would issue.
    if svc::is_active("readsb") {
        if needs_restart {
            log::info(&t!("rsb_restarting"));
            run::sudo(["systemctl", "restart", "readsb"]);
        }
    } else {
        run::sudo(["systemctl", "start", "readsb"]);
    }

    if run::dry_run() {
        return true;
    }

    run::cmd(["sleep", "3"]);

    if svc::is_active("readsb") {
        log::success(&t!("rsb_active"));
    } else {
        log::error(&t!("rsb_failed"));
        return false;
    }

    if Path::new(READSB_JSON).is_file() {
        log::success(&t!("rsb_json_ok", READSB_JSON));
    } else {
        log::warn(&t!("rsb_json_wait", READSB_JSON));
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn config(pairs: &[(&str, &str)]) -> Config {
        let values = pairs
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect::<BTreeMap<_, _>>();
        Config::from_map(values)
    }

    #[test]
    fn defaults_build_the_exact_option_strings() {
        let config = config(&[]);
        let [receiver, decoder, net, json] = option_strings(&config);
        assert_eq!(
            receiver,
            "--device 0 --device-type rtlsdr --gain auto --ppm 0 --lat  --lon "
        );
        assert_eq!(decoder, "--max-range 450 --write-json-every 1");
        assert_eq!(
            net,
            "--net --net-ri-port 30001 --net-ro-port 30002 --net-sbs-port 30003 \
             --net-bi-port 30004,30104 --net-bo-port 30005"
        );
        assert_eq!(json, "--json-location-accuracy 2 --range-outline-hours 24");
    }

    #[test]
    fn config_values_land_on_the_option_strings() {
        let config = config(&[
            ("RECEIVER_GAIN", "49.6"),
            ("RECEIVER_PPM", "2"),
            ("RECEIVER_LAT", "-23.58"),
            ("RECEIVER_LON", "-46.55"),
            ("DECODER_MAX_RANGE", "300"),
            ("NET_RI_PORT", "30011"),
            ("NET_RO_PORT", "30012"),
            ("NET_SBS_PORT", "30013"),
            ("NET_BI_PORT", "30014,30114"),
            ("NET_BO_PORT", "30015"),
            ("JSON_LOCATION_ACCURACY", "1"),
        ]);
        let [receiver, decoder, net, json] = option_strings(&config);
        assert_eq!(
            receiver,
            "--device 0 --device-type rtlsdr --gain 49.6 --ppm 2 --lat -23.58 --lon -46.55"
        );
        assert_eq!(decoder, "--max-range 300 --write-json-every 1");
        assert_eq!(
            net,
            "--net --net-ri-port 30011 --net-ro-port 30012 --net-sbs-port 30013 \
             --net-bi-port 30014,30114 --net-bo-port 30015"
        );
        assert_eq!(json, "--json-location-accuracy 1 --range-outline-hours 24");
    }

    #[test]
    fn enabled_feeders_append_their_connector_strings() {
        let config = config(&[
            ("FEEDER_ADSBEXCHANGE", "true"),
            ("FEEDER_AIRPLANESLIVE", "true"),
        ]);
        let [_, _, net, _] = option_strings(&config);
        assert_eq!(
            net,
            "--net --net-ri-port 30001 --net-ro-port 30002 --net-sbs-port 30003 \
             --net-bi-port 30004,30104 --net-bo-port 30005 \
             --net-connector feed.adsbexchange.com,30005,beast_reduce_out \
             --net-connector feed.airplanes.live,30004,beast_reduce_plus_out,feed.airplanes.live,64004"
        );
    }

    #[test]
    fn the_defaults_file_content_matches_the_bash_printf() {
        // The header is pinned by the i18n parity gate; hardcoded here so
        // the test does not depend on which language another test resolved.
        let config = config(&[("RECEIVER_LAT", "0.0"), ("RECEIVER_LON", "0.0")]);
        let [receiver, decoder, net, json] = option_strings(&config);
        let header = "# Generated by easy1090. Edit freely: the installer only rewrites this\n\
                      # file when you run install.sh again.";
        let content = format!(
            "{header}\nRECEIVER_OPTIONS=\"{receiver}\"\nDECODER_OPTIONS=\"{decoder}\"\
             \nNET_OPTIONS=\"{net}\"\nJSON_OPTIONS=\"{json}\"",
        );
        assert!(content.starts_with(
            "# Generated by easy1090. Edit freely: the installer only rewrites this\n\
             # file when you run install.sh again.\nRECEIVER_OPTIONS=\"--device 0 "
        ));
        assert!(content
            .ends_with("JSON_OPTIONS=\"--json-location-accuracy 2 --range-outline-hours 24\""));
    }
}
