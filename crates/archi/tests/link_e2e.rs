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

use serde_json::Value;

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
/// add` minted reads authored. It also holds thousands of rows minted under
/// the second standing, and the decay events the waves pressed onto them:
/// they fold too, and every one of them stands asserted. No migration was
/// run, and none is needed
/// (`archi/requirements/code-link/the-journal-says-which-rule-made-a-row.md`,
/// `archi/requirements/code-link/a-link-stands-asserted-or-it-does-not-stand.md`).
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
    // The size of the corpus is asserted at the head of this test, against
    // the journal itself. What stands live is curated by hand, and a count
    // written down here says nothing about what this test is about: both
    // kinds print, and each row reads the rule its origin recorded.
    assert!(
        guesses > 0 && claims > 0,
        "both kinds printed: {guesses} guesses, {claims} claims"
    );

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

    // The same journal holds what the second standing left behind: rows an
    // older binary minted as evidence, and the decay events its waves pressed
    // onto them. Both fold, and no row prints the word — a guess still reads
    // as a guess in `rule`, and every live row stands asserted.
    let words = |what: &str| text.lines().filter(|l| l.contains(what)).count();
    assert!(
        words("\"standing\":\"evidence\"") > 2000 && words("\"event\":\"decay\"") > 100,
        "the standing journal holds the rows and the events this is about: {} rows, {} events",
        words("\"standing\":\"evidence\""),
        words("\"event\":\"decay\"")
    );
    // The standing column, not the whole row: a `proved by` anchor may name a
    // test whose own name carries the word, and that says nothing about how
    // the row stands.
    for line in out.lines() {
        let standing = line.split_whitespace().nth(2);
        assert_eq!(standing, Some("asserted"), "{line}");
    }
}

// ---- one standing --------------------------------------------------------
//
// A link is a claim or it is not a link. The second standing left with the
// producer that minted it, and the machinery that served it left with it: no
// verb promotes a guess, no score decays one and no sweep prunes one. The
// rows stay, because the journal is append-only truth, and they load as the
// claims they now are
// (`archi/requirements/code-link/a-link-stands-asserted-or-it-does-not-stand.md`).

/// The one live row of `root`, as `link ls` prints it.
fn only_row(root: &Path) -> String {
    let out = ok(root, &["link", "ls"]);
    assert_eq!(out.lines().count(), 1, "one row stands here:\n{out}");
    out.lines().next().expect("the row").to_string()
}

/// The id of that one row — it opens the line.
fn only_id(root: &Path) -> String {
    only_row(root)
        .split_whitespace()
        .next()
        .expect("the id opens the row")
        .to_string()
}

/// Rewrite the journal as an older binary wrote it: the row it holds was
/// minted under the second standing, and a wave pressed a decay onto it. The
/// mint is real — the pins are the ones `link add` computed — and only the
/// word for the standing changes. Hands back the row's id.
fn as_an_older_binary_wrote_it(root: &Path) -> String {
    let id = only_id(root);
    let path = root.join("archi/links/journal.jsonl");
    let text = fs::read_to_string(&path).expect("the journal");
    assert!(text.contains("\"standing\":\"asserted\""), "{text}");
    let older = format!(
        "{}{{\"event\":\"decay\",\"id\":\"{id}\",\"task\":\"t9\",\"at\":\"2020-01-01T00:00:00Z\"}}\n",
        text.replace("\"standing\":\"asserted\"", "\"standing\":\"evidence\"")
    );
    fs::write(&path, older).unwrap();
    id
}

/// Every verb the second standing needed answers a usage error: nothing
/// promotes a row, nothing filters for a guess and nothing prunes one. The
/// help names none of them either
/// (`archi/requirements/code-link/a-link-stands-asserted-or-it-does-not-stand.md`).
#[test]
fn the_verbs_that_served_the_second_standing_answer_a_usage_error() {
    let (fixture, root) = bound("one-standing-surface");
    ok(&root, &[
        "link", "add", "Auth", "code/auth.rs#login", "--kind", "indirect",
    ]);
    let id = only_id(&root);

    for args in [
        vec!["link", "confirm", id.as_str()],
        vec!["link", "ls", "--evidence"],
        vec!["link", "audit", "--prune"],
    ] {
        let (success, out, err) = util::run(&root, &args);
        assert!(!success, "{args:?} is gone from the surface:\n{out}");
        assert!(err.contains("usage:"), "{args:?} answers a usage error:\n{err}");
    }

    // The usage the reader is handed names none of the three.
    let (_, _, err) = util::run(&root, &["link"]);
    for gone in ["confirm", "--evidence", "--prune"] {
        assert!(!err.contains(gone), "the usage still names `{gone}`:\n{err}");
    }

    // Nothing was written: the row stands exactly as it was minted.
    assert!(only_row(&root).contains("asserted"), "{}", only_row(&root));

    cleanup(&fixture);
}

/// The audit has two findings left — code no link claims, and spec no code
/// answers — and it names nothing else: no score, no decayed guess, and
/// nothing retired behind the reader's back
/// (`archi/requirements/code-link/a-link-stands-asserted-or-it-does-not-stand.md`,
/// `archi/requirements/code-link/the-audit-inverts-coverage.md`).
#[test]
fn the_audit_reports_dark_code_and_dark_spec_and_nothing_else() {
    let (fixture, root) = bound("one-standing-audit");
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["plan", "use", "mvp"]);
    ok(&root, &["plan", "task", "add", "Auth"]);
    ok(&root, &[
        "link", "add", "Auth", "code/auth.rs#login", "--kind", "literal",
    ]);

    // The row is one an older binary minted, and its anchor is gone: the
    // sweep that scored such a row would have called it decayed.
    as_an_older_binary_wrote_it(&root);
    fs::write(root.join("code/auth.rs"), "pub fn login() -> bool {\n    true\n}\n").unwrap();
    fs::write(
        root.join("code/rogue.rs"),
        "pub fn rogue(n: u8) -> u8 {\n    n + 7\n}\n",
    )
    .unwrap();

    let before = fs::read_to_string(root.join("archi/links/journal.jsonl")).unwrap();
    let out = ok(&root, &["link", "audit"]);
    assert!(out.contains("unaccounted delta: code/rogue.rs"), "{out}");
    assert!(
        out.contains("unlinked spec element: Gate.out wire Auth.inn"),
        "{out}"
    );
    for word in ["evidence", "confidence", "decayed", "pruned"] {
        assert!(!out.contains(word), "the audit says `{word}`:\n{out}");
    }

    // The sweep reads: the journal it read is the journal it leaves.
    let after = fs::read_to_string(root.join("archi/links/journal.jsonl")).unwrap();
    assert_eq!(before, after, "the audit retires nothing");

    cleanup(&fixture);
}

/// A journal an older binary wrote still folds: the row minted under the
/// second standing loads as the claim it now is, the decay event a wave
/// pressed onto it is read and skipped, and the record itself is never
/// rewritten to say so
/// (`archi/requirements/code-link/a-link-stands-asserted-or-it-does-not-stand.md`,
/// `archi/requirements/code-link/link-truth-is-append-only.md`).
#[test]
fn a_row_minted_as_evidence_loads_as_a_claim_and_its_decay_is_skipped() {
    let (fixture, root) = bound("one-standing-fold");
    ok(&root, &[
        "link", "add", "Auth", "code/auth.rs#login", "--kind", "literal",
    ]);
    let id = as_an_older_binary_wrote_it(&root);

    let row = only_row(&root);
    assert!(row.starts_with(&id), "{row}");
    assert!(row.contains("asserted"), "{row}");
    assert!(!row.contains("evidence"), "{row}");

    // The record still says what it always said: the fold maps the word on
    // read, and no migration rewrites append-only truth.
    let text = fs::read_to_string(root.join("archi/links/journal.jsonl")).unwrap();
    assert!(text.contains("\"standing\":\"evidence\""), "{text}");
    assert!(text.contains("\"event\":\"decay\""), "{text}");

    // The machine answer says it too, and the row carries no erosion.
    let rows: Value = serde_json::from_str(&ok(&root, &["link", "ls", "--json"])).unwrap();
    assert_eq!(rows[0]["standing"], Value::from("asserted"), "{rows}");
    assert!(rows[0].get("decays").is_none(), "{rows}");

    cleanup(&fixture);
}

/// The row grades exactly as any claim grades: clean while the code holds,
/// and failing the gate the moment the body it watches moves — where the
/// second standing never failed a verify at all
/// (`archi/requirements/code-link/a-link-stands-asserted-or-it-does-not-stand.md`,
/// `archi/requirements/code-link/verify-grades-every-claim.md`).
#[test]
fn a_row_minted_as_evidence_grades_exactly_as_a_claim() {
    let (fixture, root) = bound("one-standing-verify");
    ok(&root, &[
        "link", "add", "Auth", "code/auth.rs#login", "--kind", "literal",
    ]);
    as_an_older_binary_wrote_it(&root);

    let out = ok(&root, &["link", "verify"]);
    assert!(out.contains("clean"), "{out}");
    assert!(!out.contains("[failing]"), "{out}");
    assert!(
        !out.contains("journal:"),
        "the decay event is skipped, never absorbed with a note:\n{out}"
    );

    // The watched body moves: a claim fails the gate.
    fs::write(
        root.join("code/auth.rs"),
        "pub fn login(user: &str) -> bool {\n    !user.trim().is_empty()\n}\n",
    )
    .unwrap();
    let (success, out, err) = util::run(&root, &["link", "verify"]);
    assert!(!success, "a claim fails the gate on drift:\n{out}{err}");
    assert!(out.contains("drifted") && out.contains("[failing]"), "{out}");

    cleanup(&fixture);
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

// ---- the writer's declaration ------------------------------------------------
//
// The writer holds the answer, so the writer writes it down: one file beside
// the wave index, naming for each symbol the port or requirement it answers
// and the test that proves it. Capture reads that file and mints from it; the
// shared-term rule mints nothing at all
// (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`,
// `archi/requirements/code-link/a-declaration-names-the-test-that-proves-it.md`,
// `archi/requirements/planning/the-declaration-refusal-repairs-without-guessing.md`).

/// The test the declarations name. It is written before the wave opens, so it
/// sits in the wave-open index and is never a change of its own — and it sits
/// outside `code/auth.rs`, the one file the task's outputs claim.
const AUTH_TEST_RS: &str =
    "pub fn a_login_without_a_name_is_refused() {\n    assert!(true);\n}\n";

/// The declaration file of task `t1` in wave 1, where the wave already writes
/// its index.
const DECLARES: &str = "archi/plans/mvp/waves/w01.t1.declares.toml";

/// The test symbol every well-formed declaration below names.
const PROOF: &str = "code/auth_test.rs#a_login_without_a_name_is_refused";

/// A started plan with one task claiming `code/auth.rs`: the wave-open index
/// is written, so an edit under that output is the delta a capture reads.
fn started_plan(root: &Path) {
    write_requirement(root, REQ);
    fs::write(root.join("code/auth_test.rs"), AUTH_TEST_RS).unwrap();
    ok(root, &["version", "save", "-m", "first"]);
    ok(root, &["plan", "use", "mvp"]);
    ok(root, &["plan", "task", "add", "Auth"]);
    fs::write(
        root.join("archi/plans/mvp/t1-auth.md"),
        format!(
            "---\nnode: Auth\nowns: [{REQ}]\n---\n\n# t1 — Auth\n\nrealize the node\n\n\
             ## Spec\n\n- `Auth`\n- `Gate.out wire Auth.inn`\n\n\
             ## Inputs\n\n## Outputs\n\n- code/auth.rs\n\n## Stack\n\n## Verifications\n\n\
             ### {REQ}\n\n- test — proves the refusal\n"
        ),
    )
    .unwrap();
    ok(root, &["plan", "start"]);
}

/// The wave's delta: the symbol a declaration names moves, and a second one
/// moves beside it that no declaration names — and whose terms the old
/// shared-term rule matched against both of the task's refs.
fn change_auth(root: &Path) {
    fs::write(
        root.join("code/auth.rs"),
        "pub fn login(user: &str) -> bool {\n    !user.trim().is_empty()\n}\n\n\
         pub fn auth_gate() -> bool {\n    true\n}\n",
    )
    .unwrap();
}

/// Write `t1`'s declaration file, as its sub-agent does before it returns.
fn declare(root: &Path, text: &str) {
    fs::write(root.join(DECLARES), text).unwrap();
}

/// One `[[declares]]` table, spelled out field by field so a test can leave
/// one out or add one of its own.
fn entry(fields: &[(&str, &str)]) -> String {
    let mut out = String::from("[[declares]]\n");
    for (key, value) in fields {
        out.push_str(&format!("{key} = \"{value}\"\n"));
    }
    out
}

/// The whole declaration a well-formed file holds: the changed symbol, the
/// port it answers, and the test that proves it.
fn sound_entry() -> String {
    entry(&[
        ("symbol", "code/auth.rs#login"),
        ("answers", "Auth.inn"),
        ("proved_by", PROOF),
    ])
}

/// Run the wave's capture for `t1`, expecting the declaration to be refused;
/// hands back the refusal.
fn refused(root: &Path) -> String {
    let (success, out, err) = util::run(root, &["link", "capture", "--task", "t1"]);
    assert!(!success, "the declaration is refused:\n{out}");
    err
}

/// Every refusal a declaration raises is located: it names the file, the task
/// that owes it and the line it is about, and it quotes what stood on that
/// line. No refusal is a restatement of the grammar alone — each one names
/// the thing in *this* file that is wrong
/// (`archi/requirements/planning/the-declaration-refusal-repairs-without-guessing.md`).
fn located(err: &str, line: usize, quoted: &str) {
    assert!(err.contains(DECLARES), "names the file:\n{err}");
    assert!(err.contains("task `t1`"), "names the task:\n{err}");
    assert!(err.contains(&format!(":{line},")), "names line {line}:\n{err}");
    assert!(
        err.contains(&format!("{line} | {quoted}")),
        "quotes what stood on line {line}:\n{err}"
    );
}

/// The writer names the port and the requirement its symbol answers, and the
/// test that proves each; both land as asserted links on that symbol, both
/// read as declared, and the reverse view of the requirement hands back the
/// test beside the code. The symbol nobody named draws nothing, whatever it
/// resembles: nothing but a declaration mints
/// (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`,
/// `archi/requirements/code-link/a-declaration-names-the-test-that-proves-it.md`).
#[test]
fn a_declaration_mints_the_pair_asserted_and_the_reverse_view_names_the_test() {
    let (fixture, root) = bound("declares");
    started_plan(&root);
    change_auth(&root);
    declare(
        &root,
        &format!(
            "{}\n{}",
            sound_entry(),
            entry(&[
                ("symbol", "code/auth.rs#login"),
                ("answers", &format!("req:{REQ}")),
                ("proved_by", PROOF),
            ])
        ),
    );

    // A file with every field present and no unknown key is accepted with no
    // warning: capture says nothing about it beyond the rows it minted.
    let out = ok(&root, &["link", "capture", "--task", "t1"]);
    assert_eq!(out.matches("captured ").count(), 2, "{out}");
    assert!(!out.contains("declares.toml"), "{out}");
    assert!(!out.contains("note:"), "{out}");

    // What the file named is an asserted link on the symbol that named it, it
    // says a writer declared it, and it carries the test.
    let rows = ok(&root, &["link", "ls"]);
    assert_eq!(rows.lines().count(), 2, "{rows}");
    for line in rows.lines() {
        assert!(line.contains("asserted"), "{line}");
        assert_eq!(rule_word(line), Some("declared"), "{line}");
        assert!(line.contains("captured(t1)"), "{line}");
        assert!(line.contains("← code/auth.rs#login"), "{line}");
        assert!(line.contains(&format!("proved by {PROOF}")), "{line}");
    }
    assert!(rows.contains("Auth.inn ← code/auth.rs#login"), "{rows}");

    // The reader who doubts a pair has one place to go: the reverse view of
    // the requirement prints the test beside the code.
    let out = ok(&root, &["link", "ls", "--spec", &format!("req:{REQ}")]);
    assert_eq!(out.lines().count(), 1, "{out}");
    assert!(out.contains(&format!("proved by {PROOF}")), "{out}");

    // Nothing was minted for the symbol no declaration names.
    assert!(!rows.contains("auth_gate"), "{rows}");
    let json: Value =
        serde_json::from_str(&ok(&root, &["link", "capture", "--task", "t1", "--json"])).unwrap();
    // And the re-run is idempotent: a declaration already minted is not
    // minted twice.
    assert!(json["minted"].as_array().unwrap().is_empty(), "{json}");

    cleanup(&fixture);
}

/// A declared name that resolves against neither the model nor the
/// requirement set is refused where it stands, and the refusal names the name
/// (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`).
#[test]
fn a_declared_name_that_resolves_to_nothing_refuses_at_its_line() {
    let (fixture, root) = bound("unresolved");
    started_plan(&root);
    change_auth(&root);

    declare(
        &root,
        &entry(&[
            ("symbol", "code/auth.rs#login"),
            ("answers", "NoSuchNode"),
            ("proved_by", PROOF),
        ]),
    );
    let err = refused(&root);
    assert!(err.contains("`NoSuchNode`"), "names the name:\n{err}");
    located(&err, 3, "answers = \"NoSuchNode\"");

    // The requirement form refuses the same way, on the slug the file wrote.
    declare(
        &root,
        &entry(&[
            ("symbol", "code/auth.rs#login"),
            ("answers", "req:no-such-claim"),
            ("proved_by", PROOF),
        ]),
    );
    let err = refused(&root);
    assert!(err.contains("no-such-claim"), "names the slug:\n{err}");
    located(&err, 3, "answers = \"req:no-such-claim\"");

    // A symbol the tree does not hold is refused on its own line.
    declare(
        &root,
        &entry(&[
            ("symbol", "code/auth.rs#no_such_item"),
            ("answers", "Auth.inn"),
            ("proved_by", PROOF),
        ]),
    );
    let err = refused(&root);
    assert!(err.contains("no_such_item"), "names the symbol:\n{err}");
    located(&err, 2, "symbol = \"code/auth.rs#no_such_item\"");

    assert_eq!(ok(&root, &["link", "ls"]), "no links\n", "nothing was minted");

    cleanup(&fixture);
}

/// An edge in `answers` is refused, and the refusal names the ports behind it:
/// an edge is a caller, and the code behind a port does not know its callers
/// (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`).
#[test]
fn an_edge_in_answers_is_refused_and_the_refusal_names_its_ports() {
    let (fixture, root) = bound("edge");
    started_plan(&root);
    change_auth(&root);

    declare(
        &root,
        &entry(&[
            ("symbol", "code/auth.rs#login"),
            ("answers", "Gate.out wire Auth.inn"),
            ("proved_by", PROOF),
        ]),
    );
    let err = refused(&root);
    assert!(err.contains("`Gate.out`"), "names the source port:\n{err}");
    assert!(err.contains("`Auth.inn`"), "names the target port:\n{err}");
    located(&err, 3, "answers = \"Gate.out wire Auth.inn\"");
    assert_eq!(ok(&root, &["link", "ls"]), "no links\n", "nothing was minted");

    cleanup(&fixture);
}

/// The third name is checked like the other two: a test that resolves to
/// nothing refuses and the refusal names it, and a `proved_by` that stops at a
/// file names no test at all
/// (`archi/requirements/code-link/a-declaration-names-the-test-that-proves-it.md`).
#[test]
fn a_test_that_resolves_to_nothing_refuses_and_names_the_test() {
    let (fixture, root) = bound("proof");
    started_plan(&root);
    change_auth(&root);

    declare(
        &root,
        &entry(&[
            ("symbol", "code/auth.rs#login"),
            ("answers", "Auth.inn"),
            ("proved_by", "code/auth_test.rs#no_such_test"),
        ]),
    );
    let err = refused(&root);
    assert!(
        err.contains("code/auth_test.rs#no_such_test"),
        "names the test:\n{err}"
    );
    located(&err, 4, "proved_by = \"code/auth_test.rs#no_such_test\"");

    // A file is not a test: the name has to reach a symbol.
    declare(
        &root,
        &entry(&[
            ("symbol", "code/auth.rs#login"),
            ("answers", "Auth.inn"),
            ("proved_by", "code/auth_test.rs"),
        ]),
    );
    let err = refused(&root);
    assert!(err.contains("code/auth_test.rs"), "names the test:\n{err}");
    located(&err, 4, "proved_by = \"code/auth_test.rs\"");

    assert_eq!(ok(&root, &["link", "ls"]), "no links\n", "nothing was minted");

    cleanup(&fixture);
}

/// A malformed file is refused with the line, what was expected there and what
/// stood there — never with a restatement of the grammar alone, and never by
/// ignoring what it did not understand
/// (`archi/requirements/planning/the-declaration-refusal-repairs-without-guessing.md`).
#[test]
fn a_malformed_declaration_repairs_in_one_read() {
    let (fixture, root) = bound("malformed");
    started_plan(&root);
    change_auth(&root);

    // A missing field: the refusal says which field was expected and quotes
    // the entry that stood there without it.
    declare(
        &root,
        &entry(&[
            ("symbol", "code/auth.rs#login"),
            ("answers", "Auth.inn"),
        ]),
    );
    let err = refused(&root);
    assert!(err.contains("missing field `proved_by`"), "{err}");
    located(&err, 1, "[[declares]]");
    assert!(err.contains("2 | symbol = \"code/auth.rs#login\""), "{err}");
    assert!(err.contains("3 | answers = \"Auth.inn\""), "{err}");

    // An unknown key is refused rather than ignored, and the refusal names
    // the key and the keys that were expected.
    declare(
        &root,
        &format!("{}proves = \"code/auth_test.rs#gone\"\n", sound_entry()),
    );
    let err = refused(&root);
    assert!(err.contains("unknown field `proves`"), "{err}");
    assert!(err.contains("`symbol`"), "{err}");
    assert!(err.contains("`answers`"), "{err}");
    assert!(err.contains("`proved_by`"), "{err}");
    located(&err, 5, "proves = \"code/auth_test.rs#gone\"");

    // A misspelled table is an unknown key too — no entry is silently read as
    // no declaration.
    declare(&root, &sound_entry().replace("[[declares]]", "[[declare]]"));
    let err = refused(&root);
    assert!(err.contains("unknown field `declare`"), "{err}");
    located(&err, 1, "[[declare]]");

    // A value of the wrong type is located at the value, not at the table.
    declare(&root, &sound_entry().replace("\"Auth.inn\"", "3"));
    let err = refused(&root);
    assert!(err.contains("expected a string"), "{err}");
    located(&err, 3, "answers = 3");

    assert_eq!(ok(&root, &["link", "ls"]), "no links\n", "nothing was minted");

    cleanup(&fixture);
}

/// A wave whose task wrote no declaration file does not crash and does not
/// refuse: it mints nothing for that task, and it says so. The refusal on an
/// absent file is the wave gate's, and the wave gate is not this
/// (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`).
#[test]
fn an_absent_declaration_file_mints_nothing_and_says_so() {
    let (fixture, root) = bound("absent");
    started_plan(&root);
    change_auth(&root);

    let out = ok(&root, &["link", "capture", "--task", "t1"]);
    assert!(!out.contains("captured "), "{out}");
    assert!(out.contains("note: `t1` declares nothing"), "{out}");
    assert!(out.contains(DECLARES), "{out}");
    assert_eq!(ok(&root, &["link", "ls"]), "no links\n", "{out}");

    cleanup(&fixture);
}

// ---- repin --moved: the bulk consumer of exact candidates --------------------
//
// A crate rename orphans every link into it at once. The grader already
// proves each move — the body hash matches across the tree — and the pass is
// the missing consumer of that proof: every exact candidate accepted in one
// journal stroke per row, every inexact one reported and left with a person
// (`archi/requirements/code-link/an-exact-move-repins-in-one-pass.md`).

/// Two symbols in the one file the move fixtures rename.
const GUARD_RS: &str = "pub fn admit(user: &str) -> bool {\n    !user.is_empty()\n}\n\n\
                        pub fn expel(user: &str) -> bool {\n    user.is_empty()\n}\n";

/// A bound project with `code/guard.rs` linked at both symbols.
fn linked_guard(tag: &str) -> (PathBuf, PathBuf) {
    let (fixture, root) = bound(tag);
    fs::write(root.join("code/guard.rs"), GUARD_RS).unwrap();
    ok(&root, &["link", "add", "Gate", "code/guard.rs#admit", "--kind", "literal"]);
    ok(&root, &["link", "add", "Auth", "code/guard.rs#expel", "--kind", "literal"]);
    (fixture, root)
}

/// A renamed file's links repin to their exact candidates in one pass — one
/// row per repin, in `repin`'s own words — and a second run finds nothing
/// moved and appends nothing
/// (`archi/requirements/code-link/an-exact-move-repins-in-one-pass.md`).
#[test]
fn a_renamed_files_links_repin_to_their_exact_candidates_in_one_pass() {
    let (fixture, root) = linked_guard("moved-pass");
    fs::rename(root.join("code/guard.rs"), root.join("code/warden.rs")).unwrap();

    let out = ok(&root, &["link", "repin", "--moved"]);
    assert_eq!(
        out.lines().filter(|l| l.starts_with("repinned ")).count(),
        2,
        "{out}"
    );
    assert!(out.contains("Gate ← code/warden.rs#admit"), "{out}");
    assert!(out.contains("Auth ← code/warden.rs#expel"), "{out}");

    // The journal took the moves: the rows anchor at the new path, and the
    // whole set grades clean again.
    let ls = ok(&root, &["link", "ls"]);
    assert!(ls.contains("code/warden.rs#admit"), "{ls}");
    assert!(ls.contains("code/warden.rs#expel"), "{ls}");
    assert!(!ls.contains("guard.rs"), "{ls}");
    let verify = ok(&root, &["link", "verify"]);
    assert!(verify.contains("2 checked: 2 clean, 0 failing"), "{verify}");

    // The second run finds nothing moved and appends nothing.
    let journal = root.join("archi/links/journal.jsonl");
    let before = fs::read_to_string(&journal).unwrap();
    let again = ok(&root, &["link", "repin", "--moved"]);
    assert!(again.contains("no moved links"), "{again}");
    assert_eq!(fs::read_to_string(&journal).unwrap(), before, "{again}");

    cleanup(&fixture);
}

/// An inexact candidate is a judgement: the pass reports it — the row, the
/// candidate and the per-row repair addressed to this id — and never takes it
/// (`archi/requirements/code-link/an-exact-move-repins-in-one-pass.md`).
#[test]
fn an_inexact_candidate_is_reported_and_never_taken() {
    let (fixture, root) = bound("moved-inexact");
    fs::write(root.join("code/guard.rs"), GUARD_RS).unwrap();
    ok(&root, &["link", "add", "Gate", "code/guard.rs#admit", "--kind", "literal"]);
    let id = only_id(&root);

    // The body moves and changes in the same stroke: the new place holds
    // `admit`, but not the body the link pinned.
    fs::remove_file(root.join("code/guard.rs")).unwrap();
    fs::write(
        root.join("code/warden.rs"),
        "pub fn admit(user: &str) -> bool {\n    user.len() > 1\n}\n\n\
         pub fn expel(user: &str) -> bool {\n    user.is_empty()\n}\n",
    )
    .unwrap();

    let out = ok(&root, &["link", "repin", "--moved"]);
    assert!(!out.contains("repinned"), "{out}");
    assert!(out.contains(&format!("inexact  {id}")), "{out}");
    assert!(out.contains("candidate: `code/warden.rs#admit`"), "{out}");
    assert!(out.contains(&format!("link repin {id} --to")), "{out}");

    // Untouched: the row still anchors where it anchored, and verify still
    // grades the same move.
    assert!(only_row(&root).contains("code/guard.rs#admit"), "untouched");
    let verify = ok(&root, &["link", "verify"]);
    assert!(verify.contains("moved"), "{verify}");

    cleanup(&fixture);
}

/// Links grading clean, drifted or missing-without-candidate are not the
/// pass's rows: nothing is taken, the grading stands exactly as it stood,
/// and running the pass again changes exactly as little
/// (`archi/requirements/code-link/an-exact-move-repins-in-one-pass.md`).
#[test]
fn links_grading_anything_else_are_untouched_and_a_second_run_is_a_no_op() {
    let (fixture, root) = linked_guard("moved-else");
    ok(&root, &["link", "add", "Gate", "code/auth.rs#login", "--kind", "literal"]);
    // `admit` drifts in place; `expel` goes missing with no candidate
    // anywhere; `login` stays clean.
    fs::write(
        root.join("code/guard.rs"),
        "pub fn admit(user: &str) -> bool {\n    user.len() > 1\n}\n",
    )
    .unwrap();

    let (_, graded, _) = util::run(&root, &["link", "verify"]);
    for state in ["clean", "drifted", "missing"] {
        assert!(graded.contains(state), "{graded}");
    }
    let journal = root.join("archi/links/journal.jsonl");
    let before = fs::read_to_string(&journal).unwrap();

    let out = ok(&root, &["link", "repin", "--moved"]);
    assert!(out.contains("no moved links"), "{out}");
    let again = ok(&root, &["link", "repin", "--moved"]);
    assert!(again.contains("no moved links"), "{again}");

    assert_eq!(fs::read_to_string(&journal).unwrap(), before, "nothing appended");
    let (_, regraded, _) = util::run(&root, &["link", "verify"]);
    assert_eq!(regraded, graded, "every state held");

    cleanup(&fixture);
}

/// `repin <id> --moved` refuses, and the refusal names the two forms
/// (`archi/requirements/code-link/an-exact-move-repins-in-one-pass.md`).
#[test]
fn repin_refuses_an_id_and_moved_together_naming_the_two_forms() {
    let (fixture, root) = bound("moved-refusal");

    let (success, _, err) = util::run(&root, &["link", "repin", "l0001", "--moved"]);
    assert!(!success, "the two forms together are refused");
    let refusal = err.lines().next().unwrap_or_default();
    assert!(refusal.contains("repin <id>"), "{err}");
    assert!(refusal.contains("repin --moved"), "{err}");

    cleanup(&fixture);
}

/// `--json` carries the same rows as the render: the repinned links and the
/// reported inexact ones, one envelope
/// (`archi/requirements/code-link/an-exact-move-repins-in-one-pass.md`).
#[test]
fn the_json_envelope_carries_the_same_rows_as_the_render() {
    let (fixture, root) = linked_guard("moved-json");
    // One symbol moves verbatim, the other moves and changes: an exact and
    // an inexact candidate out of the same rename.
    fs::remove_file(root.join("code/guard.rs")).unwrap();
    fs::write(
        root.join("code/warden.rs"),
        "pub fn admit(user: &str) -> bool {\n    !user.is_empty()\n}\n\n\
         pub fn expel(user: &str) -> bool {\n    user.trim().is_empty()\n}\n",
    )
    .unwrap();

    let out = ok(&root, &["link", "repin", "--moved", "--json"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    let repinned = v["repinned"].as_array().unwrap();
    assert_eq!(repinned.len(), 1, "{out}");
    assert_eq!(repinned[0]["anchor"]["file"], "code/warden.rs", "{out}");
    assert_eq!(repinned[0]["anchor"]["symbol"], "admit", "{out}");
    let reported = v["reported"].as_array().unwrap();
    assert_eq!(reported.len(), 1, "{out}");
    assert_eq!(reported[0]["state"], "moved", "{out}");
    assert_eq!(reported[0]["exact"], false, "{out}");
    assert_eq!(reported[0]["link"]["anchor"]["file"], "code/guard.rs", "{out}");
    let note = reported[0]["note"].as_str().unwrap();
    assert!(note.contains("code/warden.rs#expel"), "{out}");

    cleanup(&fixture);
}
