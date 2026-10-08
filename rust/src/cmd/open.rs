//! Port of lib/cmd-open.sh: open a component in the current terminal.
//!
//! No sudo anywhere; only the map and the two GUIs need a graphical session,
//! and they check before trying. With no target the command lists the targets
//! on stderr and exits 0; an unknown target errors on stderr and exits 1.

use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::cmd::local_ip;
use crate::core::{log, run, util};
use crate::t;

/// `cmd::open`: dispatch on the first argument, ignoring the rest like the
/// bash `local target="${1:-}"`.
pub fn run(args: &[&str]) -> i32 {
    let target = args.first().copied().unwrap_or("");

    if target.is_empty() {
        list();
        return 0;
    }

    match target {
        "viewadsb" | "view" => exec("viewadsb", &[]),
        "sbs" | "raw" | "nc" => sbs(),
        "map" | "tar1090" | "web" => map(),
        "sdrpp" | "sdr" => gui("sdrpp"),
        "satdump" => gui("satdump-ui"),
        _ => {
            log::error(&t!("open_unknown", target));
            list();
            1
        }
    }
}

/// `open::list`: the targets, all on stderr.
fn list() {
    eprintln!("\n{}{}{}", log::bold(), t!("open_targets"), log::reset());
    eprint!(
        "{}\n{}\n{}\n{}\n{}\n\n",
        t!("open_t_viewadsb"),
        t!("open_t_sbs"),
        t!("open_t_map"),
        t!("open_t_sdrpp"),
        t!("open_t_satdump")
    );
}

/// `open::exec`: check, announce, then replace the process. `exec` keeps
/// Ctrl+C and the terminal exactly as if the command had been typed.
fn exec(cmd: &str, args: &[&str]) -> i32 {
    if !util::have_cmd(cmd) {
        log::error(&t!("open_missing", cmd));
        return 1;
    }

    // `"$cmd $*"`: with no arguments the message keeps a trailing space,
    // exactly like the bash expansion.
    log::info(&t!("open_running", format!("{cmd} {}", args.join(" "))));

    if run::dry_run() {
        let argv: Vec<&str> = std::iter::once(cmd).chain(args.iter().copied()).collect();
        log::dry_run(&run::render(argv));
        return 0;
    }

    let error = Command::new(cmd).args(args).exec();
    // Only reached when exec failed after the check above; the bash exits
    // 127 in the same situation.
    eprintln!("{cmd}: {error}");
    std::process::exit(127);
}

/// `open::sbs`: a raw netcat to the SBS port. The port comes from the
/// environment with the same 30003 default: status and open never load
/// install.conf in the bash either.
fn sbs() -> i32 {
    let port = env::var("NET_SBS_PORT")
        .ok()
        .filter(|port| !port.is_empty())
        .unwrap_or_else(|| "30003".to_string());

    if !util::have_cmd("nc") {
        log::error(&t!("open_missing", "nc"));
        return 1;
    }

    exec("nc", &["localhost", port.as_str()])
}

/// `open::map`: print the URL, then hand it to xdg-open only when a
/// graphical session exists. On a headless box the URL is the outcome.
fn map() -> i32 {
    let ip = local_ip();
    let host = if ip.is_empty() {
        "localhost"
    } else {
        ip.as_str()
    };
    let url = format!("http://{host}/tar1090/");

    log::info(&t!("open_url", &url));

    if !has_display() {
        return 0;
    }
    if !util::have_cmd("xdg-open") {
        return 0;
    }

    run::exit_code(["xdg-open", url.as_str()])
}

/// `open::gui`: the GUIs refuse to start without a graphical session.
fn gui(cmd: &str) -> i32 {
    if !has_display() {
        log::error(&t!("open_no_display", cmd));
        return 1;
    }

    exec(cmd, &[])
}

/// `open::has_display`: `[[ -n "${DISPLAY:-}" || -n "${WAYLAND_DISPLAY:-}" ]]`.
fn has_display() -> bool {
    ["DISPLAY", "WAYLAND_DISPLAY"].iter().any(|name| {
        env::var(name)
            .map(|value| !value.is_empty())
            .unwrap_or(false)
    })
}
