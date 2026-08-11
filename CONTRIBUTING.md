# Contributing

Thanks for the interest. Bug reports from real installations are the most useful contribution here, because this project exists to encode friction that only shows up on actual hardware.

## Before opening a PR

Open an issue first for anything large, anything that changes the architecture, or support for another distribution. For a bug fix, a typo or a wording change, go straight to a pull request.

## Testing

Two levels, and the cheap one catches most mistakes:

```bash
./easy1090 install --dry-run    # full preflight, prints the real commands
shellcheck -S warning -e SC2155,SC2034 easy1090 install.sh lib/*.sh
```

The rest needs an actual Arch box with an RTL-SDR attached. There is no way around that: no CI can validate a udev rule, a systemd unit or a tuner. When you test on hardware, say so in the PR, with the distro and whether it was a clean machine or a re-run.

**Re-running matters.** The installer is idempotent, so every change has to survive a second and a third run. Most bugs found so far were not in the first install, they were in the second: a config written and never applied, a file present but never loaded by the daemon.

## Rules that the CI enforces

Both translation catalogs must define exactly the same keys, and every key used in the code must exist in them. A missing translation does not crash, it just prints a raw key on someone's screen, which is why this is checked mechanically.

Catalog entries are `printf` format strings. Escape any `$` as `\$`, otherwise it is expanded when the file is sourced and the whole run dies under `set -u`.

`vendor/tar1090-install.sh` must match the checksum pinned in `install.conf.example`. If you update the vendored file, update the pin and `vendor/README.md` in the same commit.

## Style

The code follows the conventions already in `lib/`: `namespace::function` naming, `set -euo pipefail`, four space indent, and colors guarded by a TTY check.

Two habits worth keeping:

Comments explain **why**, not what. `# blacklist alone only stops autoload at boot` earns its place; `# remove the file` does not.

Anything user facing goes through `t()`. No literal strings in `log::` calls.

## Adding another distribution

Every call to a package manager lives in `lib/pkg-arch.sh` behind the `pkg::*` interface. Supporting Debian means writing `lib/pkg-debian.sh` with the same functions, without touching the modules. The distro check in `lib/00-preflight.sh` and the loader in `easy1090` are the other two places that need to know.

## Commits

One logical change per commit, present tense, no trailers.
