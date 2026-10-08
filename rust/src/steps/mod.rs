//! Ports of the numbered install step modules (10-driver.sh, 20-readsb.sh,
//! 30-tar1090.sh, 40-optional.sh, 60-validate.sh). They are shared by the
//! install, uninstall and feed commands, which is what makes them a tree of
//! their own instead of one command module each.

pub mod driver;
pub mod optional;
pub mod readsb;
pub mod tar1090;
pub mod validate;
