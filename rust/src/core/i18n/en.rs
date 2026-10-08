//! easy1090 message catalog (en).
//!
//! Generated verbatim from lib/i18n/en.sh by scripts/gen-catalogs.sh;
//! do not edit by hand. Key parity with the bash catalogs is enforced
//! in CI by scripts/i18n-parity.sh.

/// Keys appear in the bash catalog's order; lookups are linear, which
/// is fine at this size.
pub static MSG: &[(&str, &str)] = &[
    // Common
    ("yes_no", "[y/N] "),
    ("yes_chars", "SsYy"),
    ("cfg_missing_dry", "Config does not exist yet; it would be created from the example."),
    ("cfg_created", "Config created at %s (from the example)."),
    ("cfg_example_missing", "Example config not found: %s"),
    ("sudo_validating", "Validating sudo (your password may be requested now)."),
    ("sudo_failed", "Could not validate sudo."),
    ("cmd_required", "Required command not found: %s"),
    // Receiver position
    ("pos_intro", "The antenna position is used to compute range and distance to aircraft."),
    ("pos_help_title", "To find your coordinates, use one of these:"),
    ("pos_help_osm", "  OpenStreetMap   https://www.openstreetmap.org   (right click the spot, \"Show address\")"),
    ("pos_help_gmaps", "  Google Maps     https://maps.google.com        (right click the spot)"),
    ("pos_help_latlong", "  latlong.net     https://www.latlong.net        (search by address)"),
    ("pos_help_tip", "Enter decimal degrees with a dot. South and west are negative."),
    ("pos_lat", "Latitude (e.g. -23.58): "),
    ("pos_lon", "Longitude (e.g. -46.55): "),
    ("pos_required", "Latitude and longitude are required."),
    ("pos_required_yes", "RECEIVER_LAT/RECEIVER_LON are required with --yes. Fill them in %s."),
    ("pos_dry", "RECEIVER_LAT/RECEIVER_LON are empty; a real run would use the values you provide."),
    ("pos_saved", "Position saved to %s"),
    ("pos_invalid", "Invalid coordinates: %s, %s. Use decimal degrees with a dot, latitude between -90 and 90, longitude between -180 and 180."),
    // Data sharing
    ("feed_title", "Share your data with a public flight tracking network?"),
    ("feed_explain", "This sends the aircraft you receive, and your receiver position, to third party servers. In return those sites usually grant premium access to contributors."),
    ("feed_opt_none", "  1) Do not share (default, everything stays on your network)"),
    ("feed_opt_adsbx", "  2) ADSBExchange (adsbexchange.com, no aircraft filtering)"),
    ("feed_fa_note", "FlightAware is not listed here: feeding their network requires the piaware client, with its own registration and feeder ID; a plain beast connector does not work."),
    ("feed_prompt", "Choose [1]: "),
    ("feed_none", "Local feed only; nothing will be shared."),
    ("feed_enabled", "Feed enabled: %s. Your position will be shared."),
    // Preflight
    ("pre_step", "Preflight"),
    ("pre_root", "Do not run as root. Use your normal user; the script asks for sudo when needed (makepkg and yay refuse to run as root)."),
    ("pre_user_ok", "Normal user (uid %s)."),
    ("pre_tty_dry", "TTY: check relaxed under --dry-run."),
    ("pre_tty_missing", "No interactive terminal. sudo needs a real tty; run this from a terminal or a proper SSH session."),
    ("pre_tty_ok", "Interactive terminal available."),
    ("pre_osrelease", "/etc/os-release not found; could not identify the distro."),
    ("pre_distro_ok", "Compatible distro: %s"),
    ("pre_distro_bad", "v1 supports Arch and derivatives only (detected: %s). Other distros are on the roadmap; see the README."),
    ("pre_tools_missing", "Missing essential commands: %s"),
    ("pre_yay_missing", "yay not found. Install an AUR helper first (easy1090 needs it for readsb and the other AUR packages)."),
    ("pre_tools_ok", "Tools present: %s"),
    ("pre_lsusb_missing", "lsusb not found (usbutils package); skipping the dongle check."),
    ("pre_dongle_ok", "RTL-SDR detected on the USB bus (%s)."),
    ("pre_dongle_missing", "No RTL-SDR found in lsusb (%s)."),
    ("pre_dongle_warn", "The install continues, but nothing will decode without the dongle plugged in."),
    ("pre_dongle_confirm", "Continue anyway?"),
    ("pre_aborted", "Aborted by the user."),
    ("pre_done", "Preflight complete."),
    // Driver
    ("drv_step", "RTL-SDR driver"),
    ("drv_installed", "%s already installed."),
    ("drv_conflict", "The generic %s package conflicts with the RTL-SDR Blog fork."),
    ("drv_conflict_confirm", "Remove %s now?"),
    ("drv_conflict_abort", "Without removing the conflict, installing the fork fails."),
    ("drv_blacklist_ok", "Blacklist for %s already configured."),
    ("drv_blacklist_set", "Configuring blacklist for %s"),
    ("drv_module_unload", "Unloading %s (currently loaded)."),
    ("drv_module_unload_fail", "Could not unload %s; a reboot may be needed."),
    ("drv_module_absent", "%s is not loaded."),
    ("drv_test_missing", "rtl_test not found in PATH; skipping driver validation."),
    ("drv_test_running", "Validating the hardware with rtl_test."),
    ("drv_no_device", "No supported device found."),
    ("drv_no_device_hint", "Check the cable and the USB port (prefer rear ports wired straight to the motherboard)."),
    ("drv_tuner_v4", "R828D tuner detected (RTL-SDR Blog V4)."),
    ("drv_tuner_v3", "R820T/R820T2 tuner detected (v3 or clone)."),
    ("drv_tuner_unknown", "The dongle answered, but the tuner was not identified. Full output with --verbose."),
    // readsb
    ("rsb_step", "readsb (ADS-B decoder)"),
    ("rsb_installed", "%s already installed."),
    ("rsb_conflict", "%s (Mictronics fork) conflicts with %s and writes protobuf instead of JSON."),
    ("rsb_conflict_confirm", "Remove %s now?"),
    ("rsb_conflict_abort", "The two packages cannot coexist; removal is required to continue."),
    ("rsb_legacy_override", "Old systemd override found (references $USER_OPTIONS, which this package does not define)."),
    ("rsb_legacy_removed", "Legacy override removed."),
    ("rsb_udev_ok", "readsb udev rule already present."),
    ("rsb_udev_create", "Creating the udev rule for the readsb service user."),
    ("rsb_udev_comment", "# easy1090: hands the dongle to the readsb service user's group.\\n# The stock rule uses GROUP=\"plugdev\", which does not cover a session-less user."),
    ("rsb_defaults_write", "Writing %s"),
    ("rsb_defaults_header", "# Generated by easy1090. Edit freely: the installer only rewrites this\\n# file when you run install.sh again."),
    ("rsb_enabling", "Enabling and starting the readsb service."),
    ("rsb_active", "readsb is active."),
    ("rsb_failed", "readsb did not start. Check: journalctl -u readsb -n 40 --no-pager"),
    ("rsb_json_ok", "JSON being written to %s"),
    ("rsb_json_wait", "%s does not exist yet; it may take a few seconds."),
    // tar1090
    ("tar_step", "tar1090 (live web map)"),
    ("tar_confd_ok", "%s already exists."),
    ("tar_confd_create", "Creating %s (the tar1090 installer looks for it)."),
    ("tar_vendor_missing", "tar1090 installer not found at %s"),
    ("tar_pin_missing", "TAR1090_INSTALLER_SHA256 is not set in the config."),
    ("tar_pin_current", "Current checksum of the vendored file: %s"),
    ("tar_pin_hint", "Pin that value in the config to catch future changes."),
    ("tar_pin_mismatch", "tar1090 installer checksum does not match."),
    ("tar_pin_expected", "  expected: %s"),
    ("tar_pin_got", "  got:      %s"),
    ("tar_pin_abort", "Refuse to run a modified root script. Review the file before updating the pin."),
    ("tar_pin_ok", "Vendored installer matches the pin in the config."),
    ("tar_service_ok", "tar1090 service already enabled."),
    ("tar_running", "Running the official tar1090 installer (vendored)."),
    ("tar_conf_missing", "%s not found; skipping the include fix."),
    ("tar_include_ok", "lighttpd.conf already includes conf-enabled."),
    ("tar_include_add", "Adding the conf-enabled include to lighttpd.conf (the Arch default lacks it)."),
    ("tar_lighttpd_check", "Validating the lighttpd config before starting it."),
    ("tar_lighttpd_invalid", "Invalid lighttpd config. Review %s before continuing."),
    ("tar_redirect_ok", "mod_redirect already enabled."),
    ("tar_redirect_add", "Enabling mod_redirect (tar1090 uses url.redirect, and Arch does not load that module by default)."),
    ("tar_redirect_comment", "# easy1090: tar1090 uses url.redirect for the slash-less URL."),
    ("tar_lighttpd_enable", "Enabling and starting lighttpd."),
    ("tar_web_ok", "Web map responding at %s"),
    ("tar_web_slashless", "%s (no trailing slash) returned %s; the redirect is not active."),
    ("tar_web_fail", "Web map returned HTTP %s."),
    ("tar_web_hint", "Check: systemctl status lighttpd tar1090"),
    // Optional
    ("opt_sdrpp_step", "SDR++ (spectrum viewer)"),
    ("opt_sdrpp_note", "SDR++ does not decode ADS-B; it is for eyeballing RF energy at 1090 MHz."),
    ("opt_satdump_step", "SatDump (satellite decoder)"),
    ("opt_satdump_slow", "The SatDump build is long (about 45 minutes on the reference hardware)."),
    ("opt_satdump_confirm", "Continue with the SatDump install?"),
    ("opt_satdump_skipped", "SatDump skipped."),
    // Validation
    ("val_step", "Final validation"),
    ("val_unit_missing", "Unit not found: %s.service"),
    ("val_service_ok", "%s: active and enabled at boot."),
    ("val_service_bad", "%s: active=%s enabled=%s"),
    ("val_json_missing", "%s does not exist. Is readsb writing JSON?"),
    ("val_json_stale", "aircraft.json has been stale for %ss; readsb may have hung."),
    ("val_decoding_ok", "readsb decoding (JSON refreshed %ss ago, %s aircraft on screen)."),
    ("val_zero_aircraft", "Zero aircraft right now is normal: it depends on traffic, antenna and line of sight."),
    ("val_web_bad", "http://localhost/tar1090/ returned %s."),
    ("val_web_ok", "tar1090 serving map and data."),
    ("val_web_data_bad", "The map responds, but /tar1090/data/aircraft.json does not. Check the tar1090 service."),
    ("val_all_ok", "Everything is up."),
    ("val_failures", "%s check(s) failed."),
    ("val_howto", "How to watch the traffic:"),
    ("val_howto_viewadsb", "live table in the terminal"),
    ("val_howto_nc", "decoded messages (SBS/CSV)"),
    ("val_howto_map", "live web map"),
    // CLI
    ("cli_dry_warning", "--dry-run mode: nothing will be changed; the commands below are the real ones."),
    ("cli_unknown_opt", "Unknown option: %s"),
    ("cli_usage", "easy1090 %s - ADS-B stack installer (Arch and derivatives)

USAGE
    ./install.sh [options]

OPTIONS
    --full              install everything, including SDR++ and SatDump
    --lang <pt|en>      interface language
    --lat <degrees>     antenna latitude (e.g. -23.58)
    --lon <degrees>     antenna longitude (e.g. -46.55)
    --skip-tar1090      do not install the web map
    --skip-sdrpp        do not install SDR++
    --skip-satdump      do not install SatDump
    --dry-run           run preflight and print the exact commands, without executing
    --yes               do not ask anything (except the sudo password)
    --verbose           debug level logging
    --version           show version
    -h, --help          this help

EXAMPLES
    ./install.sh                                  interactive, asks for lat/lon
    ./install.sh --lat -23.58 --lon -46.55 --yes  unattended
    ./install.sh --dry-run                        show what it would do

Configuration lives in install.conf (created from the .example on first run).
The flags above override whatever is in there."),
    // status.sh
    ("sts_title", "status"),
    ("sts_hardware", "Hardware and driver"),
    ("sts_decoding", "Decoding"),
    ("sts_web", "Web"),
    ("sts_optional", "Optional"),
    ("sts_running", "running"),
    ("sts_stopped", "stopped"),
    ("sts_installed", "installed"),
    ("sts_absent", "absent"),
    ("sts_driver", "driver"),
    ("sts_service", "service"),
    ("sts_map", "web map"),
    ("sts_decode_row", "decoding"),
    ("sts_lsusb_missing", "lsusb not installed"),
    ("sts_dongle_found", "detected on USB (%s)"),
    ("sts_dongle_absent", "nothing in lsusb"),
    ("sts_unit_absent", "unit not installed"),
    ("sts_json_absent", "%s does not exist"),
    ("sts_jq_missing", "jq not installed"),
    ("sts_json_fresh", "JSON from %ss ago, %s aircraft"),
    ("sts_json_stale", "JSON stale for %ss"),
    ("sts_lighttpd_down", "lighttpd inactive"),
    ("sts_http", "HTTP %s"),
    // Restart / device busy
    ("rsb_restarting", "Config changed; restarting readsb to apply it."),
    ("tar_lighttpd_restart", "Config changed; restarting lighttpd to apply it."),
    ("drv_busy", "Dongle already in use by readsb (expected on a re-run); skipping rtl_test."),
    ("drv_busy_v4", "Dongle already in use by readsb (RTL-SDR Blog V4); skipping rtl_test."),
    // Packages
    ("pkg_installed", "Already installed: %s"),
    ("pkg_pacman", "Installing via pacman: %s"),
    ("pkg_absent", "Not installed, nothing to remove: %s"),
    ("pkg_removing", "Removing package: %s"),
    ("pkg_aur", "Installing from the AUR: %s"),
    ("pkg_clean_build", "Cleaning previous build: %s"),
    ("pkg_cloning", "Cloning PKGBUILD for %s"),
    ("pkg_building", "Building and installing (%s)"),
    ("pkg_build_missing", "Build directory not found: %s"),
    // Uninstall
    ("un_title", "Uninstall"),
    ("un_plan", "What will be removed:"),
    ("un_plan_driver", "  driver      rtl-sdr-blog-git package and the DVB module blacklist"),
    ("un_plan_readsb", "  readsb      service, package, /etc/default/readsb and the udev rule"),
    ("un_plan_tar1090", "  tar1090     service, files and the lighttpd configs"),
    ("un_plan_optional", "  optional    SDR++ and SatDump (if installed)"),
    ("un_plan_local", "  local       build cache in ~/.cache/easy1090"),
    ("un_keep", "What will NOT be touched: lighttpd, jq, the readsb and tar1090 system users, and anything you installed yourself."),
    ("un_confirm", "Confirm removal?"),
    ("un_aborted", "Nothing was removed."),
    ("un_step_tar1090", "tar1090"),
    ("un_step_readsb", "readsb"),
    ("un_step_driver", "RTL-SDR driver"),
    ("un_step_optional", "optional"),
    ("un_nothing", "Nothing to remove here."),
    ("un_step_local", "local files"),
    ("un_upstream", "Running tar1090's own uninstaller (%s)."),
    ("un_upstream_missing", "tar1090 uninstaller not found; removing what easy1090 created."),
    ("un_include_removed", "Removed the include line easy1090 appended to lighttpd.conf."),
    ("un_lighttpd_restart", "Restarting lighttpd."),
    ("un_stopping", "Stopping and disabling %s."),
    ("un_removed", "Removed: %s"),
    ("un_absent", "Does not exist, nothing to do: %s"),
    ("un_pkg_kept", "Packages kept (--keep-packages)."),
    ("un_config_ask", "Also remove install.conf (your coordinates and preferences)?"),
    ("un_done", "Uninstall complete."),
    ("un_users_note", "The readsb and tar1090 system users still exist; remove them manually with userdel if you want."),
    ("un_usage", "easy1090 %s - uninstaller

USAGE
    ./uninstall.sh [options]

OPTIONS
    --keep-packages     remove services and configs, but keep the packages
    --lang <pt|en>      interface language
    --dry-run           print the exact commands, without executing
    --yes               do not ask anything (except the sudo password)
    --verbose           debug level logging
    -h, --help          this help

Does not remove lighttpd or jq, which are general purpose packages."),
    // Entrypoint, services and open
    ("main_usage", "easy1090 %s - ADS-B stack in one command (Arch and derivatives)\\n\\nUSAGE\\n    easy1090 <command> [options]\\n\\nCOMMANDS\\n    install       install the stack (idempotent, safe to re-run)\\n    update        update package versions (install converges config)\\n    feed          feed public networks (ADSBExchange, airplanes.live)\\n    uninstall     undo the installation (best effort)\\n    status        what is running, what fell over, what is missing\\n    start         bring up readsb, lighttpd and tar1090\\n    stop          bring all three down\\n    restart       restart all three, in the right order\\n    open [target] open a component (without a target, lists the options)\\n\\nGLOBAL OPTIONS\\n    --lang <pt|en>   interface language\\n    --dry-run        print the exact commands, without executing\\n    --yes            do not ask anything (except the sudo password)\\n    --verbose        debug level logging\\n    --version        show version\\n    -h, --help       this help\\n\\nUse \"easy1090 <command> --help\" for per-command options.\\n\\nstatus and open do not need sudo.\\n"),
    ("cmd_unknown", "Unknown command: %s"),
    ("cmd_missing", "Please provide a command. Use --help for the list."),
    ("svc_step", "Services"),
    ("svc_acting", "%s: %s"),
    ("svc_not_installed", "%s is not installed; skipping."),
    ("svc_done", "Done."),
    ("open_step", "Open"),
    ("open_targets", "Available targets:"),
    ("open_t_viewadsb", "  viewadsb    live table in the terminal (ncurses)"),
    ("open_t_sbs", "  sbs         decoded message stream (CSV)"),
    ("open_t_map", "  map         web map in the browser"),
    ("open_t_sdrpp", "  sdrpp       SDR++ (graphical)"),
    ("open_t_satdump", "  satdump     SatDump (graphical)"),
    ("open_unknown", "Unknown target: %s"),
    ("open_missing", "Command not found: %s. Is the component installed?"),
    ("open_no_display", "No graphical session ($DISPLAY/$WAYLAND_DISPLAY empty); cannot open %s here."),
    ("open_url", "Web map: %s"),
    ("open_running", "Running: %s"),
    // Update
    ("upd_step_aur", "AUR packages tracked by yay"),
    ("upd_step_readsb", "readsb (built outside yay)"),
    ("upd_step_services", "Services"),
    ("upd_yay_note", "--devel is required: a -git package does not change version in the AUR when upstream commits."),
    ("upd_not_installed", "%s is not installed; nothing to update. Use \"easy1090 install\"."),
    ("upd_readsb_installed", "Installed: commit %s"),
    ("upd_readsb_upstream", "Upstream: commit %s"),
    ("upd_readsb_current", "readsb is already at the current upstream commit."),
    ("upd_readsb_behind", "There are new commits upstream."),
    ("upd_readsb_unknown", "Could not compare commits (no network, or unexpected version format); skipping."),
    ("upd_readsb_confirm", "Rebuild readsb from HEAD?"),
    ("upd_readsb_skipped", "readsb left at the current version."),
    ("upd_readsb_rebuilt", "readsb rebuilt from commit %s."),
    ("upd_services_restart", "Restarting %s to load the new binaries."),
    ("upd_services_ok", "Nothing changed; no service needs restarting."),
    ("upd_done", "Update complete."),
    ("upd_hint_install", "Run \"easy1090 install\" if you also want to converge the configuration."),
    ("upd_usage", "easy1090 %s - update

USAGE
    easy1090 update [options]

Updates package VERSIONS. To converge configuration and services, use
\"easy1090 install\".

What it does:
  1. yay -Syu --devel on the AUR packages yay tracks (driver, SDR++, SatDump)
  2. compares the readsb commit with upstream HEAD and rebuilds if behind
  3. restarts the services whose binaries changed

OPTIONS
    --skip-aur          do not run yay, update readsb only
    --skip-readsb       leave readsb alone
    --dry-run           print the exact commands, without executing
    --yes               do not ask anything (except the sudo password)
    -h, --help          this help"),
    // Feed
    ("feed_step_cfg", "Feeds"),
    ("feed_step_stats", "Stats package"),
    ("feed_step_info", "Your feeder"),
    ("feed_enabling", "Enabling the feed. Your position and the aircraft you receive will be sent to ADSBExchange."),
    ("feed_disabling", "Disabling the feed. Nothing more will be shared."),
    ("feed_already_on", "Feed is already enabled in the config."),
    ("feed_already_off", "Feed is already disabled."),
    ("feed_connected", "Connected to %s"),
    ("feed_not_connected", "No established connection to ADSBExchange right now."),
    ("feed_readsb_missing", "readsb is not installed yet; the choice was recorded in install.conf, but /etc/default/readsb was not rewritten. Run easy1090 install first."),
    ("feed_stats_present", "adsbexchange-stats service already installed and enabled."),
    ("feed_stats_intro", "The stats package is third party code, from ADSBExchange, and its installer runs as root:"),
    ("feed_stats_repo", "  %s"),
    ("feed_stats_note", "Their script does not run on Arch: it calls adduser, which does not exist here, and dies before installing anything. easy1090 creates the system user first and resolves the dependencies, then hands the rest to their installer, unmodified."),
    ("feed_stats_confirm", "Install the stats package?"),
    ("feed_stats_skipped", "Stats package not installed."),
    ("feed_stats_deps", "Making sure the script dependencies are present (%s)."),
    ("feed_stats_user", "Creating the adsbexchange system user (their script uses adduser, absent on Arch)."),
    ("feed_stats_user_ok", "User adsbexchange already exists."),
    ("feed_stats_cloning", "Cloning the stats repository."),
    ("feed_stats_running", "Running the official ADSBExchange installer."),
    ("feed_stats_ok", "Stats package installed and running."),
    ("feed_stats_failed", "The stats installer failed. Check: journalctl -u adsbexchange-stats -n 30"),
    ("feed_uuid", "Feeder UUID: %s"),
    ("feed_uuid_missing", "UUID not generated yet; the stats service creates it on first run."),
    ("feed_url_stats", "  Your feeder statistics:     %s"),
    ("feed_url_myip", "  Check that you are feeding: https://adsbexchange.com/myip/"),
    ("feed_url_account", "  Link it to an account:      https://account.adsbexchange.com/ (use the UUID above)"),
    ("feed_privacy", "Remember: with JSON_LOCATION_ACCURACY=%s, the published position is %s."),
    ("feed_privacy_exact", "exact"),
    ("feed_privacy_approx", "approximate"),
    ("feed_privacy_none", "not published"),
    ("feed_usage", "easy1090 %s - public network feeds\\n\\nUSAGE\\n    easy1090 feed [network] [options]\\n\\nWith no arguments: lists the networks and their state, changing nothing.\\n\\nNETWORKS\\n    adsbexchange        enable and install their statistics package\\n    airplaneslive       enable (readsb sends directly, no extra package)\\n\\nYou can feed both at once: they are independent connections.\\n\\nOPTIONS\\n    --status            show the current state only, change nothing\\n    --disable           disable the given network\\n    --stats             install or repair the ADSBExchange package only\\n    --dry-run           print the exact commands, without executing\\n    --yes               do not ask anything (except the sudo password)\\n    -h, --help          this help\\n\\nFeeding sends your IP and the aircraft you receive to third parties.\\n"),
    ("feed_datasource_fix", "Pointing stats at /run/readsb (by default it only looks at /run/adsbexchange-feed, from their own feed package)."),
    ("feed_datasource_ok", "Stats data source already configured."),
    ("feed_datasource_restart", "Restarting adsbexchange-stats to apply it."),
    ("feed_datasource_working", "Stats is reading data from readsb."),
    ("feed_datasource_wait", "Stats has not confirmed reading yet; check: journalctl -u adsbexchange-stats -n 20"),
    ("feed_net_adsbx", "ADSBExchange"),
    ("feed_net_alive", "airplanes.live"),
    ("feed_list_title", "Available networks:"),
    ("feed_list_adsbx", "  adsbexchange   %s   does not filter aircraft; sold to JETNET in 2023"),
    ("feed_list_alive", "  airplaneslive  %s   community run, unfiltered, started after that sale"),
    ("feed_list_on", "[on]       "),
    ("feed_list_off", "[off]      "),
    ("feed_list_hint", "To turn one on or off:\\n  easy1090 feed <network>\\n  easy1090 feed <network> --disable\\n\\nYou can feed both at once: they are independent connections."),
    ("feed_unknown_net", "Unknown network: %s. Use adsbexchange or airplaneslive."),
    ("feed_alive_enabling", "Enabling the airplanes.live feed. Your position is not sent, but your IP and the aircraft you receive are."),
    ("feed_alive_disabling", "Disabling the airplanes.live feed."),
    ("feed_alive_note", "airplanes.live needs no extra package for ADS-B: readsb sends it directly. Their official installer is for people who also want to feed MLAT, which requires a separate client."),
    ("feed_alive_url", "  Your feed status:  https://airplanes.live/myfeed/"),
    ("feed_alive_map", "  Network map:       https://globe.airplanes.live/"),
    ("feed_opt_alive", "  3) airplanes.live (community run, unfiltered, started after the ADSBExchange sale)"),
];
