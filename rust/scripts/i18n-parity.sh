#!/bin/sh
# i18n parity gate: the Rust catalogs must define exactly the same MSG keys as
# the bash catalogs, and the two languages must agree with each other.
#
# Extracts the keys with grep/sed from the bash associative-array assignments
# (MSG[key]=...) and from the ("key", ...) tuples in the Rust catalogs. Fails
# with a diff on any divergence, which .github/workflows/rust.yml turns into a
# red CI job. Runs from the rust/ directory in CI, but works from anywhere.

set -eu

# Empty CDPATH so `cd` never prints the target directory to stdout.
# shellcheck disable=SC1007
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)

bash_keys() {
    grep -o 'MSG\[[a-z0-9_]*\]' "$1" | sed 's/MSG\[\(.*\)\]/\1/' | sort -u
}

rust_keys() {
    sed -n 's/^    ("\([a-z0-9_]*\)",.*/\1/p' "$1" | sort -u
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

bash_keys "$root/lib/i18n/en.sh" >"$tmp/bash-en"
bash_keys "$root/lib/i18n/pt.sh" >"$tmp/bash-pt"
rust_keys "$root/rust/src/core/i18n/en.rs" >"$tmp/rust-en"
rust_keys "$root/rust/src/core/i18n/pt.rs" >"$tmp/rust-pt"

status=0

compare() {
    label=$1
    left=$2
    right=$3
    if diff -u "$left" "$right" >&2; then
        echo "i18n parity OK:   $label"
    else
        echo "i18n parity FAIL: $label" >&2
        status=1
    fi
}

compare "bash en vs rust en" "$tmp/bash-en" "$tmp/rust-en"
compare "bash pt vs rust pt" "$tmp/bash-pt" "$tmp/rust-pt"
compare "bash en vs bash pt" "$tmp/bash-en" "$tmp/bash-pt"
compare "rust en vs rust pt" "$tmp/rust-en" "$tmp/rust-pt"

# An extraction bug would empty every set and turn the diffs above green.
for set in "$tmp/bash-en" "$tmp/rust-en" "$tmp/rust-pt"; do
    if [ ! -s "$set" ]; then
        echo "i18n parity FAIL: nothing extracted from $set" >&2
        status=1
    fi
done

if [ "$status" -ne 0 ]; then
    echo "i18n parity FAIL (see diffs above)" >&2
    exit 1
fi

printf 'i18n parity OK: %s keys in each catalog\n' "$(wc -l <"$tmp/bash-en")"
