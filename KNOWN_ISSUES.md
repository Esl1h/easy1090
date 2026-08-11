# Known issues

A living log of the friction this project knows about. The rule here is to document before fixing: the history of a decision is worth more than the isolated patch, because these problems come back with every GCC release and every PKGBUILD change in the AUR.

## Open

### The readsb build breaks with a current GCC (Mictronics fork)

`readsb-git` (the Mictronics fork) hardcodes `-Werror` in its Makefile and has had no maintenance since around 2020. A current GCC breaks that build in two different ways.

First, new warnings that become errors because of the project's own `-Werror`, such as `unterminated-string-initialization` and `format-truncation`. Removing the flag is enough for those.

Second, and more treacherous, diagnostics that GCC 14 started treating as errors **by default**, regardless of `-Werror`: `incompatible-pointer-types`, `implicit-function-declaration`, `int-conversion` and `implicit-int`. Those need an explicit `-Wno-error=`.

easy1090 uses the wiedehopf fork, which currently compiles clean, so this does not bite. It is recorded because it is a class of problem that hits any unmaintained C package in the AUR, and because nothing guarantees the next GCC will not do the same to the newer fork. If that happens, the fix is a `prepare()` in the PKGBUILD, applied in a copied directory (see the yay entry below).

### Installing without a tty fails or hangs

Every `sudo` step needs a real terminal. Running through automation, CI or a bridge without a tty makes `sudo` either hang waiting for a password or fail silently. Preflight detects this and aborts early, but it bears repeating: easy1090 is not built to run unattended on a first install.

### The tar1090 installer prints "adduser: command not found"

Twice, right at the start. This is noise, not a failure: the upstream line is a fallback chain, `adduser ... || adduser ... || useradd -r -d "$ipath" -M tar1090`. The first two forms are Debianisms that do not exist on Arch, so the final `useradd` is what creates the user. Confirmed on a real install: the `tar1090` user is created normally. Nothing to fix on our side, documented so the message does not alarm anyone.

### lighttpd warns "unknown config-key: url.redirect (ignored)"

A symptom of the missing mod_redirect, described in the resolved section. If the message comes back after an update, it means `06-mod_redirect.conf` disappeared or stopped being loaded.

### The ADSBExchange stats installer dies on Arch

Their `stats.sh` is an eleven line bootstrap that `apt-get install`s git and clones `adsbexchange/adsbexchange-stats`. The real installer is in that repository, and on line 10 it calls `adduser` with no fallback. Arch has `useradd`, not `adduser`, and the script runs under `set -e`, so it aborts right there, having created only an empty `/usr/local/share/adsbexchange-stats`. Nothing else runs: no files copied, no service, no UUID.

The message you see on the ADSBExchange site ("You do not have the stats package configured") therefore never goes away, no matter how many times you run their command.

`easy1090 feed` works around it the same way the tar1090 module does, without patching upstream: it creates the `adsbexchange` system user first, so their `id -u` check passes and the adduser branch is skipped, installs the dependencies their script only knows how to fetch with apt or yum (`curl jq gzip perl bind`, where `bind` provides `host`), and then hands over to their `install.sh` unmodified.

Two lesser Debianisms in the same script are harmless: `ischroot` is missing on Arch but only used as an `if` condition, and `vcgencmd` is Raspberry Pi only, likewise guarded.

### No real end to end CI

The full path cannot be tested without an RTL-SDR attached. What can be automated is the static part: `shellcheck`, syntax checks, catalog consistency, the vendored checksum, and the preflight refusing a non-Arch distro. End to end validation stays manual, on a VM or physical machine, after every relevant change in the AUR or in GCC.

## Resolved in the code

These stay here because, if upstream behaviour ever changes, the reason the code exists needs to be written down somewhere.

### pacman with --noconfirm takes the wrong default on conflicts

The prompt to replace a conflicting package is `[y/N]`, and `--noconfirm` answers N, aborting the install without making the reason obvious. That is why removing a conflicting package is an explicit, confirmed step, never delegated to `--noconfirm`.

### yay resets the cached PKGBUILD

Any manual edit to `~/.cache/yay/<pkg>/PKGBUILD` is discarded on the next run. It is correct anti-tampering behaviour, and incompatible with manual patching. Builds that need a patch run from their own directory, outside yay's control.

### Blacklisting a module does not stop it being loaded by alias

`blacklist dvb_usb_rtl28xxu` in modprobe.d only stops autoload at boot. On hotplug, udev asks for the module by alias and the kernel hands it over, blacklist or not. Only the `install dvb_usb_rtl28xxu /bin/false` directive actually closes it. This is missing from almost every tutorial, and it is the difference between a system that works and one that works until you replug the dongle.

### A group-based udev rule does not cover a service user

The stock `rtl-sdr` rule uses `GROUP="plugdev"`. For an interactive user this appears to work, because systemd-logind grants a session ACL on the device, which masks the problem. The readsb service user has no session, gets no ACL, and hits EACCES. The fix is a rule dedicated to the service's own group.

### Arch's lighttpd does not load conf-enabled

The `lighttpd.conf` shipped by the Arch package has only the essentials and includes no extra configuration directory, unlike Debian. Without adding the `include_shell` line, everything the tar1090 installer writes there is dead config, silently, and the map answers 404 with everything apparently installed.

### mod_redirect is not loaded on Arch, and the slash-less URL 404s

The `88-tar1090.conf` generated by the installer uses `url.redirect` to send `/tar1090` to `/tar1090/`, but the installer only creates module loaders for `mod_alias` and `mod_setenv`. On Debian `mod_redirect` is already on in the base config; on Arch it is not. lighttpd then warns `unknown config-key: url.redirect (ignored)` and carries on, and the slash-less URL returns 404. Worse: that is exactly the URL the installer prints when it finishes. easy1090 creates `06-mod_redirect.conf` in `conf-available` and enables it by symlink.

### The tar1090 installer does not start a fresh lighttpd

The upstream script only restarts lighttpd if it was already active. A freshly installed one stays `inactive (dead)` and disabled, with everything else configured correctly.

### `systemctl enable --now` does not apply new config to a running service

Found on the second real install. The installer wrote `/etc/default/readsb` with new coordinates and `enable --now` did nothing, because the unit was already active. The result: the daemon kept running on the previous configuration, with no error anywhere, and `journalctl` showed the old lat/lon while the file on disk had the new one. The same applied to lighttpd after enabling `mod_redirect`.

The fix is for `run::sudo_write` to report whether the content actually changed (`FILE_CHANGED`), and for the modules to `restart` explicitly when it did. It is the same trap already documented in the upstream tar1090 installer, which I then reproduced.

### `rtl_test` fails while readsb is running

On a re-run, readsb already owns the device, so `rtl_test` enumerates the card but cannot claim the interface, ending with `usb_claim_interface error -6`. That is not a defect, it is a healthy system. The driver module recognises this output and skips the test instead of warning that it could not identify the tuner.

### A config file existing does not mean the service loaded it

A continuation of the previous entry, and the part the first fix did not cover. On the third real run, `mod_redirect` showed up as `[SKIP] already enabled`, because the file and the symlink existed, yet the slash-less URL still returned 404. The reason: the file had been created at 15:45 by an earlier run that did not restart lighttpd, and the daemon had been up since 15:39. Presence of a file does not prove the process read it.

The fix is `svc::predates_file`, which compares the unit's `ActiveEnterTimestamp` with the file's mtime. If the service started before the file was written, it cannot have loaded it, and a restart is scheduled even on the `[SKIP]` path.

### yay's `--removemake` breaks optional dependencies

Found while testing `--full` on a clean machine. easy1090 called yay with `--removemake`, which discards packages installed as build dependencies once the build finishes. It looks like sensible housekeeping, and it is the exact opposite.

AUR packages routinely list the same library in both `makedepends` and `optdepends`: it is needed to compile a plugin, and needed again at runtime to load it. yay only sees the build side and removes it. The program stays installed, the plugins stay on disk, and nothing complains.

Measured on `sdrpp-git`: 24 packages removed at the end of the install, leaving 10 plugins without their libraries, among them `audio_sink.so`, which means SDR++ ended up with no audio output. No error message anywhere.

The fix is not to use `--removemake`. The cost is leaving build tooling on disk, reclaimable with `yay -Yc` by anyone who cares more about space than about the plugins working.

### readsb is invisible to yay's update check

Not a defect, but a consequence worth writing down. readsb is built with `makepkg` from a fresh clone, outside yay, so that a `prepare()` patch stays possible when a new GCC breaks the build (see the first entry). The side effect is that yay never records it in its VCS database, so `yay -Syu --devel` will not offer an update for it, ever.

Measured on the reference server after a full system update: `rtl-sdr-blog-git`, `sdrpp-git` and `airspyhf-git` were tracked and checked, while `readsb-wiedehopf-git` sat at commit `g0bfd047` with upstream already at `d418ad6`.

Two separate things bite here, and the second surprises people who know the first:

`yay -Syu` alone never updates a `-git` package. It compares the installed version with the one declared in the AUR, and for a VCS package that string does not change when upstream commits. You need `yay -Syu --devel`, which checks the actual git HEAD.

Even with `--devel`, yay only checks devel packages **it** built. Anything installed by hand with `makepkg` is not in `~/.cache/yay/vcs.json` and is skipped silently.

The documented way to move readsb forward is to remove the package and re-run the installer, which rebuilds from a fresh clone of upstream HEAD. See the README.
