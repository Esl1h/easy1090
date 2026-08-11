# easy1090

**English** · [Português](README.pt-BR.md)

One command installer for a complete ADS-B stack (RTL-SDR + readsb + tar1090) on Arch and derivatives.

ADS-B guides almost always assume Raspberry Pi OS or Debian: the official scripts use `apt-get`, Debian's `lighttpd` already ships the `conf.d`/`conf-enabled` layout, and the AUR packages of older decoders are not tested against current GCC. Every one of those costs real debugging on an Arch-based box. easy1090 automates the whole thing with the friction already sorted out.

What you end up with: the correct driver for the RTL-SDR Blog V4, `readsb` decoding ADS-B at 1090 MHz, and a live web map, all with permissions, `udev` rules and `systemd` services in place, surviving reboots.

## Requirements

- Arch Linux or a derivative (EndeavourOS, Omarchy, Manjaro, CachyOS)
- An AUR helper (`yay`)
- An RTL-SDR plugged in, ideally the RTL-SDR Blog V4
- A real terminal, because `sudo` needs a tty

## Usage

```bash
git clone https://github.com/Esl1h/easy1090.git
cd easy1090
./easy1090 install
```

`./install.sh` still works as a shortcut for `./easy1090 install`.

Before running anything as root on your machine, see what it would do:

```bash
./install.sh --dry-run
```

`--dry-run` runs the full preflight (read-only) and prints the exact commands, not descriptions of them. It is the same text you could paste into a terminal yourself.

On first run it asks for your interface language (Portuguese or English), your antenna coordinates, and whether you want to share data with a public flight tracking network. All three are remembered in `install.conf`.

### Options

```
--full              install everything, including SDR++ and SatDump
--lang <pt|en>      interface language
--lat <degrees>     antenna latitude (e.g. -23.58)
--lon <degrees>     antenna longitude (e.g. -46.55)
--skip-tar1090      do not install the web map
--skip-sdrpp        do not install SDR++
--skip-satdump      do not install SatDump
--dry-run           print the exact commands, without executing
--yes               do not ask anything (except the sudo password)
--verbose           debug level logging
```

### After installing

```bash
./easy1090 status        # what is running, what fell over, what is missing
./easy1090 restart       # restart readsb, lighttpd and tar1090 in order
./easy1090 open          # list what you can open
./easy1090 open viewadsb # live table in the terminal
./easy1090 open map      # web map in the browser (prints the URL if headless)
```

`status` and `open` never ask for sudo. `open` runs in the current terminal instead of spawning a window, because this stack usually lives on a headless box.

To undo it all:

```bash
./easy1090 uninstall --dry-run   # see exactly what would be removed
./easy1090 uninstall
```

It calls tar1090's own uninstaller, removes the services, configs and packages easy1090 installed, and deliberately leaves shared packages (`lighttpd`, `jq`) and the system users alone. `--keep-packages` removes only services and configs.

And the web map at `http://YOUR-SERVER-IP/tar1090/`.

## Configuration

Configuration lives in `install.conf`, created from `install.conf.example` on first run. It is plain sourceable shell, in the `/etc/default/*` style, with no parser dependency.

Because the installer is idempotent, that file works as a declaration of the desired state of the machine: running it again converges to whatever is written there. There is no separate `--update` mode, re-running **is** the update.

Two settings worth reading before the first run:

`JSON_LOCATION_ACCURACY` controls how precisely your receiver position is exposed in the JSON and on the map. The default is exact; if the map is reachable outside your network, consider lowering it to approximate, or not publishing it at all.

`FEEDER_ADSBEXCHANGE` is off. Enabling it shares your data and your position with a third party, so it is opt-in by decision, never by oversight. The installer asks explicitly on first run.

FlightAware is deliberately not offered: feeding their network requires the `piaware` client, with its own registration and feeder ID, and a plain `--net-connector` feeds nothing.

## What it handles for you

This list is the real value of the project, and every item cost actual debugging:

pacman's `--noconfirm` answers "N" to the conflict prompt, silently aborting the install. So removing a conflicting package is always an explicit, confirmed step.

`yay` resets any edit to the cached `PKGBUILD` on every run, which is correct anti-tampering behavior but incompatible with manual patches. Builds that need a patch run from a copied directory, outside `~/.cache/yay/`.

`blacklist` in modprobe.d only stops autoload at boot. On hotplug, udev asks for the module by alias and the kernel hands it over anyway. The `install <module> /bin/false` line is what actually closes that door, and it is missing from almost every tutorial.

The stock `udev` rule grants the dongle to the `plugdev` group, which is enough for an interactive user because systemd-logind adds a session ACL. The readsb service user has no session and no ACL, so it needs a rule for its own group. The problem hides precisely because it works when you test by hand.

Arch's `lighttpd.conf` is minimal and never includes `conf-enabled`, so everything the tar1090 installer writes there is dead config, with no error at all.

Arch does not load `mod_redirect`, so tar1090's `url.redirect` is ignored and the slash-less URL returns 404, which is exactly the URL the installer prints when it finishes.

The tar1090 installer only restarts lighttpd if it was already running; a freshly installed one stays dead and disabled.

`sudo` does not work without a real tty. Preflight detects it and fails early with a clear message, instead of letting you find out in the middle of a 45 minute build.

## Security

The script runs as a normal user and aborts if executed as root, because `makepkg` and `yay` refuse to run that way. Escalation is per step, via `sudo`. A single `sudo -v` up front validates the password once, and a background keepalive holds the timestamp during long builds so you are not asked again mid-compile.

The official tar1090 installer is vendored under `vendor/` rather than fetched from the network on each run, and is checksum verified before executing. Updating means downloading the new version, reviewing the diff and changing the pin in `install.conf`. Refusing to run a modified root script is deliberate.

## Scope

v1 supports Arch and derivatives. The package manager layer is isolated in `lib/pkg-arch.sh`, so supporting another distro means writing a `pkg-debian.sh` with the same `pkg::*` interface, without touching the modules. Debian and Raspberry Pi OS are not a priority because upstream's own scripts already cover them well.

It is not a daemon nor a configuration panel. It is an installer you run when you want to.

## Structure

```
easy1090/
├── easy1090                single entrypoint, dispatches the subcommands
├── install.sh              thin wrapper for "easy1090 install"
├── install.conf.example    reference configuration
├── lib/
│   ├── common.sh           logging, dry-run execution, sudo, config
│   ├── i18n.sh             language selection and lookup
│   ├── i18n/{pt,en}.sh     message catalogs
│   ├── pkg-arch.sh         package manager layer
│   ├── cmd-install.sh      install command
│   ├── cmd-uninstall.sh    uninstall command
│   ├── cmd-status.sh       status command (read-only)
│   ├── cmd-service.sh      start / stop / restart
│   ├── cmd-open.sh         open command
│   ├── 00-preflight.sh     read-only checks
│   ├── 10-driver.sh        RTL-SDR Blog fork and DVB blacklist
│   ├── 20-readsb.sh        decoder, udev, systemd
│   ├── 30-tar1090.sh       web map and the Arch lighttpd fixes
│   ├── 40-optional.sh      SDR++ and SatDump
│   └── 60-validate.sh      end to end validation
└── vendor/
    └── tar1090-install.sh  official installer, pinned by checksum
```

## Behind the decisions

The manual walkthrough, with the reasoning behind every choice, is documented in the SDR series on my blog (in Portuguese):

- [Capturing ADS-B at 1090 MHz with the RTL-SDR v4](https://esli.blog/posts/rtl-sdr-v4-adsb-1090/)
- [From terminal to map: live ADS-B on the web with tar1090](https://esli.blog/posts/rtl-sdr-v4-tar1090/)
- [Practical guide: every way to watch ADS-B in real time](https://esli.blog/posts/guia-visualizacao-adsb/)

If you want to understand before running a root script, start there.

## Status

Under development. Tested on EndeavourOS and Omarchy with an RTL-SDR Blog V4. See [KNOWN_ISSUES.md](KNOWN_ISSUES.md).

## License

MIT
