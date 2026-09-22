layout: single
title: Design decisions
permalink: /design/
description: The Arch specific friction easy1090 resolves, and why each workaround looks the way it does.
toc: true
toc_label: "On this page"
---

# Design decisions

Every item here cost real debugging on Arch hardware. They are listed not as trivia but as the reasons the installer is shaped the way it is.

## Package layer

- **`pacman --noconfirm` answers N to conflict prompts**, silently aborting the install. Replacing a conflicting package (`rtl-sdr` when installing the Blog fork, `readsb-git` when installing the wiedehopf fork) is always an explicit, confirmed step. Never a side effect of a flag.
- **`yay` resets the cached PKGBUILD on every run.** Anti-tampering behavior, correct, and incompatible with manual patches. Builds that need a patch run from a copied directory, outside `~/.cache/yay/`; easy1090 keeps that habit even for packages that currently build clean, because the next GCC release tends to break exactly these unmaintained C projects.
- **`yay --removemake` is never used.** AUR packages routinely list the same library in both `makedepends` and `optdepends`: needed to build a plugin, needed again at runtime to load it. `yay` only sees the make side and removes it. Measured on `sdrpp-git`: ten plugins lost their libraries, including the audio sink, with no error anywhere. The cost is leaving build tooling on disk; reclaim it with `yay -Yc` if space matters more.

## Kernel and device layer

- **Blacklisting a module does not stop hotplug.** Boot-time autoload is the only thing `blacklist` blocks; udev requests the module by alias on replug and the kernel grants it. The `install <module> /bin/false` line is what holds.
- **A group-based udev rule does not cover a service user.** The stock rtl-sdr rule grants the dongle to `plugdev`, and it appears to work because systemd-logind gives your interactive session an ACL. The `readsb` service user has no session and no ACL, so it hits EACCES. easy1090 writes a dedicated rule for the service's own group. The problem hides precisely because it works when you test by hand.

## Web layer

- **Arch's lighttpd.conf never reads conf-enabled.** Everything the tar1090 installer writes there is parsed by nothing. easy1090 appends the `include_shell` line and validates the config with `lighttpd -tt` before going live.
- **Arch does not load `mod_redirect`.** tar1090's config uses `url.redirect` to send `/tar1090` to `/tar1090/`, and the URL upstream prints when it finishes is the slash-less one, which would 404. easy1090 creates and enables a module loader for it.
- **The upstream tar1090 installer only restarts lighttpd if it was already running.** A freshly installed one stays dead and disabled. easy1090 enables and starts it explicitly.

## systemd layer

- **`systemctl enable --now` is a no-op on an already running unit.** A previous real install wrote new coordinates and the daemon kept running on the old configuration, with no error anywhere. easy1090 tracks whether the file it wrote actually changed (`FILE_CHANGED`) and also compares the service's start timestamp against the file's mtime, restarting the unit when it predates its own config. Presence of a file does not prove the process read it.
- **`set -e` and `((x++))` do not mix.** The final validation counts failures with `failures=$((failures + 1))`, because a post-increment from zero evaluates to zero, returns a non-zero status and would abort the run on the first failure it was trying to count.

## The install.conf contract

Plain sourceable shell, `/etc/default/*` style, no parser dependency. It doubles as a declaration of the desired state: the installer is idempotent and re-running converges configuration and services to whatever is written there. It does **not** touch package versions on a converge; keeping packages current is `yay -Syu --devel` or `easy1090 update`, on your schedule, because pulling a new AUR version behind your back could start a 45 minute SatDump rebuild.
