//! Port of lib/cmd-service.sh: start / stop / restart.
//!
//! Drives the three units as a group, in dependency order, so you do not
//! have to remember that tar1090 exists on top of readsb and lighttpd.
//! Individual units are still accepted for the odd case. The banner is
//! printed by the entrypoint, like the bash main() does.

use crate::core::{log, run, sudo, svc};
use crate::t;

/// `SERVICE_UNITS` from cmd-service.sh.
const SERVICE_UNITS: [&str; 3] = ["readsb", "lighttpd", "tar1090"];

/// `cmd::start` / `cmd::stop` / `cmd::restart`: the same service::act with
/// the action fixed.
pub fn run(action: &str, args: &[&str]) -> i32 {
    act(action, args);
    0
}

fn act(action: &str, args: &[&str]) {
    let units: Vec<&str> = if args.is_empty() {
        let mut units = SERVICE_UNITS.to_vec();
        // Bring the stack down from the top, and up from the bottom.
        if action == "stop" {
            units = vec!["tar1090", "lighttpd", "readsb"];
        }
        units
    } else {
        args.to_vec()
    };

    log::step(&t!("svc_step"));

    sudo::init();

    for unit in units {
        if !svc::unit_exists(unit) {
            log::skip(&t!("svc_not_installed", unit));
            continue;
        }

        log::info(&t!("svc_acting", unit, action));
        // The bash guards this with `|| true`: the action runs best effort.
        run::sudo_ok(["systemctl", action, unit]);
    }

    log::success(&t!("svc_done"));
    sudo::cleanup();
}
