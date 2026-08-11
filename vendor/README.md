# vendor/

Third party code redistributed with easy1090, kept here instead of being downloaded at install time.

## Why vendored

`install.sh` runs this script as root. Fetching it from the network on every run would mean executing whatever upstream happens to be serving at that moment, unreviewed. Vendoring makes the exact bytes visible in the repository, reviewable in a diff, and pinned by checksum in `install.conf`.

## tar1090-install.sh

| | |
| --- | --- |
| Upstream | https://github.com/wiedehopf/tar1090 |
| Author | Matthias Wirth (wiedehopf) |
| License | **GPL v2 or later** (see `LICENSE.tar1090`) |
| Retrieved | 2026-08-11, from `master` |
| SHA-256 | `5f1cbfa1561e709f66c29bb1bf18794eb9b1fcfdcf1b00df9f3d97bc2ab9a95f` |

The file is kept **byte identical** to upstream. Nothing is patched here, because the checksum pin in `install.conf` is what proves it has not been tampered with. Arch specific fixes live in `lib/30-tar1090.sh` and are applied around this script, never inside it.

tar1090 itself is based on code from [flightaware/dump1090](https://github.com/flightaware/dump1090).

### Licensing

easy1090 is MIT. This vendored file is GPL v2 or later and stays under its own license: the two are distributed together but are separate programs, invoked as a subprocess, which is aggregation rather than a derived work. The full upstream license text is in `LICENSE.tar1090`.

### How to update

1. Download the new version over this file.
2. Read the diff. This runs as root, so review it rather than trusting it.
3. Recompute the checksum and update `TAR1090_INSTALLER_SHA256` in `install.conf`, plus the table above.

```bash
curl -sSL -o vendor/tar1090-install.sh \
  https://raw.githubusercontent.com/wiedehopf/tar1090/master/install.sh
git diff vendor/tar1090-install.sh
sha256sum vendor/tar1090-install.sh
```

If the checksum in the config does not match the file, easy1090 refuses to run it. That is deliberate.
