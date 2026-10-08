//! Port of lib/30-tar1090.sh: tar1090, the live web map.
//!
//! There is no tar1090 package in the AUR. Upstream ships an install.sh
//! written for Debian and Raspberry Pi OS, which we vendor under vendor/
//! instead of piping it from the network on every run. The script itself
//! stays bash and is only executed, never ported or paraphrased: the pin in
//! install.conf (TAR1090_INSTALLER_SHA256) is verified before it runs.
//!
//! Three Arch specific gaps this module closes, all silent failures:
//!   - lighttpd.conf on Arch is minimal and never includes conf-enabled, so
//!     the config the tar1090 installer drops there is simply never read.
//!   - mod_redirect is not loaded, so url.redirect is ignored and the
//!     slash-less URL 404s (the very URL upstream prints when it finishes).
//!   - the upstream installer only restarts lighttpd if it was already
//!     running; a freshly installed one stays dead and disabled.

use std::path::Path;
use std::process::Stdio;

use crate::core::cfg::Config;
use crate::core::{log, run, svc, util};
use crate::pkg::Backend;
use crate::t;

const LIGHTTPD_CONF: &str = "/etc/lighttpd/lighttpd.conf";
const LIGHTTPD_CONF_D: &str = "/etc/lighttpd/conf.d";
const LIGHTTPD_CONF_AVAILABLE: &str = "/etc/lighttpd/conf-available";
const LIGHTTPD_CONF_ENABLED: &str = "/etc/lighttpd/conf-enabled";
const TAR1090_URL_PATH: &str = "/tar1090/";

/// `${EASY1090_ROOT}/vendor/tar1090-install.sh`, resolved at runtime like
/// the other config-adjacent paths.
pub fn installer_path() -> String {
    format!("{}/vendor/tar1090-install.sh", util::root().display())
}

/// Shared with the uninstall command.
pub const TAR1090_PATH: &str = "/usr/local/share/tar1090";
pub const TAR1090_UNINSTALL: &str = "/usr/local/share/tar1090/uninstall.sh";
pub const TAR1090_DEFAULTS: &str = "/etc/default/tar1090";
pub const MOD_REDIRECT_AVAILABLE: &str = "/etc/lighttpd/conf-available/06-mod_redirect.conf";
pub const MOD_REDIRECT_ENABLED: &str = "/etc/lighttpd/conf-enabled/06-mod_redirect.conf";

/// `tar1090::run`: dependencies, the vendored installer, then the three Arch
/// fixes. False when the web map does not answer, which aborts the install
/// like `set -e` does in the bash.
pub fn run(pkg: &dyn Backend, config: &Config) -> bool {
    log::step(&t!("tar_step"));

    dependencies(pkg);
    verify_vendored(config);
    install();

    let mut needs_restart = false;
    fix_lighttpd_include(&mut needs_restart);
    fix_mod_redirect(&mut needs_restart);
    enable_lighttpd(needs_restart)
}

/// `tar1090::is_done`: the unit file listed and enabled.
pub fn is_done() -> bool {
    svc::unit_exists("tar1090") && svc::is_enabled("tar1090")
}

fn dependencies(pkg: &dyn Backend) {
    pkg.install(&["jq", "lighttpd"]);

    // The upstream installer only switches to its automatic lighttpd mode
    // when this directory exists (a Debian convention the Arch package does
    // not use).
    if Path::new(LIGHTTPD_CONF_D).is_dir() {
        log::skip(&t!("tar_confd_ok", LIGHTTPD_CONF_D));
    } else {
        log::info(&t!("tar_confd_create", LIGHTTPD_CONF_D));
        run::sudo(["mkdir", "-p", LIGHTTPD_CONF_D]);
    }
}

/// The vendored copy is pinned by checksum. Updating it is a deliberate act:
/// refresh the file, then update TAR1090_INSTALLER_SHA256 in install.conf.
fn verify_vendored(config: &Config) {
    let installer = installer_path();
    if !Path::new(&installer).is_file() {
        util::die(&t!("tar_vendor_missing", &installer));
    }

    let actual = sha256_of(&installer);

    let pin = config.get("TAR1090_INSTALLER_SHA256").unwrap_or("");
    if pin.is_empty() {
        log::warn(&t!("tar_pin_missing"));
        log::warn(&t!("tar_pin_current", &actual));
        log::warn(&t!("tar_pin_hint"));
        return;
    }

    if actual != pin {
        log::error(&t!("tar_pin_mismatch"));
        log::error(&t!("tar_pin_expected", pin));
        log::error(&t!("tar_pin_got", &actual));
        util::die(&t!("tar_pin_abort"));
    }

    log::success(&t!("tar_pin_ok"));
}

/// `sha256sum <file> | cut -d' ' -f1`; an unreadable file yields an empty
/// hash, which fails the pin comparison.
fn sha256_of(path: &str) -> String {
    run::capture(["sha256sum", path], Stdio::null())
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string()
        })
        .unwrap_or_default()
}

fn install() {
    if is_done() {
        log::skip(&t!("tar_service_ok"));
        return;
    }

    log::info(&t!("tar_running"));
    run::sudo(["bash", &installer_path()]);
}

/// Without this include, everything the tar1090 installer wrote is dead
/// config.
fn fix_lighttpd_include(needs_restart: &mut bool) {
    if !Path::new(LIGHTTPD_CONF).is_file() {
        log::warn(&t!("tar_conf_missing", LIGHTTPD_CONF));
        return;
    }

    let content = std::fs::read_to_string(LIGHTTPD_CONF).unwrap_or_default();
    if content.contains("conf-enabled") {
        log::skip(&t!("tar_include_ok"));
        if svc::predates_file("lighttpd", LIGHTTPD_CONF) {
            *needs_restart = true;
        }
        return;
    }

    log::info(&t!("tar_include_add"));
    *needs_restart = true;
    if run::dry_run() {
        // The bash composes this line by hand, redirections and all, so the
        // preview prints it as one string instead of rendering argv.
        log::dry_run(&format!(
            "echo 'include_shell \"cat {LIGHTTPD_CONF_ENABLED}/*.conf\"' | sudo tee -a {LIGHTTPD_CONF}"
        ));
    } else {
        // printf 'include_shell "cat %s/*.conf"\n' | sudo tee -a >/dev/null
        let line = format!("include_shell \"cat {LIGHTTPD_CONF_ENABLED}/*.conf\"\n");
        let ok = run::sudo_write_append(LIGHTTPD_CONF, &line);
        if !ok {
            std::process::exit(1);
        }
    }

    log::info(&t!("tar_lighttpd_check"));
    if !run::sudo(["lighttpd", "-tt", "-f", LIGHTTPD_CONF]) {
        util::die(&t!("tar_lighttpd_invalid", LIGHTTPD_CONF));
    }
}

/// The tar1090 config uses url.redirect to send /tar1090 to /tar1090/, but
/// only ships module loaders for mod_alias and mod_setenv. On Debian
/// mod_redirect is already on in the base config; on Arch it is not, so
/// lighttpd parses the directive, warns "unknown config-key: url.redirect
/// (ignored)" and moves on.
fn fix_mod_redirect(needs_restart: &mut bool) {
    if !Path::new(LIGHTTPD_CONF_AVAILABLE).is_dir() {
        return;
    }

    if Path::new(MOD_REDIRECT_AVAILABLE).is_file() && Path::new(MOD_REDIRECT_ENABLED).exists() {
        log::skip(&t!("tar_redirect_ok"));
        // Present is not the same as loaded: a lighttpd started before this
        // file was written is still serving without it.
        if svc::predates_file("lighttpd", MOD_REDIRECT_AVAILABLE) {
            *needs_restart = true;
        }
        return;
    }

    log::info(&t!("tar_redirect_add"));
    let content = format!(
        "{}\nserver.modules += ( \"mod_redirect\" )",
        t!("tar_redirect_comment")
    );
    run::sudo_write(MOD_REDIRECT_AVAILABLE, &content);
    run::sudo(["ln", "-sf", MOD_REDIRECT_AVAILABLE, MOD_REDIRECT_ENABLED]);
    *needs_restart = true;
}

fn enable_lighttpd(needs_restart: bool) -> bool {
    log::info(&t!("tar_lighttpd_enable"));
    run::sudo(["systemctl", "enable", "lighttpd"]);

    // Same trap as readsb: enabling does not reload a running daemon, and
    // this is exactly the bug we documented in the upstream installer.
    if svc::is_active("lighttpd") {
        if needs_restart {
            log::info(&t!("tar_lighttpd_restart"));
            run::sudo(["systemctl", "restart", "lighttpd"]);
        }
    } else {
        run::sudo(["systemctl", "start", "lighttpd"]);
    }

    if run::dry_run() {
        return true;
    }

    run::cmd(["sleep", "2"]);

    let url = format!("http://localhost{TAR1090_URL_PATH}");
    let code = curl_code(&url);

    if code == "200" {
        log::success(&t!("tar_web_ok", &url));

        // The installer advertises the URL without the trailing slash, so it
        // is worth confirming that the redirect really works.
        let slashless = &url[..url.len() - 1];
        let slashless_code = curl_code(slashless);
        if !matches!(slashless_code.as_str(), "200" | "301" | "302") {
            log::warn(&t!("tar_web_slashless", slashless, slashless_code));
        }
    } else {
        log::error(&t!(
            "tar_web_fail",
            if code.is_empty() { "?" } else { code.as_str() }
        ));
        log::error(&t!("tar_web_hint"));
        return false;
    }

    true
}

/// `curl -s -o /dev/null -w '%{http_code}' "$url" || true`: the code, or an
/// empty string when curl cannot run.
fn curl_code(url: &str) -> String {
    run::capture(
        ["curl", "-s", "-o", "/dev/null", "-w", "%{http_code}", url],
        Stdio::null(),
    )
    .map(|out| {
        String::from_utf8_lossy(&out.stdout)
            .trim_end_matches('\n')
            .to_string()
    })
    .unwrap_or_default()
}
