//! End to end through the real binary: code in member repositories, spec in
//! its own — refs qualified `member//file#symbol`, absence graded
//! Unreachable and never decayed, baselines per member, the audit worded per
//! member, and the memberless project untouched
//! (`archi/requirements/multi-repo/`).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const MODEL: &str = "def conn wire := * -> *\n\
                     def node Gate:\n  port serve\n\
                     def node Ledger:\n  port keep\n\
                     Gate.serve wire Ledger.keep\n";

const SERVE_RS: &str = "pub fn serve_gate(x: u8) -> u8 {\n    x + 1\n}\n";

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-mrepo-e2e-{tag}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(&dir).unwrap();
    fs::canonicalize(&dir).unwrap()
}

fn spec_project(dir: &Path, manifest_extra: &str) {
    fs::create_dir_all(dir.join("archi/src")).unwrap();
    fs::write(
        dir.join("archi.toml"),
        format!("[project]\nname = \"t\"\n{manifest_extra}"),
    )
    .unwrap();
    fs::write(dir.join("archi/src/model.arch"), MODEL).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn member_repo(dir: &Path) {
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/lib.rs"), SERVE_RS).unwrap();
    git(dir, &["init", "-q"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-qm", "seed"]);
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
fn qualified_refs_run_the_whole_link_loop_across_members() {
    let ws = scratch("loop");
    let spec = ws.join("spec");
    let backend = ws.join("backend");
    spec_project(&spec, "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n");
    member_repo(&backend);

    // The doctor sees home and the member, both reachable.
    let ls = ok(&spec, &["repo", "ls"]);
    assert!(ls.contains("home"), "{ls}");
    assert!(ls.contains("backend"), "{ls}");
    assert!(!ls.contains("unreachable"), "{ls}");

    // A qualified add round-trips through render and journal.
    let added = ok(&spec, &[
        "link", "add", "Gate", "backend//src/lib.rs#serve_gate", "--kind", "indirect",
    ]);
    assert!(added.contains("Gate ← backend//src/lib.rs#serve_gate"), "{added}");
    let journal = fs::read_to_string(spec.join("archi/links/journal.jsonl")).unwrap();
    assert!(journal.contains("\"repo\":\"backend\""), "{journal}");
    assert!(journal.contains("\"file\":\"src/lib.rs\""), "{journal}");

    // Clean while the interface stands; body churn stays clean (indirect);
    // an interface move grades Drifted.
    let verify = ok(&spec, &["link", "verify"]);
    assert!(verify.contains("clean"), "{verify}");
    fs::write(backend.join("src/lib.rs"), SERVE_RS.replace("x + 1", "x + 2")).unwrap();
    let verify = ok(&spec, &["link", "verify"]);
    assert!(verify.contains("clean"), "{verify}");
    fs::write(
        backend.join("src/lib.rs"),
        SERVE_RS.replace("(x: u8) -> u8", "(x: u8, y: u8) -> u8"),
    )
    .unwrap();
    // Drifted fails only asserted literal links
    // (requirements/code-link.md#verify-and-drift): the indirect drift is
    // reported, qualified, and exits 0.
    let (success, verify, _) = run(&spec, &["link", "verify"]);
    assert!(success, "{verify}");
    assert!(verify.contains("drifted"), "{verify}");
    assert!(verify.contains("backend//src/lib.rs#serve_gate"), "{verify}");
    assert!(verify.contains("the declared shape moved"), "{verify}");
}

#[test]
fn absence_is_reported_never_decayed_and_fails_only_in_scope() {
    let ws = scratch("absence");
    let spec = ws.join("spec");
    let backend = ws.join("backend");
    spec_project(&spec, "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n");
    member_repo(&backend);
    ok(&spec, &[
        "link", "add", "Gate", "backend//src/lib.rs#serve_gate", "--kind", "indirect",
    ]);

    // The checkout leaves; the link grades Unreachable, verify exits 0.
    fs::remove_dir_all(&backend).unwrap();
    let (success, out, _) = run(&spec, &["link", "verify"]);
    assert!(success, "absence never fails an unscoped verify:\n{out}");
    assert!(out.contains("unreachable"), "{out}");
    assert!(out.contains("archi repo map backend"), "{out}");

    // No decay observation is journaled by looking at nothing; audit
    // neither grades nor prunes what it cannot see.
    let journal = fs::read_to_string(spec.join("archi/links/journal.jsonl")).unwrap();
    assert!(!journal.contains("\"decay\""), "{journal}");
    let (_, audit, _) = run(&spec, &["link", "audit", "--prune"]);
    assert!(audit.contains("unreachable"), "{audit}");
    assert!(!audit.contains("pruned"), "{audit}");
    let after = fs::read_to_string(spec.join("archi/links/journal.jsonl")).unwrap();
    assert!(!after.contains("\"retire\""), "{after}");

    // The explicit ask is the one place absence fails.
    let (success, _, err) = run(&spec, &["link", "verify", "--repo", "backend"]);
    assert!(!success, "inside --repo scope absence is the error");
    assert!(err.contains("repo map backend"), "{err}");

    // An undeclared member in the journal names the renamed-member recovery.
    fs::write(
        spec.join("archi.toml"),
        "[project]\nname = \"t\"\n[[repo]]\nname = \"core\"\npath = \"../backend\"\n",
    )
    .unwrap();
    let (success, out, _) = run(&spec, &["link", "verify"]);
    assert!(success, "{out}");
    assert!(out.contains("does not declare"), "{out}");
    assert!(out.contains("restore its [[repo]] row"), "{out}");
}

#[test]
fn baselines_and_audit_go_per_member() {
    let ws = scratch("audit");
    let spec = ws.join("spec");
    let backend = ws.join("backend");
    spec_project(&spec, "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n");
    member_repo(&backend);

    // Save while the member is clean: its baseline lands; home has no
    // commit yet and the report says so per member.
    let saved = ok(&spec, &["version", "save", "-m", "first"]);
    assert!(saved.contains("baseline backend:"), "{saved}");
    let index = fs::read_to_string(spec.join("archi/versions/index.toml")).unwrap();
    assert!(index.contains("[version.commits.backend]"), "{index}");
    assert!(index.contains("born = \"save\""), "{index}");

    // New code in the member since its baseline: the audit tags the dark
    // delta with the qualified path and notes home's missing source apart.
    fs::write(backend.join("src/extra.rs"), "pub fn extra() {}\n").unwrap();
    let (_, audit, _) = run(&spec, &["link", "audit"]);
    assert!(audit.contains("backend//src/extra.rs"), "{audit}");
    assert!(audit.contains("no delta source for home"), "{audit}");

    // A dirty-at-save member gets no baseline; committed and anchored, the
    // baseline is anchor-born and the audit words the window honestly.
    fs::write(spec.join("archi/src/model.arch"), format!("{MODEL}def node Extra\n")).unwrap();
    let saved = ok(&spec, &["version", "save", "-m", "second"]);
    assert!(saved.contains("no baseline for `backend`"), "{saved}");
    assert!(saved.contains("dirty"), "{saved}");
    git(&backend, &["add", "-A"]);
    git(&backend, &["commit", "-qm", "extra lands"]);
    let anchored = ok(&spec, &["version", "anchor", "--repo", "backend"]);
    assert!(anchored.contains("anchor-born"), "{anchored}");
    let (_, audit, _) = run(&spec, &["link", "audit"]);
    assert!(audit.contains("anchor-born"), "{audit}");
    assert!(audit.contains("unaudited"), "{audit}");
}

#[test]
fn a_home_rooted_below_its_git_root_rebases_the_audit() {
    // The monorepo shape the CLI blesses: archi.toml two directories below
    // the git root — git speaks top-relative paths, the audit must rebase
    // them (the silent mismatch, now a covered case).
    let repo = scratch("nested");
    git(&repo, &["init", "-q"]);
    let spec = repo.join("tools").join("plan");
    spec_project(&spec, "");
    fs::write(spec.join("gadget.rs"), "pub fn gadget() {}\n").unwrap();
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "seed"]);
    ok(&spec, &["version", "save", "-m", "first"]);
    ok(&spec, &["version", "anchor"]);

    fs::write(
        spec.join("gadget.rs"),
        "pub fn gadget() {}\npub fn widget() {}\n",
    )
    .unwrap();
    let (_, audit, _) = run(&spec, &["link", "audit"]);
    assert!(
        audit.contains("unaccounted delta: gadget.rs"),
        "the hunk arrives in the project's frame, not the git root's:\n{audit}"
    );
    assert!(!audit.contains("tools/plan/gadget.rs"), "{audit}");
}

#[test]
fn a_memberless_project_is_todays_byte_for_byte() {
    let spec = scratch("plain");
    spec_project(&spec, "");

    // The doctor shows home alone; bare refs render bare; the audit keeps
    // its original no-delta-source wording.
    let ls = ok(&spec, &["repo", "ls"]);
    assert_eq!(ls.lines().count(), 1, "{ls}");
    assert!(ls.starts_with("home"), "{ls}");
    fs::write(spec.join("gate.rs"), SERVE_RS).unwrap();
    let added = ok(&spec, &["link", "add", "Gate", "gate.rs#serve_gate", "--kind", "indirect"]);
    assert!(added.contains("Gate ← gate.rs#serve_gate"), "{added}");
    assert!(!added.contains("//"), "{added}");
    let journal = fs::read_to_string(spec.join("archi/links/journal.jsonl")).unwrap();
    assert!(!journal.contains("\"repo\""), "no member field on home anchors: {journal}");
    let (_, audit, _) = run(&spec, &["link", "audit"]);
    assert!(
        audit.contains("no delta source: commit the tree"),
        "the memberless note is today's, word for word:\n{audit}"
    );
    // A save writes no commits table.
    ok(&spec, &["version", "save", "-m", "first"]);
    let index = fs::read_to_string(spec.join("archi/versions/index.toml")).unwrap();
    assert!(!index.contains("commits"), "{index}");
}
