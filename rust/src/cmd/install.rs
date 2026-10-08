//! Port of lib/cmd-install.sh, phase 4 scope: the banner and the preflight.
//! The installation steps themselves are phase 5 and stop with the
//! not-implemented message. install::parse_args (--full, --lat/--lon, the
//! --skip-* flags and -h) arrives with that port.

use crate::core::{log, run};
use crate::preflight;
use crate::t;

/// The install arm of the bash entrypoint: the banner to stderr, the
/// --dry-run warning, then the preflight. Everything the bash does after
/// the preflight is not ported yet and reports that instead.
pub fn run(_args: &[&str]) -> i32 {
    eprintln!(
        "\n{}easy1090{} {}",
        log::bold(),
        log::reset(),
        crate::VERSION
    );
    if run::dry_run() {
        log::warn(&t!("cli_dry_warning"));
    }

    preflight::run();

    log::error("\"install\" is not implemented yet.");
    1
}
