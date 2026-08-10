//! The world-fact record (`archi/requirements/world-facts/a-world-fact-carries-its-scenarios.md`,
//! `archi/requirements/world-facts/the-header-points-three-ways.md`,
//! `archi/requirements/world-facts/a-source-may-lie-outside-the-tree.md`):
//! one file under `archi/world/` holds one condition outside the system, what
//! would make that condition false, and the scenarios it dictates. Parsing is
//! best-effort as it is in [`super::schema`] — every deviation lands in the
//! diagnostics and the record keeps what was sound.
//!
//! The three frontmatter lists are the whole header. `sources` resolves here,
//! because its two entry forms are the record's own rule; `covers` and `uses`
//! are carried with the line they sit on and resolve against the model and the
//! other facts in the compiler. The `Scenarios` block is carried as text and
//! an offset — the grammar parses it, not this reader.

use std::path::Path;

use super::DocDiagnostic;
use super::md::{Heading, MdDoc};
use super::schema::{frontmatter, list, name_checks};

/// A run of prose lifted out of the file: its text, with the blank lines the
/// reader dropped put back, and the 1-based file line its first line sits on.
/// A location inside `text` maps onto the file by addition.
pub struct Block {
    /// The lines of the block, newline-terminated.
    pub text: String,
    /// 1-based file line of the block's first line.
    pub line: usize,
}

/// One `sources` entry — the two forms
/// (`archi/requirements/world-facts/a-source-may-lie-outside-the-tree.md`).
pub enum Source {
    /// No scheme: a path from the project root, resolved on disk.
    Path(String),
    /// A `scheme:` prefix: an external locator, kept verbatim and never
    /// touched on the filesystem.
    Locator(String),
}

/// One world fact — the file under `archi/world/`.
pub struct WorldDoc {
    /// The slug (= filename).
    pub slug: String,
    /// Project-relative path.
    pub file: String,
    /// 1-based line of the name — the fact in one line.
    pub line: usize,
    /// Model elements the fact conditions, with the field's line; `None` when
    /// invalid. Resolution against the model is the compiler's.
    pub covers: Option<(Vec<String>, usize)>,
    /// The material the fact rests on, with the field's line; `None` when
    /// invalid.
    pub sources: Option<(Vec<Source>, usize)>,
    /// World facts this one holds only while they hold, with the field's
    /// line; `None` when invalid. Resolution is the compiler's.
    pub uses: Option<(Vec<String>, usize)>,
    /// The conditioning paragraph; `None` when absent (already reported).
    pub condition: Option<Block>,
    /// What would make the fact false; `None` when absent or empty.
    pub killer: Option<Block>,
    /// The `Scenarios` block, unparsed; `None` when absent or empty.
    pub scenarios: Option<Block>,
    /// The optional `Open questions` section; absence and emptiness are both
    /// legal states.
    pub open_questions: Option<Block>,
}

impl WorldDoc {
    /// Whether the fact rests on no recorded material — the hypothesis state
    /// (`archi/requirements/world-facts/an-ungrounded-fact-says-so.md`). An
    /// unsound `sources` field is no claim of either state.
    pub fn ungrounded(&self) -> bool {
        self.sources.as_ref().is_some_and(|(v, _)| v.is_empty())
    }
}

/// The sections a world fact holds, in the order it holds them. Any other
/// heading is the author's own: kept, reported by nothing.
const SECTIONS: [&str; 3] = ["What kills this", "Scenarios", "Open questions"];

/// Parse a world-fact document. `root` is the project root the schemeless
/// `sources` entries resolve against.
pub fn parse(
    doc: &MdDoc,
    file: &str,
    stem: &str,
    root: &Path,
    diags: &mut Vec<DocDiagnostic>,
) -> WorldDoc {
    // The name is the fact in one line, and the conditioning paragraph
    // follows it — the same shape every doc primitive wears.
    name_checks(doc, file, stem, "a world fact", diags);
    let fm = frontmatter(
        doc,
        file,
        &["covers", "sources", "uses"],
        "covers, sources, uses",
        diags,
    );
    let covers = list(fm, "covers", file, diags);
    let sources = list(fm, "sources", file, diags)
        .map(|(entries, line)| (sources_of(&entries, root, file, line, diags), line));
    let uses = list(fm, "uses", file, diags);

    let [killer, scenarios, open_questions] = sections(doc, file, diags);
    WorldDoc {
        slug: stem.to_string(),
        file: file.to_string(),
        line: doc.name_line,
        covers,
        sources,
        uses,
        condition: block(&doc.summary),
        killer: required(
            killer,
            "What kills this",
            "what would make the fact false",
            doc,
            file,
            diags,
        ),
        scenarios: required(
            scenarios,
            "Scenarios",
            "one scenario or more",
            doc,
            file,
            diags,
        ),
        // Absence and emptiness are both legal here: what is not known yet is
        // recorded when it is known.
        open_questions: open_questions.and_then(|h| block(&h.content)),
    }
}

/// The recognized sections in canonical order, each `None` when the file does
/// not hold it. A section out of order or a second copy of one is reported
/// here; any other heading is the author's own and passes untouched.
fn sections<'a>(
    doc: &'a MdDoc,
    file: &str,
    diags: &mut Vec<DocDiagnostic>,
) -> [Option<&'a Heading>; 3] {
    let mut found: [Option<&Heading>; 3] = [None, None, None];
    let mut reached = 0;
    for h in &doc.headings {
        let Some(i) = (h.level == 2)
            .then(|| SECTIONS.iter().position(|s| *s == h.text))
            .flatten()
        else {
            continue;
        };
        if found[i].is_some() {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                format!("`{}` reappears — a world fact holds it once", h.text),
                file,
                h.line,
            ));
            continue;
        }
        if i < reached {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                "a world fact runs `What kills this`, then `Scenarios`, then the optional \
                 `Open questions`",
                file,
                h.line,
            ));
        }
        reached = reached.max(i + 1);
        found[i] = Some(h);
    }
    found
}

/// A section the schema requires: its absence is reported at the name, its
/// emptiness at its own heading, and neither state carries prose.
fn required(
    section: Option<&Heading>,
    heading: &str,
    holds: &str,
    doc: &MdDoc,
    file: &str,
    diags: &mut Vec<DocDiagnostic>,
) -> Option<Block> {
    let Some(h) = section else {
        diags.push(DocDiagnostic::new(
            "E_DOC",
            format!("a world fact holds `## {heading}`: {holds}"),
            file,
            doc.name_line,
        ));
        return None;
    };
    let prose = block(&h.content);
    if prose.is_none() {
        diags.push(DocDiagnostic::new(
            "E_DOC",
            format!("`{heading}` holds nothing: {holds}"),
            file,
            h.line,
        ));
    }
    prose
}

/// The `sources` entries in their two forms. A schemeless entry resolves on
/// disk; a schemed one is kept verbatim and the filesystem never sees it; an
/// entry that is neither is reported and carried by neither.
fn sources_of(
    entries: &[String],
    root: &Path,
    file: &str,
    line: usize,
    diags: &mut Vec<DocDiagnostic>,
) -> Vec<Source> {
    let mut out = Vec::new();
    for entry in entries {
        match source_entry(entry) {
            None => diags.push(DocDiagnostic::new(
                "E_DOC",
                format!(
                    "`{entry}` is neither `sources` form — an entry with no scheme is a path \
                     from the project root, and a schemed one is `<scheme>:<target>`, the \
                     scheme letter-first and the target present"
                ),
                file,
                line,
            )),
            Some(Source::Path(p)) => {
                if !root.join(&p).exists() {
                    diags.push(DocDiagnostic::new(
                        "E_DOC",
                        format!(
                            "`sources` names no `{p}` in the tree — an entry with no scheme is \
                             a path from the project root; an external locator carries a scheme"
                        ),
                        file,
                        line,
                    ));
                }
                out.push(Source::Path(p));
            }
            Some(locator) => out.push(locator),
        }
    }
    out
}

/// The block a heading's content forms; `None` when the section is empty.
fn block(content: &[(usize, String)]) -> Option<Block> {
    let (first, _) = *content.first()?;
    let mut text = String::new();
    let mut next = first;
    for (line, raw) in content {
        // The reader drops blank lines; put them back so line `first + i` of
        // the text is line `first + i` of the file.
        for _ in next..*line {
            text.push('\n');
        }
        text.push_str(raw);
        text.push('\n');
        next = line + 1;
    }
    Some(Block { text, line: first })
}

/// How one `sources` entry reads; `None` when it is neither form.
fn source_entry(entry: &str) -> Option<Source> {
    let Some((scheme, target)) = entry.split_once(':') else {
        return Some(Source::Path(entry.to_string()));
    };
    let well_formed = !target.is_empty()
        && !target.contains(char::is_whitespace)
        && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    well_formed.then(|| Source::Locator(entry.to_string()))
}

#[cfg(test)]
mod tests {
    use super::super::md;
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    /// The three keys, all empty — the state a minted skeleton wears.
    const HEADER: &str = "covers: []\nsources: []\nuses: []\n";

    const BODY: &str = "\
# Users open the app on a train

The carriage drops the network for minutes at a time, so a call that must reach
the server fails for a reason the user cannot fix.

## What kills this

Trackside coverage that never drops.

## Scenarios

Feature: Offline open
  Scenario: the app opens with no network
    Given the device has no network
    When the user opens the app
    Then the last synced view appears
";

    const SLUG: &str = "users-open-the-app-on-a-train";

    fn fact(header: &str, body: &str) -> String {
        format!("---\n{header}---\n\n{body}")
    }

    fn temp_root() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-world-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Read one fact the way discovery does: structure first, then schema.
    fn read(root: &Path, text: &str) -> (Option<WorldDoc>, Vec<DocDiagnostic>) {
        let file = format!("archi/world/{SLUG}.md");
        let mut diags = Vec::new();
        match md::parse(text) {
            Ok(doc) => {
                let w = parse(&doc, &file, SLUG, root, &mut diags);
                (Some(w), diags)
            }
            Err(e) => {
                diags.push(DocDiagnostic::new("E_DOC", e.message, &file, e.line));
                (None, diags)
            }
        }
    }

    fn rendered(diags: &[DocDiagnostic]) -> Vec<String> {
        diags.iter().map(ToString::to_string).collect()
    }

    /// The text without the lines that hold any of the phrases — how a slot
    /// is emptied out of the whole fact.
    fn without(text: &str, phrases: &[&str]) -> String {
        text.lines()
            .filter(|l| !phrases.iter().any(|p| l.contains(p)))
            .map(|l| format!("{l}\n"))
            .collect()
    }

    /// The 1-based line the phrase sits on.
    fn line_of(text: &str, phrase: &str) -> usize {
        text.lines()
            .position(|l| l.contains(phrase))
            .unwrap_or_else(|| panic!("no line holds `{phrase}`"))
            + 1
    }

    /// The one diagnostic, with its code and line.
    fn only(diags: &[DocDiagnostic]) -> (&'static str, usize) {
        match diags {
            [d] => (d.code, d.line),
            _ => panic!("one diagnostic, got {:#?}", rendered(diags)),
        }
    }

    fn forms(w: &WorldDoc) -> Vec<String> {
        w.sources
            .as_ref()
            .expect("sources parsed")
            .0
            .iter()
            .map(|s| match s {
                Source::Path(p) => format!("path {p}"),
                Source::Locator(l) => format!("locator {l}"),
            })
            .collect()
    }

    #[test]
    fn a_whole_fact_parses_clean() {
        let root = temp_root();
        let text = fact(HEADER, BODY);
        let (w, diags) = read(&root, &text);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        let w = w.unwrap();
        assert_eq!(w.slug, SLUG);
        assert_eq!(w.file, format!("archi/world/{SLUG}.md"));
        assert_eq!(w.line, line_of(&text, "# Users open the app"));
        assert_eq!(w.covers.as_ref().unwrap().0, Vec::<String>::new());
        assert_eq!(w.uses.as_ref().unwrap().0, Vec::<String>::new());
        assert!(w.condition.as_ref().unwrap().text.contains("carriage"));
        assert!(w.killer.as_ref().unwrap().text.contains("Trackside"));
        // The scenario block is carried, not parsed: its text and the line it
        // opens on, so the grammar can map a location back onto the file.
        let block = w.scenarios.as_ref().unwrap();
        assert_eq!(block.line, line_of(&text, "Feature: Offline open"));
        assert!(block.text.starts_with("Feature: Offline open\n"));
        assert!(block.text.contains("    Then the last synced view appears\n"));
        // The blank line inside the block keeps the following lines in place.
        assert_eq!(
            block.line + block.text.lines().count() - 1,
            line_of(&text, "Then the last synced view appears")
        );
        assert!(w.open_questions.is_none());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_missing_slots_are_located_errors() {
        let root = temp_root();
        // No name at all: the structural reader locates it.
        let (w, diags) = read(&root, &fact(HEADER, "prose with no name\n"));
        assert!(w.is_none());
        assert_eq!(only(&diags).0, "E_DOC");

        // No conditioning paragraph.
        let text = fact(
            HEADER,
            &without(BODY, &["The carriage drops", "the server fails"]),
        );
        let (_, diags) = read(&root, &text);
        assert_eq!(
            only(&diags),
            ("E_DOC", line_of(&text, "# Users open the app"))
        );

        // No killer.
        let text = fact(
            HEADER,
            &without(BODY, &["## What kills this", "Trackside coverage"]),
        );
        let (_, diags) = read(&root, &text);
        assert_eq!(
            only(&diags),
            ("E_DOC", line_of(&text, "# Users open the app"))
        );

        // A killer heading over nothing states nothing either.
        let text = fact(HEADER, &without(BODY, &["Trackside coverage"]));
        let (w, diags) = read(&root, &text);
        assert_eq!(
            only(&diags),
            ("E_DOC", line_of(&text, "## What kills this"))
        );
        assert!(w.unwrap().killer.is_none());

        // An empty `Scenarios`.
        let text = fact(
            HEADER,
            &without(
                BODY,
                &["Feature:", "Scenario:", "Given ", "When ", "Then "],
            ),
        );
        let (w, diags) = read(&root, &text);
        assert_eq!(only(&diags), ("E_DOC", line_of(&text, "## Scenarios")));
        assert!(w.unwrap().scenarios.is_none());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_sections_run_in_order() {
        let root = temp_root();
        // The two headings swapped, each over the other's prose.
        let text = fact(
            HEADER,
            &BODY
                .replace("## What kills this", "## SWAP")
                .replace("## Scenarios", "## What kills this")
                .replace("## SWAP", "## Scenarios"),
        );
        let (_, diags) = read(&root, &text);
        // The second section is the one out of order, and it says so at its
        // own line.
        assert_eq!(
            only(&diags),
            ("E_DOC", line_of(&text, "## What kills this"))
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn open_questions_is_optional_and_may_be_empty() {
        let root = temp_root();
        // Absent.
        let (_, diags) = read(&root, &fact(HEADER, BODY));
        assert_eq!(rendered(&diags), Vec::<String>::new());
        // Present and empty.
        let (w, diags) = read(&root, &fact(HEADER, &format!("{BODY}\n## Open questions\n")));
        assert_eq!(rendered(&diags), Vec::<String>::new());
        assert!(w.unwrap().open_questions.is_none());
        // Present and filled.
        let (w, diags) = read(
            &root,
            &fact(HEADER, &format!("{BODY}\n## Open questions\n\nHow long is a tunnel?\n")),
        );
        assert_eq!(rendered(&diags), Vec::<String>::new());
        assert!(
            w.unwrap()
                .open_questions
                .as_ref()
                .unwrap()
                .text
                .contains("tunnel")
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_unknown_heading_is_kept_and_raises_nothing() {
        let root = temp_root();
        let text = fact(
            HEADER,
            &format!("{BODY}\n## Notes from the field\n\nThe guard confirmed the dead zone.\n"),
        );
        let (w, diags) = read(&root, &text);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        // The author's own heading changes nothing the schema holds.
        let w = w.unwrap();
        assert!(w.killer.is_some());
        assert!(w.scenarios.is_some());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_fourth_frontmatter_key_is_a_located_error() {
        let root = temp_root();
        let text = fact(&format!("{HEADER}confidence: high\n"), BODY);
        let (_, diags) = read(&root, &text);
        assert_eq!(only(&diags), ("E_DOC", line_of(&text, "confidence: high")));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn covers_and_uses_carry_their_values_and_their_line() {
        let root = temp_root();
        // Neither list resolves here — the compiler resolves them, and it
        // needs the values and the line to locate its error.
        let text = fact(
            "covers: [AuthService, AuthService.handle_login]\n\
             sources: []\n\
             uses: [the-train-has-no-signal]\n",
            BODY,
        );
        let (w, diags) = read(&root, &text);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        let w = w.unwrap();
        let (covers, cline) = w.covers.as_ref().unwrap();
        assert_eq!(covers, &["AuthService", "AuthService.handle_login"]);
        assert_eq!(*cline, line_of(&text, "covers:"));
        let (uses, uline) = w.uses.as_ref().unwrap();
        assert_eq!(uses, &["the-train-has-no-signal"]);
        assert_eq!(*uline, line_of(&text, "uses:"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_empty_sources_parses_and_reads_as_ungrounded() {
        let root = temp_root();
        let (w, diags) = read(&root, &fact(HEADER, BODY));
        assert_eq!(rendered(&diags), Vec::<String>::new());
        assert!(w.unwrap().ungrounded());

        let (w, diags) = read(
            &root,
            &fact("covers: []\nsources: [https://example.org/thread/42]\nuses: []\n", BODY),
        );
        assert_eq!(rendered(&diags), Vec::<String>::new());
        assert!(!w.unwrap().ungrounded());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_tree_path_resolves_and_a_miss_is_located() {
        let root = temp_root();
        fs::create_dir_all(root.join("notes")).unwrap();
        fs::write(root.join("notes/train.md"), "the ride, written down\n").unwrap();
        let text = fact("covers: []\nsources: [notes/train.md]\nuses: []\n", BODY);
        let (w, diags) = read(&root, &text);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        assert_eq!(forms(&w.unwrap()), ["path notes/train.md"]);

        let text = fact("covers: []\nsources: [notes/ghost.md]\nuses: []\n", BODY);
        let (_, diags) = read(&root, &text);
        assert_eq!(only(&diags), ("E_DOC", line_of(&text, "sources:")));
        assert!(diags[0].message.contains("notes/ghost.md"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_schemed_entry_never_touches_the_filesystem() {
        // A root that does not exist: a resolved entry could not survive it.
        let root = Path::new("/nowhere/archi-world-test");
        let text = fact(
            "covers: []\n\
             sources: [https://example.org/thread/42, mailto:guard@rail.example, jira:RAIL-77]\n\
             uses: []\n",
            BODY,
        );
        let (w, diags) = read(root, &text);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        assert_eq!(
            forms(&w.unwrap()),
            [
                "locator https://example.org/thread/42",
                "locator mailto:guard@rail.example",
                "locator jira:RAIL-77",
            ]
        );
    }

    #[test]
    fn a_malformed_schemed_entry_is_a_located_error() {
        let root = temp_root();
        for entry in ["mailto:", "1jira:RAIL-77", "://example.org", "https:a b"] {
            let text = fact(
                &format!("covers: []\nsources: [{entry}]\nuses: []\n"),
                BODY,
            );
            let (w, diags) = read(&root, &text);
            assert_eq!(
                only(&diags),
                ("E_DOC", line_of(&text, "sources:")),
                "on `{entry}`"
            );
            // Neither form: the entry is reported, never carried.
            assert_eq!(forms(&w.unwrap()), Vec::<String>::new(), "on `{entry}`");
        }
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_record_mixing_both_forms_parses() {
        let root = temp_root();
        fs::create_dir_all(root.join("notes")).unwrap();
        fs::write(root.join("notes/train.md"), "the ride, written down\n").unwrap();
        let text = fact(
            "covers: []\nsources: [notes/train.md, https://example.org/thread/42]\nuses: []\n",
            BODY,
        );
        let (w, diags) = read(&root, &text);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        let w = w.unwrap();
        assert_eq!(
            forms(&w),
            ["path notes/train.md", "locator https://example.org/thread/42"]
        );
        assert!(!w.ungrounded());
        fs::remove_dir_all(&root).unwrap();
    }
}
