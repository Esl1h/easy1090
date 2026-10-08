//! Message catalogs and the `t()` equivalent, mirroring lib/i18n.sh.
//!
//! The catalogs live in `en.rs` and `pt.rs`, generated verbatim from
//! lib/i18n/{en,pt}.sh by scripts/gen-catalogs.sh; scripts/i18n-parity.sh
//! (run in CI) makes sure the key sets never diverge from the bash tree.
//!
//! Language resolution order, exactly like i18n::init: the --lang flag,
//! UI_LANGUAGE in install.conf, the system locale, and finally an
//! interactive prompt.

pub mod en;
pub mod pt;

use std::env;
use std::fmt::Display;
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use crate::core::run;

/// Supported languages, in the I18N_SUPPORTED="pt en" order; the first entry
/// is the default of the interactive prompt.
pub const SUPPORTED: [&str; 2] = ["pt", "en"];

static LANGUAGE: OnceLock<&'static str> = OnceLock::new();
static LANGUAGE_FROM_CONFIG: AtomicBool = AtomicBool::new(false);

pub fn available(lang: &str) -> bool {
    SUPPORTED.contains(&lang)
}

/// The language resolved by i18n::init; "en" until then.
pub fn language() -> &'static str {
    LANGUAGE.get().copied().unwrap_or("en")
}

/// True when the resolved language came from UI_LANGUAGE in install.conf,
/// the UI_LANGUAGE_FROM_CONFIG marker of the bash.
pub fn language_from_config() -> bool {
    LANGUAGE_FROM_CONFIG.load(Ordering::Relaxed)
}

/// Resolves the language, mirroring i18n::init. An unsupported --lang value
/// aborts with the same bilingual message and exit code as the bash.
pub fn init(config_file: Option<&Path>, cli_lang: Option<&str>) {
    let detected = detect();
    let language: &'static str;
    let mut came_from_config = false;

    if let Some(cli) = cli_lang.filter(|lang| !lang.is_empty()) {
        if !available(cli) {
            eprintln!("Idioma não suportado / unsupported language: {cli} (pt, en)");
            std::process::exit(1);
        }
        language = if cli == "pt" { "pt" } else { "en" };
    } else if let Some(path) = config_file {
        let configured = from_config(path);
        if !configured.is_empty() {
            came_from_config = true;
            // An invalid value in the config falls back to the detected
            // language, like the final i18n::available check in the bash.
            language = match configured.as_str() {
                "pt" => "pt",
                "en" => "en",
                _ => detected,
            };
        } else if run::assume_yes() || !io::stdin().is_terminal() {
            language = detected;
        } else {
            language = prompt(detected);
        }
    } else if run::assume_yes() || !io::stdin().is_terminal() {
        language = detected;
    } else {
        language = prompt(detected);
    }

    LANGUAGE_FROM_CONFIG.store(came_from_config, Ordering::Relaxed);
    let _ = LANGUAGE.set(language);
}

/// `${LC_ALL:-${LC_MESSAGES:-${LANG:-}}}`: the first set, non-empty variable
/// wins; `pt` and `pt_*` select Portuguese, everything else English. The
/// match is case-sensitive, like the bash case pattern.
fn detect() -> &'static str {
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = env::var(var) {
            if value.is_empty() {
                continue;
            }
            return if value.starts_with("pt_") || value == "pt" {
                "pt"
            } else {
                "en"
            };
        }
    }
    "en"
}

/// Reads UI_LANGUAGE from the config without a full parse: the first line
/// starting with UI_LANGUAGE= wins, the quotes around the value are optional.
fn from_config(config_file: &Path) -> String {
    let Ok(content) = std::fs::read_to_string(config_file) else {
        return String::new();
    };
    parse_ui_language(&content)
}

fn parse_ui_language(content: &str) -> String {
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("UI_LANGUAGE=") {
            // s/^UI_LANGUAGE="?([^"]*)"?[^"]*$/\1/: an optional opening
            // quote, the value up to the closing quote, the rest discarded.
            return if let Some(unquoted) = rest.strip_prefix('"') {
                unquoted.split('"').next().unwrap_or("").to_string()
            } else {
                rest.to_string()
            };
        }
    }
    String::new()
}

/// The interactive prompt, asked once in both languages so the question is
/// readable either way. The read prompt goes to stderr, like `read -p`.
fn prompt(default_lang: &'static str) -> &'static str {
    eprint!("\n  1) Português\n  2) English\n\n");
    eprint!("Idioma / Language [{default_lang}]: ");
    let _ = io::stderr().flush();

    let mut answer = String::new();
    match io::stdin().read_line(&mut answer) {
        Ok(0) | Err(_) => default_lang,
        Ok(_) => match answer.trim().to_lowercase().as_str() {
            "1" | "pt" | "pt-br" | "portugues" | "português" => "pt",
            "2" | "en" | "english" => "en",
            _ => default_lang,
        },
    }
}

/// Translates `key` in the resolved language. Unknown keys fall back to the
/// key itself, which makes a missing translation obvious without crashing
/// the run, exactly like the bash t().
pub fn t(key: &str, args: &[&dyn Display]) -> String {
    let catalog: &[(&str, &str)] = match language() {
        "pt" => pt::MSG,
        _ => en::MSG,
    };
    lookup(catalog, key, args)
}

fn lookup(catalog: &[(&str, &str)], key: &str, args: &[&dyn Display]) -> String {
    match catalog.iter().find(|(k, _)| *k == key) {
        Some((_, fmt)) => format_printf(fmt, args),
        None => key.to_string(),
    }
}

/// printf under the hood, restricted to what the catalogs use: backslash
/// escapes, %% and the %s/%d conversions. Missing arguments fill empty/zero,
/// extra arguments reuse the format, and a format without conversions is
/// printed once, all like bash printf.
fn format_printf(fmt: &str, args: &[&dyn Display]) -> String {
    let mut out = String::with_capacity(fmt.len());
    let mut consumed = 0usize;
    let mut chars = fmt.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('a') => out.push('\x07'),
                Some('b') => out.push('\x08'),
                Some('e') => out.push('\x1b'),
                Some('f') => out.push('\x0c'),
                Some('v') => out.push('\x0b'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            },
            '%' => match chars.peek() {
                Some('%') => {
                    chars.next();
                    out.push('%');
                }
                Some('s') => {
                    chars.next();
                    out.push_str(
                        &args
                            .get(consumed)
                            .map(|arg| arg.to_string())
                            .unwrap_or_default(),
                    );
                    consumed += 1;
                }
                Some('d') => {
                    chars.next();
                    let value = args
                        .get(consumed)
                        .map(|arg| arg.to_string())
                        .unwrap_or_else(|| "0".to_string());
                    out.push_str(&value);
                    consumed += 1;
                }
                _ => out.push('%'),
            },
            other => out.push(other),
        }
    }

    if consumed > 0 && consumed < args.len() {
        out.push_str(&format_printf(fmt, &args[consumed..]));
    }
    out
}

/// Shorthand for the call sites: t!("key") or t!("key", arg1, arg2).
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::core::i18n::t($key, &[])
    };
    ($key:expr, $($arg:expr),+ $(,)?) => {
        $crate::core::i18n::t($key, &[$(&$arg as &dyn std::fmt::Display),+])
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn en_t(key: &str, args: &[&dyn Display]) -> String {
        lookup(en::MSG, key, args)
    }

    #[test]
    fn init_sets_the_language_and_the_catalog() {
        init(None, Some("pt"));
        assert_eq!(language(), "pt");
        assert_eq!(
            t("cmd_unknown", &[&"frobnicate"]),
            "Comando desconhecido: frobnicate"
        );
    }

    #[test]
    fn missing_keys_fall_back_to_the_key_itself() {
        assert_eq!(en_t("no_such_key", &[]), "no_such_key");
        assert_eq!(en_t("no_such_key", &[&"ignored"]), "no_such_key");
        assert_eq!(lookup(pt::MSG, "no_such_key", &[]), "no_such_key");
    }

    #[test]
    fn formats_s_percent_with_arguments() {
        assert_eq!(
            en_t("cfg_created", &[&"/x/y"]),
            "Config created at /x/y (from the example)."
        );
        assert_eq!(
            en_t("pos_invalid", &[&"1", &"2"]),
            "Invalid coordinates: 1, 2. Use decimal degrees with a dot, latitude between -90 and 90, longitude between -180 and 180."
        );
    }

    #[test]
    fn extra_arguments_reuse_the_format_like_printf() {
        assert_eq!(
            en_t("cfg_created", &[&"a", &"b"]),
            "Config created at a (from the example).Config created at b (from the example)."
        );
    }

    #[test]
    fn missing_arguments_default_like_printf() {
        assert_eq!(format_printf("x%sx", &[]), "xx");
        assert_eq!(format_printf("x%dx", &[]), "x0x");
        assert_eq!(format_printf("[%s|%d]", &[&"a"]), "[a|0]");
        // A format without conversions consumes nothing and prints once.
        assert_eq!(format_printf("x", &[&"a", &"b"]), "x");
    }

    #[test]
    fn percent_and_backslash_escapes() {
        assert_eq!(format_printf("100%% %s", &[&"a"]), "100% a");
        assert_eq!(format_printf("a\\nb", &[]), "a\nb");
        assert_eq!(format_printf("a\\qb", &[]), "a\\qb");
        assert_eq!(format_printf("lone %", &[]), "lone %");
    }

    #[test]
    fn catalog_newlines_expand_on_translation() {
        // main_usage stores \n as two characters; t() turns them into
        // newlines, like printf does in the bash.
        let usage = en_t("main_usage", &[&"0.0.0"]);
        assert!(usage.contains("\nUSAGE\n"));
        assert!(usage.ends_with("status and open do not need sudo.\n"));
    }

    #[test]
    fn ui_language_is_parsed_like_the_bash_sed() {
        assert_eq!(parse_ui_language("UI_LANGUAGE=\"pt\"\n"), "pt");
        assert_eq!(parse_ui_language("X=1\nUI_LANGUAGE=pt\n"), "pt");
        assert_eq!(parse_ui_language("UI_LANGUAGE=\"pt\" # hand note\n"), "pt");
        assert_eq!(parse_ui_language("UI_LANGUAGE=\"\"\n"), "");
        assert_eq!(parse_ui_language("  UI_LANGUAGE=pt\n"), "");
        assert_eq!(parse_ui_language("no such line\n"), "");
    }
}
