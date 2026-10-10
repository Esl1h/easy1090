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

/// The repo's install.conf.example, compiled in so a bare release binary can
/// create its first config. It is the same file, not a copy: the one on disk
/// still wins when present.
pub const EXAMPLE_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../install.conf.example"
));

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
    /// Builds a config from an explicit map; used by the step tests to pin
    /// exact values without touching the filesystem.
    pub fn from_map(values: BTreeMap<String, String>) -> Config {
        Config { values }
    }

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
    let mut reading_example = false;
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
            reading_example = true;
        } else {
            // Creating it from the example is harmless and is what every
            // first run needs, so it is not worth a prompt the user can trip
            // over. Without the example next to the binary, the compiled-in
            // copy stands in for it.
            let created = if example_file.exists() {
                fs::copy(example_file, config_file).map(|_| ())
            } else {
                fs::write(config_file, EXAMPLE_CONFIG)
            };
            if let Err(error) = created {
                crate::core::util::die(&error.to_string());
            }
            log::info(&t!("cfg_created", config_file.display()));
        }
    }

    let content = match fs::read_to_string(&source) {
        Ok(content) => content,
        Err(_) if reading_example => EXAMPLE_CONFIG.to_string(),
        Err(_) => String::new(),
    };
    let mut config = Config {
        values: parse_str(&content),
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
    // sed -i writes a temp file in the same directory and renames it over
    // the target, so what matters is write permission on the directory, not
    // on the (possibly root-owned) file. Writing in place with fs::write
    // would need write permission on the file itself and fail where the
    // bash succeeds: an install.conf created by root, or read-only, stays
    // persistable as long as the directory is writable.
    let mut temp_name = file.as_os_str().to_os_string();
    temp_name.push(".easy1090.tmp");
    let temp = std::path::PathBuf::from(temp_name);
    let result = fs::metadata(file)
        .map(|meta| meta.permissions())
        .and_then(|permissions| {
            fs::write(&temp, &out).and_then(|_| fs::set_permissions(&temp, permissions))
        })
        .or_else(|_| fs::write(&temp, &out))
        .and_then(|_| fs::rename(&temp, file));
    if let Err(error) = result {
        let _ = fs::remove_file(&temp);
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

/// Receiver position is the only thing that cannot be guessed; prompts when
/// missing, mirroring cfg::require_position. Under `--dry-run` the prompt is
/// skipped and the position read as 0.0, which is what the bash does so the
/// preview can walk the readsb config step.
pub fn require_position(config_file: &Path, config: &mut Config) {
    require_position_impl(config_file, config, run::dry_run());
}

// The dry-run flag is a parameter so the test does not flip the process-wide
// DRY_RUN, which would race with every other test running in parallel.
fn require_position_impl(config_file: &Path, config: &mut Config, dry_run: bool) {
    let lat = config.get("RECEIVER_LAT").unwrap_or("").to_string();
    let lon = config.get("RECEIVER_LON").unwrap_or("").to_string();

    if !lat.is_empty() && !lon.is_empty() {
        validate_position(&lat, &lon);
        log::debug(&format!("position: {lat}, {lon}"));
        return;
    }

    if dry_run {
        log::warn(&t!("pos_dry"));
        config.set("RECEIVER_LAT", "0.0");
        config.set("RECEIVER_LON", "0.0");
        return;
    }

    if run::assume_yes() {
        crate::core::util::die(&t!("pos_required_yes", config_file.display()));
    }

    log::info(&t!("pos_intro"));

    // Nobody knows their coordinates by heart, so point at the tools instead
    // of leaving a bare prompt on screen.
    eprint!(
        "\n{}\n{}\n{}\n{}\n\n{}\n\n",
        t!("pos_help_title"),
        t!("pos_help_osm"),
        t!("pos_help_gmaps"),
        t!("pos_help_latlong"),
        t!("pos_help_tip"),
    );
    let _ = std::io::stderr().flush();

    // The bash `read -r -p`: prompt to stderr, IFS whitespace trimmed. The
    // two prompts overwrite whatever the config held, a partially filled
    // position is re-asked in full.
    let new_lat = prompt_value(&t!("pos_lat"));
    let new_lon = prompt_value(&t!("pos_lon"));

    if new_lat.is_empty() || new_lon.is_empty() {
        crate::core::util::die(&t!("pos_required"));
    }
    validate_position(&new_lat, &new_lon);

    config.set("RECEIVER_LAT", &new_lat);
    config.set("RECEIVER_LON", &new_lon);
    persist(config_file, "RECEIVER_LAT", &new_lat);
    persist(config_file, "RECEIVER_LON", &new_lon);
    log::success(&t!("pos_saved", config_file.display()));
}

/// One `read -r -p` answer; EOF reads as empty, which the caller rejects.
fn prompt_value(prompt: &str) -> String {
    eprint!("{prompt}");
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(0) | Err(_) => String::new(),
        Ok(_) => line.trim().to_string(),
    }
}

/// Coordinates end up on the readsb command line in /etc/default/readsb. A
/// value pasted with a comma ("23,58") would parse as the range of latitude
/// 23 and leave the rest for systemctl to trip over. The regex and the awk
/// range check of the bash, with the same abort message.
pub fn validate_position(lat: &str, lon: &str) {
    if !is_decimal_degrees(lat) || !in_range(lat, -90.0, 90.0) {
        crate::core::util::die(&t!("pos_invalid", lat, lon));
    }
    if !is_decimal_degrees(lon) || !in_range(lon, -180.0, 180.0) {
        crate::core::util::die(&t!("pos_invalid", lat, lon));
    }
}

/// The `^-?[0-9]+(\.[0-9]+)?$` shape: optional minus, one or more digits,
/// then optionally a dot with at least one digit.
fn is_decimal_degrees(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut i = 0;
    if bytes.first() == Some(&b'-') {
        i += 1;
    }
    let int_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == int_start {
        return false;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let frac_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == frac_start {
            return false;
        }
    }
    i == bytes.len()
}

/// awk's `n >= min && n <= max` over the parsed value. The regex above
/// already guarantees a plain decimal number, so the f64 parse cannot fail
/// and never yields NaN, which awk would reject.
fn in_range(value: &str, min: f64, max: f64) -> bool {
    value
        .parse::<f64>()
        .map(|number| number >= min && number <= max)
        .unwrap_or(false)
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
    fn persist_rewrites_a_read_only_file_like_sed_i() {
        // sed -i swaps the file by rename, so writing depends on the
        // directory, not on the file. A read-only install.conf (or one owned
        // by root, same effect in a container) must stay persistable; the
        // regression this test pins is fs::write in place, which fails with
        // EACCES here while the bash succeeds.
        use std::os::unix::fs::PermissionsExt;
        let path = temp_path("readonly");
        fs::write(&path, "UI_LANGUAGE=\"\"\nRECEIVER_LAT=\"1\"\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o444)).unwrap();
        persist_impl(&path, "UI_LANGUAGE", "pt", false);
        assert!(fs::read_to_string(&path)
            .unwrap()
            .starts_with("UI_LANGUAGE=\"pt\"\n"));
        // The sed -i semantics also preserve the mode bits.
        let mode = fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o444);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
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
    fn load_without_an_example_on_disk_uses_the_compiled_in_copy() {
        let dir = temp_path("load-embedded");
        fs::create_dir_all(&dir).unwrap();
        let config_file = dir.join("install.conf");
        let missing_example = dir.join("install.conf.example");

        let config = load_impl(&config_file, &missing_example, false, "en", false);

        // The example ships UI_LANGUAGE="", which persistence rewrites in
        // place; everything else is the example byte for byte.
        assert_eq!(
            fs::read_to_string(&config_file).unwrap(),
            EXAMPLE_CONFIG.replace("UI_LANGUAGE=\"\"", "UI_LANGUAGE=\"en\"")
        );
        assert_eq!(config.get("NET_BI_PORT"), Some("30004,30104"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_dry_run_without_an_example_on_disk_reads_the_compiled_in_copy() {
        let dir = temp_path("load-dry-embedded");
        fs::create_dir_all(&dir).unwrap();
        let config_file = dir.join("install.conf");
        let missing_example = dir.join("install.conf.example");

        let config = load_impl(&config_file, &missing_example, true, "en", false);

        assert!(!config_file.exists());
        assert!(config
            .get("TAR1090_INSTALLER_SHA256")
            .is_some_and(|pin| pin.len() == 64));
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

    #[test]
    fn decimal_degrees_match_the_bash_regex() {
        for ok in ["0", "-0", "0.0", "-23.58", "180", "-180", "12.345"] {
            assert!(is_decimal_degrees(ok), "{ok:?} must match");
        }
        for bad in [
            "", "-", ".", ".5", "-.5", "1.", "23,58", "+3", "1e2", "1 2", " 1", "1 ",
        ] {
            assert!(!is_decimal_degrees(bad), "{bad:?} must not match");
        }
    }

    #[test]
    fn ranges_follow_the_awk_bounds() {
        assert!(in_range("0", -90.0, 90.0));
        assert!(in_range("-90", -90.0, 90.0));
        assert!(in_range("90", -90.0, 90.0));
        assert!(!in_range("-90.1", -90.0, 90.0));
        assert!(!in_range("90.1", -90.0, 90.0));
        assert!(in_range("180", -180.0, 180.0));
        assert!(!in_range("180.5", -180.0, 180.0));
        // The regex keeps the parse from ever seeing these.
        assert!(!in_range("not-a-number", 0.0, 1.0));
    }

    #[test]
    fn require_position_dry_run_reads_zero_without_prompting() {
        let config_file = PathBuf::from("/tmp/easy1090-cfg-test-dry/install.conf");
        let mut config = Config::default();
        apply_defaults(&mut config);

        require_position_impl(&config_file, &mut config, true);

        assert_eq!(config.get("RECEIVER_LAT"), Some("0.0"));
        assert_eq!(config.get("RECEIVER_LON"), Some("0.0"));
    }
}
