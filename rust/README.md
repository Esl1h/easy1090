# easy1090, the Rust port

A second implementation of the easy1090 installer, living next to the shell
version in the repository root. One command set, two implementations: the
shell tree is the reference; this crate is the same behavior in typed,
testable code that ships as a single static binary.

What the port is not: a rewrite with opinions of its own. The shell version
under `lib/` defines the behavior, message by message, exit code by exit
code, and CI enforces that the two do not drift apart. When the two disagree,
the shell version wins and the port is fixed.

## Status

Release candidate. Every command is ported and output parity is verified
(the full `--dry-run` matrix of every subcommand, in both interface
languages, is compared line by line against the shell version in CI), but
the port has not yet been exercised end to end on real hardware the way the
shell release was. Until it is, the shell version remains the recommended
default and the binaries publish as pre-releases.

## Layout

```
rust/
├── Cargo.toml            crate manifest; the version names the artifacts
├── build.rs             stamps version and build commit into the binary
├── Makefile             build, check, release; single source of artifact names
├── src/
│   ├── main.rs           argument parsing and command dispatch
│   ├── cmd/              one module per subcommand
│   ├── steps/            driver, readsb, tar1090, optional, validate
│   ├── preflight/        read-only checks before anything mutates
│   ├── core/             log, run (dry-run), sudo, confirm, cfg, i18n
│   ├── core/i18n/        message catalogs, generated from lib/i18n/*.sh
│   └── pkg/              package manager trait + the Arch backend
├── scripts/
│   ├── gen-catalogs.sh   regenerates the catalogs from lib/i18n/*.sh
│   └── i18n-parity.sh    CI gate: the key sets must match, all four
└── tests/               integration tests, including the parity harness
```

Zero external dependencies. The crate is std-only, which is also why the
aarch64 musl build needs nothing beyond `rust-lld`: there is no C code to
cross-compile.

## Building and developing

Everything runs from this directory, never the repository root:

```bash
make build        # cargo build
make check        # fmt, clippy -D warnings, tests
make release      # both musl binaries, both tarballs, SHA256SUMS
```

The i18n catalogs are generated files: after editing `lib/i18n/*.sh`, run
`scripts/gen-catalogs.sh` and commit the result, otherwise the parity gate
in CI fails. Never edit `src/core/i18n/*.rs` by hand.

## Parity

Two gates keep the port honest:

- `scripts/i18n-parity.sh` compares the message key sets of the shell
  catalogs against the Rust ones. Four comparisons, zero tolerance.
- The `--dry-run` output of every subcommand is compared against the shell
  version line by line. What you see in dry-run is exactly what would run,
  in both implementations, byte for byte.

## Release artifacts

Built by `make release`, named by the version in `Cargo.toml`, published by
the tag workflow (`.github/workflows/release.yml`):

```
easy1090-<ver>-linux-amd64.bin     static musl binary, x86_64
easy1090-<ver>-linux-arm64.bin     static musl binary, aarch64
easy1090-<ver>-linux-amd64.tar.gz binary + README + LICENSE
easy1090-<ver>-shell.tar.gz        the shell tree, without this port
SHA256SUMS
```

## Scope

Same as the shell version: Arch and derivatives. The package layer is the
`pkg::Backend` trait with an Arch implementation; a Debian backend means one
new file implementing the trait, without touching the rest, mirroring
`lib/pkg-arch.sh` in the shell tree.

The vendored tar1090 installer is not ported: it is executed as the shell
script it is, checksum-verified against the pin in `install.conf.example`,
in both implementations.

## License

MIT, same as the repository root.
