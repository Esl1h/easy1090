#!/usr/bin/env bash
#===============================================================================
# easy1090 - internationalization
#
# Message catalogs live in lib/i18n/<lang>.sh as a single associative array.
# Everything user facing goes through t(), which is printf under the hood, so
# catalog entries can carry positional arguments (%s, %d).
#
# Language resolution order: --lang flag, UI_LANGUAGE in install.conf, the
# system locale, and finally an interactive prompt.
#
# Author: Esli
# License: MIT
#===============================================================================

[[ -n "${_EASY1090_I18N_LOADED:-}" ]] && return 0
readonly _EASY1090_I18N_LOADED=1

declare -A MSG
declare UI_LANGUAGE=""

readonly I18N_SUPPORTED="pt en"

# Translates a key. Unknown keys fall back to the key itself, which makes a
# missing translation obvious without crashing the run.
t() {
    local key="$1"
    shift
    local fmt="${MSG[$key]:-}"

    if [[ -z "$fmt" ]]; then
        printf '%s' "$key"
        return 0
    fi

    # The `--` matters: some messages start with "--dry-run", and without it
    # printf would try to parse the message as its own options.
    # shellcheck disable=SC2059
    printf -- "$fmt" "$@"
}

i18n::available() {
    local lang="$1"
    [[ " $I18N_SUPPORTED " == *" $lang "* ]]
}

i18n::detect() {
    case "${LC_ALL:-${LC_MESSAGES:-${LANG:-}}}" in
    pt_* | pt) printf 'pt' ;;
    *) printf 'en' ;;
    esac
}

i18n::load() {
    local lang="$1"
    local catalog="${EASY1090_ROOT}/lib/i18n/${lang}.sh"

    [[ -f "$catalog" ]] || catalog="${EASY1090_ROOT}/lib/i18n/en.sh"

    # shellcheck source=/dev/null
    source "$catalog"
}

# Asks once, in both languages, so the question is readable either way.
i18n::prompt() {
    local default_lang="$1"
    local answer

    printf '\n  1) Português\n  2) English\n\n' >&2
    read -r -p "$(printf 'Idioma / Language [%s]: ' "$default_lang")" answer

    case "${answer,,}" in
    1 | pt | pt-br | portugues | português) printf 'pt' ;;
    2 | en | english) printf 'en' ;;
    '') printf '%s' "$default_lang" ;;
    *) printf '%s' "$default_lang" ;;
    esac
}

i18n::init() {
    local config_file="$1"
    local cli_lang="$2"
    local detected

    detected="$(i18n::detect)"

    if [[ -n "$cli_lang" ]]; then
        i18n::available "$cli_lang" || {
            printf 'Idioma não suportado / unsupported language: %s (pt, en)\n' "$cli_lang" >&2
            exit 1
        }
        UI_LANGUAGE="$cli_lang"
    elif [[ -f "$config_file" ]] && grep -qE '^UI_LANGUAGE=' "$config_file"; then
        UI_LANGUAGE="$(grep -E '^UI_LANGUAGE=' "$config_file" | head -1 | cut -d'"' -f2)"
    elif [[ "$ASSUME_YES" == true || ! -t 0 ]]; then
        UI_LANGUAGE="$detected"
    else
        UI_LANGUAGE="$(i18n::prompt "$detected")"
    fi

    i18n::available "$UI_LANGUAGE" || UI_LANGUAGE="$detected"
    i18n::load "$UI_LANGUAGE"
}
