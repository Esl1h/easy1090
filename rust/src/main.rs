//! easy1090 (Rust port), phase 2: core foundation.
//!
//! The binary answers --version and -h/--help (text from MSG[main_usage] in
//! the selected language) and errors for everything else, with the messages
//! coming from the same catalog. --lang selects the catalog; the remaining
//! global flags set their state but have no commands to act on yet.
//! Subcommands arrive in later phases.

// Public so the core modules stay reachable even before every piece is wired
// into a subcommand; phases 3+ consume them from here.
pub mod core;

use std::process::ExitCode;

use core::{i18n, log, run};

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

/// The bash keeps install.conf next to the script (EASY1090_ROOT); the
/// binary looks next to its working directory until the layout is settled.
const CONFIG_FILE: &str = "install.conf";

fn print_help() {
    print!("{}", i18n::t("main_usage", &[&VERSION]));
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // The bash exits on --version anywhere in the argument list, before the
    // language is even resolved.
    if args.iter().any(|arg| arg == "--version") {
        println!("easy1090 {VERSION}");
        return ExitCode::SUCCESS;
    }

    // Global flags, parsed like the bash parse_global: --lang takes the next
    // argument as its value (whatever it is), the rest set state. Without
    // subcommands yet, only the language has an observable effect.
    let mut command: Option<&str> = None;
    let mut help_wanted = false;
    let mut cli_lang: Option<&str> = None;
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--lang" => cli_lang = iter.next().map(String::as_str),
            "--dry-run" => run::set_dry_run(true),
            "--yes" | "-y" => run::set_assume_yes(true),
            "--verbose" => log::set_level(3),
            "-h" | "--help" => {
                if command.is_none() {
                    help_wanted = true;
                }
            }
            _ if arg.starts_with('-') => {}
            _ => {
                if command.is_none() {
                    command = Some(arg.as_str());
                }
            }
        }
    }

    i18n::init(Some(std::path::Path::new(CONFIG_FILE)), cli_lang);

    if help_wanted {
        print_help();
        return ExitCode::SUCCESS;
    }

    match command {
        None => {
            // Unknown options land in the command args in the bash too, so a
            // bare --flag ends here, not in a cli_unknown_opt error.
            log::error(&i18n::t("cmd_missing", &[]));
            print_help();
        }
        Some(cmd) if KNOWN_COMMANDS.contains(&cmd) => {
            log::error(&format!("\"{cmd}\" is not implemented yet."));
        }
        Some(cmd) => {
            log::error(&i18n::t("cmd_unknown", &[&cmd]));
            print_help();
        }
    }

    ExitCode::FAILURE
}
