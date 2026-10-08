//! Port of lib/10-driver.sh: the RTL-SDR driver.
//!
//! Installs the RTL-SDR Blog fork of librtlsdr and gets the kernel DVB
//! driver out of the way. The fork is installed unconditionally, without
//! probing the model: it is mandatory for the V4 (R828D tuner plus the
//! internal HF upconverter, which the osmocom driver mishandles) and
//! harmless for v3 and clones.

use std::process::Stdio;

use crate::core::{confirm, log, run, util};
use crate::pkg::Backend;
use crate::t;

pub const DRIVER_PACKAGE: &str = "rtl-sdr-blog-git";
const DRIVER_CONFLICTS: &str = "rtl-sdr";
const DVB_MODULE: &str = "dvb_usb_rtl28xxu";
/// Shared with the uninstall command.
pub const DVB_BLACKLIST: &str = "/etc/modprobe.d/blacklist-rtlsdr.conf";

/// `driver::run`: the install fork, the DVB blacklist and the hardware
/// validation. False when rtl_test finds no device, which aborts the install
/// like `set -e` does in the bash.
pub fn run(pkg: &dyn Backend) -> bool {
    log::step(&t!("drv_step"));

    install_fork(pkg);
    blacklist_dvb();
    validate()
}

fn install_fork(pkg: &dyn Backend) {
    if pkg.is_installed(DRIVER_PACKAGE) {
        log::skip(&t!("drv_installed", DRIVER_PACKAGE));
        return;
    }

    // The AUR package declares a conflict with the generic rtl-sdr. Under
    // --noconfirm pacman answers "N" to the replace prompt and the install
    // aborts, so the removal has to be its own explicit step.
    if pkg.is_installed(DRIVER_CONFLICTS) {
        log::warn(&t!("drv_conflict", DRIVER_CONFLICTS));
        if !confirm::confirm(&t!("drv_conflict_confirm", DRIVER_CONFLICTS)) {
            util::die(&t!("drv_conflict_abort"));
        }
        pkg.remove(DRIVER_CONFLICTS);
    }

    pkg.install_aur(DRIVER_PACKAGE);
}

/// blacklist alone only stops autoload at boot. udev still pulls the module
/// by alias on hotplug, so the dongle gets claimed as a DVB tuner the moment
/// it is replugged. The install line is what actually closes that door.
fn blacklist_dvb() {
    // The content of the bash heredoc: the `$(cat <<EOF ...)` there strips
    // the trailing newline, so the written file ends after the install line.
    let content = format!(
        "# easy1090: keeps the digital TV driver away from the RTL-SDR dongle.\n\
         # blacklist stops autoload at boot; install stops the alias-triggered load\n\
         # when udev asks for the module on hotplug.\n\
         blacklist {DVB_MODULE}\n\
         install {DVB_MODULE} /bin/false"
    );

    let configured = std::fs::read_to_string(DVB_BLACKLIST)
        .map(|existing| existing.contains(&format!("install {DVB_MODULE} /bin/false")))
        .unwrap_or(false);
    if configured {
        log::skip(&t!("drv_blacklist_ok", DVB_MODULE));
    } else {
        log::info(&t!("drv_blacklist_set", DVB_MODULE));
        run::sudo_write(DVB_BLACKLIST, &content);
    }

    // `lsmod 2>/dev/null | grep -q "^${DVB_MODULE}"`: a loaded module means
    // the dongle is claimed as a tuner right now and has to be released.
    let loaded = run::capture(["lsmod"], Stdio::null())
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .any(|line| line.starts_with(DVB_MODULE))
        })
        .unwrap_or(false);

    if loaded {
        log::info(&t!("drv_module_unload", DVB_MODULE));
        if !run::sudo(["modprobe", "-r", DVB_MODULE]) {
            log::warn(&t!("drv_module_unload_fail", DVB_MODULE));
        }
    } else {
        log::skip(&t!("drv_module_absent", DVB_MODULE));
    }
}

fn validate() -> bool {
    if run::dry_run() {
        log::dry_run("rtl_test -t");
        return true;
    }

    if !util::have_cmd("rtl_test") {
        log::warn(&t!("drv_test_missing"));
        return true;
    }

    log::info(&t!("drv_test_running"));

    // rtl_test -t always ends with "No E4000 tuner found, aborting." on this
    // hardware. That is the E4000 specific gain test, not a failure, so we
    // look at the detection lines instead of the exit code. `timeout 15`
    // and `2>&1` with `|| true`, so both streams land in the variable; the
    // grep checks do not care about interleaving order, so concatenating the
    // two captures is exact enough.
    let output = match run::capture(["timeout", "15", "rtl_test", "-t"], Stdio::piped()) {
        Some(out) => {
            let mut merged = out.stdout;
            merged.extend_from_slice(&out.stderr);
            String::from_utf8_lossy(&merged).into_owned()
        }
        None => String::new(),
    };
    log::debug(&output);

    if output.contains("No supported devices found") {
        log::error(&t!("drv_no_device"));
        log::error(&t!("drv_no_device_hint"));
        return false;
    }

    // On a re-run readsb already owns the device, so rtl_test enumerates it
    // but cannot claim the interface. That is a healthy system, not a
    // failure: the device line still tells us which model it is.
    if output.contains("usb_claim_interface error")
        || output.contains("Failed to open rtlsdr device")
    {
        if output.contains("Blog V4") {
            log::skip(&t!("drv_busy_v4"));
        } else {
            log::skip(&t!("drv_busy"));
        }
        return true;
    }

    if output.contains("R828D") {
        log::success(&t!("drv_tuner_v4"));
    } else if output.contains("R820T") {
        log::success(&t!("drv_tuner_v3"));
    } else {
        log::warn(&t!("drv_tuner_unknown"));
    }

    true
}
