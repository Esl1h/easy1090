//! Phase 2 core: ports of lib/common.sh and lib/i18n.sh.
//!
//! - `log`: leveled output with colors, exactly as the bash `log::*` prints
//! - `run`: command execution with a `--dry-run` preview of the real command
//! - `sudo`: `sudo -v` validation plus the timestamp keepalive loop
//! - `confirm`: the `util::confirm` prompt, defaults and EOF behavior
//! - `cfg`: `install.conf` parsing and in-place single-key persistence
//! - `i18n`: the message catalogs and the `t()` equivalent

pub mod cfg;
pub mod confirm;
pub mod i18n;
pub mod log;
pub mod run;
pub mod sudo;
pub mod svc;
pub mod util;
