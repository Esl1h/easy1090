//! Compile-time metadata: the crate version and the build commit become rustc
//! env vars, read back in main.rs with `option_env!`. This is phase 1
//! groundwork; the release phase consumes the commit in the published builds.

use std::process::Command;

fn git_commit() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let commit = String::from_utf8(out.stdout).ok()?;
    let commit = commit.trim();
    if commit.is_empty() {
        None
    } else {
        Some(commit.to_string())
    }
}

fn main() {
    // The version comes from Cargo.toml, never written by hand here.
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=EASY1090_VERSION={version}");

    let commit = git_commit().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=EASY1090_BUILD_COMMIT={commit}");
}
