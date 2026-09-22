---
layout: default
title: Security
nav_order: 6
permalink: /security/
description: What runs as root, how it is made auditable, and what never gets escalated.
---

# Security model

An installer for hardware access that touches systemd, udev and package management has to run steps as root. easy1090 keeps that surface small and auditable.

## Refuses to run as root

The opposite of the typical installer: it aborts if executed as root, because `makepkg` and `yay` refuse that way and because per-step escalation is more auditable than one giant root shell. A single `sudo -v` validates the password up front and a background keepalive holds the timestamp through long builds, so you are not asked again mid-compile.

## The vendored tar1090 installer

Fetching and executing a script from the network as root on every run (`curl | sudo bash`) is exactly the pattern easy1090 refuses to copy. Instead:

- The official tar1090 installer lives in `vendor/tar1090-install.sh`, byte-identical to upstream (GPL v2+ by Matthias Wirth, with its own license file alongside).
- A SHA-256 pin lives in `install.conf`. Every run verifies the file against the pin and aborts on a mismatch. Updating the pin is a deliberate act: download the new version, read the diff, change the pin.
- CI asserts that the committed bytes still match the pinned checksum, so a drifted vendor file cannot merge without the pin moving with it.

The ADSBExchange stats code is pulled from their repository on install (cloned in full, then executed) rather than vendored, because it is optional and rarely updated; their bootstrap is skipped entirely and their installer is invoked directly, still unmodified.

## What never escalates

`status` and `open` are read-only: no sudo, no writes, safe anywhere. `--dry-run` prints the exact commands a real run would execute and never touches root. The preflight runs read-only in full even under dry-run, which is how you can inspect what will happen without running it.

## What is deliberately kept out

- FlightAware feeding is not offered (their client owns the registration flow).
- Feeding public networks, both of which share your received data and your position, defaults to off and is asked explicitly on the first install. `FEEDER_ADSBEXCHANGE=false` stays off until you decide.
- A conflicting package is never removed without a prompt; a root script that silently trades packages for you is not something this project ships.
