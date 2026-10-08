//! Shared utilities from lib/common.sh, starting with `util::die`.

use crate::core::log;

/// `log::error` followed by exit 1, as the bash `util::die` does.
pub fn die(message: &str) -> ! {
    log::error(message);
    std::process::exit(1);
}
