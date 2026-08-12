# Changelog

Notable changes to easy1090.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- airplanes.live as a second feed network. `easy1090 feed` now lists the networks and their state instead of assuming ADSBExchange, and takes the network as an argument: `easy1090 feed airplaneslive`. Both can be fed at once, since they are independent connectors on the same readsb.

## [0.2.0] - 2026-08-11

Two new commands, both born from things that went wrong on a real machine.

### Added

- `easy1090 feed`, to enable the ADSBExchange feed and install their statistics package. Their installer aborts on Arch because it calls `adduser` with no fallback, under `set -e`, so it dies before copying a single file. easy1090 creates the system user and the dependencies first, then runs their script unmodified.
- `easy1090 update`, for package versions, keeping the separation from `install`, which converges configuration and services. It runs `yay -Syu --devel` and then compares readsb's installed commit with upstream HEAD, because readsb is built outside yay and no yay flag will ever check it.

### Fixed

- The ADSBExchange stats service installed, reported `active` and did nothing: their `json-status` only looks at `/run/adsbexchange-feed`, while a readsb that feeds directly writes to `/run/readsb`. `USE_OLD_PATH=1` in `/etc/default/adsbexchange-stats` is their own escape hatch, and their installer only writes it on a Raspberry Pi image.
- `declare` at file level in the command modules created function-local variables that vanished, since the modules are sourced from inside a function. `declare -g` now. This had made `uninstall` fail on an unbound variable after the single-entrypoint refactor.
- `--dry-run` claimed a config change, and a service restart, on every run: the content comparison was skipped in preview mode.
- Debug messages and the package layer were still in Portuguese inside an otherwise English codebase.

### Changed

- Documentation no longer says "re-running is the update". Re-running converges configuration and services; package versions are `yay -Syu --devel`, or `easy1090 update`.
- `install.conf.example` is bilingual, and the dead `PREFERRED_TERMINAL` key is gone. It came from an earlier design of `open` that spawned terminal windows; the shipped one runs in the current terminal on purpose.
- `KNOWN_ISSUES.md` is in English, like the rest of the project documentation.

## [0.1.0] - 2026-08-11

First release. Tested end to end on a clean Omarchy (Arch) machine with an RTL-SDR Blog V4.

### Added

- Single `easy1090` entrypoint with subcommands: `install`, `uninstall`, `status`, `start`, `stop`, `restart` and `open`. `status` and `open` never escalate privileges.
- Idempotent install: every module detects what is already in place and skips it, so re-running converges configuration and services. Package versions are left to `yay -Syu`.
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

[Unreleased]: https://github.com/Esl1h/easy1090/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Esl1h/easy1090/releases/tag/v0.2.0
[0.1.0]: https://github.com/Esl1h/easy1090/releases/tag/v0.1.0
