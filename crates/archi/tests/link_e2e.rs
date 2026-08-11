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
