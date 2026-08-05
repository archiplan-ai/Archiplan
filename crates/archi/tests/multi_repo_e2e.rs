//! End to end through the real binary: code in member repositories, spec in
//! its own — refs qualified `member//file#symbol`, absence graded
//! Unreachable and never decayed, baselines per member, the audit worded per
//! member, and the memberless project untouched
//! (`archi/requirements/multi-repo/`).

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use util::{git, ok, run};

const MODEL: &str = "def conn wire := * -> *\n\
                     def node Gate:\n  port serve\n\
                     def node Ledger:\n  port keep\n\
                     Gate.serve wire Ledger.keep\n";

const SERVE_RS: &str = "pub fn serve_gate(x: u8) -> u8 {\n    x + 1\n}\n";

fn scratch(tag: &str) -> PathBuf {
    util::scratch("archi-mrepo-e2e", tag)
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

fn git_out(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn member_repo(dir: &Path) {
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/lib.rs"), SERVE_RS).unwrap();
    git(dir, &["init", "-q"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-qm", "seed"]);
}

/// Mint the spec worktree and map each member to its real checkout — its
/// relative `[[repo]]` paths resolve nowhere from the worktree.
fn worktree_mapped(spec: &Path, members: &[(&str, &Path)]) -> PathBuf {
    let wt = util::worktree(spec);
    for (name, dir) in members {
        ok(&wt, &["repo", "map", name, dir.to_str().unwrap()]);
    }
    wt
}

#[test]
fn qualified_refs_run_the_whole_link_loop_across_members() {
    let ws = scratch("loop");
    let spec = ws.join("spec");
    let backend = ws.join("backend");
    spec_project(&spec, "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n");
    member_repo(&backend);
    let spec = worktree_mapped(&spec, &[("backend", &backend)]);

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
    // (archi/requirements/code-link/verify-grades-every-claim.md): the indirect drift is
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
    let spec = worktree_mapped(&spec, &[("backend", &backend)]);
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
    // The overlay goes with the declaration — a row naming an undeclared
    // member is its own loud error, not the one under test.
    fs::remove_file(spec.join("archi/repos.local.toml")).unwrap();
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
    let spec = worktree_mapped(&spec, &[("backend", &backend)]);
    // Home stays dirty at save — no home baseline, the per-member wording
    // below keeps its subject.
    fs::write(spec.join("wip.txt"), "uncommitted\n").unwrap();

    // Save while the member is clean: its baseline lands; home has no
    // commit and the report says so per member.
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
    // The monorepo worktree: home sits two directories below the worktree
    // root exactly as it sat below the git root.
    ok(&spec, &["worktree", "mint", "wt"]);
    let name = repo.file_name().unwrap().to_str().unwrap();
    let wt = repo.parent().unwrap().join(format!("{name}-worktrees")).join("wt");
    let spec = wt.join("tools").join("plan");
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
    let spec = util::worktree(&spec);

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

#[test]
fn map_refuses_a_linked_worktree_and_names_the_main_checkout() {
    let ws = scratch("wtgate");
    let spec = ws.join("spec");
    let backend = ws.join("backend");
    spec_project(&spec, "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n");
    member_repo(&backend);
    let spec = util::worktree(&spec);

    // The field incident's shape: a scratch worktree of the member offered
    // as its mapping. The row would outlive the worktree and a later mint
    // would base worktrees on the dead branch standing there.
    let wt = ws.join("backend-scratch");
    git(&backend, &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "feature/dead"]);
    let (success, out, err) = run(&spec, &["repo", "map", "backend", wt.to_str().unwrap()]);
    assert!(!success, "a linked worktree must not become a mapping:\n{out}\n{err}");
    assert!(err.contains("archi: "), "{err}");
    assert!(err.contains("linked worktree"), "{err}");
    assert!(err.contains("`feature/dead`"), "the standing branch is named:\n{err}");
    assert!(
        err.contains(backend.to_str().unwrap()),
        "the main checkout is the ready repair:\n{err}"
    );
    assert!(err.contains("archi repo map backend"), "{err}");
    let overlay = spec.join("archi/repos.local.toml");
    let row_written =
        overlay.exists() && fs::read_to_string(&overlay).unwrap().contains("backend");
    assert!(!row_written, "the refusal wrote no overlay row");

    // The named repair — the main checkout — maps exactly as before.
    let mapped = ok(&spec, &["repo", "map", "backend", backend.to_str().unwrap()]);
    assert!(mapped.contains("mapped backend ->"), "{mapped}");
    assert!(
        fs::read_to_string(&overlay).unwrap().contains("backend"),
        "the main-checkout row lands"
    );
    let ls = ok(&spec, &["repo", "ls"]);
    assert!(!ls.contains("unreachable"), "{ls}");
}

#[test]
fn check_reports_an_unresolved_member_path_from_either_source() {
    let ws = scratch("chk-path");
    let spec = ws.join("spec");
    let backend = ws.join("backend");
    spec_project(
        &spec,
        "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n\
         [[repo]]\nname = \"frontend\"\npath = \"../frontend\"\n",
    );
    member_repo(&backend);
    let spec = worktree_mapped(&spec, &[("backend", &backend)]);

    // backend's mapped checkout leaves; frontend's manifest convention
    // resolves nowhere from the worktree. Both decay modes are advisory: the
    // check still exits 0.
    fs::remove_dir_all(&backend).unwrap();
    let (success, out, err) = run(&spec, &["check"]);
    assert!(success, "member findings never fail the check:\n{out}\n{err}");
    assert!(out.contains("unresolved member path: `backend`"), "{out}");
    assert!(
        out.contains("archi/repos.local.toml"),
        "the overlay is named as backend's source:\n{out}"
    );
    assert!(out.contains("archi repo map backend"), "{out}");
    assert!(out.contains("unresolved member path: `frontend`"), "{out}");
    assert!(
        out.contains("archi.toml `path`"),
        "the manifest is named as frontend's source:\n{out}"
    );
    assert!(out.contains("archi repo map frontend"), "{out}");
}

#[test]
fn check_reports_a_linked_worktree_standing_in_the_map() {
    let ws = scratch("chk-wt");
    let spec = ws.join("spec");
    let backend = ws.join("backend");
    spec_project(&spec, "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n");
    member_repo(&backend);
    let spec = worktree_mapped(&spec, &[("backend", &backend)]);

    // The map decays under the write-time gate: a scratch worktree comes to
    // stand where the row points (the row predates the worktree).
    let wt = ws.join("backend-scratch");
    git(&backend, &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "feature/dead"]);
    fs::write(
        spec.join("archi/repos.local.toml"),
        format!("backend = {:?}\n", wt.to_str().unwrap()),
    )
    .unwrap();
    let (success, out, err) = run(&spec, &["check"]);
    assert!(success, "{out}\n{err}");
    assert!(out.contains("linked worktree: `backend`"), "{out}");
    assert!(out.contains("`feature/dead`"), "the standing branch is named:\n{out}");
    assert!(
        out.contains("archi/repos.local.toml"),
        "the overlay is named as the source:\n{out}"
    );
    assert!(
        out.contains(&format!("archi repo map backend {}", backend.display())),
        "the main checkout is the ready repair:\n{out}"
    );
}

#[test]
fn check_reports_a_wrong_clone_and_lets_absent_urls_be() {
    let ws = scratch("chk-url");
    let spec = ws.join("spec");
    let backend = ws.join("backend");
    let frontend = ws.join("frontend");
    spec_project(
        &spec,
        "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n\
         url = \"https://github.com/acme/backend.git\"\n\
         [[repo]]\nname = \"frontend\"\npath = \"../frontend\"\n\
         url = \"https://github.com/acme/frontend.git\"\n",
    );
    member_repo(&backend);
    member_repo(&frontend);
    git(&backend, &["remote", "add", "origin", "git@github.com:acme/other.git"]);
    // frontend declares a url but its checkout has no origin remote —
    // absence is not drift, that probe stays silent.
    let spec = worktree_mapped(&spec, &[("backend", &backend), ("frontend", &frontend)]);

    let (success, out, err) = run(&spec, &["check"]);
    assert!(success, "{out}\n{err}");
    assert!(out.contains("wrong clone: `backend`"), "{out}");
    assert!(
        out.contains("a different clone stands at the mapped path"),
        "{out}"
    );
    assert!(
        out.contains("https://github.com/acme/backend.git"),
        "the declared url arrives verbatim:\n{out}"
    );
    assert!(
        out.contains("git@github.com:acme/other.git"),
        "origin's url arrives verbatim:\n{out}"
    );
    assert!(out.contains("archi.toml declares"), "the source is named:\n{out}");
    assert_eq!(
        out.matches("wrong clone").count(),
        1,
        "frontend's originless checkout is not drift:\n{out}"
    );
}

#[test]
fn check_reports_a_squash_stranded_baseline() {
    let ws = scratch("chk-base");
    let spec = ws.join("spec");
    let backend = ws.join("backend");
    spec_project(&spec, "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n");
    member_repo(&backend);
    let spec = worktree_mapped(&spec, &[("backend", &backend)]);
    let saved = ok(&spec, &["version", "save", "-m", "first"]);
    assert!(saved.contains("baseline backend:"), "{saved}");
    let index = fs::read_to_string(spec.join("archi/versions/index.toml")).unwrap();
    let sha7 = index.split("sha = \"").nth(1).unwrap()[..7].to_string();

    // History rewrites out from under the record: a new root replaces the
    // branch and the old one retires — the recorded sha may still exist as
    // an object, but `branch --contains` answers empty, the squashed-landing
    // shape.
    let old = git_out(&backend, &["rev-parse", "--abbrev-ref", "HEAD"]);
    git(&backend, &["checkout", "-q", "--orphan", "fresh"]);
    git(&backend, &["commit", "-qm", "squashed root"]);
    git(&backend, &["branch", "-D", &old]);

    let (success, out, err) = run(&spec, &["check"]);
    assert!(success, "{out}\n{err}");
    assert!(out.contains("stranded baseline: `backend`"), "{out}");
    assert!(
        out.contains("the baseline sits on no branch — a squashed landing"),
        "{out}"
    );
    assert!(out.contains(&sha7), "the recorded sha7 is named:\n{out}");
    assert!(out.contains("archi version anchor --repo backend"), "{out}");
}

#[test]
fn a_healthy_member_map_adds_nothing_to_check() {
    let ws = scratch("chk-healthy");
    let spec = ws.join("spec");
    let backend = ws.join("backend");
    spec_project(
        &spec,
        "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n\
         url = \"https://GitHub.com/acme/backend.git\"\n",
    );
    member_repo(&backend);
    // origin is the ssh form of the declared https url: normalization meets
    // in the middle, no drift.
    git(&backend, &["remote", "add", "origin", "ssh://git@github.com/acme/backend/"]);
    let spec = worktree_mapped(&spec, &[("backend", &backend)]);
    // A recorded baseline standing on its branch — the healthy shape of the
    // fourth probe too.
    let saved = ok(&spec, &["version", "save", "-m", "first"]);
    assert!(saved.contains("baseline backend:"), "{saved}");

    let out = ok(&spec, &["check"]);
    assert!(
        out.contains("no findings"),
        "a healthy map adds nothing to the report:\n{out}"
    );
}

#[test]
fn a_dangling_baseline_degrades_alone() {
    let ws = scratch("dangling");
    let spec = ws.join("spec");
    let backend = ws.join("backend");
    let frontend = ws.join("frontend");
    spec_project(
        &spec,
        "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n\
         [[repo]]\nname = \"frontend\"\npath = \"../frontend\"\n",
    );
    member_repo(&backend);
    member_repo(&frontend);
    let spec = worktree_mapped(&spec, &[("backend", &backend), ("frontend", &frontend)]);

    // Both baselines land on clean members.
    let saved = ok(&spec, &["version", "save", "-m", "first"]);
    assert!(saved.contains("baseline backend:"), "{saved}");
    assert!(saved.contains("baseline frontend:"), "{saved}");

    // backend collects its baseline commit out from under the record: an amend
    // orphans it, gc prunes it — the same object-database hole a shallow clone
    // or a rebase leaves. frontend stays whole and gains a dark delta.
    fs::write(backend.join("src/lib.rs"), "pub fn serve_gate(x: u8) -> u8 { x + 9 }\n").unwrap();
    git(&backend, &["add", "-A"]);
    git(&backend, &["commit", "--amend", "--no-edit", "-q"]);
    git(&backend, &["reflog", "expire", "--expire=now", "--all"]);
    git(&backend, &["gc", "--prune=now", "-q"]);
    fs::write(frontend.join("src/extra.rs"), "pub fn extra() {}\n").unwrap();

    // The audit no longer aborts on the missing object: it names backend's
    // unresolvable baseline and still surfaces frontend's dark delta — each
    // member degrades alone (`multi-repo/an-unresolvable-baseline-says-so`).
    let (success, audit, err) = run(&spec, &["link", "audit"]);
    assert!(success, "audit aborted instead of degrading:\n{audit}\n{err}");
    assert!(!err.contains("bad object"), "raw git error leaked to stderr: {err}");
    assert!(
        audit.contains("backend") && audit.contains("does not resolve"),
        "backend's unresolvable baseline was not named:\n{audit}"
    );
    assert!(
        audit.contains("frontend//src/extra.rs"),
        "frontend's delta was lost to backend's failure:\n{audit}"
    );
}
