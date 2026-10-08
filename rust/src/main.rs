//! easy1090 (Rust port), phase 3: read-only commands.
//!
//! `status` and `open` are ported from lib/cmd-status.sh and lib/cmd-open.sh;
//! every other subcommand still reports "not implemented yet". --version and
//! -h/--help keep working from phase 1.

pub mod cmd;
pub mod core;

use std::process::ExitCode;

use core::{i18n, log, run};

/// Set by build.rs (`cargo:rustc-env`); falls back to "unknown" if unset.
pub(crate) const VERSION: &str = match option_env!("EASY1090_VERSION") {
    Some(version) => version,
    None => "unknown",
};

/// Commands the bash entrypoint dispatches to. The ones without a port yet
/// report "not implemented".
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
    // argument as its value (whatever it is), the rest set state. Everything
    // that is not a recognized global flag is handed to the subcommand; -h
    // and --help only mean "help" before a command has been seen.
    let mut command: Option<&str> = None;
    let mut command_args: Vec<&str> = Vec::new();
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
                } else {
                    command_args.push(arg.as_str());
                }
            }
            _ if arg.starts_with('-') => command_args.push(arg.as_str()),
            _ => {
                if command.is_none() {
                    command = Some(arg.as_str());
                } else {
                    command_args.push(arg.as_str());
                }
            }
        }
    }

    i18n::init(Some(std::path::Path::new(CONFIG_FILE)), cli_lang);

    if help_wanted {
        print_help();
        return ExitCode::SUCCESS;
    }

    let code = match command {
        None => {
            // Unknown options land in the command args in the bash too, so a
            // bare --flag ends here, not in a cli_unknown_opt error.
            log::error(&i18n::t("cmd_missing", &[]));
            print_help();
            1
        }
        Some("status") => cmd::status::run(),
        Some("open") => cmd::open::run(&command_args),
        Some(cmd) if KNOWN_COMMANDS.contains(&cmd) => {
            log::error(&format!("\"{cmd}\" is not implemented yet."));
            1
        }
        Some(cmd) => {
            log::error(&i18n::t("cmd_unknown", &[&cmd]));
            print_help();
            1
        }
    };

    if code == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(code.clamp(0, 255) as u8)
    }
}
