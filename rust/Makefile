# Rust port. All targets run from this directory (never the repo root).

CARGO ?= cargo

.PHONY: build dev test fmt-check clippy check release

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

release:
	$(CARGO) build --release
