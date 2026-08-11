//! The world-fact record (`archi/requirements/world-facts/a-world-fact-carries-its-scenarios.md`,
//! `archi/requirements/world-facts/the-header-points-three-ways.md`,
//! `archi/requirements/world-facts/a-source-is-reachable-and-lives-in-the-world.md`):
//! one file under `archi/world/facts/` holds one condition outside the system,
//! what would make that condition false, and the scenarios it dictates.
//! Parsing is best-effort as it is in [`super::schema`] — every deviation
//! lands in the diagnostics and the record keeps what was sound.
//!
//! The three frontmatter lists are the whole header. `sources` resolves here,
//! because its one entry form is the record's own rule: a path from the
//! project root to a file under [`WORLD`], and nowhere else. `covers` and
//! `uses` are carried with the line they sit on and resolve against the model
//! and the other facts in the compiler. The `Scenarios` block is carried as
//! text and an offset — the grammar parses it, not this reader.
//!
//! A section is what stands under its heading, and that includes the deeper
//! headings [`super::md`] lifted out of it. `Scenarios` is written as `### `
//! headings over step lines, so a section rebuilt from its prose alone would
//! arrive empty and the grammar would never see a scenario. Each such heading
//! is put back as the line it was, at the line it was, so a location the
//! grammar reports is still the line in the fact file.

use std::path::Path;

use super::DocDiagnostic;
use super::md::MdDoc;
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

/// The folder every `sources` entry names a file inside of
/// (`archi/requirements/world-facts/a-source-is-reachable-and-lives-in-the-world.md`).
pub const WORLD: &str = "archi/world/";

/// One world fact — the file under `archi/world/facts/`.
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
    /// The material the fact rests on — each entry a path from the project
    /// root to a file under [`WORLD`], with the field's line; `None` when
    /// invalid. An entry the rule refuses is reported and not carried.
    pub sources: Option<(Vec<String>, usize)>,
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
        open_questions: open_questions.and_then(|at| block(&lines_of(doc, at))),
    }
}

/// Everything a section holds, as the lines it was written as: its own prose,
/// and every deeper heading under it with the prose under that. `at` indexes
/// [`MdDoc::headings`], and the section ends at the first heading no deeper
/// than its own.
///
/// The structural reader keeps a heading as level and text, so the line is
/// written back from those two; the file line it came from rides with it, and
/// [`block`] puts the blank lines between them back. A section holding no
/// deeper heading yields exactly the prose the reader gave.
fn lines_of(doc: &MdDoc, at: usize) -> Vec<(usize, String)> {
    let head = &doc.headings[at];
    let mut out = head.content.clone();
    for h in &doc.headings[at + 1..] {
        if h.level <= head.level {
            break;
        }
        out.push((h.line, format!("{} {}", "#".repeat(h.level), h.text)));
        out.extend(h.content.iter().cloned());
    }
    out
}

/// The recognized sections in canonical order, each `None` when the file does
/// not hold it, and each carried as its index in [`MdDoc::headings`] — the
/// section is the heading and everything under it, and [`lines_of`] needs the
/// place to read on from. A section out of order or a second copy of one is
/// reported here; any other heading is the author's own and passes untouched.
fn sections(doc: &MdDoc, file: &str, diags: &mut Vec<DocDiagnostic>) -> [Option<usize>; 3] {
    let mut found: [Option<usize>; 3] = [None, None, None];
    let mut reached = 0;
    for (at, h) in doc.headings.iter().enumerate() {
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
        found[i] = Some(at);
    }
    found
}

/// A section the schema requires: its absence is reported at the name, its
/// emptiness at its own heading, and neither state carries prose.
fn required(
    section: Option<usize>,
    heading: &str,
    holds: &str,
    doc: &MdDoc,
    file: &str,
    diags: &mut Vec<DocDiagnostic>,
) -> Option<Block> {
    let Some(at) = section else {
        diags.push(DocDiagnostic::new(
            "E_DOC",
            format!("a world fact holds `## {heading}`: {holds}"),
            file,
            doc.name_line,
        ));
        return None;
    };
    let prose = block(&lines_of(doc, at));
    if prose.is_none() {
        diags.push(DocDiagnostic::new(
            "E_DOC",
            format!("`{heading}` holds nothing: {holds}"),
            file,
            doc.headings[at].line,
        ));
    }
    prose
}

/// The `sources` entries. Every one names a file under [`WORLD`], and every
/// one resolves on disk. An entry that points outside the world is reported
/// and the message says why; an entry that reaches nothing is reported. A
/// refused entry is carried by nothing — the field holds what stood up.
fn sources_of(
    entries: &[String],
    root: &Path,
    file: &str,
    line: usize,
    diags: &mut Vec<DocDiagnostic>,
) -> Vec<String> {
    let mut out = Vec::new();
    for entry in entries {
        if !inside_the_world(entry) {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                format!(
                    "`{entry}` lies outside `{WORLD}` — a `sources` entry is a path from the \
                     project root to a file under `{WORLD}`: an external locator is a claim \
                     about evidence and not evidence, and a path into the spec grounds the \
                     fact in what the fact explains"
                ),
                file,
                line,
            ));
            continue;
        }
        if !root.join(entry).is_file() {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                format!(
                    "`sources` names no file `{entry}` — a source that cannot be read is no \
                     source: the material comes into `{WORLD}` or the field stays empty"
                ),
                file,
                line,
            ));
            continue;
        }
        out.push(entry.clone());
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

/// Whether the entry names a place inside the world: a relative path that
/// opens with [`WORLD`] and walks no step back out of it. A URI carries a
/// scheme where the folder should stand, so it fails the first test and needs
/// no test of its own.
fn inside_the_world(entry: &str) -> bool {
    entry.starts_with(WORLD) && !entry.split('/').any(|part| part == "..")
}

#[cfg(test)]
mod tests {
    use super::super::{gherkin, md};
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

### the app opens with no network

Given the device has no network
When the user opens the app
Then the last synced view appears

### the queue drains on reconnect

Given a queued write
When the network returns
Then the write reaches the server
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

    /// The project root the standing facts live in — the repository itself,
    /// read from where this crate stands.
    fn project_root() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    /// The `.md` files that stand directly in `dir`.
    fn md_files(dir: &Path) -> Vec<PathBuf> {
        let Ok(entries) = fs::read_dir(dir) else {
            return Vec::new();
        };
        entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("md"))
            .collect()
    }

    /// The standing facts of this repository — the files under `facts/`.
    fn standing_facts() -> Vec<PathBuf> {
        md_files(&project_root().join("archi/world/facts"))
    }

    /// Read one fact the way discovery does: structure first, then schema.
    fn read(root: &Path, text: &str) -> (Option<WorldDoc>, Vec<DocDiagnostic>) {
        let file = format!("archi/world/facts/{SLUG}.md");
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

    /// The entries the record carried — the ones that survived the rule.
    fn carried(w: &WorldDoc) -> Vec<String> {
        w.sources.as_ref().expect("sources parsed").0.clone()
    }

    #[test]
    fn a_whole_fact_parses_clean() {
        let root = temp_root();
        let text = fact(HEADER, BODY);
        let (w, diags) = read(&root, &text);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        let w = w.unwrap();
        assert_eq!(w.slug, SLUG);
        assert_eq!(w.file, format!("archi/world/facts/{SLUG}.md"));
        assert_eq!(w.line, line_of(&text, "# Users open the app"));
        assert_eq!(w.covers.as_ref().unwrap().0, Vec::<String>::new());
        assert_eq!(w.uses.as_ref().unwrap().0, Vec::<String>::new());
        assert!(w.condition.as_ref().unwrap().text.contains("carriage"));
        assert!(w.killer.as_ref().unwrap().text.contains("Trackside"));
        // The scenario block is carried, not parsed: its text and the line it
        // opens on, so the grammar can map a location back onto the file.
        let block = w.scenarios.as_ref().unwrap();
        assert_eq!(block.line, line_of(&text, "### the app opens"));
        assert!(block.text.starts_with("### the app opens with no network\n"));
        assert!(block.text.contains("Then the last synced view appears\n"));
        // The blank line inside the block keeps the following lines in place.
        assert_eq!(
            block.line + block.text.lines().count() - 1,
            line_of(&text, "Then the write reaches the server")
        );
        assert!(w.open_questions.is_none());
        fs::remove_dir_all(&root).unwrap();
    }

    /// The `Scenarios` section reaches the grammar whole
    /// (`archi/requirements/world-facts/a-world-fact-carries-its-scenarios.md`).
    /// The structural reader lifts every `###` line into a heading of its own,
    /// so the section arrives with its sub-headings taken out of it; the
    /// reader puts them back, at the lines they stood on, and the grammar
    /// walks the section the author wrote.
    #[test]
    fn the_scenarios_section_carries_its_headings_and_steps_to_the_grammar() {
        let root = temp_root();
        let text = fact(HEADER, BODY);
        let (w, diags) = read(&root, &text);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        let w = w.unwrap();
        let block = w.scenarios.as_ref().expect("the section is not empty");
        // Every heading and every step line stands in the block, and stands
        // at the line it holds in the file.
        for phrase in [
            "### the app opens with no network",
            "Given the device has no network",
            "When the user opens the app",
            "Then the last synced view appears",
            "### the queue drains on reconnect",
            "Given a queued write",
            "When the network returns",
            "Then the write reaches the server",
        ] {
            let inside = block
                .text
                .lines()
                .position(|l| l == phrase)
                .unwrap_or_else(|| panic!("the block holds `{phrase}`"));
            assert_eq!(block.line + inside, line_of(&text, phrase), "on `{phrase}`");
        }
        // The grammar is called on it, and it reads the two scenarios whole.
        let mut diags = Vec::new();
        let parsed = gherkin::parse(block, &w.file, &mut diags).expect("the grammar reads it");
        assert_eq!(rendered(&diags), Vec::<String>::new());
        let names: Vec<&str> = parsed.scenarios.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "the app opens with no network",
                "the queue drains on reconnect"
            ]
        );
        assert_eq!(parsed.scenarios[0].line, line_of(&text, "### the app opens"));
        assert_eq!(parsed.scenarios[0].steps.len(), 3);
        assert_eq!(parsed.scenarios[1].steps.len(), 3);
        assert_eq!(
            parsed.scenarios[0].steps[0].line,
            line_of(&text, "Given the device")
        );
        fs::remove_dir_all(&root).unwrap();
    }

    /// A section ends where the next `##` opens: the headings the reader puts
    /// back are the ones that stood under `Scenarios`, and no other.
    #[test]
    fn a_heading_after_the_section_stays_out_of_the_block() {
        let root = temp_root();
        let text = fact(
            HEADER,
            &format!("{BODY}\n## Open questions\n\n### How long is a tunnel?\n\nNobody timed one.\n"),
        );
        let (w, diags) = read(&root, &text);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        let w = w.unwrap();
        let block = w.scenarios.as_ref().unwrap();
        assert!(!block.text.contains("tunnel"), "{}", block.text);
        assert_eq!(
            block.line + block.text.lines().count() - 1,
            line_of(&text, "Then the write reaches the server")
        );
        // The open questions keep their own sub-heading, at its own line.
        let open = w.open_questions.as_ref().unwrap();
        assert_eq!(open.line, line_of(&text, "### How long is a tunnel?"));
        assert!(open.text.contains("Nobody timed one."));
        fs::remove_dir_all(&root).unwrap();
    }

    /// The shape the standing facts are written in: the `### ` heading is the
    /// scenario and the fact's own title is the feature, so no `Feature:` and
    /// no `Scenario:` line is left under `archi/world/facts/`.
    #[test]
    fn no_fact_under_archi_world_holds_a_feature_or_a_scenario_line() {
        let mut left: Vec<String> = Vec::new();
        for path in standing_facts() {
            for (i, line) in fs::read_to_string(&path).unwrap().lines().enumerate() {
                let line = line.trim_start();
                if line.starts_with("Feature:") || line.starts_with("Scenario:") {
                    left.push(format!("{}:{}", path.display(), i + 1));
                }
            }
        }
        assert_eq!(left, Vec::<String>::new());
    }

    /// The folder decides what a file is
    /// (`archi/requirements/world-facts/the-world-holds-four-layers.md`), so a
    /// record standing directly in `archi/world/` has no kind at all. The four
    /// standing facts live under `facts/` and nothing is left beside them.
    #[test]
    fn no_record_stands_directly_under_archi_world() {
        let loose = md_files(&project_root().join("archi/world"));
        assert_eq!(loose, Vec::<PathBuf>::new());
        assert_eq!(standing_facts().len(), 4, "{:?}", standing_facts());
    }

    /// Every `sources` entry the tree holds names a file inside the world
    /// (`archi/requirements/world-facts/a-source-is-reachable-and-lives-in-the-world.md`).
    /// The migration wrote the intent each fact was lifted from into this
    /// field; the field is empty now, and nothing outside `archi/world/` may
    /// go back into it.
    #[test]
    fn no_sources_entry_in_the_tree_names_a_path_outside_the_world() {
        let mut outside: Vec<String> = Vec::new();
        for path in standing_facts() {
            let text = fs::read_to_string(&path).unwrap();
            // The frontmatter: past the opening fence, up to the closing one.
            for line in text.lines().skip(1).take_while(|l| l.trim() != "---") {
                let Some(rest) = line.strip_prefix("sources:") else {
                    continue;
                };
                let inner = rest.trim().trim_start_matches('[').trim_end_matches(']');
                for item in inner.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                    if !item.starts_with(WORLD) {
                        outside.push(format!("{}: {item}", path.display()));
                    }
                }
            }
        }
        assert_eq!(outside, Vec::<String>::new());
    }

    /// The four standing facts rest on nothing anybody recorded, and each one
    /// says so — the state the wing counts
    /// (`archi/requirements/world-facts/an-ungrounded-fact-says-so.md`).
    #[test]
    fn the_four_standing_facts_carry_no_source_and_report_ungrounded() {
        let root = project_root();
        let mut grounded: Vec<String> = Vec::new();
        for path in standing_facts() {
            let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
            let text = fs::read_to_string(&path).unwrap();
            let doc = md::parse(&text).unwrap_or_else(|e| panic!("{stem}: {}", e.message));
            let mut diags = Vec::new();
            let file = format!("archi/world/facts/{stem}.md");
            let w = parse(&doc, &file, &stem, &root, &mut diags);
            assert_eq!(rendered(&diags), Vec::<String>::new(), "on `{stem}`");
            if !w.ungrounded() {
                grounded.push(stem);
            }
        }
        assert_eq!(grounded, Vec::<String>::new());
    }

    /// The member a scenario runs in, read off the anchor its link carries —
    /// the whole rule, and the only place it is written
    /// (`archi/requirements/world-facts/a-scenario-runs-where-its-link-points.md`).
    fn runner(anchor: &crate::links::Anchor) -> &str {
        anchor.repo.as_deref().unwrap_or(crate::members::HOME)
    }

    /// A scenario carries no declaration of where it runs: the anchor's member
    /// prefix says so.
    #[test]
    fn an_anchor_s_member_prefix_names_where_its_scenario_runs() {
        let anchor = crate::links::Anchor::parse("backend//tests/walk.rs#opens").unwrap();
        assert_eq!(anchor.repo.as_deref(), Some("backend"));
        assert_eq!(runner(&anchor), "backend");
        // The scan key the member prefix produces names the same member.
        assert_eq!(anchor.qualified_file(), "backend//tests/walk.rs");
    }

    /// A bare anchor runs in the project's own repository — the home member,
    /// whose name is the empty one and whose key stays unqualified.
    #[test]
    fn a_bare_anchor_runs_in_the_project_s_own_repository() {
        let anchor = crate::links::Anchor::parse("tests/walk.rs#opens").unwrap();
        assert_eq!(anchor.repo, None);
        assert_eq!(runner(&anchor), crate::members::HOME);
        assert_eq!(anchor.qualified_file(), "tests/walk.rs");
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
            &without(BODY, &["### the ", "Given ", "When ", "Then "]),
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

    /// A world that holds one file in each of the three loose folders.
    fn world_with_material() -> PathBuf {
        let root = temp_root();
        for (folder, name, text) in [
            ("notes", "train.md", "the ride, written down\n"),
            ("hypotheses", "repeaters.md", "somebody means to measure it\n"),
            ("resources", "guard.txt", "the transcript, raw\n"),
        ] {
            let dir = root.join("archi/world").join(folder);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join(name), text).unwrap();
        }
        root
    }

    /// The header the fact wears when it names one source.
    fn with_source(entry: &str) -> String {
        format!("covers: []\nsources: [{entry}]\nuses: []\n")
    }

    #[test]
    fn an_empty_sources_parses_and_reads_as_ungrounded() {
        let root = world_with_material();
        let (w, diags) = read(&root, &fact(HEADER, BODY));
        assert_eq!(rendered(&diags), Vec::<String>::new());
        assert!(w.unwrap().ungrounded());

        let (w, diags) = read(
            &root,
            &fact(&with_source("archi/world/notes/train.md"), BODY),
        );
        assert_eq!(rendered(&diags), Vec::<String>::new());
        assert!(!w.unwrap().ungrounded());
        fs::remove_dir_all(&root).unwrap();
    }

    /// A source is material this project holds, so the entry names a file in
    /// one of the world's loose folders and the reader resolves it
    /// (`archi/requirements/world-facts/a-source-is-reachable-and-lives-in-the-world.md`).
    #[test]
    fn a_source_under_the_world_s_loose_folders_resolves() {
        let root = world_with_material();
        let entries = [
            "archi/world/notes/train.md",
            "archi/world/hypotheses/repeaters.md",
            "archi/world/resources/guard.txt",
        ];
        let text = fact(&with_source(&entries.join(", ")), BODY);
        let (w, diags) = read(&root, &text);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        let w = w.unwrap();
        assert_eq!(carried(&w), entries);
        assert!(!w.ungrounded());
        fs::remove_dir_all(&root).unwrap();
    }

    /// An entry that reaches nothing is not a source: the material comes into
    /// the world or the field stays empty.
    #[test]
    fn a_source_that_reaches_nothing_is_a_located_error() {
        let root = world_with_material();
        for entry in [
            "archi/world/notes/ghost.md",
            // A folder is not a file, and neither is the world itself.
            "archi/world/notes",
        ] {
            let text = fact(&with_source(entry), BODY);
            let (w, diags) = read(&root, &text);
            assert_eq!(
                only(&diags),
                ("E_DOC", line_of(&text, "sources:")),
                "on `{entry}`"
            );
            assert!(diags[0].message.contains(entry), "{}", diags[0].message);
            assert_eq!(carried(&w.unwrap()), Vec::<String>::new(), "on `{entry}`");
        }
        fs::remove_dir_all(&root).unwrap();
    }

    /// A path into the spec grounds the fact in what the fact explains, so it
    /// is refused and the message says that. The migration wrote exactly this
    /// entry into all four standing facts.
    #[test]
    fn a_source_outside_the_world_is_a_located_error_that_says_why() {
        let root = world_with_material();
        for entry in [
            "archi/requirements/modeling-language/modeling-language.md",
            "notes/train.md",
            "archi/world/../notes/train.md",
            "/etc/hosts",
        ] {
            let text = fact(&with_source(entry), BODY);
            let (w, diags) = read(&root, &text);
            assert_eq!(
                only(&diags),
                ("E_DOC", line_of(&text, "sources:")),
                "on `{entry}`"
            );
            let message = &diags[0].message;
            assert!(message.contains(entry), "{message}");
            assert!(message.contains("archi/world/"), "{message}");
            assert!(message.contains("lies outside"), "{message}");
            assert_eq!(carried(&w.unwrap()), Vec::<String>::new(), "on `{entry}`");
        }
        fs::remove_dir_all(&root).unwrap();
    }

    /// An external locator is a claim about evidence and not evidence, so it
    /// is refused by the one rule, with the one message.
    #[test]
    fn a_source_carrying_a_uri_scheme_raises_the_same_error() {
        let root = world_with_material();
        for entry in [
            "https://example.org/thread/42",
            "mailto:guard@rail.example",
            "jira:RAIL-77",
        ] {
            let text = fact(&with_source(entry), BODY);
            let (w, diags) = read(&root, &text);
            assert_eq!(
                only(&diags),
                ("E_DOC", line_of(&text, "sources:")),
                "on `{entry}`"
            );
            let message = &diags[0].message;
            assert!(message.contains(entry), "{message}");
            assert!(message.contains("lies outside"), "{message}");
            assert_eq!(carried(&w.unwrap()), Vec::<String>::new(), "on `{entry}`");
        }
        fs::remove_dir_all(&root).unwrap();
    }
}
