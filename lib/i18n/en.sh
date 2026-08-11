#!/usr/bin/env bash
#===============================================================================
# easy1090 - message catalog (English)
#===============================================================================

# shellcheck disable=SC2034

#-------------------------------------------------------------------------------
# Common
#-------------------------------------------------------------------------------
MSG[yes_no]="[y/N] "
MSG[yes_chars]="SsYy"
MSG[cfg_missing_dry]="Config does not exist yet; it would be created from the example."
MSG[cfg_created]="Config created at %s (from the example)."
MSG[cfg_example_missing]="Example config not found: %s"
MSG[sudo_validating]="Validating sudo (your password may be requested now)."
MSG[sudo_failed]="Could not validate sudo."
MSG[cmd_required]="Required command not found: %s"

#-------------------------------------------------------------------------------
# Receiver position
#-------------------------------------------------------------------------------
MSG[pos_intro]="The antenna position is used to compute range and distance to aircraft."
MSG[pos_help_title]="To find your coordinates, use one of these:"
MSG[pos_help_osm]="  OpenStreetMap   https://www.openstreetmap.org   (right click the spot, \"Show address\")"
MSG[pos_help_gmaps]="  Google Maps     https://maps.google.com        (right click the spot)"
MSG[pos_help_latlong]="  latlong.net     https://www.latlong.net        (search by address)"
MSG[pos_help_tip]="Enter decimal degrees with a dot. South and west are negative."
MSG[pos_lat]="Latitude (e.g. -23.58): "
MSG[pos_lon]="Longitude (e.g. -46.55): "
MSG[pos_required]="Latitude and longitude are required."
MSG[pos_required_yes]="RECEIVER_LAT/RECEIVER_LON are required with --yes. Fill them in %s."
MSG[pos_dry]="RECEIVER_LAT/RECEIVER_LON are empty; a real run would use the values you provide."
MSG[pos_saved]="Position saved to %s"

#-------------------------------------------------------------------------------
# Data sharing
#-------------------------------------------------------------------------------
MSG[feed_title]="Share your data with a public flight tracking network?"
MSG[feed_explain]="This sends the aircraft you receive, and your receiver position, to third party servers. In return those sites usually grant premium access to contributors."
MSG[feed_opt_none]="  1) Do not share (default, everything stays on your network)"
MSG[feed_opt_adsbx]="  2) ADSBExchange (adsbexchange.com, no aircraft filtering)"
MSG[feed_fa_note]="FlightAware is not listed here: feeding their network requires the piaware client, with its own registration and feeder ID; a plain beast connector does not work."
MSG[feed_prompt]="Choose [1]: "
MSG[feed_none]="Local feed only; nothing will be shared."
MSG[feed_enabled]="Feed enabled: %s. Your position will be shared."

#-------------------------------------------------------------------------------
# Preflight
#-------------------------------------------------------------------------------
MSG[pre_step]="Preflight"
MSG[pre_root]="Do not run as root. Use your normal user; the script asks for sudo when needed (makepkg and yay refuse to run as root)."
MSG[pre_user_ok]="Normal user (uid %s)."
MSG[pre_tty_dry]="TTY: check relaxed under --dry-run."
MSG[pre_tty_missing]="No interactive terminal. sudo needs a real tty; run this from a terminal or a proper SSH session."
MSG[pre_tty_ok]="Interactive terminal available."
MSG[pre_osrelease]="/etc/os-release not found; could not identify the distro."
MSG[pre_distro_ok]="Compatible distro: %s"
MSG[pre_distro_bad]="v1 supports Arch and derivatives only (detected: %s). Other distros are on the roadmap; see the README."
MSG[pre_tools_missing]="Missing essential commands: %s"
MSG[pre_yay_missing]="yay not found. Install an AUR helper first (easy1090 needs it for readsb and the other AUR packages)."
MSG[pre_tools_ok]="Tools present: %s"
MSG[pre_lsusb_missing]="lsusb not found (usbutils package); skipping the dongle check."
MSG[pre_dongle_ok]="RTL-SDR detected on the USB bus (%s)."
MSG[pre_dongle_missing]="No RTL-SDR found in lsusb (%s)."
MSG[pre_dongle_warn]="The install continues, but nothing will decode without the dongle plugged in."
MSG[pre_dongle_confirm]="Continue anyway?"
MSG[pre_aborted]="Aborted by the user."
MSG[pre_done]="Preflight complete."

#-------------------------------------------------------------------------------
# Driver
#-------------------------------------------------------------------------------
MSG[drv_step]="RTL-SDR driver"
MSG[drv_installed]="%s already installed."
MSG[drv_conflict]="The generic %s package conflicts with the RTL-SDR Blog fork."
MSG[drv_conflict_confirm]="Remove %s now?"
MSG[drv_conflict_abort]="Without removing the conflict, installing the fork fails."
MSG[drv_blacklist_ok]="Blacklist for %s already configured."
MSG[drv_blacklist_set]="Configuring blacklist for %s"
MSG[drv_module_unload]="Unloading %s (currently loaded)."
MSG[drv_module_unload_fail]="Could not unload %s; a reboot may be needed."
MSG[drv_module_absent]="%s is not loaded."
MSG[drv_test_missing]="rtl_test not found in PATH; skipping driver validation."
MSG[drv_test_running]="Validating the hardware with rtl_test."
MSG[drv_no_device]="No supported device found."
MSG[drv_no_device_hint]="Check the cable and the USB port (prefer rear ports wired straight to the motherboard)."
MSG[drv_tuner_v4]="R828D tuner detected (RTL-SDR Blog V4)."
MSG[drv_tuner_v3]="R820T/R820T2 tuner detected (v3 or clone)."
MSG[drv_tuner_unknown]="The dongle answered, but the tuner was not identified. Full output with --verbose."

#-------------------------------------------------------------------------------
# readsb
#-------------------------------------------------------------------------------
MSG[rsb_step]="readsb (ADS-B decoder)"
MSG[rsb_installed]="%s already installed."
MSG[rsb_conflict]="%s (Mictronics fork) conflicts with %s and writes protobuf instead of JSON."
MSG[rsb_conflict_confirm]="Remove %s now?"
MSG[rsb_conflict_abort]="The two packages cannot coexist; removal is required to continue."
MSG[rsb_legacy_override]="Old systemd override found (references \$USER_OPTIONS, which this package does not define)."
MSG[rsb_legacy_removed]="Legacy override removed."
MSG[rsb_udev_ok]="readsb udev rule already present."
MSG[rsb_udev_create]="Creating the udev rule for the readsb service user."
MSG[rsb_udev_comment]="# easy1090: hands the dongle to the readsb service user's group.\n# The stock rule uses GROUP=\"plugdev\", which does not cover a session-less user."
MSG[rsb_defaults_write]="Writing %s"
MSG[rsb_defaults_header]="# Generated by easy1090. Edit freely: the installer only rewrites this\n# file when you run install.sh again."
MSG[rsb_enabling]="Enabling and starting the readsb service."
MSG[rsb_active]="readsb is active."
MSG[rsb_failed]="readsb did not start. Check: journalctl -u readsb -n 40 --no-pager"
MSG[rsb_json_ok]="JSON being written to %s"
MSG[rsb_json_wait]="%s does not exist yet; it may take a few seconds."

#-------------------------------------------------------------------------------
# tar1090
#-------------------------------------------------------------------------------
MSG[tar_step]="tar1090 (live web map)"
MSG[tar_confd_ok]="%s already exists."
MSG[tar_confd_create]="Creating %s (the tar1090 installer looks for it)."
MSG[tar_vendor_missing]="tar1090 installer not found at %s"
MSG[tar_pin_missing]="TAR1090_INSTALLER_SHA256 is not set in the config."
MSG[tar_pin_current]="Current checksum of the vendored file: %s"
MSG[tar_pin_hint]="Pin that value in the config to catch future changes."
MSG[tar_pin_mismatch]="tar1090 installer checksum does not match."
MSG[tar_pin_expected]="  expected: %s"
MSG[tar_pin_got]="  got:      %s"
MSG[tar_pin_abort]="Refuse to run a modified root script. Review the file before updating the pin."
MSG[tar_pin_ok]="Vendored installer matches the pin in the config."
MSG[tar_service_ok]="tar1090 service already enabled."
MSG[tar_running]="Running the official tar1090 installer (vendored)."
MSG[tar_conf_missing]="%s not found; skipping the include fix."
MSG[tar_include_ok]="lighttpd.conf already includes conf-enabled."
MSG[tar_include_add]="Adding the conf-enabled include to lighttpd.conf (the Arch default lacks it)."
MSG[tar_lighttpd_check]="Validating the lighttpd config before starting it."
MSG[tar_lighttpd_invalid]="Invalid lighttpd config. Review %s before continuing."
MSG[tar_redirect_ok]="mod_redirect already enabled."
MSG[tar_redirect_add]="Enabling mod_redirect (tar1090 uses url.redirect, and Arch does not load that module by default)."
MSG[tar_redirect_comment]="# easy1090: tar1090 uses url.redirect for the slash-less URL."
MSG[tar_lighttpd_enable]="Enabling and starting lighttpd."
MSG[tar_web_ok]="Web map responding at %s"
MSG[tar_web_slashless]="%s (no trailing slash) returned %s; the redirect is not active."
MSG[tar_web_fail]="Web map returned HTTP %s."
MSG[tar_web_hint]="Check: systemctl status lighttpd tar1090"

#-------------------------------------------------------------------------------
# Optional
#-------------------------------------------------------------------------------
MSG[opt_sdrpp_step]="SDR++ (spectrum viewer)"
MSG[opt_sdrpp_note]="SDR++ does not decode ADS-B; it is for eyeballing RF energy at 1090 MHz."
MSG[opt_satdump_step]="SatDump (satellite decoder)"
MSG[opt_satdump_slow]="The SatDump build is long (about 45 minutes on the reference hardware)."
MSG[opt_satdump_confirm]="Continue with the SatDump install?"
MSG[opt_satdump_skipped]="SatDump skipped."

#-------------------------------------------------------------------------------
# Validation
#-------------------------------------------------------------------------------
MSG[val_step]="Final validation"
MSG[val_unit_missing]="Unit not found: %s.service"
MSG[val_service_ok]="%s: active and enabled at boot."
MSG[val_service_bad]="%s: active=%s enabled=%s"
MSG[val_json_missing]="%s does not exist. Is readsb writing JSON?"
MSG[val_json_stale]="aircraft.json has been stale for %ss; readsb may have hung."
MSG[val_decoding_ok]="readsb decoding (JSON refreshed %ss ago, %s aircraft on screen)."
MSG[val_zero_aircraft]="Zero aircraft right now is normal: it depends on traffic, antenna and line of sight."
MSG[val_web_bad]="http://localhost/tar1090/ returned %s."
MSG[val_web_ok]="tar1090 serving map and data."
MSG[val_web_data_bad]="The map responds, but /tar1090/data/aircraft.json does not. Check the tar1090 service."
MSG[val_all_ok]="Everything is up."
MSG[val_failures]="%s check(s) failed."
MSG[val_howto]="How to watch the traffic:"
MSG[val_howto_viewadsb]="live table in the terminal"
MSG[val_howto_nc]="decoded messages (SBS/CSV)"
MSG[val_howto_map]="live web map"

#-------------------------------------------------------------------------------
# CLI
#-------------------------------------------------------------------------------
MSG[cli_dry_warning]="--dry-run mode: nothing will be changed; the commands below are the real ones."
MSG[cli_unknown_opt]="Unknown option: %s"
MSG[cli_usage]="easy1090 %s - ADS-B stack installer (Arch and derivatives)

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
The flags above override whatever is in there.
"

#-------------------------------------------------------------------------------
# status.sh
#-------------------------------------------------------------------------------
MSG[sts_title]="status"
MSG[sts_hardware]="Hardware and driver"
MSG[sts_decoding]="Decoding"
MSG[sts_web]="Web"
MSG[sts_optional]="Optional"
MSG[sts_running]="running"
MSG[sts_stopped]="stopped"
MSG[sts_installed]="installed"
MSG[sts_absent]="absent"
MSG[sts_driver]="driver"
MSG[sts_service]="service"
MSG[sts_map]="web map"
MSG[sts_decode_row]="decoding"
MSG[sts_lsusb_missing]="lsusb not installed"
MSG[sts_dongle_found]="detected on USB (%s)"
MSG[sts_dongle_absent]="nothing in lsusb"
MSG[sts_unit_absent]="unit not installed"
MSG[sts_json_absent]="%s does not exist"
MSG[sts_jq_missing]="jq not installed"
MSG[sts_json_fresh]="JSON from %ss ago, %s aircraft"
MSG[sts_json_stale]="JSON stale for %ss"
MSG[sts_lighttpd_down]="lighttpd inactive"
MSG[sts_http]="HTTP %s"

#-------------------------------------------------------------------------------
# Restart / device busy
#-------------------------------------------------------------------------------
MSG[rsb_restarting]="Config changed; restarting readsb to apply it."
MSG[tar_lighttpd_restart]="Config changed; restarting lighttpd to apply it."
MSG[drv_busy]="Dongle already in use by readsb (expected on a re-run); skipping rtl_test."
MSG[drv_busy_v4]="Dongle already in use by readsb (RTL-SDR Blog V4); skipping rtl_test."

#-------------------------------------------------------------------------------
# Packages
#-------------------------------------------------------------------------------
MSG[pkg_installed]="Already installed: %s"
MSG[pkg_pacman]="Installing via pacman: %s"
MSG[pkg_absent]="Not installed, nothing to remove: %s"
MSG[pkg_removing]="Removing package: %s"
MSG[pkg_aur]="Installing from the AUR: %s"
MSG[pkg_clean_build]="Cleaning previous build: %s"
MSG[pkg_cloning]="Cloning PKGBUILD for %s"
MSG[pkg_building]="Building and installing (%s)"
MSG[pkg_build_missing]="Build directory not found: %s"

#-------------------------------------------------------------------------------
# Uninstall
#-------------------------------------------------------------------------------
MSG[un_title]="Uninstall"
MSG[un_plan]="What will be removed:"
MSG[un_plan_driver]="  driver      rtl-sdr-blog-git package and the DVB module blacklist"
MSG[un_plan_readsb]="  readsb      service, package, /etc/default/readsb and the udev rule"
MSG[un_plan_tar1090]="  tar1090     service, files and the lighttpd configs"
MSG[un_plan_optional]="  optional    SDR++ and SatDump (if installed)"
MSG[un_plan_local]="  local       build cache in ~/.cache/easy1090"
MSG[un_keep]="What will NOT be touched: lighttpd, jq, the readsb and tar1090 system users, and anything you installed yourself."
MSG[un_confirm]="Confirm removal?"
MSG[un_aborted]="Nothing was removed."
MSG[un_step_tar1090]="tar1090"
MSG[un_step_readsb]="readsb"
MSG[un_step_driver]="RTL-SDR driver"
MSG[un_step_optional]="optional"
MSG[un_nothing]="Nothing to remove here."
MSG[un_step_local]="local files"
MSG[un_upstream]="Running tar1090's own uninstaller (%s)."
MSG[un_upstream_missing]="tar1090 uninstaller not found; removing what easy1090 created."
MSG[un_include_removed]="Removed the include line easy1090 appended to lighttpd.conf."
MSG[un_lighttpd_restart]="Restarting lighttpd."
MSG[un_stopping]="Stopping and disabling %s."
MSG[un_removed]="Removed: %s"
MSG[un_absent]="Does not exist, nothing to do: %s"
MSG[un_pkg_kept]="Packages kept (--keep-packages)."
MSG[un_config_ask]="Also remove install.conf (your coordinates and preferences)?"
MSG[un_done]="Uninstall complete."
MSG[un_users_note]="The readsb and tar1090 system users still exist; remove them manually with userdel if you want."
MSG[un_usage]="easy1090 %s - uninstaller

USAGE
    ./uninstall.sh [options]

OPTIONS
    --keep-packages     remove services and configs, but keep the packages
    --lang <pt|en>      interface language
    --dry-run           print the exact commands, without executing
    --yes               do not ask anything (except the sudo password)
    --verbose           debug level logging
    -h, --help          this help

Does not remove lighttpd or jq, which are general purpose packages.
"

#-------------------------------------------------------------------------------
# Entrypoint, services and open
#-------------------------------------------------------------------------------
MSG[main_usage]="easy1090 %s - ADS-B stack in one command (Arch and derivatives)\n\nUSAGE\n    easy1090 <command> [options]\n\nCOMMANDS\n    install       install the stack (idempotent: re-running is the update)\n    uninstall     undo the installation (best effort)\n    status        what is running, what fell over, what is missing\n    start         bring up readsb, lighttpd and tar1090\n    stop          bring all three down\n    restart       restart all three, in the right order\n    open [target] open a component (without a target, lists the options)\n\nGLOBAL OPTIONS\n    --lang <pt|en>   interface language\n    --dry-run        print the exact commands, without executing\n    --yes            do not ask anything (except the sudo password)\n    --verbose        debug level logging\n    --version        show version\n    -h, --help       this help\n\nUse \"easy1090 <command> --help\" for per-command options.\n\nstatus and open do not need sudo.\n"
MSG[cmd_unknown]="Unknown command: %s"
MSG[cmd_missing]="Please provide a command. Use --help for the list."
MSG[svc_step]="Services"
MSG[svc_acting]="%s: %s"
MSG[svc_not_installed]="%s is not installed; skipping."
MSG[svc_done]="Done."
MSG[open_step]="Open"
MSG[open_targets]="Available targets:"
MSG[open_t_viewadsb]="  viewadsb    live table in the terminal (ncurses)"
MSG[open_t_sbs]="  sbs         decoded message stream (CSV)"
MSG[open_t_map]="  map         web map in the browser"
MSG[open_t_sdrpp]="  sdrpp       SDR++ (graphical)"
MSG[open_t_satdump]="  satdump     SatDump (graphical)"
MSG[open_unknown]="Unknown target: %s"
MSG[open_missing]="Command not found: %s. Is the component installed?"
MSG[open_no_display]="No graphical session (\$DISPLAY/\$WAYLAND_DISPLAY empty); cannot open %s here."
MSG[open_url]="Web map: %s"
MSG[open_running]="Running: %s"
