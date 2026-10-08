//! Sudo validation and timestamp keepalive, mirroring sudo::init and
//! sudo::cleanup in lib/common.sh. The timestamp is validated once and then
//! kept warm in a background loop, so a long build never hits a password
//! prompt in the middle.

use std::sync::Mutex;

use crate::core::log;
use crate::core::run;
use crate::t;

static KEEPALIVE: Mutex<Option<std::process::Child>> = Mutex::new(None);

/// Validates sudo with `sudo -v` and starts the keepalive loop. Under
/// `--dry-run` it does nothing, like the bash.
pub fn init() {
    if run::dry_run() {
        return;
    }

    log::info(&t!("sudo_validating"));
    let ok = std::process::Command::new("sudo")
        .arg("-v")
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
        .map(|status| status.success())
        .unwrap_or(false);
    if !ok {
        crate::core::util::die(&t!("sudo_failed"));
    }

    // Same loop as the bash subshell: refresh the timestamp every 60s and
    // exit quietly when sudo stops answering.
    match std::process::Command::new("sh")
        .arg("-c")
        .arg("while true; do sleep 60; sudo -n true 2>/dev/null || exit 0; done")
        .spawn()
    {
        Ok(child) => {
            log::debug(&format!("sudo keepalive at pid {}", child.id()));
            *KEEPALIVE
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(child);
        }
        Err(error) => log::debug(&format!("sudo keepalive failed to start: {error}")),
    }
}

/// Kills the keepalive loop if one was started. Call before exiting.
pub fn cleanup() {
    if let Some(mut child) = KEEPALIVE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
    {
        let _ = child.kill();
        let _ = child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test-only view of the keepalive slot, to prove the dry-run skip.
    fn keepalive_running() -> bool {
        KEEPALIVE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    #[test]
    fn dry_run_skips_validation_and_keepalive() {
        run::set_dry_run(true);
        init();
        assert!(
            !keepalive_running(),
            "the bash sudo::init returns immediately under --dry-run"
        );
        run::set_dry_run(false);
        cleanup();
    }
}
