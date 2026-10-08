//! Command execution with a dry-run preview, mirroring run::cmd and run::sudo
//! in lib/common.sh. The dry-run mode prints the exact command that would run
//! (rendered like bash `printf '%q'`), never a description of it.

use std::ffi::OsString;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::core::log;

static DRY_RUN: AtomicBool = AtomicBool::new(false);
static ASSUME_YES: AtomicBool = AtomicBool::new(false);

pub fn set_dry_run(value: bool) {
    DRY_RUN.store(value, Ordering::Relaxed);
}

pub fn dry_run() -> bool {
    DRY_RUN.load(Ordering::Relaxed)
}

pub fn set_assume_yes(value: bool) {
    ASSUME_YES.store(value, Ordering::Relaxed);
}

pub fn assume_yes() -> bool {
    ASSUME_YES.load(Ordering::Relaxed)
}

/// Renders argv as a copy-pasteable command line. This is what `--dry-run`
/// prints, so it has to be the real command: safe characters verbatim, the
/// rest backslash-escaped, and strings with control characters in the `$'...'`
/// form, like bash `printf '%q'`.
pub fn render<I, S>(args: I) -> String
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut out = String::new();
    for arg in args {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(&quote(&arg.into().to_string_lossy()));
    }
    out
}

/// One argument through the `%q` rules: alphanumerics and `_./:=@%+-` stay
/// bare, `#` and `~` are escaped only at word start (that is where the shell
/// treats them specially), and every other character gets a backslash.
fn quote(arg: &str) -> String {
    if arg.is_empty() {
        return "''".to_string();
    }
    if arg.chars().any(char::is_control) {
        return ansi_c_quote(arg);
    }

    let mut out = String::with_capacity(arg.len());
    for (index, c) in arg.chars().enumerate() {
        let safe = c.is_ascii_alphanumeric()
            || matches!(c, '_' | '.' | '/' | ':' | '=' | '@' | '%' | '+' | '-');
        let word_start = index == 0 && matches!(c, '#' | '~');
        let in_word_hash_or_tilde = !word_start && matches!(c, '#' | '~');
        if safe || !c.is_ascii() || in_word_hash_or_tilde {
            out.push(c);
        } else {
            out.push('\\');
            out.push(c);
        }
    }
    out
}

/// The ANSI-C quoting `%q` switches to when the argument contains control
/// characters. Non-ASCII bytes stay literal, matching a UTF-8 locale.
fn ansi_c_quote(arg: &str) -> String {
    let mut out = String::from("$'");
    for c in arg.chars() {
        match c {
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\x07' => out.push_str("\\a"),
            '\x08' => out.push_str("\\b"),
            '\x1b' => out.push_str("\\E"),
            '\x0c' => out.push_str("\\f"),
            '\x0b' => out.push_str("\\v"),
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}

/// Runs a command with inherited stdio (the bash runs `"$@"` as-is). Under
/// `--dry-run` it previews the command instead and reports success, so the
/// whole flow can be walked without executing anything.
pub fn cmd<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let argv: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let rendered = render(argv.iter().cloned());

    if dry_run() {
        log::dry_run(&rendered);
        return true;
    }

    log::debug(&format!("exec: {rendered}"));
    let Some((program, rest)) = argv.split_first() else {
        return true;
    };
    Command::new(program)
        .args(rest)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// `run::sudo`: the same execution path with `sudo` in front.
pub fn sudo<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut argv: Vec<OsString> = vec![OsString::from("sudo")];
    argv.extend(args.into_iter().map(Into::into));
    cmd(argv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_safe_arguments_verbatim() {
        assert_eq!(
            render(["sudo", "systemctl", "enable", "readsb"]),
            "sudo systemctl enable readsb"
        );
    }

    #[test]
    fn renders_arguments_the_way_bash_percent_q_does() {
        assert_eq!(render(["cp", "a b.txt", "x"]), "cp a\\ b.txt x");
        assert_eq!(render(["echo", ""]), "echo ''");
        assert_eq!(render(["x", "30004,30104"]), "x 30004\\,30104");
        assert_eq!(render(["it's"]), "it\\'s");
        assert_eq!(render(["#tag"]), "\\#tag");
        assert_eq!(render(["a#b"]), "a#b");
        assert_eq!(render(["~/.x"]), "\\~/.x");
        assert_eq!(render(["a~b"]), "a~b");
        assert_eq!(render(["a$b"]), "a\\$b");
        assert_eq!(render(["a|b"]), "a\\|b");
        assert_eq!(render(["café"]), "café");
        assert_eq!(render(["a\tb"]), "$'a\\tb'");
    }
}
