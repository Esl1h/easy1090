//! The bare release binary has to work without install.conf.example or
//! vendor/ next to it: both are compiled in. Each test runs the real binary in
//! an empty directory, the way someone who downloaded only the .bin would.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// An empty working directory, removed when the test ends.
struct EmptyDir(PathBuf);

impl EmptyDir {
    fn new(name: &str) -> EmptyDir {
        let dir =
            std::env::temp_dir().join(format!("easy1090-standalone-{name}-{}", std::process::id()));
        fs::remove_dir_all(&dir).ok();
        fs::create_dir_all(&dir).unwrap();
        EmptyDir(dir)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_easy1090"))
            .current_dir(&self.0)
            .env("LC_ALL", "C")
            .args(args)
            .output()
            .unwrap()
    }
}

impl Drop for EmptyDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).ok();
    }
}

#[test]
fn first_run_creates_the_config_without_the_example_next_to_the_binary() {
    let dir = EmptyDir::new("real");

    let out = dir.run(&["--lang", "en", "feed", "--status"]);
    let err = String::from_utf8(out.stderr).unwrap();

    assert!(out.status.success(), "{err}");
    assert!(!err.contains("Example config not found"), "{err}");
    assert!(err.contains("==> Feeds\n"), "{err}");

    let config = fs::read_to_string(dir.0.join("install.conf")).unwrap();
    assert!(config.contains("UI_LANGUAGE=\"en\""), "{config}");
    assert!(config.contains("NET_BI_PORT=\"30004,30104\""), "{config}");
}

#[test]
fn dry_run_without_the_example_writes_nothing_and_still_succeeds() {
    let dir = EmptyDir::new("dry");

    let out = dir.run(&["--lang", "en", "feed", "--status", "--dry-run"]);
    let err = String::from_utf8(out.stderr).unwrap();

    assert!(out.status.success(), "{err}");
    assert!(err.contains("[DRY  ] cp "), "{err}");
    assert!(!dir.0.join("install.conf").exists());
}
