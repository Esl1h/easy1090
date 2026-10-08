//! easy1090 (Rust port), phase 5: the mutating commands.
//!
//! `install`, `uninstall`, `start`, `stop`, `restart`, `update` and `feed`
//! are now ported from lib/cmd-*.sh with the package layer behind the
//! `pkg::Backend` trait. Every mutation goes through the run layer, so
//! `--dry-run` previews the exact commands, and the state probes (pacman,
//! systemctl, files) still run for real, exactly like the bash.

pub mod cmd;
pub mod core;
pub mod pkg;
pub mod preflight;
pub mod steps;

use std::process::ExitCode;

use core::{i18n, log, run, util};

/// Set by build.rs (`cargo:rustc-env`); falls back to "unknown" if unset.
pub(crate) const VERSION: &str = match option_env!("EASY1090_VERSION") {
    Some(version) => version,
    None => "unknown",
};

/// The build commit, stamped by the same build.rs. Nothing prints it (the
/// bash has no equivalent), but the reference in main() keeps the value in
/// the binary, so `strings` on a published build shows what was shipped.
pub(crate) const BUILD_COMMIT: &str = match option_env!("EASY1090_BUILD_COMMIT") {
    Some(commit) => commit,
    None => "unknown",
};

/// The bash keeps install.conf next to the script (EASY1090_ROOT); the
/// binary resolves its start directory the same way, so the config and the
/// vendored installer are found next to it and every printed path is
/// absolute like the bash prints them.
pub(crate) fn config_file() -> std::path::PathBuf {
    util::root().join("install.conf")
}

pub(crate) fn config_example() -> std::path::PathBuf {
    util::root().join("install.conf.example")
}

fn print_help() {
    print!("{}", i18n::t("main_usage", &[&VERSION]));
}

/// `banner` from the bash entrypoint: the version line and the --dry-run
/// warning, both on stderr.
fn banner() {
    eprintln!(
        "\n{}easy1090{} {}",
        log::bold(),
        log::reset(),
        crate::VERSION
    );
    if run::dry_run() {
        log::warn(&i18n::t("cli_dry_warning", &[]));
    }
}

/// The uninstall arm of the bash entrypoint prints its own header line with
/// the title appended, then the same dry-run warning.
fn uninstall_banner() {
    eprintln!(
        "\n{}easy1090{} {} - {}",
        log::bold(),
        log::reset(),
        crate::VERSION,
        i18n::t("un_title", &[])
    );
    if run::dry_run() {
        log::warn(&i18n::t("cli_dry_warning", &[]));
    }
}

fn main() -> ExitCode {
    // Keeps BUILD_COMMIT in the binary; no output depends on it.
    std::hint::black_box(BUILD_COMMIT);

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

    i18n::init(Some(&config_file()), cli_lang);

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
        // status and open are read-only and print their own headers, so
        // they skip the banner and never touch sudo.
        Some("status") => cmd::status::run(),
        Some("open") => cmd::open::run(&command_args),
        Some("install") => {
            banner();
            cmd::install::run(&command_args, &config_file(), &config_example())
        }
        Some("uninstall") => {
            uninstall_banner();
            cmd::uninstall::run(&command_args, &config_file())
        }
        Some("start") => {
            banner();
            cmd::service::run("start", &command_args)
        }
        Some("stop") => {
            banner();
            cmd::service::run("stop", &command_args)
        }
        Some("restart") => {
            banner();
            cmd::service::run("restart", &command_args)
        }
        Some("update") => {
            banner();
            cmd::update::run(&command_args)
        }
        Some("feed") => {
            banner();
            cmd::feed::run(&command_args, &config_file(), &config_example())
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
