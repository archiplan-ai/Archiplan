//! The scenario grammar
//! (`archi/requirements/world-facts/the-grammar-is-a-named-subset.md`,
//! `archi/requirements/world-facts/scenarios-parse-or-the-check-fails.md`,
//! `archi/requirements/world-facts/a-scenario-names-where-it-runs.md`): the
//! `Scenarios` block of a world fact reads as six keywords and a tag line.
//!
//! The `gherkin` crate reads the whole language; this module reads it and
//! then refuses everything outside the subset — backgrounds, rules, outlines,
//! examples, docstrings, data tables, free prose and any other step keyword —
//! with one located error naming the construct and the six keywords the
//! subset holds (`archi/decisions/the-grammar-stays-small.md`). Every location
//! the crate reports sits inside the block, so it is mapped onto the fact file
//! before it is reported.
//!
//! A `@runs:<member>` tag names the member whose tree runs the scenario. It
//! resolves against the declared members, and its absence is the claim that
//! the project's own repository runs it.

use gherkin::{Feature, GherkinEnv};

use super::DocDiagnostic;
use super::world::Block;
use crate::members::{HOME, MemberSet};

/// What every refusal names — the whole grammar in one clause.
const SUBSET: &str = "the grammar reads `Feature`, `Scenario`, `Given`, `When`, `Then` and \
                      `And`, and a tag line above a feature or a scenario";

/// The step keywords the subset holds.
const STEPS: [&str; 4] = ["Given", "When", "Then", "And"];

/// The tag prefix that names where a scenario runs.
const RUNS: &str = "runs:";

/// One `Scenarios` block, parsed and held to the subset.
pub struct ScenarioBlock {
    /// The feature's name — the block in one line.
    pub feature: String,
    /// 1-based fact-file line of the `Feature` keyword.
    pub line: usize,
    /// The scenarios the subset accepted, in source order.
    pub scenarios: Vec<Scenario>,
}

/// One scenario — the address a later `archi link` anchors to running code.
pub struct Scenario {
    /// The scenario's name.
    pub name: String,
    /// 1-based fact-file line of the `Scenario` keyword.
    pub line: usize,
    /// The steps, in source order.
    pub steps: Vec<Step>,
    /// The member whose tree runs it; [`HOME`] — the project's own repository
    /// — when no `@runs:` tag names one.
    pub runs: String,
    /// Every tag the scenario carries, `@` stripped, `runs:` among them.
    pub tags: Vec<String>,
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

/// Parse one `Scenarios` block. `block` is the text and its offset as the
/// world-fact reader carried them; `members` are the declared members the
/// `@runs:` tags resolve against. Every deviation lands in the diagnostics and
/// the result keeps the scenarios that were sound; `None` means none was.
pub fn parse(
    block: &Block,
    file: &str,
    members: &MemberSet,
    diags: &mut Vec<DocDiagnostic>,
) -> Option<ScenarioBlock> {
    // Blank lines and comments state no scenario: the heading stands over
    // nothing.
    if block.text.lines().all(|l| {
        let line = l.trim_start();
        line.is_empty() || line.starts_with('#')
    }) {
        diags.push(err(
            file,
            block.line,
            format!("`Scenarios` holds no scenario — {SUBSET}"),
        ));
        return None;
    }
    // The crate reads the whole language, so what it refuses outright is
    // prose in place of Gherkin, or a line no keyword at all opens.
    let feature = match Feature::parse(&block.text, GherkinEnv::default()) {
        Ok(f) => f,
        Err(e) => {
            let line = line_in(block, reported_line(&e.to_string()));
            diags.push(err(file, line, refusal("this line")));
            return None;
        }
    };

    let line = line_in(block, feature.position.line);
    if feature.keyword != "Feature" {
        diags.push(err(file, line, refusal(&quoted(&feature.keyword))));
    }
    // A description is free prose, and a scenario a machine cannot read is a
    // paragraph wearing a costume.
    if feature.description.is_some() {
        diags.push(err(file, line, refusal("free prose")));
    }
    if let Some(b) = &feature.background {
        let at = line_in(block, b.position.line);
        diags.push(err(file, at, refusal(&quoted(&b.keyword))));
    }
    for r in &feature.rules {
        let at = line_in(block, r.position.line);
        diags.push(err(file, at, refusal(&quoted(&r.keyword))));
    }
    if feature.scenarios.is_empty() {
        diags.push(err(
            file,
            line,
            format!("`Feature` holds one `Scenario` or more — {SUBSET}"),
        ));
    }
    // A `runs:` tag above the feature is the default its scenarios inherit —
    // Gherkin tags read down — and a scenario states its own.
    let inherited = runs_tag(&feature.tags, file, line, members, diags)
        .unwrap_or_else(|| HOME.to_string());

    let mut scenarios = Vec::new();
    for s in &feature.scenarios {
        // A scenario any refusal touched is reported and dropped; the block
        // keeps what was sound, as every doc primitive does.
        let before = diags.len();
        let line = line_in(block, s.position.line);
        if s.keyword != "Scenario" {
            diags.push(err(file, line, refusal(&quoted(&s.keyword))));
        }
        if s.description.is_some() {
            diags.push(err(file, line, refusal("free prose")));
        }
        for e in &s.examples {
            let at = line_in(block, e.position.line);
            diags.push(err(file, at, refusal(&quoted(&e.keyword))));
        }
        if s.steps.is_empty() {
            diags.push(err(
                file,
                line,
                format!("`Scenario` holds one step or more — {SUBSET}"),
            ));
        }
        let mut steps = Vec::new();
        for st in &s.steps {
            let at = line_in(block, st.position.line);
            // The crate keeps the keyword as written, trailing space and all.
            let keyword = st.keyword.trim();
            if !STEPS.contains(&keyword) {
                diags.push(err(file, at, refusal(&quoted(keyword))));
                continue;
            }
            // A docstring carries no location of its own: it opens under its
            // step, and the step's line is where the trimming starts.
            if st.docstring.is_some() {
                diags.push(err(file, at, refusal("a docstring")));
            }
            if let Some(t) = &st.table {
                let at = line_in(block, t.position.line);
                diags.push(err(file, at, refusal("a data table")));
            }
            steps.push(Step {
                keyword: keyword.to_string(),
                text: st.value.clone(),
                line: at,
            });
        }
        let runs =
            runs_tag(&s.tags, file, line, members, diags).unwrap_or_else(|| inherited.clone());
        if diags.len() == before {
            scenarios.push(Scenario {
                name: s.name.clone(),
                line,
                steps,
                runs,
                tags: s.tags.clone(),
            });
        }
    }
    (!scenarios.is_empty()).then(|| ScenarioBlock {
        feature: feature.name.clone(),
        line,
        scenarios,
    })
}

/// The member a tag line names, resolved against the declared members;
/// `None` when the tags name none. Home is never named on a tag: absence is
/// the claim that the project's own repository runs the scenario.
fn runs_tag(
    tags: &[String],
    file: &str,
    line: usize,
    members: &MemberSet,
    diags: &mut Vec<DocDiagnostic>,
) -> Option<String> {
    let named: Vec<&str> = tags.iter().filter_map(|t| t.strip_prefix(RUNS)).collect();
    match named.as_slice() {
        [] => None,
        [name] if members.declared().iter().any(|m| m.name == *name) => {
            Some((*name).to_string())
        }
        [name] => {
            diags.push(err(file, line, unknown_member(name, members)));
            None
        }
        // A walk that crosses members is one scenario per member, joined by
        // the fact that carries them.
        _ => {
            diags.push(err(
                file,
                line,
                format!("`@{RUNS}` appears twice — one scenario runs in one place"),
            ));
            None
        }
    }
}

/// The refusal an unresolved `@runs:` earns: it names what the manifest
/// declares, because the repair is either the name or the declaration.
fn unknown_member(name: &str, members: &MemberSet) -> String {
    let declared: Vec<&str> = members.declared().iter().map(|m| m.name.as_str()).collect();
    let known = if declared.is_empty() {
        "archi.toml declares no member".to_string()
    } else {
        format!("archi.toml declares {}", declared.join(", "))
    };
    format!(
        "`@{RUNS}{name}` names no declared member — {known}; a scenario with no `@{RUNS}` tag \
         runs in the project's own repository"
    )
}

/// The 1-based block line a crate parse failure sits on. The crate keeps the
/// position private and renders it as `Error at <line>:<col>: …`, so the
/// rendering is the only exposure; a rendering that reads otherwise falls
/// back to the block's first line.
fn reported_line(rendered: &str) -> usize {
    rendered
        .strip_prefix("Error at ")
        .and_then(|rest| rest.split_once(':'))
        .and_then(|(line, _)| line.parse().ok())
        .unwrap_or(1)
}

/// Every refusal is one located `E_DOC`: the code `check` blocks on.
fn err(file: &str, line: usize, message: impl Into<String>) -> DocDiagnostic {
    DocDiagnostic::new("E_DOC", message, file, line)
}

/// A keyword as a refusal names it.
fn quoted(keyword: &str) -> String {
    format!("`{keyword}`")
}

/// The one refusal every construct outside the subset earns: it names what it
/// found and what the grammar reads, so trimming a foreign feature file is
/// mechanical rather than a guessing game.
fn refusal(construct: &str) -> String {
    format!("{construct} is outside the scenario grammar — {SUBSET}")
}

/// A location the grammar reports, mapped onto the fact file: line `i` of the
/// block is line `block.line + i - 1` of the file.
fn line_in(block: &Block, line: usize) -> usize {
    block.line + line.saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::members::Member;
    use std::path::PathBuf;

    const FILE: &str = "archi/world/users-open-the-app-on-a-train.md";

    /// The block's first line in the fact file. Every reported line is this
    /// plus the line inside the block, so a test that confused the two would
    /// be off by eleven.
    const AT: usize = 12;

    const BLOCK: &str = "\
Feature: Offline open

  @runs:backend
  Scenario: the app opens with no network
    Given the device has no network
    When the user opens the app
    Then the last synced view appears
    And the banner names the outage

  Scenario: the queue drains on reconnect
    Given a queued write
    When the network returns
    Then the write reaches the server
";

    const BACKGROUND: &str = "\
Feature: Offline open

  Background:
    Given the device has no network

  Scenario: the app opens with no network
    Given the device has no network
    Then the last synced view appears
";

    const RULE: &str = "\
Feature: Offline open

  Scenario: the app opens with no network
    Given the device has no network
    Then the last synced view appears

  Rule: the carriage drops the network

    Scenario: the queue drains on reconnect
      Given a queued write
      Then the write reaches the server
";

    const OUTLINE: &str = "\
Feature: Offline open

  Scenario Outline: the app opens with no network
    Given the device has <signal>
    Then the last synced view appears
";

    const EXAMPLES: &str = "\
Feature: Offline open

  Scenario: the app opens with no network
    Given the device has no network
    Then the last synced view appears

    Examples:
      | signal |
      | none   |
";

    const DOCSTRING: &str = "\
Feature: Offline open

  Scenario: the app opens with no network
    Given the payload
      \"\"\"
      {\"queued\": 1}
      \"\"\"
    Then the last synced view appears
";

    const TABLE: &str = "\
Feature: Offline open

  Scenario: the app opens with no network
    Given the rows
      | device | state |
      | phone  | dark  |
    Then the last synced view appears
";

    /// A step keyword the crate reads and the subset refuses.
    const BUT: &str = "\
Feature: Offline open

  Scenario: the app opens with no network
    Given the device has no network
    But the banner never appears
    Then the last synced view appears
";

    const STAR: &str = "\
Feature: Offline open

  Scenario: the app opens with no network
    Given the device has no network
    * the banner never appears
    Then the last synced view appears
";

    /// A step keyword nothing reads — the grammar stops on the line.
    const MISSPELT: &str = "\
Feature: Offline open

  Scenario: the app opens with no network
    Given the device has no network
    Whn the user opens the app
    Then the last synced view appears
";

    const PROSE: &str = "\
The carriage drops the network for minutes at a time, so the app opens on
whatever it synced last.
";

    /// A paragraph inside a scenario: Gherkin reads it as a description, the
    /// subset as prose wearing a costume.
    const PARAGRAPH: &str = "\
Feature: Offline open

  Scenario: the app opens with no network
    The user opens the app and the last synced view appears.
    Given the device has no network
    Then the last synced view appears
";

    /// A member set with the named members declared — home first, as
    /// [`MemberSet::declared`] assumes.
    fn members(declared: &[&str]) -> MemberSet {
        let member = |name: &str| Member {
            name: name.to_string(),
            url: None,
            declared_path: None,
            mapped_path: None,
            root: None,
        };
        let mut members = vec![member(HOME)];
        members.extend(declared.iter().map(|n| member(n)));
        MemberSet {
            project_root: PathBuf::from("/nowhere"),
            members,
        }
    }

    /// Parse a block sitting at [`AT`] in the fact file.
    fn read(text: &str, declared: &[&str]) -> (Option<ScenarioBlock>, Vec<DocDiagnostic>) {
        let block = Block {
            text: text.to_string(),
            line: AT,
        };
        let mut diags = Vec::new();
        let out = parse(&block, FILE, &members(declared), &mut diags);
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

    /// Whether a refusal lists the six keywords the subset holds.
    fn lists_the_six(message: &str) -> bool {
        ["`Feature`", "`Scenario`", "`Given`", "`When`", "`Then`", "`And`"]
            .iter()
            .all(|k| message.contains(k))
    }

    fn steps(s: &Scenario) -> Vec<String> {
        s.steps
            .iter()
            .map(|st| format!("{} {}", st.keyword, st.text))
            .collect()
    }

    #[test]
    fn a_feature_with_two_scenarios_parses() {
        let (b, diags) = read(BLOCK, &["backend"]);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        let b = b.unwrap();
        assert_eq!(b.feature, "Offline open");
        assert_eq!(b.line, at(BLOCK, "Feature: Offline open"));
        assert_eq!(
            b.scenarios.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            [
                "the app opens with no network",
                "the queue drains on reconnect"
            ]
        );
        let first = &b.scenarios[0];
        assert_eq!(first.line, at(BLOCK, "Scenario: the app opens"));
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
        assert_eq!(b.scenarios[1].steps.len(), 3);
    }

    #[test]
    fn every_construct_outside_the_subset_is_a_located_error() {
        for (construct, text, marker) in [
            ("`Background`", BACKGROUND, "Background:"),
            ("`Rule`", RULE, "Rule: the carriage"),
            ("`Scenario Outline`", OUTLINE, "Scenario Outline:"),
            ("`Examples`", EXAMPLES, "Examples:"),
            ("a docstring", DOCSTRING, "Given the payload"),
            ("a data table", TABLE, "| device | state |"),
        ] {
            let (_, diags) = read(text, &[]);
            let (code, line, message) = only(&diags);
            assert_eq!((code, line), ("E_DOC", at(text, marker)), "on {construct}");
            assert!(message.contains(construct), "{message}");
            assert!(lists_the_six(message), "{message}");
        }
    }

    #[test]
    fn but_and_star_are_refused_like_any_other_unknown_step_keyword() {
        for (named, text, marker) in [
            (Some("`But`"), BUT, "But the banner"),
            (Some("`*`"), STAR, "* the banner"),
            (None, MISSPELT, "Whn the user"),
        ] {
            let (_, diags) = read(text, &[]);
            let (code, line, message) = only(&diags);
            assert_eq!((code, line), ("E_DOC", at(text, marker)), "on {marker}");
            assert!(lists_the_six(message), "{message}");
            if let Some(named) = named {
                assert!(message.contains(named), "{message}");
            }
        }
    }

    #[test]
    fn prose_and_emptiness_are_errors() {
        // A `Scenarios` heading over prose: the grammar stops on its first
        // line, and the line is the one in the fact file.
        let (b, diags) = read(PROSE, &[]);
        assert!(b.is_none());
        let (code, line, message) = only(&diags);
        assert_eq!((code, line), ("E_DOC", AT));
        assert!(lists_the_six(message), "{message}");

        // A paragraph in place of steps is refused at its scenario, and the
        // scenario it costumed is dropped.
        let (b, diags) = read(PARAGRAPH, &[]);
        assert!(b.is_none());
        let (code, line, message) = only(&diags);
        assert_eq!((code, line), ("E_DOC", at(PARAGRAPH, "Scenario: the app opens")));
        assert!(message.contains("free prose"), "{message}");

        // A block of blank lines states no scenario either.
        let (b, diags) = read("\n\n", &[]);
        assert!(b.is_none());
        assert_eq!(only(&diags).0, "E_DOC");
        assert_eq!(only(&diags).1, AT);
    }

    #[test]
    fn a_runs_tag_resolves_against_the_declared_members() {
        // The tagged scenario runs in the member's tree; the untagged one in
        // the project's own repository.
        let (b, diags) = read(BLOCK, &["backend"]);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        let b = b.unwrap();
        assert_eq!(b.scenarios[0].runs, "backend");
        assert_eq!(b.scenarios[0].tags, ["runs:backend"]);
        assert_eq!(b.scenarios[1].runs, HOME);

        // A name no declaration carries is a located error at the scenario.
        let (b, diags) = read(BLOCK, &["frontend"]);
        let (code, line, message) = only(&diags);
        assert_eq!((code, line), ("E_DOC", at(BLOCK, "Scenario: the app opens")));
        assert!(message.contains("backend"), "{message}");
        assert!(message.contains("frontend"), "the declared members: {message}");
        // The unsound scenario is dropped; the sound one stands.
        assert_eq!(b.unwrap().scenarios.len(), 1);
    }

    #[test]
    fn a_tag_above_the_feature_is_the_default_every_scenario_inherits() {
        let text = format!("@runs:backend\n{BLOCK}");
        let block = Block {
            text: text.clone(),
            line: AT,
        };
        let mut diags = Vec::new();
        let b = parse(&block, FILE, &members(&["backend"]), &mut diags);
        assert_eq!(rendered(&diags), Vec::<String>::new());
        let b = b.unwrap();
        assert_eq!(b.scenarios[0].runs, "backend");
        assert_eq!(b.scenarios[1].runs, "backend", "the tag reads down");
    }
}
