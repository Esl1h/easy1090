//! End-to-end checks of the phase 4 install entry point: banner, --dry-run
//! warning and the ported preflight. The expectations are built from the
//! machine's own /etc/os-release and uid instead of assuming one, so the
//! refusal-path checks run wherever the distro is not Arch-derived (the CI
//! runner included) and skip themselves on Arch.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};

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

/// The machine's os-release, parsed the way the binary does: KEY=VALUE with
/// an optional surrounding quote pair, PRETTY_NAME defaulting to "?".
fn os_release() -> (String, String, String) {
    let content = fs::read_to_string("/etc/os-release").expect("/etc/os-release must exist");
    let mut id = String::new();
    let mut id_like = String::new();
    let mut pretty = String::from("?");
    for line in content.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = value.trim_end().trim_matches('"');
        match key {
            "ID" => id = value.to_string(),
            "ID_LIKE" => id_like = value.to_string(),
            "PRETTY_NAME" => pretty = value.to_string(),
            _ => {}
        }
    }
    (id, id_like, pretty)
}

/// `[[ "$id" == "arch" || "$id_like" == *arch* ]]` over the real file.
fn arch_derived() -> bool {
    let (id, id_like, _) = os_release();
    id == "arch" || id_like.contains("arch")
}

/// The effective uid, from /proc/self/status like the binary reads it.
fn effective_uid() -> u32 {
    fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find_map(|line| {
            let mut fields = line.split_whitespace();
            (fields.next() == Some("Uid:")).then(|| fields.next())
        })
        .flatten()
        .and_then(|uid| uid.parse().ok())
        .unwrap_or(0)
}

#[test]
fn dry_run_install_on_a_non_arch_distro_refuses_like_the_bash() {
    if arch_derived() {
        eprintln!("skipping: this machine is Arch-derived, the refusal needs a non-Arch distro");
        return;
    }

    let (_, _, pretty) = os_release();
    let out = bin()
        .args(["--lang", "en", "--dry-run", "install"])
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty(), "stdout must stay empty");
    let expected = format!(
        "\neasy1090 {}\n\
         [WARN ] --dry-run mode: nothing will be changed; the commands below are the real ones.\n\
         \n==> Preflight\n\
         [OK   ] Normal user (uid {}).\n\
         [INFO ] TTY: check relaxed under --dry-run.\n\
         [ERROR] v1 supports Arch and derivatives only (detected: {}). Other distros are on the roadmap; see the README.\n",
        version(),
        effective_uid(),
        pretty
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(stderr, expected);
}

#[test]
fn dry_run_install_refuses_in_portuguese_too() {
    if arch_derived() {
        eprintln!("skipping: this machine is Arch-derived, the refusal needs a non-Arch distro");
        return;
    }

    let (_, _, pretty) = os_release();
    let out = bin()
        .args(["--lang", "pt", "--dry-run", "install"])
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty(), "stdout must stay empty");
    let expected = format!(
        "\neasy1090 {}\n\
         [WARN ] Modo --dry-run: nada será alterado; os comandos abaixo são os reais.\n\
         \n==> Preflight\n\
         [OK   ] Usuário normal (uid {}).\n\
         [INFO ] TTY: verificação relaxada em --dry-run.\n\
         [ERROR] A v1 suporta apenas Arch e derivados (detectado: {}). Outras distros estão no roadmap; veja o README.\n",
        version(),
        effective_uid(),
        pretty
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(stderr, expected);
}

#[test]
fn the_refusal_path_never_invokes_sudo() {
    if arch_derived() {
        eprintln!("skipping: this machine is Arch-derived, the refusal needs a non-Arch distro");
        return;
    }

    // A PATH holding only a fake sudo that stamps a marker if anything runs
    // it. The refusal happens at the distro check, before tooling or any
    // command execution, so the marker must stay absent: the preflight is
    // read-only, sudo included.
    let dir: PathBuf = std::env::temp_dir().join(format!("easy1090-test-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let marker = dir.join("sudo-called");
    let sudo_script = dir.join("sudo");
    fs::write(
        &sudo_script,
        format!(
            "#!/bin/sh\necho called > \"{}\"\nexit 0\n",
            marker.display()
        ),
    )
    .unwrap();
    let mut perms = fs::metadata(&sudo_script).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&sudo_script, perms).unwrap();

    let out = bin()
        .env("PATH", &dir)
        .args(["--lang", "en", "--dry-run", "install"])
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(1));
    assert!(
        !marker.exists(),
        "the refusal path must not run anything, sudo included"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("[ERROR] v1 supports Arch and derivatives only"));

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn install_without_a_tty_stops_at_the_tty_check() {
    if effective_uid() == 0 {
        eprintln!("skipping: running as root exercises the root refusal instead");
        return;
    }

    // stdin forced to /dev/null so the check is deterministic even when
    // cargo test runs in a real terminal, where stdin is a tty.
    let out = bin()
        .stdin(Stdio::null())
        .args(["--lang", "en", "install"])
        .output()
        .unwrap();

    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty(), "stdout must stay empty");
    let expected = format!(
        "\neasy1090 {}\n\
         \n==> Preflight\n\
         [OK   ] Normal user (uid {}).\n\
         [ERROR] No interactive terminal. sudo needs a real tty; run this from a terminal or a proper SSH session.\n",
        version(),
        effective_uid()
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert_eq!(stderr, expected);
}
