//! Subcommand ports, one module per lib/cmd-*.sh file.

pub mod install;
pub mod open;
pub mod status;

use std::process::Stdio;

use crate::core::run;

/// `ip route get 1.1.1.1 | grep -o 'src [0-9.]*' | cut -d' ' -f2`, the
/// pipeline cmd-status.sh and cmd-open.sh share. The entrypoint runs under
/// `set -euo pipefail`, so a failing pipeline (ip missing, no `src` match)
/// aborts the command silently with exit 1; that is reproduced here instead
/// of returning a wrong address.
pub(crate) fn local_ip() -> String {
    let Some(out) = run::capture(["ip", "route", "get", "1.1.1.1"], Stdio::null()) else {
        std::process::exit(1);
    };
    if !out.status.success() {
        std::process::exit(1);
    }
    match src_address(&out.stdout) {
        Some(ip) => ip,
        None => std::process::exit(1),
    }
}

/// The `grep -o 'src [0-9.]*' | cut -d' ' -f2` part: the address after the
/// first `src ` on the route output, possibly empty when nothing follows it.
fn src_address(stdout: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(stdout);
    let start = text.find("src ")? + "src ".len();
    Some(
        text[start..]
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::src_address;

    #[test]
    fn src_address_parses_the_ip_route_output() {
        let line = b"1.1.1.1 via 192.168.1.1 dev wlan0 src 192.168.1.5 uid 1000\n";
        assert_eq!(src_address(line).as_deref(), Some("192.168.1.5"));
        assert_eq!(src_address(b"unreachable\n"), None);
        assert_eq!(src_address(b"1.1.1.1 dev eth0 src \n").as_deref(), Some(""));
    }
}
