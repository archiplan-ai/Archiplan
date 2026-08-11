//! The scenario grammar
//! (`archi/requirements/world-facts/the-grammar-is-a-named-subset.md`,
//! `archi/requirements/world-facts/scenarios-parse-or-the-check-fails.md`):
//! the `Scenarios` block of a world fact reads as level-three headings and
//! four step keywords.
//!
//! A `### ` heading opens a scenario, and its text is the name — the address
//! a later `archi link` anchors to running code. `Given`, `When`, `Then` and
//! `And` open its step lines, and those four are the whole vocabulary. Every
//! other non-blank line is one located `E_DOC` naming what it found
//! (`archi/decisions/the-grammar-stays-small.md`). `Feature:` and `Scenario:`
//! earn a refusal of their own, because they are the shape this one replaced:
//! the fact's own title is the feature and the heading is the scenario, so
//! each names where it now belongs.
//!
//! The reader is a walk over the lines of the block, so it needs no parser.
//! Every location it reports is the line in the fact file: line `i` of the
//! block is line `block.line + i` of the file.

use super::DocDiagnostic;
use super::world::Block;

/// The step keywords — the whole vocabulary.
const STEPS: [&str; 4] = ["Given", "When", "Then", "And"];

/// The vocabulary as every refusal names it, so the repair is mechanical
/// rather than a guessing game.
const VOCABULARY: &str = "a step line opens with `Given`, `When`, `Then` or `And`";

/// The heading as every refusal names it.
const HEADING: &str = "`### <name>` opens a scenario";

/// What a `Feature:` line earns: the fact's own title is the feature.
const FEATURE: &str = "a `Feature:` line names the feature twice — the fact's own title is the \
                       feature, and a scenario block holds none of its own";

/// What a `Scenario:` line earns: the heading is the scenario.
const SCENARIO: &str = "a `Scenario:` line names the scenario twice — the `### ` heading is the \
                        scenario, and the heading text is the name";

/// What a heading with no text earns: a scenario with no name has no address.
const NAMELESS: &str = "a `### ` heading with no text names no scenario — the heading text is the \
                        name, and the name is the address";

/// One `Scenarios` block, read.
pub struct ScenarioBlock {
    /// The scenarios the grammar accepted, in source order.
    pub scenarios: Vec<Scenario>,
}

/// One scenario — the address a later `archi link` anchors to running code.
pub struct Scenario {
    /// The heading text: the scenario's name.
    pub name: String,
    /// 1-based fact-file line of the heading.
    pub line: usize,
    /// The steps, in source order.
    pub steps: Vec<Step>,
}

/// One step of a scenario.
pub struct Step {
    /// `Given`, `When`, `Then` or `And`.
    pub keyword: String,
    /// The text after the keyword.
    pub text: String,
    /// 1-based fact-file line.
    pub line: usize,
}

/// The scenario the walk is inside: what it has read so far, and whether any
/// refusal touched it.
struct Open {
    scenario: Scenario,
    refused: bool,
}

/// Parse one `Scenarios` block. `block` is the text and its offset as the
/// world-fact reader carried them. Every deviation lands in the diagnostics
/// and the result keeps the scenarios that were sound; `None` means none was.
pub fn parse(block: &Block, file: &str, diags: &mut Vec<DocDiagnostic>) -> Option<ScenarioBlock> {
    // A block with no heading is one refusal at the block, not one per line:
    // the whole block is in the wrong shape, and a refusal on every line of
    // it says one thing many times. The two replaced keywords are reported
    // even there, because each names the repair.
    let headed = block.text.lines().any(|l| heading(l).is_some());
    if !headed {
        diags.push(err(
            file,
            block.line,
            format!("`Scenarios` holds no scenario — {HEADING}, and {VOCABULARY}"),
        ));
    }

    let mut out: Vec<Scenario> = Vec::new();
    let mut open: Option<Open> = None;
    // Every heading the block carried, sound or not: the name is the address,
    // and a name that resolved twice would address nothing.
    let mut named: Vec<(String, usize)> = Vec::new();

    for (i, raw) in block.text.lines().enumerate() {
        // Line `i` of the block is line `block.line + i` of the fact file.
        let at = block.line + i;
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        // `Feature:` and `Scenario:` are the shape this one replaced, so each
        // names where it now belongs instead of what the grammar reads.
        if let Some(message) = replaced(line) {
            diags.push(err(file, at, message));
            refuse(&mut open);
            continue;
        }
        if let Some(name) = heading(line) {
            close(open.take(), &mut out, file, diags);
            let mut this = Open {
                scenario: Scenario {
                    name: name.to_string(),
                    line: at,
                    steps: Vec::new(),
                },
                refused: false,
            };
            if name.is_empty() {
                diags.push(err(file, at, NAMELESS));
                this.refused = true;
            }
            if let Some((_, first)) = named.iter().find(|(n, _)| n == name) {
                diags.push(
                    err(
                        file,
                        at,
                        format!(
                            "two scenarios are named `{name}` — the name is the address, and one \
                             fact holds it once"
                        ),
                    )
                    .with_note("first named here", file, *first),
                );
                this.refused = true;
            }
            named.push((name.to_string(), at));
            open = Some(this);
            continue;
        }
        // A line before the first heading stands under no scenario; a block
        // that holds none was refused whole above.
        let Some(this) = open.as_mut() else {
            if headed {
                diags.push(err(
                    file,
                    at,
                    format!(
                        "`{}` stands under no scenario — {HEADING}, and {VOCABULARY}",
                        opener(line)
                    ),
                ));
            }
            continue;
        };
        let (keyword, text) = split(line);
        if !STEPS.contains(&keyword) {
            diags.push(err(
                file,
                at,
                format!("`{keyword}` opens no step — {VOCABULARY}"),
            ));
            this.refused = true;
            continue;
        }
        if text.is_empty() {
            diags.push(err(
                file,
                at,
                format!(
                    "`{keyword}` states nothing — {VOCABULARY}, and what follows the keyword is \
                     the step"
                ),
            ));
            this.refused = true;
            continue;
        }
        this.scenario.steps.push(Step {
            keyword: keyword.to_string(),
            text: text.to_string(),
            line: at,
        });
    }
    close(open, &mut out, file, diags);
    (!out.is_empty()).then_some(ScenarioBlock { scenarios: out })
}

/// Finish the scenario the walk was inside. A scenario any refusal touched is
/// reported and dropped, and the block keeps what was sound, as every doc
/// primitive does.
fn close(open: Option<Open>, out: &mut Vec<Scenario>, file: &str, diags: &mut Vec<DocDiagnostic>) {
    let Some(Open { scenario, refused }) = open else {
        return;
    };
    if scenario.steps.is_empty() {
        // A heading a refusal already touched carries its error; a second one
        // at the same place says the same thing twice.
        if !refused {
            diags.push(err(
                file,
                scenario.line,
                format!("`{}` holds no step — {VOCABULARY}", scenario.name),
            ));
        }
        return;
    }
    if !refused {
        out.push(scenario);
    }
}

/// The refusal a line of the replaced shape earns; `None` when the line is
/// neither.
fn replaced(line: &str) -> Option<&'static str> {
    if line.starts_with("Feature:") {
        Some(FEATURE)
    } else if line.starts_with("Scenario:") {
        Some(SCENARIO)
    } else {
        None
    }
}

/// The name a level-three heading carries; `None` when the line is no such
/// heading. A deeper heading is not one: `### ` is the whole opener.
///
/// It is the one opener rule. A link resolves an address over the same block
/// ([`crate::links`]) and reads its headings through this function, so the two
/// readers can never part on what opens a scenario, however far their answers
/// stand apart after that.
pub(crate) fn heading(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("###")?;
    let name = rest
        .strip_prefix(' ')
        .or_else(|| rest.is_empty().then_some(rest))?;
    Some(name.trim())
}

/// The first word of a line, and what follows it.
fn split(line: &str) -> (&str, &str) {
    match line.split_once(char::is_whitespace) {
        Some((first, rest)) => (first, rest.trim()),
        None => (line, ""),
    }
}

/// What a line opens with — what a refusal names it by.
fn opener(line: &str) -> &str {
    split(line).0
}

/// A scenario the walk is inside, refused: the line that failed is reported
/// where it sits, and the scenario around it is dropped.
fn refuse(open: &mut Option<Open>) {
    if let Some(this) = open.as_mut() {
        this.refused = true;
    }
}

/// Every refusal is one located `E_DOC`: the code `check` blocks on.
fn err(file: &str, line: usize, message: impl Into<String>) -> DocDiagnostic {
    DocDiagnostic::new("E_DOC", message, file, line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    const FILE: &str = "archi/world/users-open-the-app-on-a-train.md";

    /// The block's first line in the fact file. Every reported line is this
    /// plus the line inside the block, so a test that confused the two would
    /// be off by eleven.
    const AT: usize = 12;

    const BLOCK: &str = "\
### the app opens with no network

Given the device has no network
When the user opens the app
Then the last synced view appears
And the banner names the outage

### the queue drains on reconnect

Given a queued write
When the network returns
Then the write reaches the server
";

    /// The shape this one replaced, whole: a feature line, a scenario line
    /// and no heading at all.
    const REPLACED: &str = "\
Feature: Offline open

  Scenario: the app opens with no network
    Given the device has no network
    Then the last synced view appears
";

    /// A `Feature:` line over a sound scenario — a fact half moved.
    const FEATURE_LINE: &str = "\
Feature: Offline open

### the app opens with no network

Given the device has no network
Then the last synced view appears
";

    /// A `Scenario:` line under the heading that replaced it.
    const SCENARIO_LINE: &str = "\
### the app opens with no network

Scenario: the app opens with no network
Given the device has no network
Then the last synced view appears
";

    const PARAGRAPH: &str = "\
### the app opens with no network

The user opens the app and the last synced view appears.
Given the device has no network
Then the last synced view appears
";

    const BUT: &str = "\
### the app opens with no network

Given the device has no network
But the banner never appears
Then the last synced view appears
";

    const STAR: &str = "\
### the app opens with no network

Given the device has no network
* the banner never appears
Then the last synced view appears
";

    const MISSPELT: &str = "\
### the app opens with no network

Given the device has no network
Whn the user opens the app
Then the last synced view appears
";

    const STEPLESS: &str = "\
### the app opens with no network

### the queue drains on reconnect

Given a queued write
Then the write reaches the server
";

    const PROSE: &str = "\
The carriage drops the network for minutes at a time, so the app opens on
whatever it synced last.
";

    const TWICE: &str = "\
### the app opens with no network

Given the device has no network
Then the last synced view appears

### the app opens with no network

Given a queued write
Then the write reaches the server
";

    /// Parse a block sitting at [`AT`] in the fact file.
    fn read(text: &str) -> (Option<ScenarioBlock>, Vec<DocDiagnostic>) {
        read_at(text, AT)
    }

    /// Parse a block sitting at `line` in the fact file.
    fn read_at(text: &str, line: usize) -> (Option<ScenarioBlock>, Vec<DocDiagnostic>) {
        let block = Block {
            text: text.to_string(),
            line,
        };
        let mut diags = Vec::new();
        let out = parse(&block, FILE, &mut diags);
        (out, diags)
    }

    fn rendered(diags: &[DocDiagnostic]) -> Vec<String> {
        diags.iter().map(ToString::to_string).collect()
    }

    /// The one diagnostic: its code, its line and its message.
    fn only(diags: &[DocDiagnostic]) -> (&'static str, usize, &str) {
        match diags {
            [d] => (d.code, d.line, d.message.as_str()),
            _ => panic!("one diagnostic, got {:#?}", rendered(diags)),
        }
    }

    /// The fact-file line of the block line holding `phrase`.
    fn at(text: &str, phrase: &str) -> usize {
        text.lines()
            .position(|l| l.contains(phrase))
            .unwrap_or_else(|| panic!("no line holds `{phrase}`"))
            + AT
    }

    /// Whether a refusal lists the four step keywords.
    fn lists_the_four(message: &str) -> bool {
        ["`Given`", "`When`", "`Then`", "`And`"]
            .iter()
            .all(|k| message.contains(k))
    }

    fn names(b: &ScenarioBlock) -> Vec<&str> {
        b.scenarios.iter().map(|s| s.name.as_str()).collect()
    }

    fn steps(s: &Scenario) -> Vec<String> {
        s.steps
            .iter()
            .map(|st| format!("{} {}", st.keyword, st.text))
            .collect()
    }

    /// Every block the grammar refuses, for the claims made about all of them
    /// at once.
    const MALFORMED: [&str; 9] = [
        REPLACED,
        FEATURE_LINE,
        SCENARIO_LINE,
        PARAGRAPH,
        BUT,
        STAR,
        MISSPELT,
        STEPLESS,
        PROSE,
    ];

    /// The shape itself: a heading names a scenario, and the four keywords
    /// open its steps (`the-grammar-is-a-named-subset`).
    #[test]
    fn two_headings_and_their_steps_parse_into_two_named_scenarios() {
        let (b, diags) = read(BLOCK);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        let b = b.unwrap();
        assert_eq!(
            names(&b),
            [
                "the app opens with no network",
                "the queue drains on reconnect"
            ]
        );
        let first = &b.scenarios[0];
        assert_eq!(first.line, at(BLOCK, "### the app opens"));
        assert_eq!(
            steps(first),
            [
                "Given the device has no network",
                "When the user opens the app",
                "Then the last synced view appears",
                "And the banner names the outage",
            ]
        );
        // Every step carries its line in the fact file, not in the block.
        assert_eq!(first.steps[0].line, at(BLOCK, "Given the device"));
        assert_eq!(first.steps[3].line, at(BLOCK, "And the banner"));
        let second = &b.scenarios[1];
        assert_eq!(second.line, at(BLOCK, "### the queue drains"));
        assert_eq!(second.steps.len(), 3);
    }

    /// The fact's title is the feature, so a `Feature:` line says it twice.
    #[test]
    fn a_feature_line_names_the_fact_s_title_as_its_place() {
        let (b, diags) = read(FEATURE_LINE);
        let (code, line, message) = only(&diags);
        assert_eq!((code, line), ("E_DOC", at(FEATURE_LINE, "Feature:")));
        assert!(message.contains("`Feature:`"), "{message}");
        assert!(message.contains("title"), "the place: {message}");
        // The line stands over no scenario, so the scenario under it stands.
        assert_eq!(names(&b.unwrap()), ["the app opens with no network"]);
    }

    /// The heading is the scenario, so a `Scenario:` line says it twice.
    #[test]
    fn a_scenario_line_names_the_heading_as_its_place() {
        let (b, diags) = read(SCENARIO_LINE);
        let (code, line, message) = only(&diags);
        assert_eq!((code, line), ("E_DOC", at(SCENARIO_LINE, "Scenario:")));
        assert!(message.contains("`Scenario:`"), "{message}");
        assert!(message.contains("heading"), "the place: {message}");
        // The refusal touched the scenario, so the scenario is dropped.
        assert!(b.is_none());
    }

    /// Free prose in place of a step is a paragraph wearing a costume, and
    /// the refusal names the vocabulary it is not written in.
    #[test]
    fn prose_under_a_heading_names_the_four_step_keywords() {
        let (b, diags) = read(PARAGRAPH);
        let (code, line, message) = only(&diags);
        assert_eq!((code, line), ("E_DOC", at(PARAGRAPH, "The user opens")));
        assert!(message.contains("`The`"), "what it found: {message}");
        assert!(lists_the_four(message), "{message}");
        assert!(b.is_none());
    }

    /// The four keywords are the whole vocabulary: the keywords Gherkin holds
    /// beyond them read as any other unknown opener does.
    #[test]
    fn but_and_star_are_refused_like_any_other_opener() {
        let mut shapes = Vec::new();
        for (opener, text, marker) in [
            ("But", BUT, "But the banner"),
            ("*", STAR, "* the banner"),
            ("Whn", MISSPELT, "Whn the user"),
        ] {
            let (b, diags) = read(text);
            let (code, line, message) = only(&diags);
            assert_eq!((code, line), ("E_DOC", at(text, marker)), "on `{opener}`");
            assert!(message.contains(&format!("`{opener}`")), "{message}");
            assert!(lists_the_four(message), "{message}");
            assert!(b.is_none(), "on `{opener}`");
            shapes.push(message.replace(opener, "<opener>"));
        }
        // One refusal covers every opener: `But` and `*` earn no error of
        // their own.
        assert_eq!(shapes[0], shapes[1]);
        assert_eq!(shapes[1], shapes[2]);
    }

    /// A heading over nothing states no scenario.
    #[test]
    fn a_heading_with_no_step_is_a_located_error() {
        let (b, diags) = read(STEPLESS);
        let (code, line, message) = only(&diags);
        assert_eq!((code, line), ("E_DOC", at(STEPLESS, "### the app opens")));
        assert!(lists_the_four(message), "{message}");
        // The stepless scenario is dropped; the sound one beside it stands.
        assert_eq!(names(&b.unwrap()), ["the queue drains on reconnect"]);
    }

    /// A block with no heading is one error at the block: the whole block is
    /// in the wrong shape, and a refusal on every line says one thing many
    /// times.
    #[test]
    fn a_block_with_no_heading_is_a_located_error() {
        for text in [PROSE, "\n\n"] {
            let (b, diags) = read(text);
            let (code, line, message) = only(&diags);
            assert_eq!((code, line), ("E_DOC", AT));
            assert!(lists_the_four(message), "{message}");
            assert!(message.contains("### "), "the opener: {message}");
            assert!(b.is_none());
        }
    }

    /// The shape this one replaced, whole: the block-level refusal, and the
    /// two lines that name where each half now belongs.
    #[test]
    fn the_replaced_shape_is_refused_line_by_line() {
        let (b, diags) = read(REPLACED);
        assert!(b.is_none());
        let lines: Vec<usize> = diags.iter().map(|d| d.line).collect();
        assert_eq!(
            lines,
            [
                AT,
                at(REPLACED, "Feature:"),
                at(REPLACED, "Scenario: the app opens")
            ]
        );
        assert!(diags[1].message.contains("title"), "{}", diags[1].message);
        assert!(diags[2].message.contains("heading"), "{}", diags[2].message);
    }

    /// The name is the address, and an ambiguous address names nothing.
    #[test]
    fn two_headings_sharing_a_name_name_both_lines() {
        let (b, diags) = read(TWICE);
        let [d] = &diags[..] else {
            panic!("one diagnostic, got {:#?}", rendered(&diags));
        };
        // The second heading carries the error; the first is the note.
        assert_eq!((d.code, d.line), ("E_DOC", AT + 5));
        assert!(d.message.contains("the app opens with no network"), "{d}");
        let note = d.note.as_ref().expect("the first line");
        assert_eq!((note.file.as_str(), note.line), (FILE, AT));
        // The first scenario is unambiguous in itself and stands.
        assert_eq!(names(&b.unwrap()), ["the app opens with no network"]);
    }

    /// Every location is the line in the fact file, computed from the block's
    /// offset (`scenarios-parse-or-the-check-fails`): the same block read at
    /// two offsets reports the same lines, moved by the difference.
    #[test]
    fn every_error_reports_its_line_in_the_fact_file() {
        for text in MALFORMED {
            let (_, here) = read_at(text, AT);
            let (_, further) = read_at(text, AT + 100);
            let moved: Vec<usize> = here.iter().map(|d| d.line + 100).collect();
            let there: Vec<usize> = further.iter().map(|d| d.line).collect();
            assert_eq!(moved, there, "on {text}");
            // Every line falls inside the block it was read from.
            let last = AT + text.lines().count();
            assert!(
                here.iter().all(|d| (AT..last).contains(&d.line)),
                "on {text}"
            );
        }
    }

    /// Every refusal carries `E_DOC` — the code the docs pass
    /// ([`super::super::check`]) collects and `archi check` exits non-zero
    /// on. A malformed block blocks; it is never a finding.
    #[test]
    fn a_malformed_block_is_the_code_the_check_blocks_on() {
        for text in MALFORMED {
            let (_, diags) = read(text);
            assert!(!diags.is_empty(), "on {text}");
            assert!(diags.iter().all(|d| d.code == "E_DOC"), "on {text}");
        }
    }

    /// Four keywords and a markdown heading need no parser: nothing in the
    /// workspace depends on the `gherkin` crate any more.
    #[test]
    fn no_cargo_toml_of_the_workspace_names_the_gherkin_crate() {
        let root = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
        let mut stack = vec![root];
        let mut named: Vec<String> = Vec::new();
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).unwrap().flatten() {
                let path = entry.path();
                let name = entry.file_name();
                if path.is_dir() {
                    if name != "target" && name != ".git" {
                        stack.push(path);
                    }
                } else if name == "Cargo.toml"
                    && fs::read_to_string(&path).unwrap().contains("gherkin")
                {
                    named.push(path.display().to_string());
                }
            }
        }
        assert_eq!(named, Vec::<String>::new());
    }
}
