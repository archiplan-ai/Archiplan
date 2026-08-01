//! End to end through the real binary: one command searches everything
//! (`archi/requirements/agent-retrieval/one-verb-searches-everything`), a
//! dark corpus stays partial and never breaks the exit code, advisory doc
//! states search fine, and a search perturbs nothing a `version save`
//! could notice.

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const MODEL: &str = "def node AuthService: // password login for the api\n  port handle_login\n\
                     def node RateLimiter // sheds the replay burst before hashing\n\
                     Service type_of AuthService\n";

fn temp_project() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-search-e2e-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(dir.join("archi/src")).unwrap();
    fs::write(
        dir.join("archi.toml"),
        "[project]\nname = \"t\"\npreset = \"default\"\n",
    )
    .unwrap();
    fs::write(dir.join("archi/src/model.arch"), MODEL).unwrap();
    util::worktree(&dir)
}

fn put(root: &Path, rel_path: &str, text: &str) {
    let path = root.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// The doc side: an intent, a satisfied requirement, a deferred one, a
/// closed round with its breaking stressor.
fn docs(root: &Path) {
    put(
        root,
        "archi/requirements/secure-auth/secure-auth.md",
        "# Secure auth\n\nPassword authentication that leaks nothing.\n",
    );
    put(
        root,
        "archi/requirements/secure-auth/rate-limit-logins.md",
        "---\nkind: functional\norigin: stressor(credential-stuffing)\nsatisfied-by: [RateLimiter]\ndeferred:\n---\n\n# Rate limit logins\n\nRate limiting sheds the replay burst.\n\n## System Context\n\n## Satisfy\n\n`RateLimiter` sheds the burst.\n\n- test — replay a burst, organic logins stay fast\n",
    );
    put(
        root,
        "archi/requirements/secure-auth/token-rotation.md",
        "---\nkind: functional\norigin: intent\nsatisfied-by: []\ndeferred: postponed\n---\n\n# Token rotation\n\nKeys rotate on a fixed cadence.\n\n## System Context\n\n## Satisfy\n",
    );
    put(
        root,
        "archi/stress/auth-hardening/auth-hardening.md",
        "---\nversion: v0001\nclosed: v0001\n---\n\n# Auth hardening\n\nFirst adversarial round.\n",
    );
    put(
        root,
        "archi/stress/auth-hardening/credential-stuffing.md",
        "---\naffects: [AuthService]\noutcome: breaking\n---\n\n# Credential stuffing\n\nBots replay leaked pairs at 100x the organic rate limiting budget.\n\n## Attractor\n\nThe login path saturates.\n\n## Resolution\n\nRate limiting takes the burst off the hot path.\n",
    );
}

fn run(root: &Path, args: &[&str]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_archi"))
        .args(args)
        .args(["--project", root.to_str().unwrap()])
        .output()
        .expect("archi runs");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn ok(root: &Path, args: &[&str]) -> String {
    let (success, stdout, stderr) = run(root, args);
    assert!(success, "archi {args:?} failed:\n{stdout}\n{stderr}");
    stdout
}

#[test]
fn one_verb_spans_kinds_narrows_bounds_and_speaks_json() {
    let root = temp_project();
    docs(&root);

    // One phrase, hits across kinds, each with its address.
    let out = ok(&root, &["search", "rate", "limiting"]);
    assert!(out.contains("element     RateLimiter"), "{out}");
    assert!(out.contains("requirement rate-limit-logins"), "{out}");
    assert!(out.contains("stressor    credential-stuffing"), "{out}");
    assert!(out.contains("rate-limit-logins.md:8"), "{out}");

    // --kind narrows; --limit bounds.
    let out = ok(&root, &["search", "rate", "limiting", "--kind", "requirement"]);
    assert!(!out.contains("element"), "{out}");
    let out = ok(&root, &["search", "rate", "limiting", "--limit", "1"]);
    assert_eq!(out.lines().filter(|l| !l.starts_with(' ')).count(), 1, "{out}");

    // The JSON envelope: status, query, dark, hits — and it repeats
    // byte-identically (same tree, same phrase).
    let a = ok(&root, &["search", "versioning", "--kind", "requirement", "--limit", "3", "--json"]);
    let b = ok(&root, &["search", "versioning", "--kind", "requirement", "--limit", "3", "--json"]);
    assert_eq!(a, b);
    let v: serde_json::Value = serde_json::from_str(&a).unwrap();
    assert_eq!(v["status"], "ok");
    assert_eq!(v["query"], "versioning");
    assert_eq!(v["dark"].as_array().unwrap().len(), 0);
    for h in v["hits"].as_array().unwrap() {
        assert_eq!(h["kind"], "requirement");
    }

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_dark_model_keeps_doc_hits_and_the_exit_stays_zero() {
    let root = temp_project();
    docs(&root);
    // Break the model: an edge onto a node that does not exist.
    fs::write(
        root.join("archi/src/model.arch"),
        format!("{MODEL}Ghost.out wire Phantom.inn\n"),
    )
    .unwrap();

    // check exits one; search still answers from the docs, exit zero,
    // dark names the model corpus.
    let (success, _, _) = run(&root, &["check"]);
    assert!(!success);
    let out = ok(&root, &["search", "rate", "limiting", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(
        v["dark"][0]
            .as_str()
            .unwrap()
            .starts_with("model: it does not compile"),
        "{out}"
    );
    assert!(!v["hits"].as_array().unwrap().is_empty(), "{out}");
    assert!(
        v["hits"]
            .as_array()
            .unwrap()
            .iter()
            .all(|h| h["kind"] != "element"),
        "{out}"
    );
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn advisory_states_search_fine_and_a_save_still_reports_unchanged() {
    let root = temp_project();
    docs(&root);

    // The KB carries findings (a deferred requirement); the command is
    // unbothered — advisory states are content, never blockers.
    let out = ok(&root, &["search", "token", "rotation"]);
    assert!(out.contains("state: deferred"), "{out}");

    // A search perturbs nothing a save could notice: save, search, save —
    // the second save still sees the model unchanged since the first.
    ok(&root, &["version", "save", "-m", "first"]);
    ok(&root, &["search", "rate", "limiting"]);
    let (_, stdout, stderr) = run(&root, &["version", "save", "-m", "again"]);
    assert!(
        (stdout.clone() + &stderr).contains("unchanged since v0001"),
        "{stdout}\n{stderr}"
    );
    fs::remove_dir_all(&root).unwrap();
}
