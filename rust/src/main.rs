//! easy1090 (Rust port), phase 1: scaffold.
//!
//! The binary answers only `--version` and `-h`/`--help`. Subcommands and the
//! global flags arrive in later phases, mirroring the bash entrypoint.

use std::process::ExitCode;

/// Set by build.rs (`cargo:rustc-env`); falls back to "unknown" if unset.
const VERSION: &str = match option_env!("EASY1090_VERSION") {
    Some(version) => version,
    None => "unknown",
};

/// Commands the bash entrypoint dispatches to. Not implemented yet.
const KNOWN_COMMANDS: &[&str] = &[
    "install",
    "update",
    "feed",
    "uninstall",
    "status",
    "start",
    "stop",
    "restart",
    "open",
];

/// Global flags the bash entrypoint accepts. Not implemented yet.
const KNOWN_GLOBAL_FLAGS: &[&str] = &["--lang", "--dry-run", "--yes", "-y", "--verbose"];

/// Byte-for-byte copy of MSG[main_usage] from lib/i18n/en.sh (English default).
/// The `%s` placeholder carries the crate version, exactly like the bash `t`.
const MAIN_USAGE: &str = "easy1090 %s - ADS-B stack in one command (Arch and derivatives)\n\nUSAGE\n    easy1090 <command> [options]\n\nCOMMANDS\n    install       install the stack (idempotent, safe to re-run)\n    update        update package versions (install converges config)\n    feed          feed public networks (ADSBExchange, airplanes.live)\n    uninstall     undo the installation (best effort)\n    status        what is running, what fell over, what is missing\n    start         bring up readsb, lighttpd and tar1090\n    stop          bring all three down\n    restart       restart all three, in the right order\n    open [target] open a component (without a target, lists the options)\n\nGLOBAL OPTIONS\n    --lang <pt|en>   interface language\n    --dry-run        print the exact commands, without executing\n    --yes            do not ask anything (except the sudo password)\n    --verbose        debug level logging\n    --version        show version\n    -h, --help       this help\n\nUse \"easy1090 <command> --help\" for per-command options.\n\nstatus and open do not need sudo.\n";

/// MSG[cmd_missing] from lib/i18n/en.sh.
const MSG_CMD_MISSING: &str = "Please provide a command. Use --help for the list.";

/// MSG[cmd_unknown] from lib/i18n/en.sh.
const MSG_CMD_UNKNOWN: &str = "Unknown command: %s";

/// MSG[cli_unknown_opt] from lib/i18n/en.sh.
const MSG_CLI_UNKNOWN_OPT: &str = "Unknown option: %s";

fn print_help() {
    print!("{}", MAIN_USAGE.replace("%s", VERSION));
}

/// Same shape as log::error in the bash tree (stderr, `[ERROR] ` prefix).
fn error(message: &str) {
    eprintln!("[ERROR] {message}");
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // The bash entrypoint exits on --version anywhere in the argument list.
    if args.iter().any(|arg| arg == "--version") {
        println!("easy1090 {VERSION}");
        return ExitCode::SUCCESS;
    }

    // Likewise -h/--help; per-command help does not exist yet, so this is
    // always the main usage.
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_help();
        return ExitCode::SUCCESS;
    }

    match args.first().map(String::as_str) {
        None => {
            error(MSG_CMD_MISSING);
            print_help();
        }
        Some(arg) if arg.starts_with('-') => {
            if KNOWN_GLOBAL_FLAGS.contains(&arg) {
                error(&format!("\"{arg}\" is not implemented yet."));
            } else {
                error(&MSG_CLI_UNKNOWN_OPT.replace("%s", arg));
                print_help();
            }
        }
        Some(cmd) if KNOWN_COMMANDS.contains(&cmd) => {
            error(&format!("\"{cmd}\" is not implemented yet."));
        }
        Some(cmd) => {
            error(&MSG_CMD_UNKNOWN.replace("%s", cmd));
            print_help();
        }
    }

    ExitCode::FAILURE
}
