//! The `util::confirm` equivalent: bold prompt, `[y/N]`-style suffix from the
//! catalog, and an EOF that reads as a no. Always true under `--yes` or
//! `--dry-run`.

use std::io::{self, BufRead, Write};

use crate::core::{log, run};
use crate::t;

/// Asks for confirmation. The bash prints the prompt with `read -p`, which
/// writes to stderr, so this does too.
pub fn confirm(prompt: &str) -> bool {
    if run::assume_yes() || run::dry_run() {
        return true;
    }
    eprint!("{}{}{} {}", log::bold(), prompt, log::reset(), t!("yes_no"));
    let _ = io::stderr().flush();
    read_answer(&mut io::stdin().lock())
}

/// One answer from the reader; EOF is a no, like `read` failing in the bash.
fn read_answer(reader: &mut dyn BufRead) -> bool {
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) | Err(_) => false,
        Ok(_) => matches_answer(&line),
    }
}

/// The bash `read -r` strips leading and trailing IFS whitespace, then the
/// answer must match `^[SsYy]$` (`yes_chars` from the catalog).
fn matches_answer(line: &str) -> bool {
    let answer = line.trim_matches([' ', '\t', '\n']);
    let yes_chars = t!("yes_chars");
    let mut chars = answer.chars();
    matches!(chars.next(), Some(c) if yes_chars.contains(c)) && chars.next().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn eof_reads_as_no() {
        let mut empty = Cursor::new(Vec::<u8>::new());
        assert!(!read_answer(&mut empty));
    }

    #[test]
    fn answers_parse_like_the_bash_read_and_regex() {
        assert!(matches_answer("y\n"));
        assert!(matches_answer("Y\n"));
        assert!(matches_answer(" y \n"));
        assert!(matches_answer("s"));
        assert!(!matches_answer("n\n"));
        assert!(!matches_answer("yes\n"));
        assert!(!matches_answer(""));
    }

    #[test]
    fn assume_yes_and_dry_run_shortcut_the_prompt() {
        run::set_assume_yes(true);
        assert!(confirm("anything"));
        run::set_assume_yes(false);
        run::set_dry_run(true);
        assert!(confirm("anything"));
        run::set_dry_run(false);
    }
}
