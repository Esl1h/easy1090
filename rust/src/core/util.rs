//! Shared utilities from lib/common.sh: `util::die` and `util::have_cmd`.

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::core::log;

/// `log::error` followed by exit 1, as the bash `util::die` does.
pub fn die(message: &str) -> ! {
    log::error(message);
    std::process::exit(1);
}

/// `util::have_cmd`: true when the name is an executable file found on PATH.
/// A name containing a slash is checked at that path, like `command -v`.
pub fn have_cmd(name: &str) -> bool {
    if name.contains('/') {
        return executable(Path::new(name));
    }

    match env::var_os("PATH") {
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
}
