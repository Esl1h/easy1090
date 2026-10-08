//! Shared utilities from lib/common.sh: `util::die` and `util::have_cmd`.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::core::log;

/// `log::error` followed by exit 1, as the bash `util::die` does. The bash
/// traps EXIT for the sudo keepalive, so the cleanup runs here too.
pub fn die(message: &str) -> ! {
    log::error(message);
    crate::core::sudo::cleanup();
    std::process::exit(1);
}

/// The `EASY1090_ROOT` of the bash: the script's directory, resolved once at
/// startup. The binary keeps install.conf next to its working directory, so
/// the equivalent is the canonicalized start directory; canonicalizing makes
/// every printed path absolute, exactly what `cd dirname && pwd` gives the
/// bash.
pub fn root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        env::current_dir()
            .ok()
            .and_then(|dir| dir.canonicalize().ok())
            .or_else(|| env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."))
    })
}

/// `$HOME` for the build cache paths (`${HOME}/.cache/easy1090/...`).
pub fn home() -> String {
    env::var("HOME").unwrap_or_default()
}

/// `util::have_cmd`: true when the name is an executable file found on PATH.
/// A name containing a slash is checked at that path, like `command -v`.
pub fn have_cmd(name: &str) -> bool {
    have_cmd_in(name, env::var_os("PATH"))
}
/// The same check against an explicit search path; the preflight tests build
/// fake tooling in a temp dir without touching the environment.
pub fn have_cmd_in(name: &str, path: Option<OsString>) -> bool {
    if name.contains('/') {
        return executable(Path::new(name));
    }

    match path {
        Some(path) => env::split_paths(&path).any(|dir| executable(&dir.join(name))),
        // bash falls back to its compiled-in search path when PATH is unset.
        None => ["/usr/bin", "/bin"]
            .iter()
            .any(|dir| executable(&Path::new(dir).join(name))),
    }
}

/// The check behind `command -v`: a regular, executable file (bash does not
/// consider a directory a command even with the execute bit set).
fn executable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// awk's string-to-number conversion: the leading numeric prefix, 0 when the
/// string does not start with one. Shared by the status and validate JSON
/// freshness checks, both of which feed it a `.now` timestamp from jq.
pub fn awk_number(value: &str) -> f64 {
    let text = value.trim_start();
    let bytes = text.as_bytes();
    let mut end = 0;

    if end < bytes.len() && (bytes[end] == b'+' || bytes[end] == b'-') {
        end += 1;
    }
    let mut digits = false;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
        digits = true;
    }
    if end < bytes.len() && bytes[end] == b'.' {
        end += 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
            digits = true;
        }
    }

    if digits {
        text[..end].parse().unwrap_or(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_commands_on_path_and_rejects_directories() {
        // `sh` is on PATH everywhere these tests run; a random name is not.
        assert!(have_cmd("sh"));
        assert!(!have_cmd("easy1090-no-such-command-42"));
        assert!(!have_cmd("/usr"));
        assert!(!have_cmd("/etc/hostname"));
    }

    #[test]
    fn awk_number_reads_the_leading_numeric_prefix() {
        assert_eq!(awk_number("42"), 42.0);
        assert_eq!(awk_number("-3.5x"), -3.5);
        assert_eq!(awk_number("abc"), 0.0);
        assert_eq!(awk_number("  7"), 7.0);
        assert_eq!(awk_number("+1.25"), 1.25);
        assert_eq!(awk_number("."), 0.0);
    }
}
