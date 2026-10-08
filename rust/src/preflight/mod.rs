//! Port of lib/00-preflight.sh: the read-only checks that run before any
//! installation step, in full even under --dry-run. Same checks, same order,
//! same messages, same exit codes and same abort behavior as the bash.

use std::io::IsTerminal;
use std::path::Path;
use std::process::Stdio;

use crate::core::{log, run, util};
use crate::t;

/// RTLSDR_USB_VENDOR / RTLSDR_USB_PRODUCT from 00-preflight.sh, also used by
/// the readsb udev rule in 20-readsb.sh.
pub const RTLSDR_USB_VENDOR: &str = "0bda";
pub const RTLSDR_USB_PRODUCT: &str = "2838";

pub fn run() {
    log::step(&t!("pre_step"));

    not_root();
    interactive_tty();
    distro();
    tooling();
    dongle();

    log::success(&t!("pre_done"));
}

// makepkg and yay refuse to run as root, but udev/systemd/pacman -U need it.
// The design is: run as a normal user, escalate per step.
fn not_root() {
    let euid = effective_uid();
    if euid == 0 {
        util::die(&t!("pre_root"));
    }
    log::success(&t!("pre_user_ok", euid));
}

/// $EUID from the bash: the effective uid. std has no geteuid and this tool
/// is Linux-only, so /proc/self/status is the source. Unparseable status
/// reads as root: refusing is safer than running makepkg as root.
fn effective_uid() -> u32 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| euid_in(&status))
        .unwrap_or(0)
}

/// The second number on the Uid: line (real, effective, saved, fs).
fn euid_in(status: &str) -> Option<u32> {
    for line in status.lines() {
        let mut fields = line.split_whitespace();
        if fields.next() != Some("Uid:") {
            continue;
        }
        if let Some(euid) = fields.next().and_then(|field| field.parse().ok()) {
            return Some(euid);
        }
    }
    None
}

// sudo needs a real tty for the password. Without it an automation pipeline
// either hangs or fails silently, and it always happens at the worst moment.
fn interactive_tty() {
    if run::dry_run() {
        log::info(&t!("pre_tty_dry"));
        return;
    }

    // `[[ -t 0 ]]`: stdin must be a terminal.
    if !std::io::stdin().is_terminal() {
        util::die(&t!("pre_tty_missing"));
    }
    log::success(&t!("pre_tty_ok"));
}

fn distro() {
    // `[[ -r /etc/os-release ]]`: missing or unreadable aborts the same way.
    let Some(release) = OsRelease::read(Path::new("/etc/os-release")) else {
        util::die(&t!("pre_osrelease"));
    };

    if release.arch_compatible() {
        log::success(&t!("pre_distro_ok", &release.pretty));
        return;
    }

    util::die(&t!("pre_distro_bad", &release.pretty));
}

fn tooling() {
    let required = ["pacman", "git", "curl"];
    let missing = find_missing(&required, util::have_cmd);

    if !missing.is_empty() {
        // ${missing[*]}: space-joined, like the bash expansion.
        util::die(&t!("pre_tools_missing", missing.join(" ")));
    }

    if !util::have_cmd("yay") {
        util::die(&t!("pre_yay_missing"));
    }

    log::success(&t!("pre_tools_ok", "pacman git curl yay"));
}

/// The tools that `have` reports absent, in the required order.
fn find_missing<'a>(required: &[&'a str], have: impl Fn(&str) -> bool) -> Vec<&'a str> {
    required
        .iter()
        .filter(|tool| !have(tool))
        .copied()
        .collect()
}

fn dongle() {
    if !util::have_cmd("lsusb") {
        log::warn(&t!("pre_lsusb_missing"));
        return;
    }

    let usb_id = format!("{RTLSDR_USB_VENDOR}:{RTLSDR_USB_PRODUCT}");

    // `lsusb | grep -qi "$usb_id"`: stderr of lsusb stays visible, like the
    // bash pipeline.
    let found = run::capture(["lsusb"], Stdio::inherit())
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .to_lowercase()
                .contains(usb_id.as_str())
        })
        .unwrap_or(false);

    if found {
        log::success(&t!("pre_dongle_ok", usb_id));
        return;
    }

    log::warn(&t!("pre_dongle_missing", &usb_id));
    log::warn(&t!("pre_dongle_warn"));

    if !crate::core::confirm::confirm(&t!("pre_dongle_confirm")) {
        util::die(&t!("pre_aborted"));
    }
}

/// The three variables the bash extracts by sourcing /etc/os-release, with
/// the same defaults: ID_LIKE empty, PRETTY_NAME `?`.
struct OsRelease {
    id: String,
    id_like: String,
    pretty: String,
}

impl OsRelease {
    /// Reads and parses the file; None when it cannot be read, which is the
    /// pre_osrelease die case.
    fn read(path: &Path) -> Option<OsRelease> {
        Some(OsRelease::parse(&std::fs::read_to_string(path).ok()?))
    }

    /// The sourcing equivalent for the os-release shape: assignments of the
    /// KEY="value" or KEY=value form, comments and other lines ignored.
    fn parse(content: &str) -> OsRelease {
        let mut id = String::new();
        let mut id_like = String::new();
        let mut pretty = String::from("?");

        for line in content.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim_end();
            let value = unquote(value.trim_end());
            match key {
                "ID" => id = value,
                "ID_LIKE" => id_like = value,
                "PRETTY_NAME" => pretty = value,
                _ => {}
            }
        }

        OsRelease {
            id,
            id_like,
            pretty,
        }
    }

    /// `[[ "$id" == "arch" || "$id_like" == *arch* ]]`.
    fn arch_compatible(&self) -> bool {
        self.id == "arch" || self.id_like.contains("arch")
    }
}

/// The shell removes the surrounding quote pair of a sourced value, so
/// ID="arch" becomes arch. os-release quotes the whole value or not at all.
fn unquote(value: &str) -> String {
    match value.as_bytes() {
        [b'"', .., b'"'] | [b'\'', .., b'\''] => value[1..value.len() - 1].to_string(),
        _ => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "easy1090-preflight-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// An executable stub for `name` inside `dir`, so the tests can build a
    /// fake PATH without touching the environment.
    fn fake_tool(dir: &std::path::Path, name: &str) {
        let file = dir.join(name);
        fs::write(&file, "#!/bin/sh\n").unwrap();
        let mut perms = fs::metadata(&file).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&file, perms).unwrap();
    }

    #[test]
    fn parses_os_release_like_the_bash_sourcing() {
        let release =
            OsRelease::parse("ID=\"arch\"\nID_LIKE=\"arch linux\"\nPRETTY_NAME=\"Arch Linux\"\n");
        assert_eq!(release.id, "arch");
        assert_eq!(release.id_like, "arch linux");
        assert_eq!(release.pretty, "Arch Linux");
        assert!(release.arch_compatible());
    }

    #[test]
    fn missing_values_default_like_the_bash() {
        let release = OsRelease::parse("# comment\nID=manjaro\nNO_EQUALS\n");
        assert_eq!(release.id, "manjaro");
        assert_eq!(release.id_like, "");
        assert_eq!(release.pretty, "?");
    }

    #[test]
    fn arch_derivatives_pass_and_others_refuse() {
        // The id itself.
        assert!(OsRelease::parse("ID=arch\n").arch_compatible());
        // id_like containing "arch" anywhere, the *arch* glob.
        assert!(OsRelease::parse("ID=manjaro\nID_LIKE=\"arch linux\"\n").arch_compatible());
        assert!(OsRelease::parse("ID=endeavouros\nID_LIKE=\"archlinux\"\n").arch_compatible());
        // Non-Arch ids, with and without id_like.
        assert!(!OsRelease::parse("ID=fedora\n").arch_compatible());
        assert!(!OsRelease::parse("ID=debian\nID_LIKE=\"debian\"\n").arch_compatible());
        // id_like without "arch" does not rescue a non-arch id.
        assert!(!OsRelease::parse("ID=ubuntu\nID_LIKE=\"debian\"\n").arch_compatible());
    }

    #[test]
    fn reads_os_release_from_an_injectable_path() {
        let dir = temp_dir();
        let file = dir.join("os-release");
        fs::write(
            &file,
            "ID=manjaro\nID_LIKE=\"arch\"\nPRETTY_NAME=\"Manjaro Linux\"\n",
        )
        .unwrap();

        let release = OsRelease::read(&file).unwrap();
        assert!(release.arch_compatible());
        assert_eq!(release.pretty, "Manjaro Linux");

        // The unreadable case is the pre_osrelease die.
        assert!(OsRelease::read(&dir.join("missing")).is_none());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn tool_detection_respects_the_search_path() {
        let dir = temp_dir();
        for name in ["pacman", "git", "curl"] {
            fake_tool(&dir, name);
        }

        let path = Some(OsString::from(dir.as_os_str()));
        let have = |tool: &str| util::have_cmd_in(tool, path.clone());

        let missing = find_missing(&["pacman", "git", "curl"], have);
        assert!(
            missing.is_empty(),
            "all three tools are present: {missing:?}"
        );

        // yay is deliberately absent: the AUR helper check is the separate
        // pre_yay_missing die.
        assert!(!util::have_cmd_in("yay", path.clone()));

        // A non-executable file is not a command either.
        fs::write(dir.join("not-script"), "").unwrap();
        assert!(!util::have_cmd_in("not-script", path));

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_tools_stay_in_the_required_order() {
        let dir = temp_dir();
        fake_tool(&dir, "git");

        let path = Some(OsString::from(dir.as_os_str()));
        let missing = find_missing(&["pacman", "git", "curl"], |tool| {
            util::have_cmd_in(tool, path.clone())
        });
        assert_eq!(missing, ["pacman", "curl"]);
        // The message joins them with spaces, ${missing[*]} in the bash.
        assert_eq!(missing.join(" "), "pacman curl");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn effective_uid_reads_the_status_line() {
        let status = "Name:\teasy1090\nUid:\t1000\t1000\t1000\t1000\nGid:\t1000\n";
        assert_eq!(euid_in(status), Some(1000));
        assert_eq!(euid_in("nothing here\n"), None);
        assert_eq!(euid_in("Uid:\n"), None);
    }

    #[test]
    fn the_real_effective_uid_agrees_with_proc() {
        // /proc/self is owned by the test process, so the uid from the
        // status line must match its metadata (the two differ only under
        // setuid, which cargo test is not).
        let owner = fs::metadata("/proc/self").unwrap().uid();
        assert_eq!(effective_uid(), owner);
    }
}
