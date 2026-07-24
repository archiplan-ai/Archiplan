//! `archi batch` — many verbs, one invocation, zero drift.
//!
//! Commands arrive on stdin, one per line; each line is split shell-style
//! and re-executed as the running binary itself, so every verb the CLI
//! grows is batchable the day it lands — there is no second dispatch
//! surface to hardcode or to rot. Execution is fail-fast: the first
//! non-zero line stops the batch with its line number; what ran, ran —
//! what follows, never started. The exit code tells the truth.

use std::io::Read;
use std::process::{Command, ExitCode};

/// Split one batch line into argv: whitespace separates, `'…'` is
/// literal, `"…"` understands `\"`, `\\`, `\n` and `\t` — the escape is
/// how a one-line command carries a multiline value.
pub fn split_line(line: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut started = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            c if c.is_whitespace() => {
                if started {
                    out.push(std::mem::take(&mut cur));
                    started = false;
                }
            }
            '\'' => {
                started = true;
                loop {
                    match chars.next() {
                        Some('\'') => break,
                        Some(c) => cur.push(c),
                        None => return Err("unterminated `'` quote".into()),
                    }
                }
            }
            '"' => {
                started = true;
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => match chars.next() {
                            Some('"') => cur.push('"'),
                            Some('\\') => cur.push('\\'),
                            Some('n') => cur.push('\n'),
                            Some('t') => cur.push('\t'),
                            Some(other) => {
                                cur.push('\\');
                                cur.push(other);
                            }
                            None => return Err("unterminated `\"` quote".into()),
                        },
                        Some(c) => cur.push(c),
                        None => return Err("unterminated `\"` quote".into()),
                    }
                }
            }
            c => {
                started = true;
                cur.push(c);
            }
        }
    }
    if started {
        out.push(cur);
    }
    Ok(out)
}

/// Run the batch: lines from stdin, blank lines and `#` comments skipped,
/// trailing `\r` tolerated, `--project` forwarded to every line that does
/// not carry its own. Fail-fast with the offending line named.
pub fn run(project: Option<&str>) -> ExitCode {
    let mut text = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut text) {
        eprintln!("archi: batch reads its commands from stdin: {e}");
        return ExitCode::from(2);
    }
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("archi: cannot re-invoke itself: {e}");
            return ExitCode::from(1);
        }
    };
    let mut done = 0usize;
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim_end_matches('\r').trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut argv = match split_line(line) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("archi: batch line {}: {e}", i + 1);
                return ExitCode::from(2);
            }
        };
        if argv.first().map(String::as_str) == Some("batch") {
            eprintln!("archi: batch line {}: batch does not nest", i + 1);
            return ExitCode::from(2);
        }
        if let Some(p) = project {
            if !argv.iter().any(|a| a == "--project") {
                argv.push("--project".to_string());
                argv.push(p.to_string());
            }
        }
        println!("[{}] {line}", done + 1);
        match Command::new(&exe).args(&argv).status() {
            Ok(status) if status.success() => done += 1,
            Ok(status) => {
                eprintln!(
                    "archi: batch stopped at line {} (`{line}`) — {} command{} ran before it",
                    i + 1,
                    done,
                    if done == 1 { "" } else { "s" }
                );
                return ExitCode::from(status.code().unwrap_or(1).clamp(0, 255) as u8);
            }
            Err(e) => {
                eprintln!("archi: batch line {}: cannot run: {e}", i + 1);
                return ExitCode::from(1);
            }
        }
    }
    println!("batch: {done} command{} done", if done == 1 { "" } else { "s" });
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::split_line;

    #[test]
    fn splits_words_and_both_quote_kinds() {
        assert_eq!(
            split_line("plan task desc t1 'persist the rows'").unwrap(),
            vec!["plan", "task", "desc", "t1", "persist the rows"]
        );
        assert_eq!(
            split_line("plan problem \"a tiny store\"").unwrap(),
            vec!["plan", "problem", "a tiny store"]
        );
    }

    #[test]
    fn double_quotes_carry_escapes_single_stay_literal() {
        assert_eq!(
            split_line(r#"task desc t1 "line one\nline two — a \"quote\"""#).unwrap(),
            vec!["task", "desc", "t1", "line one\nline two — a \"quote\""]
        );
        assert_eq!(
            split_line(r"x 'no \n escape'").unwrap(),
            vec!["x", r"no \n escape"]
        );
    }

    #[test]
    fn empty_quotes_make_empty_args_and_unterminated_refuse() {
        assert_eq!(split_line("a '' b").unwrap(), vec!["a", "", "b"]);
        assert!(split_line("a 'open").is_err());
        assert!(split_line("a \"open").is_err());
    }
}
