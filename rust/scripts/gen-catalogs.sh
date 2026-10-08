#!/usr/bin/env bash
# Regenerates rust/src/core/i18n/{en,pt}.rs verbatim from lib/i18n/{en,pt}.sh.
#
# The catalogs are committed, and scripts/i18n-parity.sh (run in CI) makes
# sure the committed key sets never diverge from the bash catalogs. Run this
# after touching lib/i18n/*.sh, from anywhere in the tree.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
out_dir="$root/rust/src/core/i18n"
mkdir -p "$out_dir"

# Source both catalogs. Each one defines MSG, so stash them under distinct
# names first; sourcing also applies the bash double-quote semantics, which is
# exactly the text the runtime sees.
declare -A MSG
# shellcheck source=/dev/null
source "$root/lib/i18n/en.sh"
declare -A EN
# EN is read through the nameref in generate(), which shellcheck cannot see.
# shellcheck disable=SC2034
for key in "${!MSG[@]}"; do EN[$key]=${MSG[$key]}; done
unset MSG

declare -A MSG
# shellcheck source=/dev/null
source "$root/lib/i18n/pt.sh"
declare -A PT
# shellcheck disable=SC2034
for key in "${!MSG[@]}"; do PT[$key]=${MSG[$key]}; done

# Escapes a resolved catalog value as a Rust string literal. The values hold
# printf-style sequences (like \n) as literal backslash characters, so they
# must round-trip: backslash becomes \\ and the only other escapable character
# here is the double quote.
rust_escape() {
    local value="$1"
    value="${value//\\/\\\\}"
    value="${value//\"/\\\"}"
    printf '%s' "$value"
}

# Walks en.sh as the structural template (both bash catalogs define the same
# keys in the same order, enforced by the parity gate) and fills in the values
# from the requested language.
generate() {
    local lang="$1"
    local out="$out_dir/$lang.rs"
    local -n catalog
    if [[ "$lang" == "en" ]]; then
        catalog=EN
    else
        catalog=PT
    fi

    {
        printf '//! easy1090 message catalog (%s).\n//!\n' "$lang"
        printf '//! Generated verbatim from lib/i18n/%s.sh by scripts/gen-catalogs.sh;\n' "$lang"
        printf '//! do not edit by hand. Key parity with the bash catalogs is enforced\n'
        printf '//! in CI by scripts/i18n-parity.sh.\n\n'
        printf '/// Keys appear in the bash catalog'"'"'s order; lookups are linear, which\n'
        printf '/// is fine at this size.\n'
        printf 'pub static MSG: &[(&str, &str)] = &[\n'
    } >"$out"

    local previous_was_banner=false
    local key line escaped
    while IFS= read -r line; do
        case "$line" in
        '#----'*)
            # Section banner; the section title is the next comment line.
            previous_was_banner=true
            ;;
        '# '*)
            if [[ "$previous_was_banner" == true ]]; then
                printf '    // %s\n' "${line#\# }" >>"$out"
            fi
            previous_was_banner=false
            ;;
        MSG\[*\]*)
            previous_was_banner=false
            key="${line#MSG[}"
            key="${key%%]*}"
            if [[ -z ${catalog[$key]+x} ]]; then
                echo "error: the $lang catalog is missing key '$key'" >&2
                exit 1
            fi
            # Command substitution strips trailing newlines, which would
            # silently drop the final newline of the multi-line usage
            # messages. A sentinel keeps them; it is removed right after.
            escaped=$(rust_escape "${catalog[$key]}"; printf 'X')
            escaped="${escaped%X}"
            printf '    ("%s", "%s"),\n' "$key" "$escaped" >>"$out"
            ;;
        *)
            previous_was_banner=false
            ;;
        esac
    done <"$root/lib/i18n/en.sh"

    printf '];\n' >>"$out"
}

generate en
generate pt
echo "regenerated $out_dir/en.rs and $out_dir/pt.rs"
