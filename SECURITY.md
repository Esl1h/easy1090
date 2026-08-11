# Security policy

## What this project does with privileges

easy1090 installs system packages, writes files under `/etc`, creates a `udev` rule and enables `systemd` units. It runs as your normal user and escalates with `sudo` only on the steps that need it, and it refuses to run as root at all, because `makepkg` and `yay` do.

Two things are worth knowing before you run it:

**It executes a third party script as root.** `vendor/tar1090-install.sh` comes from [wiedehopf/tar1090](https://github.com/wiedehopf/tar1090). It is vendored rather than downloaded at install time, so the exact bytes are visible in the repository and reviewable in a diff, and it is verified against `TAR1090_INSTALLER_SHA256` in your config before running. If the checksum does not match, easy1090 aborts instead of running it. Updating the pin is a deliberate act, documented in [vendor/README.md](vendor/README.md).

**You can see everything first.** `easy1090 install --dry-run` runs the full read-only preflight and prints the exact commands it would execute, not descriptions of them. Reading that output before a real run is encouraged, and is the reason the flag exists.

## Privacy

Your receiver position is written to `/etc/default/readsb` and, depending on `JSON_LOCATION_ACCURACY`, published in the JSON the web map serves. If the map is reachable outside your network, consider lowering that setting.

Feeding a public tracking network is off by default and asked explicitly on first run. Nothing is shared with third parties unless you turn it on.

`install.conf` holds your coordinates and is git-ignored. Do not commit it.

## Reporting a vulnerability

Do not open a public issue for a security problem.

Write to **not.announced@simplelogin.fr**, ideally with a description of the impact and the steps to reproduce it. I will confirm receipt and keep you informed of the fix.

## Supported versions

This is a young project with a single active line of development. Fixes go into `main` and the next tag; there is no backporting to older tags.
