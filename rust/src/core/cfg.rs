//! install.conf parsing and persistence, mirroring cfg::load and cfg::_persist
//! in lib/common.sh. The file is the /etc/default style the bash sources:
//! KEY="value" lines, blank lines and # comments. Values may be empty, may
//! contain commas (NET_BI_PORT="30004,30104"), and hand edits may drop the
//! quotes; persistence always writes quotes back.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::core::{i18n, log, run};
use crate::t;

/// Defaults applied to missing or empty values, from cfg::_apply_defaults.
const DEFAULTS: &[(&str, &str)] = &[
    ("RECEIVER_GAIN", "auto"),
    ("RECEIVER_PPM", "0"),
    ("DECODER_MAX_RANGE", "450"),
    ("JSON_LOCATION_ACCURACY", "2"),
    ("COMPONENT_TAR1090", "true"),
    ("COMPONENT_SDRPP", "false"),
    ("COMPONENT_SATDUMP", "false"),
    ("NET_RI_PORT", "30001"),
    ("NET_RO_PORT", "30002"),
    ("NET_SBS_PORT", "30003"),
    ("NET_BI_PORT", "30004,30104"),
    ("NET_BO_PORT", "30005"),
    ("FEEDER_ADSBEXCHANGE", "false"),
    ("FEEDER_AIRPLANESLIVE", "false"),
    ("FEEDER_ASKED", "false"),
    ("TAR1090_INSTALLER_SHA256", ""),
];

/// The parsed configuration. Key order is sorted, which makes iteration
/// deterministic without changing the meaning of the flat key/value format.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    values: BTreeMap<String, String>,
}

impl Config {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// Loads install.conf, creating it from the example on the first run. The
/// language must already be resolved (i18n::init); cfg::load re-persists it
/// here so the next run does not ask again, exactly like the bash.
pub fn load(config_file: &Path, example_file: &Path) -> Config {
    load_impl(
        config_file,
        example_file,
        run::dry_run(),
        i18n::language(),
        i18n::language_from_config(),
    )
}

fn load_impl(
    config_file: &Path,
    example_file: &Path,
    dry_run: bool,
    language: &str,
    language_from_config: bool,
) -> Config {
    let mut source = config_file.to_path_buf();
    if !config_file.exists() {
        if dry_run {
            // Nothing is written in dry-run, so read the example instead;
            // without this the run would report defaults that differ from
            // what a real install would use.
            log::info(&t!("cfg_missing_dry"));
            log::dry_run(&format!(
                "cp {} {}",
                example_file.display(),
                config_file.display()
            ));
            source = example_file.to_path_buf();
        } else {
            // Creating it from the example is harmless and is what every
            // first run needs, so it is not worth a prompt the user can trip
            // over.
            if !example_file.exists() {
                crate::core::util::die(&t!("cfg_example_missing", example_file.display()));
            }
            if let Err(error) = fs::copy(example_file, config_file) {
                crate::core::util::die(&error.to_string());
            }
            log::info(&t!("cfg_created", config_file.display()));
        }
    }

    let mut config = Config {
        values: parse_str(&fs::read_to_string(&source).unwrap_or_default()),
    };
    apply_defaults(&mut config);

    // Persist the language so the next run does not ask again. Testing for
    // the key is not enough: the example ships UI_LANGUAGE="", what matters
    // is having a value, and UI_LANGUAGE_FROM_CONFIG says it came from one.
    if !dry_run && !language_from_config {
        persist_impl(config_file, "UI_LANGUAGE", language, false);
    }

    config
}

fn apply_defaults(config: &mut Config) {
    for (key, default) in DEFAULTS {
        if config.get(key).is_none_or(str::is_empty) {
            config.set(key, default);
        }
    }
}

/// Parses the file contents: KEY="value" (quotes required only by
/// persistence), KEY=value, comments and blank lines. Returns the values
/// keyed in sorted order.
pub fn parse_str(content: &str) -> BTreeMap<String, String> {
    let mut values = BTreeMap::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some(eq) = line.find('=') else {
            continue;
        };
        let key = line[..eq].trim();
        if key.is_empty() || !key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        values.insert(key.to_string(), parse_value(&line[eq + 1..]));
    }
    values
}

/// The value side of KEY=...: a quoted value runs to the closing quote
/// (trailing comments allowed, as in install.conf.example), a bare one runs
/// to a # comment preceded by whitespace.
fn parse_value(raw: &str) -> String {
    let rest = raw.trim_start();
    if let Some(unquoted) = rest.strip_prefix('"') {
        match unquoted.find('"') {
            Some(end) => unquoted[..end].to_string(),
            None => unquoted.to_string(),
        }
    } else {
        match rest.find(" #") {
            Some(index) => rest[..index].trim_end().to_string(),
            None => rest.trim_end().to_string(),
        }
    }
}

/// Rewrites a single key in place, preserving the rest of the file and its
/// comments, like cfg::_persist.
pub fn persist(file: &Path, key: &str, value: &str) {
    persist_impl(file, key, value, run::dry_run());
}

fn persist_impl(file: &Path, key: &str, value: &str, dry_run: bool) {
    // sed replacement semantics: & expands to the match, | is our delimiter.
    let escaped = value
        .replace('\\', "\\\\")
        .replace('&', "\\&")
        .replace('|', "\\|");

    if dry_run {
        log::dry_run(&sed_preview(file, key, &escaped));
        return;
    }

    let content = fs::read_to_string(file).unwrap_or_default();
    let prefix = format!("{key}=");
    let replacement = format!("{key}=\"{escaped}\"");
    let mut out = String::with_capacity(content.len() + replacement.len() + 1);
    let mut found = false;
    for line in content.split_inclusive('\n') {
        if line.starts_with(&prefix) {
            found = true;
            out.push_str(&replacement);
            out.push('\n');
        } else {
            out.push_str(line);
        }
    }
    if !found {
        // The append branch writes the raw value, not the sed-escaped one.
        out.push_str(&format!("{key}=\"{value}\"\n"));
    }
    if let Err(error) = fs::write(file, out) {
        crate::core::util::die(&error.to_string());
    }
}

/// The exact sed line the dry-run preview shows.
fn sed_preview(file: &Path, key: &str, escaped: &str) -> String {
    format!(
        "sed -i 's|^{key}=.*|{key}=\"{escaped}\"|' {}",
        file.display()
    )
}

/// Feeds a public network only when asked explicitly, mirroring
/// cfg::require_feeder. Already-enabled or non-interactive runs return at
/// once; otherwise it asks 1/2/3 and records the choice in the file.
pub fn require_feeder(config_file: &Path, config: &mut Config) {
    if config.get("FEEDER_ADSBEXCHANGE") == Some("true")
        || config.get("FEEDER_AIRPLANESLIVE") == Some("true")
    {
        return;
    }
    if run::dry_run() || run::assume_yes() {
        return;
    }
    let already_asked = fs::read_to_string(config_file)
        .map(|content| {
            content
                .lines()
                .any(|l| l.starts_with("FEEDER_ASKED=\"true\""))
        })
        .unwrap_or(false);
    if already_asked {
        return;
    }

    eprint!(
        "\n{}\n{}\n\n{}\n{}\n{}\n\n{}\n\n",
        t!("feed_title"),
        t!("feed_explain"),
        t!("feed_opt_none"),
        t!("feed_opt_adsbx"),
        t!("feed_opt_alive"),
        t!("feed_fa_note"),
    );
    let _ = std::io::stderr().flush();
    eprint!("{}", t!("feed_prompt"));
    let _ = std::io::stderr().flush();

    let mut answer = String::new();
    let answer = std::io::stdin()
        .read_line(&mut answer)
        .map(|_| answer.trim().to_string())
        .unwrap_or_default();

    match answer.as_str() {
        "2" => {
            config.set("FEEDER_ADSBEXCHANGE", "true");
            persist(config_file, "FEEDER_ADSBEXCHANGE", "true");
        }
        "3" => {
            config.set("FEEDER_AIRPLANESLIVE", "true");
            persist(config_file, "FEEDER_AIRPLANESLIVE", "true");
        }
        _ => log::info(&t!("feed_none")),
    }

    config.set("FEEDER_ASKED", "true");
    persist(config_file, "FEEDER_ASKED", "true");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("easy1090-cfg-test-{}-{name}", std::process::id()))
    }

    #[test]
    fn parses_plain_and_quoted_keys() {
        let config = parse_str("PLAIN=value\nQUOTED=\"hello world\"\n");
        assert_eq!(config.get("PLAIN").map(String::as_str), Some("value"));
        assert_eq!(
            config.get("QUOTED").map(String::as_str),
            Some("hello world")
        );
    }

    #[test]
    fn values_with_commas_survive() {
        let config = parse_str("NET_BI_PORT=\"30004,30104\"\n");
        assert_eq!(
            config.get("NET_BI_PORT").map(String::as_str),
            Some("30004,30104")
        );
    }

    #[test]
    fn empty_values_are_kept() {
        let config = parse_str("UI_LANGUAGE=\"\"\nRECEIVER_LAT=\"\"\n");
        assert_eq!(config.get("UI_LANGUAGE").map(String::as_str), Some(""));
        assert_eq!(config.get("RECEIVER_LAT").map(String::as_str), Some(""));
    }

    #[test]
    fn comments_and_blank_lines_are_ignored() {
        let config = parse_str("# full line\n\nX=\"1\"        # trailing\n  # indented\nY=2\n");
        assert_eq!(config.get("X").map(String::as_str), Some("1"));
        assert_eq!(config.get("Y").map(String::as_str), Some("2"));
        assert_eq!(config.len(), 2);
    }

    #[test]
    fn persist_updates_only_the_target_line() {
        let path = temp_path("persist");
        fs::write(&path, "# keep me\nUI_LANGUAGE=\"\"\nRECEIVER_LAT=\"1\"\n").unwrap();
        persist_impl(&path, "UI_LANGUAGE", "pt", false);
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "# keep me\nUI_LANGUAGE=\"pt\"\nRECEIVER_LAT=\"1\"\n"
        );
        fs::remove_file(&path).ok();
    }

    #[test]
    fn persist_appends_missing_keys() {
        let path = temp_path("append");
        fs::write(&path, "RECEIVER_LAT=\"1\"\n").unwrap();
        persist_impl(&path, "UI_LANGUAGE", "pt", false);
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "RECEIVER_LAT=\"1\"\nUI_LANGUAGE=\"pt\"\n"
        );
        fs::remove_file(&path).ok();
    }

    #[test]
    fn persist_escapes_sed_metacharacters() {
        let path = temp_path("escape");
        fs::write(&path, "KEY=\"old\"\n").unwrap();
        persist_impl(&path, "KEY", "a\\b&c|d", false);
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "KEY=\"a\\\\b\\&c\\|d\"\n"
        );
        fs::remove_file(&path).ok();
    }

    #[test]
    fn persist_dry_run_previews_the_sed_command() {
        let preview = sed_preview(Path::new("/etc/install.conf"), "UI_LANGUAGE", "pt");
        assert_eq!(
            preview,
            "sed -i 's|^UI_LANGUAGE=.*|UI_LANGUAGE=\"pt\"|' /etc/install.conf"
        );
    }

    #[test]
    fn load_creates_the_config_from_the_example_and_keeps_its_bytes() {
        let dir = temp_path("load");
        fs::create_dir_all(&dir).unwrap();
        let config_file = dir.join("install.conf");
        let example = dir.join("install.conf.example");
        fs::write(
            &example,
            "# comment\nRECEIVER_GAIN=\"\"\nRECEIVER_LAT=\"-23.5\"\n",
        )
        .unwrap();

        let config = load_impl(&config_file, &example, false, "pt", false);

        // Defaults fill the map, not the file; only UI_LANGUAGE is persisted.
        assert_eq!(config.get("RECEIVER_GAIN"), Some("auto"));
        assert_eq!(config.get("RECEIVER_LAT"), Some("-23.5"));
        assert_eq!(
            fs::read_to_string(&config_file).unwrap(),
            "# comment\nRECEIVER_GAIN=\"\"\nRECEIVER_LAT=\"-23.5\"\nUI_LANGUAGE=\"pt\"\n"
        );
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_dry_run_reads_the_example_without_writing() {
        let dir = temp_path("load-dry");
        fs::create_dir_all(&dir).unwrap();
        let config_file = dir.join("install.conf");
        let example = dir.join("install.conf.example");
        fs::write(&example, "NET_RI_PORT=\"30001\"\n").unwrap();

        let config = load_impl(&config_file, &example, true, "en", false);

        assert!(!config_file.exists());
        assert_eq!(config.get("NET_RI_PORT"), Some("30001"));
        assert_eq!(config.get("NET_BI_PORT"), Some("30004,30104"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_keeps_a_language_that_came_from_the_config() {
        let dir = temp_path("load-lang");
        fs::create_dir_all(&dir).unwrap();
        let config_file = dir.join("install.conf");
        let example = dir.join("install.conf.example");
        fs::write(&example, "UI_LANGUAGE=\"pt\"\n").unwrap();
        fs::copy(&example, &config_file).unwrap();

        // language_from_config=true: cfg::load must not rewrite UI_LANGUAGE.
        let _ = load_impl(&config_file, &example, false, "pt", true);

        assert_eq!(
            fs::read_to_string(&config_file).unwrap(),
            "UI_LANGUAGE=\"pt\"\n"
        );
        fs::remove_dir_all(&dir).ok();
    }
}
