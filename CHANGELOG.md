# Changelog

Notable changes to easy1090.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-11

First release. Tested end to end on a clean Omarchy (Arch) machine with an RTL-SDR Blog V4.

### Added

- Single `easy1090` entrypoint with subcommands: `install`, `uninstall`, `status`, `start`, `stop`, `restart` and `open`. `status` and `open` never escalate privileges.
- Idempotent install: re-running is the update, and every module detects what is already in place and skips it.
- `--dry-run` that runs the full read-only preflight and prints the exact commands it would execute.
- Interface in English and Portuguese, chosen on first run and remembered in the config.
- Map links (OpenStreetMap, Google Maps, latlong.net) when asking for the antenna coordinates.
- Explicit opt-in question about feeding a public flight tracking network, off by default.
- `uninstall` that calls tar1090's own uninstaller and cleans up only what easy1090 added, leaving shared packages and system users alone.
- The tar1090 installer vendored under `vendor/` and verified against a checksum pin before running as root.

### Fixed

These are the frictions the installer encodes. Each one was found on real hardware and produces no error message of its own.

- pacman answers "N" to the conflict prompt under `--noconfirm`, silently aborting the install. Removing a conflicting package is now an explicit, confirmed step.
- `blacklist` in modprobe.d only stops autoload at boot; udev still pulls the module by alias on hotplug. The `install <module> /bin/false` line is what actually closes it.
- The stock udev rule grants the dongle to `plugdev`, which works for an interactive user because logind adds a session ACL, and fails for the readsb service user, which has neither.
- Arch's `lighttpd.conf` never includes `conf-enabled`, so everything the tar1090 installer writes there is dead config.
- Arch does not load `mod_redirect`, so tar1090's `url.redirect` is ignored and the slash-less URL returns 404, which is exactly the URL upstream prints when it finishes.
- `systemctl enable --now` does nothing to an already running unit, leaving the daemon on the previous configuration. Services are now restarted when their config actually changes.
- A config file existing does not mean the daemon loaded it. Services whose start time predates the config file are restarted too.
- `yay --removemake` deletes libraries that AUR packages list as both build and optional runtime dependencies. On sdrpp-git that silently broke 10 plugins, including the audio sink. The flag is gone.
- `rtl_test` cannot claim the device while readsb holds it, which is a healthy system on a re-run, not a failure.

[Unreleased]: https://github.com/Esl1h/easy1090/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Esl1h/easy1090/releases/tag/v0.1.0
