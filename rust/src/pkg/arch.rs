//! The Arch backend: `pkg::is_installed` via pacman, everything else through
//! the run layer so `--dry-run` previews it. The struct is a unit marker; the
//! shared bodies live on the trait, so a Debian backend would only replace
//! this one probe and keep the same install/remove messages.

use std::process::Stdio;

use crate::core::run;
use crate::pkg::Backend;

/// The only Arch-specific state probe: `pacman -Q <package> &>/dev/null`.
/// Runs in dry-run too, like the bash, because it is a read, not a command
/// the preview should fake.
fn pacman_has(package: &str) -> bool {
    run::capture(["pacman", "-Q", package], Stdio::null())
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// The backend for Arch and derivatives, selected by cmd::install and the
/// other commands.
pub struct Arch;

impl Backend for Arch {
    fn is_installed(&self, package: &str) -> bool {
        pacman_has(package)
    }
}

#[cfg(test)]
mod tests {
    use super::super::parent_of;
    use super::*;

    #[test]
    fn arch_is_installed_reads_false_where_pacman_answers_no() {
        // A random package name: pacman answers "not found" and the probe
        // returns false without printing anything.
        assert!(!pacman_has("easy1090-no-such-package-42"));
    }

    #[test]
    fn parent_of_splits_like_dirname() {
        assert_eq!(
            parent_of("/home/u/.cache/easy1090/readsb-wiedehopf-git"),
            "/home/u/.cache/easy1090"
        );
        assert_eq!(parent_of("/top"), "/");
    }
}
