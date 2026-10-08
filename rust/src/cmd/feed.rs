//! Port of lib/cmd-feed.sh: the ADSBExchange and airplanes.live feeds.
//!
//! Two separate things, which the ADSBExchange documentation tends to blur:
//!
//!   Feeding is just a --net-connector in readsb. That is all it takes for
//!   your data to reach them, and easy1090 already handles it through the
//!   config.
//!
//!   The stats package is optional and separate. It generates a feeder UUID
//!   and pushes receiver statistics, which is what makes your feeder appear
//!   on their site and lets you claim it under an account.
//!
//! Their installer does not run on Arch. It calls `adduser` with no
//! fallback, and with `set -e` the script dies on that line before
//! installing anything. We create the system user first, so their `id -u`
//! check passes and the adduser branch is skipped, then hand the rest to
//! their script unmodified. Same principle as the tar1090 module: work
//! around it, never patch it.

use std::path::Path;
use std::process::Stdio;

use crate::core::cfg::{self, Config};
use crate::core::{confirm, log, run, sudo, svc};
use crate::pkg::{self, Backend};
use crate::steps::readsb::{self, READSB_PACKAGE};
use crate::t;

const FEED_STATS_REPO: &str = "https://github.com/adsbexchange/adsbexchange-stats";
/// `${HOME}/.cache/easy1090/adsbexchange-stats`.
fn stats_clone() -> String {
    format!(
        "{}/.cache/easy1090/adsbexchange-stats",
        crate::core::util::home()
    )
}
const FEED_STATS_UNIT: &str = "adsbexchange-stats";
const FEED_STATS_USER: &str = "adsbexchange";
const FEED_STATS_PATH: &str = "/usr/local/share/adsbexchange-stats";
const FEED_STATS_DEFAULTS: &str = "/etc/default/adsbexchange-stats";
/// create-uuid.sh writes to one of these, depending on how the host was set
/// up.
const FEED_UUID_FILES: [&str; 2] = [
    "/boot/adsbx-uuid",
    "/usr/local/share/adsbexchange/adsbx-uuid",
];

/// The FEED_ACTION of the bash: what the command does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    List,
    Status,
    Disable,
    Stats,
    Enable,
}

/// The FEED_NETWORK of the bash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Network {
    AdsbExchange,
    AirplanesLive,
}

/// `cmd::feed`: parse, load the config, then dispatch on the action. The
/// banner is printed by the entrypoint, like the bash main() does.
pub fn run(args: &[&str], config_file: &Path, example_file: &Path) -> i32 {
    let Some((action, network)) = parse_args(args) else {
        return 1;
    };

    let mut config = cfg::load(config_file, example_file);
    let backend = pkg::arch::Arch;

    match action {
        Action::List | Action::Status => show_status(&config),
        Action::Stats => {
            sudo::init();
            install_stats(&backend, true);
            show_info(&config);
        }
        Action::Disable => {
            sudo::init();
            set_enabled(&backend, &mut config, network, false, config_file);
        }
        Action::Enable => {
            sudo::init();
            set_enabled(&backend, &mut config, network, true, config_file);
            // Only ADSBExchange has a separate stats package. airplanes.live
            // shows the feed on their site straight from the connector.
            if network == Network::AdsbExchange {
                install_stats(&backend, false);
                show_info(&config);
            } else {
                eprint!(
                    "\n{}\n\n{}\n{}\n\n",
                    t!("feed_alive_note"),
                    t!("feed_alive_url"),
                    t!("feed_alive_map"),
                );
            }
        }
    }

    sudo::cleanup();
    0
}

/// feed::parse_args: the state machine over the positional arguments. The
/// action defaults to "list", which a network keyword turns into "enable".
fn parse_args(args: &[&str]) -> Option<(Action, Network)> {
    let mut action = Action::List;
    let mut network: Option<Network> = None;

    for arg in args {
        match *arg {
            "--status" => action = Action::Status,
            "--disable" => {
                action = Action::Disable;
                if network.is_none() {
                    network = Some(Network::AdsbExchange);
                }
            }
            "--stats" => {
                action = Action::Stats;
                network = Some(Network::AdsbExchange);
            }
            "adsbexchange" | "adsbx" => {
                network = Some(Network::AdsbExchange);
                if action == Action::List {
                    action = Action::Enable;
                }
            }
            "airplaneslive" | "airplanes.live" | "alive" => {
                network = Some(Network::AirplanesLive);
                if action == Action::List {
                    action = Action::Enable;
                }
            }
            "-h" | "--help" => {
                print!("{}", t!("feed_usage", crate::VERSION));
                std::process::exit(0);
            }
            other => {
                log::error(&t!("cli_unknown_opt", other));
                return None;
            }
        }
    }

    Some((action, network.unwrap_or(Network::AdsbExchange)))
}

/// Flips the config key and reconverges readsb, which is what actually puts
/// the --net-connector on the running process.
fn set_enabled(
    backend: &dyn Backend,
    config: &mut Config,
    network: Network,
    wanted: bool,
    config_file: &Path,
) {
    log::step(&t!("feed_step_cfg"));

    let (unchanged, value) = flip_enabled(config, network, wanted);
    if unchanged {
        log::skip(&t!(if wanted {
            "feed_already_on"
        } else {
            "feed_already_off"
        }));
    } else {
        let message = match (network, wanted) {
            (Network::AirplanesLive, true) => t!("feed_alive_enabling"),
            (Network::AirplanesLive, false) => t!("feed_alive_disabling"),
            (_, true) => t!("feed_enabling"),
            (_, false) => t!("feed_disabling"),
        };
        if wanted {
            log::warn(&message);
        } else {
            log::info(&message);
        }
        let key = if network == Network::AirplanesLive {
            "FEEDER_AIRPLANESLIVE"
        } else {
            "FEEDER_ADSBEXCHANGE"
        };
        cfg::persist(config_file, key, &value);
    }

    // The connector only exists when readsb is installed. Without this guard
    // a feed on a fresh machine writes /etc/default/readsb with an empty
    // --lat/--lon and then systemctl enable fails on the missing unit.
    if backend.is_installed(READSB_PACKAGE) {
        // Rewrites /etc/default/readsb and restarts only if the content
        // changed.
        let needs_restart = readsb::configure(config);
        readsb::enable(needs_restart);
    } else {
        log::warn(&t!("feed_readsb_missing"));
    }
}

/// `feed::stats_installed`: the unit file exists.
fn stats_installed() -> bool {
    svc::unit_exists(FEED_STATS_UNIT)
}

fn install_stats(backend: &dyn Backend, force: bool) -> bool {
    log::step(&t!("feed_step_stats"));

    if stats_installed() && !force {
        log::skip(&t!("feed_stats_present"));
        return true;
    }

    eprint!(
        "\n{}\n{}\n\n{}\n\n",
        t!("feed_stats_intro"),
        t!("feed_stats_repo", FEED_STATS_REPO),
        t!("feed_stats_note"),
    );

    if !confirm::confirm(&t!("feed_stats_confirm")) {
        log::info(&t!("feed_stats_skipped"));
        return true;
    }

    stats_dependencies(backend);
    stats_user();
    stats_run()
}

/// Their script installs these with apt or yum only, so on Arch it silently
/// installs nothing and the service misbehaves later. `host` comes from
/// bind.
fn stats_dependencies(backend: &dyn Backend) {
    let needed = ["curl", "jq", "gzip", "perl", "bind"];

    log::info(&t!("feed_stats_deps", needed.join(" ")));
    backend.install(&needed);
}

fn stats_user() {
    let exists = run::capture(["id", "-u", FEED_STATS_USER], Stdio::null())
        .map(|out| out.status.success())
        .unwrap_or(false);
    if exists {
        log::skip(&t!("feed_stats_user_ok"));
        return;
    }

    log::info(&t!("feed_stats_user"));
    run::sudo([
        "useradd",
        "--system",
        "--home-dir",
        FEED_STATS_PATH,
        "--no-create-home",
        "--shell",
        "/usr/bin/nologin",
        FEED_STATS_USER,
    ]);
}

/// Skips their stats.sh bootstrap, which only exists to apt-get install git
/// and then clone this repository. Cloning it ourselves keeps apt out of the
/// picture and makes the code visible before it runs.
fn stats_run() -> bool {
    let clone = stats_clone();
    if Path::new(&clone).is_dir() {
        run::cmd(["rm", "-rf", &clone]);
    }
    run::cmd(["mkdir", "-p", &parent_of(&clone)]);

    log::info(&t!("feed_stats_cloning"));
    run::cmd(["git", "clone", "--depth", "1", FEED_STATS_REPO, &clone]);

    log::info(&t!("feed_stats_running"));
    if run::dry_run() {
        log::dry_run(&format!("cd {clone} && sudo bash install.sh"));
    } else {
        // (cd "$FEED_STATS_CLONE" && sudo bash install.sh) || { error }
        if !run::sudo_in(&clone, ["bash", "install.sh"]) {
            log::error(&t!("feed_stats_failed"));
            return false;
        }

        if svc::is_active(FEED_STATS_UNIT) {
            log::success(&t!("feed_stats_ok"));
        }
    }

    // Runs in dry-run too, so the preview shows every step the real run
    // takes.
    stats_datasource()
}

/// Their json-status only looks at /run/adsbexchange-feed, which is created
/// by the ADSBExchange feed package. We feed straight from readsb, so our
/// JSON is in /run/readsb and the service loops on "No valid data source
/// directory".
///
/// The escape hatch is theirs: USE_OLD_PATH=1 makes it try /run/readsb
/// first. Their installer only writes it when it detects a Raspberry Pi
/// image, which is why it never lands on a normal machine.
fn stats_datasource() -> bool {
    let content = "# easy1090: look at /run/readsb, where our readsb writes its JSON.\n\
                   # Without this, json-status only checks /run/adsbexchange-feed and reports\n\
                   # \"No valid data source directory found\" forever.\n\
                   USE_OLD_PATH=1";

    let configured = std::fs::read_to_string(FEED_STATS_DEFAULTS)
        .map(|existing| {
            existing
                .lines()
                .any(|line| line.starts_with("USE_OLD_PATH=1"))
        })
        .unwrap_or(false);
    if configured {
        log::skip(&t!("feed_datasource_ok"));
        return true;
    }

    log::info(&t!("feed_datasource_fix"));
    run::sudo_write(FEED_STATS_DEFAULTS, content);

    log::info(&t!("feed_datasource_restart"));
    run::sudo(["systemctl", "restart", FEED_STATS_UNIT]);

    if run::dry_run() {
        return true;
    }

    run::cmd(["sleep", "5"]);
    let working = run::capture(
        [
            "journalctl",
            "-u",
            FEED_STATS_UNIT,
            "--since",
            "-1min",
            "--no-pager",
        ],
        Stdio::null(),
    )
    .map(|out| String::from_utf8_lossy(&out.stdout).contains("Using JSON directory"))
    .unwrap_or(false);

    if working {
        log::success(&t!("feed_datasource_working"));
    } else {
        log::warn(&t!("feed_datasource_wait"));
    }
    true
}

/// `feed::uuid`: the first readable UUID file, whitespace stripped.
fn uuid() -> Option<String> {
    for file in FEED_UUID_FILES {
        if let Ok(content) = std::fs::read_to_string(file) {
            return Some(content.chars().filter(|c| !c.is_whitespace()).collect());
        }
    }
    None
}

fn show_info(config: &Config) {
    log::step(&t!("feed_step_info"));

    match uuid() {
        Some(uuid) if !uuid.is_empty() => {
            log::success(&t!("feed_uuid", &uuid));
            eprint!(
                "\n{}\n{}\n{}\n\n",
                t!(
                    "feed_url_stats",
                    format!("https://www.adsbexchange.com/api/feeders/?feed={uuid}")
                ),
                t!("feed_url_myip"),
                t!("feed_url_account"),
            );
        }
        _ => {
            log::warn(&t!("feed_uuid_missing"));
            eprint!("\n{}\n\n", t!("feed_url_myip"));
        }
    }

    privacy_note(config);
}

fn privacy_note(config: &Config) {
    let accuracy = config.get("JSON_LOCATION_ACCURACY").unwrap_or("2");
    let human = match accuracy {
        "0" => t!("feed_privacy_none"),
        "1" => t!("feed_privacy_approx"),
        _ => t!("feed_privacy_exact"),
    };

    log::info(&t!("feed_privacy", accuracy, &human));
}

/// Read-only: no sudo, no changes.
fn show_status(config: &Config) {
    log::step(&t!("feed_step_cfg"));

    let adsbx = if config.get("FEEDER_ADSBEXCHANGE") == Some("true") {
        t!("feed_list_on")
    } else {
        t!("feed_list_off")
    };
    let alive = if config.get("FEEDER_AIRPLANESLIVE") == Some("true") {
        t!("feed_list_on")
    } else {
        t!("feed_list_off")
    };

    eprint!(
        "\n{}\n{}\n{}\n\n{}\n\n",
        t!("feed_list_title"),
        t!("feed_list_adsbx", &adsbx),
        t!("feed_list_alive", &alive),
        t!("feed_list_hint"),
    );

    // Column 4 is the peer, so this only matches connections we opened out
    // to someone else's 30005. A LAN client attached to our own beast port
    // shows up with an ephemeral peer port and is correctly ignored.
    let peer = established_peer();
    if !peer.is_empty() {
        log::success(&t!("feed_connected", &peer));
    } else {
        log::warn(&t!("feed_not_connected"));
    }

    log::step(&t!("feed_step_stats"));
    if stats_installed() {
        log::success(&t!("feed_stats_present"));
    } else {
        log::warn(&t!("feed_stats_skipped"));
    }

    show_info(config);
}

/// `ss -tn state established 2>/dev/null | awk '$4 ~ /:30005$/ {print $4}' |
/// head -1`: the first peer address with a :30005 remote port, empty when
/// there is none.
fn established_peer() -> String {
    let Some(out) = run::capture(["ss", "-tn", "state", "established"], Stdio::null()) else {
        return String::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| {
            let field = line.split_whitespace().nth(3)?;
            field.ends_with(":30005").then(|| field.to_string())
        })
        .next()
        .unwrap_or_default()
}

/// `$(dirname "$clone")` for the mkdir step; the clone is always an absolute
/// path ending in the repository name.
fn parent_of(path: &str) -> String {
    match path.rfind('/') {
        Some(index) if index > 0 => path[..index].to_string(),
        Some(_) => "/".to_string(),
        None => ".".to_string(),
    }
}

/// The config flip behind feed::set_enabled, extracted so the transitions
/// are testable without the file and the readsb reconverge.
fn flip_enabled(config: &mut Config, network: Network, wanted: bool) -> (bool, String) {
    let key = if network == Network::AirplanesLive {
        "FEEDER_AIRPLANESLIVE"
    } else {
        "FEEDER_ADSBEXCHANGE"
    };
    let current = config.get(key) == Some("true");
    let wanted_value = if wanted { "true" } else { "false" };
    if current != wanted {
        config.set(key, wanted_value);
    }
    (current == wanted, wanted_value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn config() -> Config {
        let values = [
            ("FEEDER_ADSBEXCHANGE", "false"),
            ("FEEDER_AIRPLANESLIVE", "false"),
            ("JSON_LOCATION_ACCURACY", "2"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<BTreeMap<_, _>>();
        Config::from_map(values)
    }

    #[test]
    fn enabling_flips_only_the_requested_network() {
        let mut config = config();

        let (unchanged, value) = flip_enabled(&mut config, Network::AdsbExchange, true);
        assert!(!unchanged);
        assert_eq!(value, "true");
        assert_eq!(config.get("FEEDER_ADSBEXCHANGE"), Some("true"));
        assert_eq!(config.get("FEEDER_AIRPLANESLIVE"), Some("false"));

        flip_enabled(&mut config, Network::AirplanesLive, true);
        assert_eq!(config.get("FEEDER_AIRPLANESLIVE"), Some("true"));
    }

    #[test]
    fn disabling_reports_unchanged_when_already_off() {
        let mut config = config();
        let (unchanged, value) = flip_enabled(&mut config, Network::AdsbExchange, false);
        assert!(unchanged, "already off must read as unchanged");
        assert_eq!(value, "false");
    }

    #[test]
    fn re_enabling_reports_unchanged_when_already_on() {
        let mut config = config();
        flip_enabled(&mut config, Network::AirplanesLive, true);
        let (unchanged, _) = flip_enabled(&mut config, Network::AirplanesLive, true);
        assert!(unchanged);
    }

    #[test]
    fn parse_args_walks_the_state_machine_like_the_bash() {
        // A network keyword turns the default list action into enable.
        let (action, network) = parse_args(&["adsbexchange"]).unwrap();
        assert_eq!(action, Action::Enable);
        assert_eq!(network, Network::AdsbExchange);

        // --disable defaults to ADSBExchange when no network was given.
        let (action, network) = parse_args(&["--disable"]).unwrap();
        assert_eq!(action, Action::Disable);
        assert_eq!(network, Network::AdsbExchange);

        // A network after --disable only changes which one.
        let (action, network) = parse_args(&["--disable", "airplanes.live"]).unwrap();
        assert_eq!(action, Action::Disable);
        assert_eq!(network, Network::AirplanesLive);

        // --stats pins ADSBExchange whatever came before.
        let (action, network) = parse_args(&["alive", "--stats"]).unwrap();
        assert_eq!(action, Action::Stats);
        assert_eq!(network, Network::AdsbExchange);

        // --status changes nothing else.
        let (action, _) = parse_args(&["--status"]).unwrap();
        assert_eq!(action, Action::Status);

        assert!(parse_args(&["--frobnicate"]).is_none());
    }

    #[test]
    fn privacy_level_picks_the_human_label() {
        let mut config = config();
        config.set("JSON_LOCATION_ACCURACY", "0");
        let human = match config.get("JSON_LOCATION_ACCURACY").unwrap_or("2") {
            "0" => t!("feed_privacy_none"),
            "1" => t!("feed_privacy_approx"),
            _ => t!("feed_privacy_exact"),
        };
        assert!(human.contains("not published") || human.contains("não publica"));
    }
}
