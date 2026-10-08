//! Command execution with a dry-run preview, mirroring run::cmd and run::sudo
//! in lib/common.sh. The dry-run mode prints the exact command that would run
//! (rendered like bash `printf '%q'`), never a description of it.

use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, Output, Stdio};
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

/// Runs a command with inherited stdio (the bash runs `"$@"` as-is). The
/// inherit is explicit on all three streams: `sudo` gets the caller's real
/// tty for the password prompt, and stdout/stderr stream live instead of
/// being buffered. Under `--dry-run` it previews the command instead and
/// reports success, so the whole flow can be walked without executing
/// anything.
///
/// A nonzero exit aborts the run, like a bare command under `set -e` in
/// the bash: nothing extra is printed (the failed command's own stderr has
/// already streamed), the exit code is preserved, and the sudo keepalive
/// is cleaned up on the way out, like the bash EXIT trap. Call sites the
/// bash guards with `||` use cmd_ok and sudo_ok instead, where a failure
/// is just a false return.
pub fn cmd<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    abort_on_failure(cmd_in_impl(None, args))
}

/// `run::cmd` from inside a directory, the `( cd dir && cmd )` subshell of
/// the bash. The preview shows the command alone, exactly like the bash
/// renders `run::cmd` inside the subshell. Same abort-on-failure contract
/// as cmd: the subshell under `set -e` takes the parent with it.
pub fn cmd_in<I, S>(dir: impl AsRef<OsStr>, args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    abort_on_failure(cmd_in_impl(Some(dir.as_ref()), args))
}

/// `run::cmd` for the call sites the bash guards with `||`: a failure is
/// reported as false, the run continues.
pub fn cmd_ok<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    cmd_in_impl(None, args) == 0
}

/// `run::sudo` for the `||`-guarded call sites, same non-fatal contract.
pub fn sudo_ok<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut argv: Vec<OsString> = vec![OsString::from("sudo")];
    argv.extend(args.into_iter().map(Into::into));
    cmd_ok(argv)
}

/// The set -e half of the contract: exit with the failed command's own
/// status code, after the keepalive cleanup the bash EXIT trap does.
fn abort_on_failure(code: i32) -> bool {
    if code != 0 {
        crate::core::sudo::cleanup();
        std::process::exit(code);
    }
    true
}

fn cmd_in_impl<I, S>(dir: Option<&OsStr>, args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let argv: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let rendered = render(argv.iter().cloned());

    if dry_run() {
        log::dry_run(&rendered);
        return 0;
    }

    log::debug(&format!("exec: {rendered}"));
    let Some((program, rest)) = argv.split_first() else {
        return 0;
    };
    let mut command = Command::new(program);
    command.args(rest);
    if let Some(dir) = dir {
        command.current_dir(dir);
    }
    match command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        Ok(status) => status.code().unwrap_or(1),
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

/// `run::sudo`: the same execution path with `sudo` in front. Same
/// abort-on-failure contract as cmd.
pub fn sudo<I, S>(args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut argv: Vec<OsString> = vec![OsString::from("sudo")];
    argv.extend(args.into_iter().map(Into::into));
    cmd(argv)
}

/// `run::sudo` from inside a directory, the `( cd dir && sudo cmd )` shape
/// feed::stats_run uses for the third party installer. Non-fatal by
/// contract: the bash guards that call with `|| { ...; return 1; }`.
pub fn sudo_in<I, S>(dir: impl AsRef<OsStr>, args: I) -> bool
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut argv: Vec<OsString> = vec![OsString::from("sudo")];
    argv.extend(args.into_iter().map(Into::into));
    cmd_in_impl(Some(dir.as_ref()), argv) == 0
}

/// `run::sudo` with stderr silenced and the failure ignored, the
/// `run::sudo ... 2>/dev/null || true` call sites. The preview stays the
/// same: redirections are not part of argv, so the bash renders the command
/// without them too.
pub fn sudo_quiet<I, S>(args: I) -> bool
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
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Writes a file as root, the `run::sudo_write` of the bash. Kept separate
/// from run::sudo because the content has to land on the privileged side of
/// the pipe, not in our process.
///
/// Returns whether the file changed: callers decide whether a service needs
/// restarting, without which `systemctl enable --now` is a no-op on an
/// already running unit. The comparison runs in dry-run too, so the preview
/// claims a change (and a restart) exactly when a real run would.
pub fn sudo_write(path: &str, content: &str) -> bool {
    // `$(cat "$path" 2>/dev/null)` strips trailing newlines, which is what
    // makes the round trip stable: tee writes content plus one newline.
    let unchanged = std::fs::read_to_string(path)
        .map(|existing| existing.trim_end_matches('\n') == content)
        .unwrap_or(false);
    if unchanged {
        log::debug(&format!("unchanged: {path}"));
        return false;
    }

    if dry_run() {
        log::dry_run(&format!("sudo tee {path} <<'EOF'"));
        // `printf '%s\n' "$content" | sed 's/^/        /'`: every line of the
        // content, eight spaces in front, blank lines included.
        for line in content.split('\n') {
            eprintln!("        {line}");
        }
        log::dry_run("EOF");
        return true;
    }

    log::debug(&format!("writing {path}"));
    let mut child = match Command::new("sudo")
        .arg("tee")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let _ = child.stdin.take().map(|mut stdin| {
        stdin
            .write_all(content.as_bytes())
            .and_then(|_| stdin.write_all(b"\n"))
            .and_then(|_| stdin.flush())
    });
    // `printf ... | sudo tee` under set -e and pipefail: a failed tee
    // aborts the run, silently, preserving the exit code.
    match child.wait() {
        Ok(status) if status.success() => {}
        Ok(status) => {
            abort_on_failure(status.code().unwrap_or(1));
        }
        Err(_) => {
            abort_on_failure(1);
        }
    }
    true
}

/// Appends one line to a file as root, the `printf ... | sudo tee -a` shape
/// tar1090::fix_lighttpd_include uses. No preview of its own: the bash
/// composes the dry-run line by hand at the call site. The bash call site
/// is a bare statement, so a failed tee aborts the run, like sudo_write.
pub fn sudo_write_append(path: &str, line: &str) -> bool {
    let mut child = match Command::new("sudo")
        .arg("tee")
        .arg("-a")
        .arg(path)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            eprintln!("{error}");
            return abort_on_failure(1);
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        if stdin
            .write_all(line.as_bytes())
            .and_then(|_| stdin.flush())
            .is_err()
        {
            return abort_on_failure(1);
        }
    }
    match child.wait() {
        Ok(status) if status.success() => {}
        Ok(status) => {
            abort_on_failure(status.code().unwrap_or(1));
        }
        Err(_) => {
            abort_on_failure(1);
        }
    }
    true
}

/// Runs a command capturing stdout, the shape of the bash `$(cmd ...)`.
/// `stderr` carries the redirection of each call site: `Stdio::null()` where
/// the bash writes `2>/dev/null`, `Stdio::inherit()` where it leaves stderr
/// alone. `None` when the program cannot be spawned, the bash 127 case.
pub fn capture<I, S>(args: I, stderr: Stdio) -> Option<Output>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let argv: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let (program, rest) = argv.split_first()?;
    Command::new(program)
        .args(rest)
        .stderr(stderr)
        .output()
        .ok()
}

/// Runs a command with inherited stdio and returns its exit status, for the
/// call sites where the bash propagates the status (open::map's xdg-open).
/// Under `--dry-run` it previews the command and reports success, like
/// `run::cmd`.
pub fn exit_code<I, S>(args: I) -> i32
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let argv: Vec<OsString> = args.into_iter().map(Into::into).collect();
    let rendered = render(argv.iter().cloned());

    if dry_run() {
        log::dry_run(&rendered);
        return 0;
    }

    log::debug(&format!("exec: {rendered}"));
    let Some((program, rest)) = argv.split_first() else {
        return 0;
    };
    match Command::new(program)
        .args(rest)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        // A signal-terminated child reports 128 + signal, like the shell.
        Ok(status) => status
            .code()
            .unwrap_or_else(|| 128 + status.signal().unwrap_or(0)),
        // Spawn failure: the shell reports 127 for a command not found.
        Err(_) => 127,
    }
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

    /// Manual proof of the sudo TTY story: run in a real terminal with
    /// `cargo test sudo_inherited_stdio -- --ignored`. With cached sudo
    /// credentials `sudo true` succeeds with no prompt, through the exact
    /// inherited-stdio path run::sudo uses; without them the password
    /// prompt appears on the terminal, which is the behavior being proven.
    #[test]
    #[ignore = "needs a real terminal and possibly a sudo password"]
    fn sudo_inherited_stdio() {
        set_dry_run(false);
        assert!(sudo(["true"]), "sudo true failed");
    }
}
