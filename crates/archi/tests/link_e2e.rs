//! End to end through the real binary: the thread between a written claim
//! and the code that answers it. What the audit says is promised and
//! unbuilt, what the record hands an assistant that did not write the
//! obligation, and what still answers once the author and their history are
//! gone (`archi/requirements/code-link/`,
//! `archi/world/facts/a-design-written-apart-from-the-code-falls-behind-it.md`,
//! `archi/world/facts/an-assistant-guesses-which-files-answer-a-written-obligation.md`).

mod util;

use std::fs;
use std::path::{Path, PathBuf};

use util::ok;

const MODEL: &str = "def conn wire := * -> *\n\
                     def node Gate:\n  port out\n\
                     def node Auth:\n  port inn\n\
                     Gate.out wire Auth.inn\n";

const AUTH_RS: &str = "pub fn login(user: &str) -> bool {\n    !user.is_empty()\n}\n";

/// A committed project and the worktree bound to it — mutating verbs answer
/// only from the worktree, so every test writes and runs there. Returns
/// (fixture, worktree).
fn bound(tag: &str) -> (PathBuf, PathBuf) {
    let fixture = util::scratch("archi-link-e2e", tag);
    fs::create_dir_all(fixture.join("archi/src")).unwrap();
    fs::create_dir_all(fixture.join("code")).unwrap();
    fs::write(
        fixture.join("archi.toml"),
        "[project]\nname = \"t\"\npreset = \"default\"\n",
    )
    .unwrap();
    fs::write(fixture.join("archi/src/model.arch"), MODEL).unwrap();
    fs::write(fixture.join("code/auth.rs"), AUTH_RS).unwrap();
    let wt = util::worktree(&fixture);
    (fixture, wt)
}

fn cleanup(fixture: &Path) {
    let name = fixture.file_name().unwrap().to_str().unwrap();
    let _ = fs::remove_dir_all(fixture.parent().unwrap().join(format!("{name}-worktrees")));
    let _ = fs::remove_dir_all(fixture);
}

/// The slug of the requirement the fixtures address.
const REQ: &str = "a-login-is-refused-without-a-name";

/// One requirement where the discovery walks for it: the intent folder, its
/// anchor, and the requirement beside it.
fn write_requirement(root: &Path, slug: &str) {
    let dir = root.join("archi/requirements/access");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("access.md"), "# Access\n\nWho gets in, and on what.\n").unwrap();
    fs::write(
        dir.join(format!("{slug}.md")),
        "---\nkind: functional\nsatisfied-by: [Auth]\ndeferred:\n---\n\n\
         # A login is refused without a name\n\nAn empty name is refused.\n",
    )
    .unwrap();
}

/// The producing-rule column of a rendered row.
fn rule_word(line: &str) -> Option<&str> {
    line.split_whitespace().nth(4)
}

/// A writer asks what answers a written requirement and the record answers
/// with an address of its own — the requirement, not the element that stands
/// near it — and answers from the rows a person stood behind
/// (`archi/requirements/code-link/a-requirement-is-addressable-in-the-journal.md`,
/// `archi/requirements/code-link/the-journal-says-which-rule-made-a-row.md`).
#[test]
fn a_requirement_is_addressed_and_the_reverse_view_answers_from_it() {
    let (fixture, root) = bound("requirement");
    write_requirement(&root, REQ);
    let spec = format!("req:{REQ}");

    ok(&root, &[
        "link", "add", &spec, "code/auth.rs#login", "--kind", "indirect",
    ]);
    let out = ok(&root, &["link", "ls", "--spec", &spec]);
    assert!(out.contains(&format!("{spec} ← code/auth.rs#login")), "{out}");
    assert_eq!(out.lines().count(), 1, "{out}");
    assert_eq!(rule_word(out.lines().next().unwrap()), Some("authored"), "{out}");

    // A `req:` naming no requirement is refused, and the refusal names the
    // slug rather than the tree it searched.
    let (success, _, err) = util::run(&root, &[
        "link", "add", "req:no-such-claim", "code/auth.rs#login", "--kind", "indirect",
    ]);
    assert!(!success, "a slug nothing holds is refused");
    assert!(err.contains("no-such-claim"), "{err}");

    // A requirement carries no version pin.
    let (success, _, err) = util::run(&root, &[
        "link", "add", &format!("{spec}@v0001"), "code/auth.rs#login", "--kind", "indirect",
    ]);
    assert!(!success, "a requirement takes no slot");
    assert!(err.contains("carries no version pin"), "{err}");

    // The audit's tally names the three rules.
    let out = ok(&root, &["link", "audit"]);
    let head = out.lines().next().unwrap();
    for word in ["declared", "inferred", "authored"] {
        assert!(head.contains(word), "{out}");
    }

    // No verb retires rows in bulk by their producing rule.
    let (success, _, err) = util::run(&root, &["link", "rm", "--rule", "inferred", "--yes"]);
    assert!(!success, "there is no such selector");
    assert!(err.contains("--rule"), "{err}");
    assert_eq!(ok(&root, &["link", "ls"]).lines().count(), 1, "nothing retired");

    cleanup(&fixture);
}

/// The journal this project already carries was written before a row said
/// which rule produced it. It loads, grades and prints as it always did, and
/// each of those rows reads the rule its origin already recorded: what
/// capture minted under the shared-term rule reads inferred, and what `link
/// add` minted reads authored. No migration was run, and none is needed
/// (`archi/requirements/code-link/the-journal-says-which-rule-made-a-row.md`).
#[test]
fn the_standing_journal_takes_its_rule_from_its_origin_with_no_migration() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the project this crate lives in");
    let text = fs::read_to_string(repo.join("archi/links/journal.jsonl"))
        .expect("the standing journal");

    // The rows written before the field existed: an add that names no rule.
    // Each one is sorted by the provenance it does name.
    let mut captured: Vec<&str> = Vec::new();
    let mut authored: Vec<&str> = Vec::new();
    for line in text
        .lines()
        .filter(|l| l.contains("\"event\":\"add\"") && !l.contains("\"rule\":"))
    {
        let Some(id) = line
            .split("\"id\":\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
        else {
            continue;
        };
        if line.contains("\"origin\":{\"kind\":\"captured\"") {
            captured.push(id);
        } else if line.contains("\"origin\":{\"kind\":\"authored\"") {
            authored.push(id);
        }
    }
    assert!(
        captured.len() > 2000 && authored.len() > 100,
        "the standing journal holds both kinds of row this test is about: \
         {} captured, {} authored",
        captured.len(),
        authored.len()
    );

    let out = ok(&repo, &["link", "ls"]);
    let (mut guesses, mut claims) = (0, 0);
    for line in out.lines() {
        let id = line.split_whitespace().next().unwrap_or_default();
        if captured.contains(&id) {
            assert_eq!(rule_word(line), Some("inferred"), "{line}");
            guesses += 1;
        } else if authored.contains(&id) {
            assert_eq!(rule_word(line), Some("authored"), "{line}");
            claims += 1;
        }
    }
    assert!(guesses > 2000, "every standing guess printed: {guesses}");
    assert!(claims > 100, "every standing claim printed: {claims}");

    // The rows a person reviewed hardest — a link onto a scenario of a world
    // fact, the only spec side that holds a `#` — were all minted by hand,
    // and they stay claims.
    let scenarios: Vec<&str> = out
        .lines()
        .filter(|l| l.split(" ← ").next().is_some_and(|left| left.contains('#')))
        .collect();
    for line in &scenarios {
        assert_eq!(rule_word(line), Some("authored"), "{line}");
    }
    assert!(!scenarios.is_empty(), "the world links are in the journal");
}

/// A claim the model makes and no code answers is named — promised and
/// unbuilt — when somebody asks what is unaccounted for; code recorded
/// against it lifts the finding, and the claim still waiting keeps its line
/// (`archi/requirements/code-link/the-audit-inverts-coverage.md`,
/// `archi/world/facts/a-design-written-apart-from-the-code-falls-behind-it.md`,
/// "A claim is written that no code answers").
#[test]
fn the_audit_names_a_claim_no_code_answers() {
    let (fixture, root) = bound("unbuilt");
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);
    ok(&root, &["plan", "task", "add", "Auth"]);

    // The node and the edge into it are both written and both dark: the
    // question "what is unaccounted for" names each one.
    let out = ok(&root, &["link", "audit"]);
    assert!(
        out.contains("unlinked spec element: Auth — no asserted code-link"),
        "{out}"
    );
    assert!(
        out.contains("unlinked spec element: Gate.out wire Auth.inn — no asserted code-link"),
        "{out}"
    );

    // Code answers the node; the edge is still promised and still named.
    ok(&root, &[
        "link", "add", "Auth", "code/auth.rs#login", "--kind", "indirect",
    ]);
    let out = ok(&root, &["link", "audit"]);
    assert!(!out.contains("unlinked spec element: Auth —"), "{out}");
    assert!(
        out.contains("unlinked spec element: Gate.out wire Auth.inn"),
        "{out}"
    );

    cleanup(&fixture);
}

/// An assistant that did not write the obligation asks the record which
/// files answer it and is told, instead of reading the tree and guessing;
/// and work landing in a file no record names is reported rather than
/// assumed to be covered
/// (`archi/requirements/code-link/code-link.md`,
/// `archi/world/facts/an-assistant-guesses-which-files-answer-a-written-obligation.md`,
/// "Work arrives over an obligation the assistant did not write").
#[test]
fn the_record_hands_over_the_files_and_names_what_it_does_not_cover() {
    let (fixture, root) = bound("handover");
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &[
        "link", "add", "Auth", "code/auth.rs#login", "--kind", "indirect",
    ]);
    ok(&root, &[
        "link",
        "add",
        "Gate.out wire Auth.inn",
        "code/auth.rs#login",
        "--kind",
        "indirect",
    ]);

    // Asked to change what `Auth` covers, the assistant asks the record: it
    // answers with the file and the item inside it, and with that
    // obligation's files alone — the other claim over the same file stays
    // out of the answer.
    let out = ok(&root, &["link", "ls", "--spec", "Auth"]);
    assert!(out.contains("Auth ← code/auth.rs#login"), "{out}");
    assert_eq!(
        out.lines().count(),
        1,
        "the obligation's own files, nothing else:\n{out}"
    );

    // The second obligation answers on its own name, over the same file.
    let out = ok(&root, &["link", "ls", "--spec", "Gate.out wire Auth.inn"]);
    assert!(
        out.contains("Gate.out wire Auth.inn ← code/auth.rs#login"),
        "{out}"
    );

    // Work lands in a file no record names: it is reported, never taken as
    // covered because it sits in the tree.
    fs::write(
        root.join("code/rogue.rs"),
        "pub fn rogue(n: u8) -> u8 {\n    n + 7\n}\n",
    )
    .unwrap();
    let out = ok(&root, &["link", "audit"]);
    assert!(out.contains("unaccounted delta: code/rogue.rs"), "{out}");
    assert!(out.contains("no link claims it"), "{out}");

    cleanup(&fixture);
}

/// The record outlives its author: a successor tree carries the spec, the
/// code and the journal and holds no repository at all — no commit history
/// to walk — and the same question gets the same answer
/// (`archi/requirements/code-link/code-link.md`,
/// `archi/world/facts/an-assistant-guesses-which-files-answer-a-written-obligation.md`,
/// "The thread is recovered after its author has gone").
#[test]
fn the_record_answers_with_no_history_to_walk() {
    let (fixture, root) = bound("recovered");
    ok(&root, &[
        "link", "add", "Auth", "code/auth.rs#login", "--kind", "indirect",
    ]);
    let answered = ok(&root, &["link", "ls", "--spec", "Auth"]);
    assert!(answered.contains("Auth ← code/auth.rs#login"), "{answered}");

    // Everything the author left behind, minus the history: spec, code and
    // the journal, in a tree git knows nothing about.
    let heir = util::scratch("archi-link-e2e", "heir");
    fs::create_dir_all(heir.join("archi/src")).unwrap();
    fs::create_dir_all(heir.join("archi/links")).unwrap();
    fs::create_dir_all(heir.join("code")).unwrap();
    for (from, to) in [
        ("archi.toml", "archi.toml"),
        ("archi/src/model.arch", "archi/src/model.arch"),
        ("code/auth.rs", "code/auth.rs"),
        ("archi/links/journal.jsonl", "archi/links/journal.jsonl"),
    ] {
        fs::copy(root.join(from), heir.join(to)).unwrap();
    }
    assert!(!heir.join(".git").exists(), "no history to walk");

    // The question is put to the record alone, and the record answers the
    // same files — byte for byte with the answer its author got.
    let out = ok(&heir, &["link", "ls", "--spec", "Auth"]);
    assert_eq!(out, answered);

    let _ = fs::remove_dir_all(&heir);
    cleanup(&fixture);
}
