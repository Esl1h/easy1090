layout: single
title: Troubleshooting
permalink: /troubleshooting
description: Behaviors that look like bugs but are not, and the third party quirks easy1090 works around.
toc: true
toc_label: "On this page"
---

# Troubleshooting

## Things that are not failures

- **`rtl_test -t` ends with "No E4000 tuner found, aborting."** The `-t` flag is an E4000 specific gain test, and the V4 has an R828D. Not a failure. The detection lines above it are what matter.
- **`rtl_test` reports an interface claim error on a re-run.** readsb already owns the device. That is a healthy system: the device line still tells you the model was recognized.
- **The tar1090 installer prints `adduser: command not found` twice.** Noise. The upstream line is a fallback chain whose Debian branches do not exist on Arch; the final `useradd` is what creates the user.
- **lighttpd warns `unknown config-key: url.redirect (ignored)`.** A symptom of the missing `mod_redirect`, solved by the installer. If it comes back after an update, the module loader file disappeared or stopped being read.
- **SatDump packaging lint: "package contains reference to $srcdir".** Informational only: some binaries reference the build path in metadata.

## Third party incompatibilities easy1090 resolves

### ADSBExchange stats installer aborts on Arch

Their installer calls `adduser` with no fallback and runs under `set -e`, so it dies on line 10 having created only an empty directory. The message on their site ("You do not have the stats package configured") then persists no matter how many times you rerun their command. easy1090 creates the `adsbexchange` system user first (so their `id -u` check passes and the `adduser` branch is skipped), installs the dependencies their script only fetches with `apt`/`yum` (`curl jq gzip perl bind`), then hands their script over unmodified.

### The stats service loops on "No valid data source directory"

Their `json-status` only checks `/run/adsbexchange-feed`, created by their separate *feed* package. easy1090 feeds straight from readsb, so the JSON lives in `/run/readsb`. The escape hatch is theirs: `USE_OLD_PATH=1` in `/etc/default/adsbexchange-stats` makes it try `/run/readsb` first. Their installer only writes it when it detects a Raspberry Pi image, which is why it never lands on an ordinary machine. easy1090 writes it and restarts the service.qui

The nasty part: the service reports `active`, `enabled` and green the whole time, with no message that the failure is silent.

### airplanes.live installer: same `adduser` bug

Their `update.sh` is for the MLAT client, which easy1090 does not need for ADS-B feeding: the standard feed is a single `--net-connector` on readsb. Their installer exists only for MLAT.

## Updating after install

```bash
./easy1090 update
```

Runs `yay -Syu --devel` (required: AUR `-git` packages never move in a plain `yay -Syu`, the reasoning is on the [design page](/easy1090/design)) and then handles readsb itself, comparing the installed commit with upstream HEAD. The `.gXXXXXXX` suffix printed by `pacman -Q readsb-wiedehopf-git` is the upstream commit the package was built from.

## Two things not yet exercised by real hardware tests

- The removal path for a conflicting Mictronics `readsb-git` install needs a machine that already has the old fork.
- A full `--full` run including SatDump has not yet been exercised end to end on real hardware; both open gaps are also listed in the repository's KNOWN_ISSUES.md.

If nothing here matches your symptom, open an issue on GitHub; bug reports from real installations are the most valuable contributions to this project.
