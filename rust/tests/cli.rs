//! End-to-end checks of the binary: version, help and error paths, in both
//! languages. The locale is pinned so the language detection never depends on
//! the machine running the tests.

use std::process::Command;

fn bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_easy1090"));
    cmd.env("LC_ALL", "C")
        .env("LC_MESSAGES", "C")
        .env("LANG", "C");
    cmd
}

fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[test]
fn version_flag_prints_name_and_version() {
    let out = bin().arg("--version").output().unwrap();
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout, format!("easy1090 {}\n", version()));
}

#[test]
fn help_flag_prints_main_usage() {
    let out = bin().arg("--help").output().unwrap();
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with(&format!(
        "easy1090 {} - ADS-B stack in one command",
        version()
    )));
    assert!(stdout.contains("COMMANDS\n    install"));
    assert!(stdout.contains("GLOBAL OPTIONS"));
    assert!(stdout.ends_with("status and open do not need sudo.\n"));
}

#[test]
fn short_help_flag_also_works() {
    let out = bin().arg("-h").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("USAGE\n    easy1090 <command> [options]"));
}

#[test]
fn no_arguments_errors_with_usage() {
    let out = bin().output().unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Please provide a command. Use --help for the list."));
    assert!(stdout.contains("USAGE"));
}

#[test]
fn unknown_command_errors_with_usage() {
    let out = bin().arg("frobnicate").output().unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Unknown command: frobnicate"));
    assert!(stdout.contains("USAGE"));
}

#[test]
fn unimplemented_command_errors() {
    // install has a phase 4 port now (banner + preflight); the rest still
    // report the generic message.
    let out = bin().arg("update").output().unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("\"update\" is not implemented yet."));
    assert!(stdout.is_empty());
}

#[test]
fn unknown_option_errors_with_missing_command() {
    // The bash sends unknown flags to the (absent) subcommand and ends in
    // cmd_missing; cli_unknown_opt is for the installer's own parser.
    let out = bin().arg("--frobnicate").output().unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Please provide a command. Use --help for the list."));
    assert!(stdout.contains("USAGE"));
}

#[test]
fn lang_pt_selects_the_portuguese_help() {
    let out = bin().args(["--lang", "pt", "--help"]).output().unwrap();
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with(&format!(
        "easy1090 {} - stack ADS-B em um comando",
        version()
    )));
    assert!(stdout.contains("COMANDOS\n    install"));
    assert!(stdout.contains("OPÇÕES GLOBAIS"));
    assert!(stdout.ends_with("status e open não precisam de sudo.\n"));
}

#[test]
fn lang_en_keeps_the_english_help() {
    let out = bin().args(["--lang", "en", "--help"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with(&format!(
        "easy1090 {} - ADS-B stack in one command",
        version()
    )));
}

#[test]
fn lang_pt_selects_portuguese_errors() {
    let out = bin().args(["--lang", "pt"]).output().unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Informe um comando. Use --help para ver a lista."));
    assert!(stdout.contains("USO\n"));

    let out = bin().args(["--lang", "pt", "frobnicate"]).output().unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Comando desconhecido: frobnicate"));
}

#[test]
fn unsupported_language_is_rejected_like_the_bash() {
    let out = bin().args(["--lang", "fr", "--help"]).output().unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Idioma não suportado / unsupported language: fr (pt, en)"));
}

#[test]
fn version_wins_before_language_validation() {
    // The bash exits on --version while parsing flags, before i18n::init.
    let out = bin().args(["--lang", "fr", "--version"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout, format!("easy1090 {}\n", version()));
}
