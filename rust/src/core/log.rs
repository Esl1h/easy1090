//! Leveled, colored logging, mirroring the `log::*` family in lib/common.sh.
//!
//! Every line goes to stderr with the exact bash shape:
//! `${color}[%-5s]${RESET} %s\n`. Colors are defined only when stdout is a
//! tty (the bash checks `[[ -t 1 ]]` once at load time), so piped output
//! stays plain.

use std::io::IsTerminal;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::OnceLock;

const RED: &str = "\x1b[0;31m";
const GREEN: &str = "\x1b[0;32m";
const YELLOW: &str = "\x1b[0;33m";
const BLUE: &str = "\x1b[0;34m";
const MAGENTA: &str = "\x1b[0;35m";
const CYAN: &str = "\x1b[0;36m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// 0=error, 1=warn, 2=info, 3=debug. Defaults to 2, like `declare LOG_LEVEL=2`.
static LOG_LEVEL: AtomicU8 = AtomicU8::new(2);

/// Sets the log level (`--verbose` maps to 3, like the bash).
pub fn set_level(level: u8) {
    LOG_LEVEL.store(level, Ordering::Relaxed);
}

fn level() -> u8 {
    LOG_LEVEL.load(Ordering::Relaxed)
}

/// Decided once, like the bash `[[ -t 1 ]]` at load time.
fn colors_on() -> bool {
    static COLORS: OnceLock<bool> = OnceLock::new();
    *COLORS.get_or_init(|| std::io::stdout().is_terminal())
}

fn color(code: &str) -> &str {
    if colors_on() {
        code
    } else {
        ""
    }
}

/// The bold sequence for confirm prompts; empty when not a tty.
pub fn bold() -> &'static str {
    if colors_on() {
        BOLD
    } else {
        ""
    }
}

/// The reset sequence after a colored or bold span; empty when not a tty.
pub fn reset() -> &'static str {
    if colors_on() {
        RESET
    } else {
        ""
    }
}

/// The GREEN sequence for `status` rows; empty when not a tty.
pub fn green() -> &'static str {
    color(GREEN)
}

/// The RED sequence for `status` rows; empty when not a tty.
pub fn red() -> &'static str {
    color(RED)
}

/// The YELLOW sequence for `status` rows with an unknown state; empty when
/// not a tty.
pub fn yellow() -> &'static str {
    color(YELLOW)
}

/// `${color}[%-5s]${RESET} %s` with the level padded to five columns, as
/// log::_output renders it.
fn line(level: &str, code: &str, message: &str) -> String {
    format!("{}[{:<5}]{} {}", color(code), level, reset(), message)
}

/// `log::_output`: writes the rendered line to stderr.
fn output(level: &str, code: &str, message: &str) {
    eprintln!("{}", line(level, code, message));
}

pub fn debug(message: &str) {
    if level() >= 3 {
        output("DEBUG", CYAN, message);
    }
}

pub fn info(message: &str) {
    if level() >= 2 {
        output("INFO", BLUE, message);
    }
}

pub fn warn(message: &str) {
    if level() >= 1 {
        output("WARN", YELLOW, message);
    }
}

pub fn error(message: &str) {
    output("ERROR", RED, message);
}

pub fn success(message: &str) {
    if level() >= 2 {
        output("OK", GREEN, message);
    }
}

pub fn skip(message: &str) {
    if level() >= 2 {
        output("SKIP", CYAN, message);
    }
}

pub fn dry_run(message: &str) {
    output("DRY", MAGENTA, message);
}

/// `log::step`: a blank line, a bold `==> ` title and another blank line, all
/// on stderr, regardless of the log level.
pub fn step(message: &str) {
    eprintln!("\n{}==> {}{}", color(BOLD), message, reset());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_labels_are_left_aligned_in_five_columns() {
        // stdout is piped under cargo test, so colors are off and the labels
        // are the whole story.
        assert_eq!(line("DEBUG", CYAN, "x"), "[DEBUG] x");
        assert_eq!(line("INFO", BLUE, "x"), "[INFO ] x");
        assert_eq!(line("WARN", YELLOW, "x"), "[WARN ] x");
        assert_eq!(line("ERROR", RED, "x"), "[ERROR] x");
        assert_eq!(line("OK", GREEN, "x"), "[OK   ] x");
        assert_eq!(line("SKIP", CYAN, "x"), "[SKIP ] x");
        assert_eq!(line("DRY", MAGENTA, "x"), "[DRY  ] x");
    }

    #[test]
    fn bold_and_reset_are_empty_without_a_tty() {
        assert_eq!(bold(), "");
        assert_eq!(reset(), "");
    }
}
