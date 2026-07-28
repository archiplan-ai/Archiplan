//! End to end through the real binary: the worktree seat discipline —
//! mutation runs only inside a seated worktree (unconditionally: the guard
//! sits at the router), protected branches refuse local merges, the
//! registry moves by verbs, context follows the checkout
//! (`archi/requirements/worktree-parallelism/`).

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const MODEL: &str = "def conn wire := * -> *\n\
                     def node Gate:\n  port serve\n\
                     def node Ledger:\n  port keep\n\
                     Gate.serve wire Ledger.keep\n";

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-wt-e2e-{tag}-{}-{}",
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

/// Run a save on `main` the disciplined way: a bootstrap seat maps the
/// members, saves, commits, and merges back — mutations never run unbound,
/// and the retire leaves the registry empty.
fn seat_save(spec: &Path, slug: &str, map: &[(&str, &Path)], msg: &str) {
    ok(spec, &["worktree", "mint", slug]);
    let name = spec.file_name().unwrap().to_str().unwrap();
    let wt = spec.parent().unwrap().join(format!("{name}-worktrees")).join(slug);
    for (member, dir) in map {
        ok(&wt, &["repo", "map", member, dir.to_str().unwrap()]);
    }
    ok(&wt, &["version", "save", "-m", msg]);
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", msg]);
    ok(spec, &["worktree", "merge", slug]);
}

/// A committed spec repo on `main` with one saved version, an empty
/// registry, and no protected list — local merges run free.
fn open_repo(tag: &str) -> (PathBuf, PathBuf) {
    let ws = scratch(tag);
    let spec = ws.join("spec");
    spec_project(&spec, "");
    git(&spec, &["init", "-q", "-b", "main"]);
    // repo-local identity: the binary's own `git merge` commits with it
    git(&spec, &["config", "user.email", "t@t"]);
    git(&spec, &["config", "user.name", "t"]);
    git(&spec, &["config", "commit.gpgsign", "false"]);
    git(&spec, &["add", "-A"]);
    git(&spec, &["commit", "-qm", "seed"]);
    seat_save(&spec, "boot", &[], "seed");
    (ws, spec)
}

/// [`open_repo`] plus `protected = ["main"]` — `main` refuses local merges.
fn protected_repo(tag: &str) -> (PathBuf, PathBuf) {
    let (ws, spec) = open_repo(tag);
    spec_project(&spec, "protected = [\"main\"]\n");
    git(&spec, &["add", "-A"]);
    git(&spec, &["commit", "-qm", "protection"]);
    (ws, spec)
}

#[test]
fn an_unbound_checkout_mints_the_seat_and_the_worktree_proceeds() {
    // No protected list: the discipline is unconditional — any unbound
    // checkout refuses, the primary on `main` included.
    let (_ws, spec) = open_repo("guard");

    // Mutation on main refuses, mints, and prints the path to enter.
    let (success, _out, err) = run(&spec, &["plan", "use", "auth"]);
    assert!(!success, "mutation on an unbound checkout must refuse");
    assert!(err.contains("unbound"), "{err}");
    assert!(err.contains("seated worktree"), "{err}");
    assert!(err.contains("archi/auth"), "{err}");
    assert!(err.contains("never changes your directory"), "{err}");
    let wt = spec.parent().unwrap().join("spec-worktrees/auth");
    assert!(wt.is_dir(), "the worktree was minted");

    // The main checkout never switched.
    let list = Command::new("git")
        .args(["-C", spec.to_str().unwrap(), "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&list.stdout).trim(), "main");

    // The same verb from the minted worktree proceeds and binds.
    let out = ok(&wt, &["plan", "use", "auth"]);
    assert!(out.contains("created plan `auth`"), "{out}");
    let ls = ok(&spec, &["worktree", "ls", "--plan", "auth"]);
    assert!(ls.contains("plan auth"), "{ls}");
    assert!(ls.contains("archi/auth"), "{ls}");

    // Any other checkout mutating the bound plan refuses with the owner.
    let (success, _out, err) = run(&spec, &["plan", "use", "auth"]);
    assert!(!success);
    assert!(err.contains("seated at"), "{err}");
    assert!(err.contains("spec-worktrees"), "{err}");
}

#[test]
fn read_verbs_answer_on_an_unbound_checkout() {
    let (_ws, spec) = protected_repo("reads");
    ok(&spec, &["check"]);
    ok(&spec, &["version", "list"]);
    // plan reads pass the guard and fail on their own terms — the
    // authoring reads (suggest, list) included.
    let (_, _, err) = run(&spec, &["plan", "verify"]);
    assert!(err.contains("no active plan"), "past the guard, not stopped by it: {err}");
    let (_, _, err) = run(&spec, &["plan", "scenarios", "list"]);
    assert!(err.contains("no active plan"), "past the guard, not stopped by it: {err}");
    ok(&spec, &["plan", "list"]);
}

#[test]
fn a_verb_with_no_work_to_name_gets_both_recipes() {
    let (_ws, spec) = open_repo("recipes");
    let (success, _out, err) = run(&spec, &["version", "save", "-m", "x"]);
    assert!(!success);
    assert!(err.contains("plan use"), "{err}");
    assert!(err.contains("worktree mint"), "{err}");
}

#[test]
fn a_gitless_project_refuses_mutation_loudly() {
    // No manifest opt-in anywhere in sight: the discipline never evaporates.
    let ws = scratch("gitless");
    let spec = ws.join("spec");
    spec_project(&spec, "");
    let (success, _out, err) = run(&spec, &["version", "save", "-m", "seed"]);
    assert!(!success, "gitless mutation is a stop, not a workflow");
    assert!(err.contains("not a git repository"), "{err}");
    assert!(err.contains("git init"), "{err}");
    assert!(err.contains("or cancel"), "{err}");
    // status names the same condition for the agent's opening read
    let st = ok(&spec, &["status"]);
    assert!(st.contains("not a git repository"), "{st}");
    assert!(st.contains("git init"), "{st}");
}

#[test]
fn mint_without_a_plan_seats_spec_work_and_drop_retires_it() {
    let (_ws, spec) = protected_repo("effort");
    let out = ok(&spec, &["worktree", "mint", "storm"]);
    assert!(out.contains("minted"), "{out}");
    assert!(out.contains("archi/storm"), "{out}");
    let ls = ok(&spec, &["worktree", "ls", "--spec", "storm"]);
    assert!(ls.contains("spec storm"), "{ls}");

    // Filters that match nothing say so instead of listing everything.
    let ls = ok(&spec, &["worktree", "ls", "--plan", "nope"]);
    assert!(ls.contains("no worktrees match"), "{ls}");

    let out = ok(&spec, &["worktree", "drop", "storm"]);
    assert!(out.contains("dropped"), "{out}");
    assert!(out.contains("branch archi/storm stays"), "{out}");
    let ls = ok(&spec, &["worktree", "ls", "--spec", "storm"]);
    assert!(ls.contains("no worktrees match"), "{ls}");
    let wt = spec.parent().unwrap().join("spec-worktrees/storm");
    assert!(!wt.exists(), "drop removed the worktree");
}

#[test]
fn status_names_the_checkout_and_its_open_work() {
    let (_ws, spec) = protected_repo("status");
    let _ = run(&spec, &["plan", "use", "auth"]); // refuses on main, mints the seat
    let wt = spec.parent().unwrap().join("spec-worktrees/auth");
    ok(&wt, &["plan", "use", "auth"]);

    let st = ok(&wt, &["status"]);
    assert!(st.contains("checkout:"), "{st}");
    assert!(st.contains("archi/auth"), "{st}");
    assert!(st.contains("plan auth"), "{st}");
    assert!(st.contains("plan: auth @ v0001 (draft)"), "{st}");
    assert!(st.contains("version: at v0001"), "{st}");
    assert!(st.contains("stress: no open round"), "{st}");
    assert!(st.contains("open plan: auth"), "{st}");

    // The main checkout is unbound and plan-less: the plan was born in the
    // worktree and travels by branch, not by osmosis.
    let st = ok(&spec, &["status"]);
    assert!(st.contains("on main"), "{st}");
    assert!(st.contains("binding: none"), "{st}");
    assert!(st.contains("plan: none active here"), "{st}");
    assert!(st.contains("open plans: none"), "{st}");
}

#[test]
fn a_clean_merge_lands_the_work_and_retires_the_seat() {
    let (_ws, spec) = open_repo("merge");
    ok(&spec, &["worktree", "mint", "feature"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/feature");
    fs::write(wt.join("notes.md"), "landed\n").unwrap();
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "work"]);

    let out = ok(&spec, &["worktree", "merge", "feature"]);
    assert!(out.contains("merged archi/feature"), "{out}");
    assert!(out.contains("retired"), "{out}");
    assert!(spec.join("notes.md").is_file(), "the work landed on main");
    assert!(!wt.exists(), "the worktree is gone");
    let ls = ok(&spec, &["worktree", "ls", "--spec", "feature"]);
    assert!(ls.contains("no worktrees match"), "{ls}");
}

#[test]
fn a_conflicted_merge_stops_and_keeps_the_seat() {
    let (_ws, spec) = open_repo("conflict");
    ok(&spec, &["worktree", "mint", "feature"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/feature");
    fs::write(wt.join("notes.md"), "theirs\n").unwrap();
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "theirs"]);
    // main moves the same file the other way
    git(&spec, &["switch", "-q", "main"]);
    fs::write(spec.join("notes.md"), "ours\n").unwrap();
    git(&spec, &["add", "-A"]);
    git(&spec, &["commit", "-qm", "ours"]);

    let (success, out, _err) = run(&spec, &["worktree", "merge", "feature"]);
    assert!(!success, "a conflict is a stop, not a success");
    assert!(out.contains("version remint"), "{out}");
    assert!(out.contains("plan repin"), "{out}");
    assert!(out.contains("session fold"), "{out}");
    assert!(wt.is_dir(), "the worktree stays");
    let ls = ok(&spec, &["worktree", "ls", "--spec", "feature"]);
    assert!(ls.contains("spec feature"), "the binding stays: {ls}");
    git(&spec, &["merge", "--abort"]);
}

#[test]
fn a_seat_lands_only_after_its_plan_closes() {
    let (_ws, spec) = open_repo("plan-gate");
    ok(&spec, &["worktree", "mint", "feat", "--plan", "feat"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/feat");
    ok(&wt, &["plan", "use", "feat"]);
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "plan born"]);

    let (success, _out, err) = run(&spec, &["worktree", "merge", "feat"]);
    assert!(!success, "an open plan never merges");
    assert!(err.contains("is draft"), "{err}");
    assert!(err.contains("plan close"), "{err}");
    assert!(wt.is_dir(), "the seat stays");

    ok(&wt, &["plan", "close"]);
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "plan closed"]);
    let out = ok(&spec, &["worktree", "merge", "feat"]);
    assert!(out.contains("retired"), "{out}");
    assert!(!wt.exists());
}

#[test]
fn a_protected_branch_never_receives_a_local_merge() {
    let (_ws, spec) = protected_repo("no-local-merge");
    ok(&spec, &["worktree", "mint", "feature"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/feature");
    fs::write(wt.join("notes.md"), "landed\n").unwrap();
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "work"]);

    let (success, _out, err) = run(&spec, &["worktree", "merge", "feature"]);
    assert!(!success, "landing on a protected branch is a PR ceremony");
    assert!(err.contains("never receives a local merge"), "{err}");
    assert!(err.contains("--to"), "{err}");
    assert!(wt.is_dir(), "nothing retired");
    assert!(!spec.join("notes.md").exists(), "main untouched");
}

#[test]
fn to_lands_the_worktree_head_on_a_new_branch_without_merging() {
    let (_ws, spec) = protected_repo("land");
    ok(&spec, &["worktree", "mint", "feature"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/feature");
    fs::write(wt.join("notes.md"), "landed elsewhere\n").unwrap();
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "work"]);

    let out = ok(&spec, &["worktree", "merge", "feature", "--to", "feat/x"]);
    assert!(out.contains("landed archi/feature on new branch feat/x"), "{out}");
    assert!(out.contains("retired"), "{out}");
    assert!(!spec.join("notes.md").exists(), "main is untouched");
    let out = Command::new("git")
        .args(["-C", spec.to_str().unwrap(), "rev-parse", "--verify", "refs/heads/feat/x"])
        .output()
        .unwrap();
    assert!(out.status.success(), "feat/x exists");
}

/// A member repo beside the spec: committed, on `main`, repo-local identity.
fn member_repo(dir: &Path) {
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn serve() {}\n").unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["config", "user.email", "t@t"]);
    git(dir, &["config", "user.name", "t"]);
    git(dir, &["config", "commit.gpgsign", "false"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-qm", "seed"]);
}

/// Spec + one member `backend`, committed and saved so the version entry
/// records the member baseline the cascade anchors on.
fn cascade_repo(tag: &str) -> (PathBuf, PathBuf, PathBuf) {
    let ws = scratch(tag);
    let spec = ws.join("spec");
    let backend = ws.join("backend");
    member_repo(&backend);
    spec_project(&spec, "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n");
    git(&spec, &["init", "-q", "-b", "main"]);
    git(&spec, &["config", "user.email", "t@t"]);
    git(&spec, &["config", "user.name", "t"]);
    git(&spec, &["config", "commit.gpgsign", "false"]);
    git(&spec, &["add", "-A"]);
    git(&spec, &["commit", "-qm", "seed"]);
    // The bootstrap seat maps the real member so the save records its
    // baseline — the seat's own relative `../backend` resolves nowhere.
    seat_save(&spec, "boot", &[("backend", &backend)], "seed");
    (ws, spec, backend)
}

#[test]
fn the_cascade_mints_member_worktrees_and_the_seat_overlay() {
    let (ws, spec, backend) = cascade_repo("cascade");
    let bare = ws.join("origin.git");
    git(&ws, &["init", "-q", "--bare", bare.to_str().unwrap()]);
    git(&backend, &["remote", "add", "origin", bare.to_str().unwrap()]);

    let out = ok(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    assert!(out.contains("member backend:"), "{out}");
    let wt = spec.parent().unwrap().join("spec-worktrees/feat");
    let bwt = backend.parent().unwrap().join("backend-worktrees/feat");
    assert!(bwt.is_dir(), "the member worktree cascaded");

    // The seat's overlay points the member at its worktree — resolution
    // inside the seat sees the cascade, not someone else's checkout.
    let overlay = fs::read_to_string(wt.join("archi/repos.local.toml")).unwrap();
    assert!(overlay.contains("backend-worktrees"), "{overlay}");
    let ls = ok(&wt, &["repo", "ls"]);
    assert!(ls.contains("backend-worktrees"), "{ls}");

    let ls = ok(&spec, &["worktree", "ls"]);
    assert!(ls.contains("member backend:"), "{ls}");
    assert!(ls.contains("(base main) — ok"), "{ls}");

    // Close: member work goes by push, spec by local merge, all retired.
    fs::write(bwt.join("src/lib.rs"), "pub fn serve() { /* new */ }\n").unwrap();
    git(&bwt, &["add", "-A"]);
    git(&bwt, &["commit", "-qm", "member work"]);
    fs::write(wt.join("notes.md"), "spec work\n").unwrap();
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "spec work"]);
    let out = ok(&spec, &["worktree", "merge", "feat"]);
    assert!(out.contains("member backend: pushed"), "{out}");
    assert!(out.contains("merged archi/feat"), "{out}");
    assert!(out.contains("retired"), "{out}");
    assert!(!bwt.exists(), "member worktree retired");
    assert!(!wt.exists(), "spec worktree retired");
    let heads = Command::new("git")
        .args(["-C", bare.to_str().unwrap(), "branch", "--format=%(refname:short)"])
        .output()
        .unwrap();
    let heads = String::from_utf8_lossy(&heads.stdout).into_owned();
    assert!(heads.contains("archi/feat"), "the member branch reached the remote: {heads}");
}

#[test]
fn a_seat_extension_resolves_members_from_the_seat() {
    // The unit lives in its seat: member declarations and anchors made
    // there exist on no other branch — the mid-unit extension must read
    // them from the seat, not the primary checkout.
    let (ws, spec) = open_repo("extend");
    let svc = ws.join("svc");
    member_repo(&svc);
    ok(&spec, &["worktree", "mint", "unit"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/unit");
    let manifest = fs::read_to_string(wt.join("archi.toml")).unwrap();
    fs::write(
        wt.join("archi.toml"),
        format!("{manifest}[[repo]]\nname = \"svc\"\npath = \"../svc\"\n"),
    )
    .unwrap();
    ok(&wt, &["repo", "map", "svc", svc.to_str().unwrap()]);
    ok(&wt, &["version", "anchor", "--repo", "svc"]);
    let out = ok(&wt, &["worktree", "mint", "unit", "--repos", "svc"]);
    assert!(out.contains("member svc:"), "{out}");
    assert!(out.contains("(base main)"), "{out}");
    let swt = svc.parent().unwrap().join("svc-worktrees/unit");
    assert!(swt.is_dir(), "the member worktree cascaded from the seat");
    let ls = ok(&wt, &["worktree", "ls"]);
    assert!(ls.contains("member svc:"), "{ls}");
    assert!(ls.contains("— ok"), "{ls}");
}

#[test]
fn a_baseline_off_the_branch_refuses_with_the_base_escape() {
    let (_ws, spec, backend) = cascade_repo("off-branch");
    // the latest baseline lands on a side branch the checkout then leaves —
    // reachable from `side`, not from `main`
    git(&backend, &["switch", "-qc", "side"]);
    git(&backend, &["commit", "-qm", "side work", "--allow-empty"]);
    fs::write(
        spec.join("archi/src/model.arch"),
        format!("{MODEL}def node Extra:\n  port x\n"),
    )
    .unwrap();
    git(&spec, &["add", "-A"]);
    git(&spec, &["commit", "-qm", "model grows"]);
    seat_save(&spec, "boot2", &[("backend", &backend)], "v2 with side baseline");
    git(&backend, &["switch", "-q", "main"]);

    let (success, _out, err) = run(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    assert!(!success, "an unreachable baseline is a question, not a guess");
    assert!(err.contains("is not on `main`"), "{err}");
    assert!(err.contains("side"), "the candidate branch is named: {err}");
    assert!(err.contains("--base backend="), "{err}");
    let out = ok(&spec, &["worktree", "mint", "feat", "--repos", "backend", "--base", "backend=main"]);
    assert!(out.contains("member backend:"), "{out}");
    assert!(out.contains("(base main)"), "{out}");
}

#[test]
fn a_missing_baseline_names_both_repairs() {
    let ws = scratch("no-baseline");
    let spec = ws.join("spec");
    let backend = ws.join("backend");
    member_repo(&backend);
    // the save happens BEFORE the member is declared — no baseline recorded
    spec_project(&spec, "");
    git(&spec, &["init", "-q", "-b", "main"]);
    git(&spec, &["config", "user.email", "t@t"]);
    git(&spec, &["config", "user.name", "t"]);
    git(&spec, &["add", "-A"]);
    git(&spec, &["commit", "-qm", "seed"]);
    seat_save(&spec, "boot", &[], "seed");
    spec_project(&spec, "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n");
    let (success, _out, err) = run(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    assert!(!success);
    assert!(err.contains("version anchor --repo backend"), "{err}");
    assert!(err.contains("--base backend="), "{err}");
}

#[test]
fn a_partial_cascade_rolls_back_whole() {
    let (ws, spec, backend) = cascade_repo("rollback");
    let web = ws.join("web");
    member_repo(&web);
    // web participates but its target path is squatted — the cascade must fail
    fs::create_dir_all(ws.join("web-worktrees/feat")).unwrap();
    let manifest = "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n\n[[repo]]\nname = \"web\"\npath = \"../web\"\n";
    spec_project(&spec, manifest);
    let (success, _out, err) = run(&spec, &["worktree", "mint", "feat", "--repos", "backend,web"]);
    assert!(!success, "{err}");
    assert!(!backend.parent().unwrap().join("backend-worktrees/feat").exists(), "rolled back");
    assert!(!spec.parent().unwrap().join("spec-worktrees/feat").exists(), "spec worktree rolled back");
    let ls = ok(&spec, &["worktree", "ls"]);
    assert!(!ls.contains("feat"), "no registry entry survived: {ls}");
}

#[test]
fn a_refused_push_keeps_the_member_until_repaired() {
    let (ws, spec, backend) = cascade_repo("push-refused");
    ok(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    let bwt = backend.parent().unwrap().join("backend-worktrees/feat");
    fs::write(bwt.join("note.txt"), "work\n").unwrap();
    git(&bwt, &["add", "-A"]);
    git(&bwt, &["commit", "-qm", "work"]);

    // no remote: the push refuses, the member stays bound, nothing retires
    let (success, out, _err) = run(&spec, &["worktree", "merge", "feat"]);
    assert!(!success);
    assert!(out.contains("member backend: kept"), "{out}");
    assert!(bwt.is_dir(), "member worktree stays");

    // repair the remote, re-run: idempotent close finishes the retire
    let bare = ws.join("origin.git");
    git(&ws, &["init", "-q", "--bare", bare.to_str().unwrap()]);
    git(&backend, &["remote", "add", "origin", bare.to_str().unwrap()]);
    let out = ok(&spec, &["worktree", "merge", "feat"]);
    assert!(out.contains("member backend: pushed"), "{out}");
    assert!(out.contains("retired"), "{out}");
    assert!(!bwt.exists());
}

#[test]
fn drop_cascades_over_member_worktrees() {
    let (_ws, spec, backend) = cascade_repo("drop-cascade");
    ok(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    let bwt = backend.parent().unwrap().join("backend-worktrees/feat");
    assert!(bwt.is_dir());
    let out = ok(&spec, &["worktree", "drop", "feat"]);
    assert!(out.contains("dropped"), "{out}");
    assert!(!bwt.exists(), "member worktree dropped");
    assert!(out.contains("member backend: branch archi/feat stays"), "{out}");
}

#[test]
fn a_doctored_plan_pin_surfaces_as_a_stale_pin_finding() {
    let ws = scratch("stale-plan");
    let spec = ws.join("spec");
    spec_project(&spec, "");
    let spec = util::seat(&spec);
    ok(&spec, &["version", "save", "-m", "seed"]);
    ok(&spec, &["plan", "use", "auth"]);
    let state_path = spec.join("archi/plans/auth/state.json");
    let text = fs::read_to_string(&state_path).unwrap();
    assert!(text.contains("\"version_hash\": \"sha256:"), "use stamps the hash: {text}");

    let out = ok(&spec, &["check"]);
    assert!(!out.contains("stale plan pin"), "an honest pin is silent: {out}");

    let doctored = text.replace("\"version_hash\": \"sha256:", "\"version_hash\": \"sha256:0000");
    fs::write(&state_path, doctored).unwrap();
    let out = ok(&spec, &["check"]);
    assert!(out.contains("stale plan pin"), "{out}");
    assert!(out.contains("archi plan repin"), "{out}");
}

#[test]
fn a_doctored_session_stamp_surfaces_as_a_stale_stamp_finding() {
    let ws = scratch("stale-session");
    let spec = ws.join("spec");
    spec_project(&spec, "");
    let spec = util::seat(&spec);
    ok(&spec, &["version", "save", "-m", "seed"]);
    let round = spec.join("archi/stress/round");
    fs::create_dir_all(&round).unwrap();
    fs::write(
        round.join("round.md"),
        "---\nversion: v0001\nclosed:\n---\n\n# Round\n\nPresses the seed model.\n",
    )
    .unwrap();
    // The unchanged save closes the round and stamps id + content hash.
    let out = ok(&spec, &["version", "save", "-m", "close"]);
    assert!(out.contains("closed stress session `round`"), "{out}");
    let text = fs::read_to_string(round.join("round.md")).unwrap();
    assert!(text.contains("closed: v0001"), "{text}");
    assert!(text.contains("version-hash: sha256:"), "the stamp carries the hash: {text}");

    fs::write(
        round.join("round.md"),
        text.replace("version-hash: sha256:", "version-hash: sha256:0000"),
    )
    .unwrap();
    let out = ok(&spec, &["check"]);
    assert!(out.contains("stale session stamp"), "{out}");
    assert!(out.contains("--session round"), "{out}");
}

#[test]
fn a_dirty_spec_outside_a_seat_fails_check_and_build() {
    let (_ws, spec) = open_repo("verdict");
    // clean unbound tree: both verdicts answer
    ok(&spec, &["check"]);
    ok(&spec, &["build"]);
    // an ungoverned spec edit: both refuse with the seat recipe
    fs::write(
        spec.join("archi/src/model.arch"),
        format!("{MODEL}def node Rogue:\n  port x\n"),
    )
    .unwrap();
    for verb in ["check", "build"] {
        let (success, _out, err) = run(&spec, &[verb]);
        assert!(!success, "{verb} blessed ungoverned work");
        assert!(err.contains("uncommitted"), "{err}");
        assert!(err.contains("model.arch"), "{err}");
        assert!(err.contains("worktree mint"), "{err}");
    }
    // a non-spec edit alone never trips it
    git(&spec, &["add", "-A"]);
    git(&spec, &["commit", "-qm", "grow"]);
    fs::write(spec.join("notes.md"), "scratch\n").unwrap();
    ok(&spec, &["check"]);
    // the same edit inside a seat is governed work: check answers
    ok(&spec, &["worktree", "mint", "grow"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/grow");
    fs::write(
        wt.join("archi/src/model.arch"),
        format!("{MODEL}def node Seated:\n  port x\n"),
    )
    .unwrap();
    ok(&wt, &["check"]);
}

#[test]
fn a_hand_removed_worktree_heals_out_of_the_registry() {
    let (_ws, spec) = protected_repo("heal");
    ok(&spec, &["worktree", "mint", "storm"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/storm");
    git(&spec, &["worktree", "remove", "--force", wt.to_str().unwrap()]);
    let ls = ok(&spec, &["worktree", "ls", "--spec", "storm"]);
    assert!(ls.contains("no worktrees match"), "{ls}");
}
