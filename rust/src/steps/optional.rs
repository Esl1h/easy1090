//! Port of lib/40-optional.sh: the optional GUI companions (SDR++ and
//! SatDump).
//!
//! Neither decodes ADS-B. They are here because they complete the SDR bench:
//! SDR++ to eyeball RF energy at 1090 MHz on the waterfall, SatDump for the
//! 137 MHz satellite side. Both are off by default in install.conf.

use crate::core::cfg::Config;
use crate::core::{confirm, log};
use crate::pkg::Backend;
use crate::t;

pub const SDRPP_PACKAGE: &str = "sdrpp-git";
pub const SATDUMP_PACKAGE: &str = "satdump";

/// `optional::run`: the two companions, only when the config asks for them.
pub fn run(pkg: &dyn Backend, config: &Config) {
    if config.get("COMPONENT_SDRPP") == Some("true") {
        sdrpp(pkg);
    }
    if config.get("COMPONENT_SATDUMP") == Some("true") {
        satdump(pkg);
    }
}

fn sdrpp(pkg: &dyn Backend) {
    log::step(&t!("opt_sdrpp_step"));

    if pkg.is_installed(SDRPP_PACKAGE) {
        log::skip(&t!("drv_installed", SDRPP_PACKAGE));
        return;
    }

    log::info(&t!("opt_sdrpp_note"));
    pkg.install_aur(SDRPP_PACKAGE);
}

fn satdump(pkg: &dyn Backend) {
    log::step(&t!("opt_satdump_step"));

    if pkg.is_installed(SATDUMP_PACKAGE) {
        log::skip(&t!("drv_installed", SATDUMP_PACKAGE));
        return;
    }

    // Stable release on purpose, not the -git: it is a large C++/CMake
    // project with many plugins, and the tagged version is far less likely
    // to break against whatever GCC the system is on.
    log::warn(&t!("opt_satdump_slow"));
    if !confirm::confirm(&t!("opt_satdump_confirm")) {
        log::info(&t!("opt_satdump_skipped"));
        return;
    }

    pkg.install_aur(SATDUMP_PACKAGE);
}
