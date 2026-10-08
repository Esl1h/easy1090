# Rust port. All targets run from this directory (never the repo root).

CARGO ?= cargo
# Stripping happens in-profile ([profile.release] strip = true in
# Cargo.toml), which is cross-arch safe: the linker strips, so no external
# strip tool is needed for the aarch64 binary on an x86_64 host. For the
# aarch64 link itself, rust-lld (shipped by rustup) is enough, since the
# crate has no C dependencies:
#   CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=rust-lld make release

# Release targets; the artifact names below are fixed by the delegation spec.
# CI only calls `make release` and never names a file itself.
TARGET_AMD64 ?= x86_64-unknown-linux-musl
TARGET_ARM64 ?= aarch64-unknown-linux-musl
DIST ?= dist

# The version comes from Cargo.toml, never written by hand a second time.
VERSION := $(shell sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' Cargo.toml | head -n 1)
ifeq ($(VERSION),)
$(error could not read the version from Cargo.toml)
endif

BIN_AMD64 := easy1090-$(VERSION)-linux-amd64.bin
BIN_ARM64 := easy1090-$(VERSION)-linux-arm64.bin
TGZ_AMD64 := easy1090-$(VERSION)-linux-amd64.tar.gz
TGZ_SHELL := easy1090-$(VERSION)-shell.tar.gz

.PHONY: build dev test fmt-check clippy check release clean \
	release-amd64 release-arm64 release-tarball release-shell release-checksums

build:
	$(CARGO) build

dev:
	$(CARGO) run

test:
	$(CARGO) test

fmt-check:
	$(CARGO) fmt -- --check

clippy:
	$(CARGO) clippy --all-targets -- -D warnings

check: fmt-check clippy test

# Both static musl binaries, the two tarballs and SHA256SUMS, under $(DIST)/.
release: release-checksums

release-amd64:
	$(CARGO) build --release --target $(TARGET_AMD64)
	install -Dm755 target/$(TARGET_AMD64)/release/easy1090 $(DIST)/$(BIN_AMD64)

release-arm64:
	$(CARGO) build --release --target $(TARGET_ARM64)
	install -Dm755 target/$(TARGET_ARM64)/release/easy1090 $(DIST)/$(BIN_ARM64)

# Binary + README + LICENSE, flat at the archive root so extracting gives a
# runnable ./easy1090. The repo ships no shell completions to add here.
release-tarball: release-amd64
	rm -rf $(DIST)/stage
	mkdir -p $(DIST)/stage
	cp ../README.md ../LICENSE $(DIST)/stage/
	cp $(DIST)/$(BIN_AMD64) $(DIST)/stage/easy1090
	tar -C $(DIST)/stage -czf $(DIST)/$(TGZ_AMD64) easy1090 README.md LICENSE
	rm -rf $(DIST)/stage

# The whole bash tree at HEAD, the Rust port excluded. `git archive` takes
# exclude pathspecs itself, so no temp checkout is needed; .git is never part
# of an archive.
release-shell:
	git -C .. archive --format=tar.gz --output="$(CURDIR)/$(DIST)/$(TGZ_SHELL)" \
		HEAD -- ':(exclude)rust'

release-checksums: release-amd64 release-arm64 release-tarball release-shell
	cd $(DIST) && sha256sum $(BIN_AMD64) $(BIN_ARM64) $(TGZ_AMD64) $(TGZ_SHELL) > SHA256SUMS

clean:
	rm -rf $(DIST)
