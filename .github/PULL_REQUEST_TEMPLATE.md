## What this changes

<!-- One or two sentences. If it fixes an issue, link it. -->

## Why

<!-- The reasoning, especially if you are working around distro behaviour.
     Comments in this project explain why, and so do pull requests. -->

## How it was tested

- [ ] `./easy1090 install --dry-run` (says what it would do, changes nothing)
- [ ] `shellcheck -S warning -e SC2155,SC2034 easy1090 install.sh lib/*.sh`
- [ ] Ran on real hardware
- [ ] Ran **twice**, to confirm idempotency

Distro and version:
Dongle:
Clean machine or existing install:

## Checklist

- [ ] User facing strings go through `t()` and exist in both catalogs
- [ ] `$` inside catalog entries is escaped as `\$`
- [ ] If `vendor/` changed, the checksum pin and `vendor/README.md` were updated
- [ ] `CHANGELOG.md` updated if the change is user visible
