//! Port of lib/cmd-install.sh: the install command.
//!
//! Idempotent: re-running is how you update. Each step detects what is
//! already in place and skips it. The banner is printed by the entrypoint,
//! like the bash main() does, so this module starts at the argument parser.

use std::path::Path;

use crate::core::cfg::{self, Config};
use crate::core::{log, sudo};
use crate::pkg;
use crate::preflight;
use crate::steps;
use crate::t;

/// The parsed command line, the install::parse_args locals of the bash.
struct Opts {
    lat: Option<String>,
    lon: Option<String>,
    full: bool,
    skip_tar1090: bool,
    skip_sdrpp: bool,
    skip_satdump: bool,
}

/// `cmd::install`: parse, preflight, config, then the steps in the order of
/// the bash. The exit code is the validate failure count, or 1 when a step
/// fails or an argument is unknown.
pub fn run(args: &[&str], config_file: &Path, example_file: &Path) -> i32 {
    let mut opts = Opts {
        lat: None,
        lon: None,
        full: false,
        skip_tar1090: false,
        skip_sdrpp: false,
        skip_satdump: false,
    };
    if !parse_args(args, &mut opts) {
        return 1;
    }

    preflight::run();

    let mut config = cfg::load(config_file, example_file);
    apply_overrides(&mut config, &opts);
    cfg::require_position(config_file, &mut config);
    // Remember CLI coordinates so the next run does not ask again.
    if let Some(lat) = opts.lat.as_deref().filter(|lat| !lat.is_empty()) {
        cfg::persist(config_file, "RECEIVER_LAT", lat);
    }
    if let Some(lon) = opts.lon.as_deref().filter(|lon| !lon.is_empty()) {
        cfg::persist(config_file, "RECEIVER_LON", lon);
    }
    cfg::require_feeder(config_file, &mut config);

    sudo::init();
    let backend = pkg::arch::Arch;

    let mut code = 1;
    if steps::driver::run(&backend)
        && steps::readsb::run(&backend, &config)
        && (config.get("COMPONENT_TAR1090") != Some("true")
            || steps::tar1090::run(&backend, &config))
    {
        steps::optional::run(&backend, &config);
        code = steps::validate::run(&config);
    }

    sudo::cleanup();
    code
}

/// install::parse_args: the flags, -h prints the installer usage and exits
/// zero before anything else runs, unknown options error and fail the
/// command.
fn parse_args(args: &[&str], opts: &mut Opts) -> bool {
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        match *arg {
            "--full" => opts.full = true,
            "--lat" => opts.lat = iter.next().map(|value| value.to_string()),
            "--lon" => opts.lon = iter.next().map(|value| value.to_string()),
            "--skip-tar1090" => opts.skip_tar1090 = true,
            "--skip-sdrpp" => opts.skip_sdrpp = true,
            "--skip-satdump" => opts.skip_satdump = true,
            "-h" | "--help" => {
                print!("{}", t!("cli_usage", crate::VERSION));
                std::process::exit(0);
            }
            other => {
                log::error(&t!("cli_unknown_opt", other));
                return false;
            }
        }
    }
    true
}

/// CLI wins over install.conf, and coordinates given here are remembered in
/// install.conf, exactly like the ones asked on the first run.
fn apply_overrides(config: &mut Config, opts: &Opts) {
    if let Some(lat) = opts.lat.as_deref().filter(|lat| !lat.is_empty()) {
        let lon = opts
            .lon
            .as_deref()
            .filter(|lon| !lon.is_empty())
            .unwrap_or("0.0");
        cfg::validate_position(lat, lon);
        config.set("RECEIVER_LAT", lat);
    }
    if let Some(lon) = opts.lon.as_deref().filter(|lon| !lon.is_empty()) {
        let lat = opts
            .lat
            .as_deref()
            .filter(|lat| !lat.is_empty())
            .unwrap_or("0.0");
        cfg::validate_position(lat, lon);
        config.set("RECEIVER_LON", lon);
    }

    if opts.full {
        config.set("COMPONENT_TAR1090", "true");
        config.set("COMPONENT_SDRPP", "true");
        config.set("COMPONENT_SATDUMP", "true");
    }

    if opts.skip_tar1090 {
        config.set("COMPONENT_TAR1090", "false");
    }
    if opts.skip_sdrpp {
        config.set("COMPONENT_SDRPP", "false");
    }
    if opts.skip_satdump {
        config.set("COMPONENT_SATDUMP", "false");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn empty_config() -> Config {
        let values = [
            ("COMPONENT_TAR1090", "true"),
            ("COMPONENT_SDRPP", "false"),
            ("COMPONENT_SATDUMP", "false"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<BTreeMap<_, _>>();
        Config::from_map(values)
    }

    #[test]
    fn full_enables_everything_and_skip_wins_after_it() {
        let mut config = empty_config();
        let opts = Opts {
            lat: None,
            lon: None,
            full: true,
            skip_tar1090: true,
            skip_sdrpp: false,
            skip_satdump: false,
        };
        // --full then --skip-tar1090: the skip is applied last, like the
        // bash checks them in that order.
        apply_overrides(&mut config, &opts);
        assert_eq!(config.get("COMPONENT_TAR1090"), Some("false"));
        assert_eq!(config.get("COMPONENT_SDRPP"), Some("true"));
        assert_eq!(config.get("COMPONENT_SATDUMP"), Some("true"));
    }

    #[test]
    fn cli_latitude_overrides_the_config_value() {
        let mut config = empty_config();
        config.set("RECEIVER_LAT", "-1.0");
        let opts = Opts {
            lat: Some("-23.58".to_string()),
            lon: None,
            full: false,
            skip_tar1090: false,
            skip_sdrpp: false,
            skip_satdump: false,
        };
        apply_overrides(&mut config, &opts);
        assert_eq!(config.get("RECEIVER_LAT"), Some("-23.58"));
    }

    #[test]
    fn parse_args_collects_the_flags() {
        let mut opts = Opts {
            lat: None,
            lon: None,
            full: false,
            skip_tar1090: false,
            skip_sdrpp: false,
            skip_satdump: false,
        };
        assert!(parse_args(
            &[
                "--full",
                "--lat",
                "-23.58",
                "--lon",
                "-46.55",
                "--skip-sdrpp",
            ],
            &mut opts
        ));
        assert!(opts.full);
        assert_eq!(opts.lat.as_deref(), Some("-23.58"));
        assert_eq!(opts.lon.as_deref(), Some("-46.55"));
        assert!(opts.skip_sdrpp);
    }

    #[test]
    fn parse_args_rejects_unknown_options() {
        let mut opts = Opts {
            lat: None,
            lon: None,
            full: false,
            skip_tar1090: false,
            skip_sdrpp: false,
            skip_satdump: false,
        };
        assert!(!parse_args(&["--frobnicate"], &mut opts));
    }
}
