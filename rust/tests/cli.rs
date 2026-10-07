//! End-to-end checks of the phase 1 binary: version, help and error paths.

use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_easy1090"))
}

#[test]
fn version_flag_prints_name_and_version() {
    let out = bin().arg("--version").output().unwrap();
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout, format!("easy1090 {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn help_flag_prints_main_usage() {
    let out = bin().arg("--help").output().unwrap();
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("easy1090 0.1.0 - ADS-B stack in one command"));
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
    let out = bin().arg("install").output().unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("\"install\" is not implemented yet."));
    assert!(stdout.is_empty());
}

#[test]
fn unknown_option_errors() {
    let out = bin().arg("--frobnicate").output().unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Unknown option: --frobnicate"));
}
