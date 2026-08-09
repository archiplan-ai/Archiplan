//! End to end through the real binary: the binding discipline —
//! a mutation runs only inside a bound worktree, and the guard sits at
//! the router. Protected branches refuse local merges, the registry
//! moves by commands, context follows the checkout
//! (`archi/requirements/worktree-parallelism/`).

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use util::{git, ok, run};

const MODEL: &str = "def conn wire := * -> *\n\
                     def node Gate:\n  port serve\n\
                     def node Ledger:\n  port keep\n\
                     Gate.serve wire Ledger.keep\n";

fn scratch(tag: &str) -> PathBuf {
    util::scratch("archi-wt-e2e", tag)
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

fn head(dir: &Path) -> String {
    let out = Command::new("git")
        .args(["-C", dir.to_str().unwrap(), "rev-parse", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Run a save on `main` the governed way: a bootstrap worktree maps the
/// members, saves, commits, and merges back — mutations never run unbound,
/// and the retire leaves the registry empty.
fn worktree_save(spec: &Path, slug: &str, map: &[(&str, &Path)], msg: &str) {
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
    worktree_save(&spec, "boot", &[], "seed");
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
fn an_unbound_checkout_mints_the_worktree_and_the_work_proceeds() {
    // No protected list: the discipline is unconditional — any unbound
    // checkout refuses, the primary on `main` included.
    let (_ws, spec) = open_repo("guard");

    // Mutation on main refuses, mints, and prints the path to enter.
    let (success, _out, err) = run(&spec, &["plan", "use", "auth"]);
    assert!(!success, "mutation on an unbound checkout must refuse");
    assert!(err.contains("unbound"), "{err}");
    assert!(err.contains("bound worktree"), "{err}");
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

    // The same command from the minted worktree proceeds and binds.
    let out = ok(&wt, &["plan", "use", "auth"]);
    assert!(out.contains("created plan `auth`"), "{out}");
    let ls = ok(&spec, &["worktree", "ls", "--plan", "auth"]);
    assert!(ls.contains("plan auth"), "{ls}");
    assert!(ls.contains("archi/auth"), "{ls}");

    // Any other checkout mutating the bound plan refuses with the owner.
    let (success, _out, err) = run(&spec, &["plan", "use", "auth"]);
    assert!(!success);
    assert!(err.contains("is bound to"), "{err}");
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
fn mint_without_a_plan_binds_spec_work_and_close_retires_it() {
    let (_ws, spec) = protected_repo("effort");
    let out = ok(&spec, &["worktree", "mint", "storm"]);
    assert!(out.contains("minted"), "{out}");
    assert!(out.contains("archi/storm"), "{out}");
    let ls = ok(&spec, &["worktree", "ls", "--spec", "storm"]);
    assert!(ls.contains("spec storm"), "{ls}");

    // Filters that match nothing say so instead of listing everything.
    let ls = ok(&spec, &["worktree", "ls", "--plan", "nope"]);
    assert!(ls.contains("no worktrees match"), "{ls}");

    let out = ok(&spec, &["worktree", "close", "storm"]);
    assert!(out.contains("closed"), "{out}");
    assert!(out.contains("the row stays as the record"), "{out}");
    assert!(out.contains("branch archi/storm stays"), "{out}");
    let wt = spec.parent().unwrap().join("spec-worktrees/storm");
    assert!(!wt.exists(), "close removed the worktree");
    // The folder went; the row is the record of what this machine carried.
    let ls = ok(&spec, &["worktree", "ls", "--spec", "storm"]);
    assert!(ls.contains("spec storm — closed"), "the listing keeps it: {ls}");
    let ls = ok(&spec, &["worktree", "ls", "--spec", "storm", "--status", "active"]);
    assert!(ls.contains("no worktrees match"), "history is not live work: {ls}");
}

#[test]
fn a_bare_drop_refuses_toward_close() {
    let (_ws, spec) = protected_repo("drop-retired");
    ok(&spec, &["worktree", "mint", "storm"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/storm");
    let (success, _out, err) = run(&spec, &["worktree", "drop", "storm"]);
    assert!(!success, "the retired verb refuses instead of doing nothing");
    assert!(
        err.contains(
            "`drop` retired — the registry keeps its rows: `archi worktree close <slug>`"
        ),
        "{err}"
    );
    assert!(wt.is_dir(), "the refusal moved nothing");
    // the bare verb, with no handle at all, answers the same
    let (success, _out, err) = run(&spec, &["worktree", "drop"]);
    assert!(!success);
    assert!(err.contains("archi worktree close <slug>"), "{err}");
}

#[test]
fn ls_filters_by_status_and_refuses_an_unknown_one() {
    let (_ws, spec) = protected_repo("status-filter");
    ok(&spec, &["worktree", "mint", "live"]);
    ok(&spec, &["worktree", "mint", "done"]);
    ok(&spec, &["worktree", "close", "done"]);

    let all = ok(&spec, &["worktree", "ls"]);
    assert!(all.contains("spec live"), "{all}");
    assert!(all.contains("spec done — closed"), "{all}");
    let active = ok(&spec, &["worktree", "ls", "--status", "active"]);
    assert!(active.contains("spec live"), "{active}");
    assert!(!active.contains("spec done"), "{active}");
    let closed = ok(&spec, &["worktree", "ls", "--status", "closed"]);
    assert!(closed.contains("spec done — closed"), "{closed}");
    assert!(!closed.contains("spec live"), "{closed}");
    // it composes with the filters that were already there
    let ls = ok(&spec, &["worktree", "ls", "--spec", "live", "--status", "closed"]);
    assert!(ls.contains("no worktrees match"), "{ls}");
    let ls = ok(&spec, &["worktree", "ls", "--spec", "live", "--status", "active"]);
    assert!(ls.contains("spec live"), "{ls}");
    assert!(!ls.contains("spec done"), "{ls}");

    let (success, _out, err) = run(&spec, &["worktree", "ls", "--status", "gone"]);
    assert!(!success, "an unknown state is a refusal, not a silent all");
    assert!(err.contains("`--status gone` names no state"), "{err}");
    assert!(err.contains("active, closed or all"), "{err}");
}

#[test]
fn status_in_an_unbound_checkout_names_the_standing_rows() {
    let (_ws, spec) = protected_repo("pointer");
    // the fixture's own seat landed and closed: nothing stands
    let st = ok(&spec, &["status"]);
    assert!(st.contains("binding: none — this checkout is unbound"), "{st}");
    assert!(st.contains("standing work: none"), "{st}");

    ok(&spec, &["worktree", "mint", "storm", "--plan", "gale"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/storm");
    let st = ok(&spec, &["status"]);
    assert!(
        st.contains(&format!("standing work: {} on archi/storm — plan gale", wt.display())),
        "{st}"
    );
    // closing it leaves the checkout pointing at nothing again
    ok(&spec, &["worktree", "close", "storm"]);
    let st = ok(&spec, &["status"]);
    assert!(st.contains("standing work: none"), "{st}");
}

#[test]
fn a_closed_row_leaves_its_checkout_unbound() {
    let (_ws, spec) = protected_repo("closed-unbound");
    ok(&spec, &["worktree", "mint", "storm"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/storm");
    // while the row stands, the seat licenses its mutations
    fs::write(
        wt.join("archi/src/model.arch"),
        format!("{MODEL}def node Governed:\n  port x\n"),
    )
    .unwrap();
    ok(&wt, &["version", "save", "-m", "governed"]);
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "saved"]);
    ok(&spec, &["worktree", "close", "storm"]);

    // the operator brings the folder back by hand: the row is history, and
    // history licenses nothing
    git(&spec, &["worktree", "add", "-q", wt.to_str().unwrap(), "archi/storm"]);
    let st = ok(&wt, &["status"]);
    assert!(st.contains("binding: none"), "{st}");
    assert!(st.contains("row closed"), "{st}");
    let (success, _out, err) = run(&wt, &["version", "save", "-m", "ungoverned"]);
    assert!(!success, "a closed row is no license to mutate");
    assert!(err.contains("unbound"), "{err}");
}

#[test]
fn status_names_the_checkout_and_its_open_work() {
    let (_ws, spec) = protected_repo("status");
    let _ = run(&spec, &["plan", "use", "auth"]); // refuses on main, mints the worktree
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
fn a_clean_merge_lands_the_work_and_retires_the_worktree() {
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
    // the folder is gone, the row is the record
    let ls = ok(&spec, &["worktree", "ls", "--spec", "feature"]);
    assert!(ls.contains("spec feature — closed"), "{ls}");
    let ls = ok(&spec, &["worktree", "ls", "--spec", "feature", "--status", "active"]);
    assert!(ls.contains("no worktrees match"), "{ls}");
}

#[test]
fn a_conflicted_merge_stops_and_keeps_the_worktree() {
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
fn a_worktree_lands_only_after_its_plan_closes() {
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
    assert!(wt.is_dir(), "the worktree stays");

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
    assert!(!spec.join("notes.md").exists(), "main is untouched");
    let verify = Command::new("git")
        .args(["-C", spec.to_str().unwrap(), "rev-parse", "--verify", "refs/heads/feat/x"])
        .output()
        .unwrap();
    assert!(verify.status.success(), "feat/x exists");

    // A departure, not an arrival: the seat survives the review window and
    // the report says what is left to do.
    assert!(!out.contains("retired"), "nothing retired on the sideways path: {out}");
    assert!(out.contains(&format!("{} stays", wt.display())), "{out}");
    assert!(out.contains("push feat/x and open the PR"), "{out}");
    assert!(out.contains("the row closes once main carries the work"), "{out}");
    assert!(out.contains("the next archi command frees the folder"), "{out}");
    assert!(wt.is_dir(), "the worktree stays until the work lands");
    let ls = ok(&spec, &["worktree", "ls", "--spec", "feature"]);
    assert!(ls.contains("spec feature — waiting on feat/x → main"), "{ls}");
}

#[test]
fn the_sweep_frees_an_integrated_seat_and_keeps_one_with_stray_work() {
    let (_ws, spec) = protected_repo("sweep-merge");
    // ignored build output is the reason the folder is worth freeing at all
    fs::write(spec.join(".gitignore"), "junk/\n").unwrap();
    git(&spec, &["add", "-A"]);
    git(&spec, &["commit", "-qm", "ignore the build output"]);
    ok(&spec, &["worktree", "mint", "one"]);
    ok(&spec, &["worktree", "mint", "two"]);
    let one = spec.parent().unwrap().join("spec-worktrees/one");
    let two = spec.parent().unwrap().join("spec-worktrees/two");
    for (wt, name) in [(&one, "one"), (&two, "two")] {
        fs::write(wt.join(format!("{name}.md")), "work\n").unwrap();
        git(wt, &["add", "-A"]);
        git(wt, &["commit", "-qm", "work"]);
    }
    ok(&spec, &["worktree", "merge", "one", "--to", "feat/one"]);
    ok(&spec, &["worktree", "merge", "two", "--to", "feat/two"]);

    // nothing arrived yet: the reading command sweeps and stays silent
    let ls = ok(&spec, &["worktree", "ls"]);
    assert!(!ls.contains("freed"), "an empty sweep says nothing: {ls}");
    assert!(ls.contains("spec one — waiting on feat/one → main"), "{ls}");
    assert!(one.is_dir() && two.is_dir());

    // the forge merges both pull requests; one seat holds ignored build
    // output, the other an untracked file no rule covers
    git(&spec, &["merge", "--no-edit", "feat/one"]);
    git(&spec, &["merge", "--no-edit", "feat/two"]);
    fs::create_dir_all(one.join("junk")).unwrap();
    fs::write(one.join("junk/build.bin"), "tens of gigabytes\n").unwrap();
    fs::write(two.join("stray.txt"), "unfinished\n").unwrap();

    let ls = ok(&spec, &["worktree", "ls"]);
    assert!(
        ls.contains(&format!("freed {} — spec integrated into main", one.display())),
        "{ls}"
    );
    assert!(ls.contains(&format!("closed {}", one.display())), "{ls}");
    assert!(!one.exists(), "ignored files never veto a cleanup");
    assert!(two.is_dir(), "an unignored untracked file keeps its seat");
    assert!(ls.contains("spec one — closed"), "{ls}");
    // the kept seat is live work again — its landing record no longer stands
    assert!(!ls.contains("waiting on feat/two"), "{ls}");
}

#[test]
fn the_sweep_frees_a_seat_the_forge_squashed() {
    let (_ws, spec) = protected_repo("sweep-squash");
    ok(&spec, &["worktree", "mint", "solo"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/solo");
    fs::write(wt.join("notes.md"), "landed\n").unwrap();
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "work"]);
    let landed = head(&wt);
    ok(&spec, &["worktree", "merge", "solo", "--to", "feat/solo"]);

    // the forge squashes the pull request: main takes the content under a
    // sha of its own, so ancestry answers no and the content answers yes
    git(&spec, &["merge", "--squash", "feat/solo"]);
    git(&spec, &["commit", "-qm", "squashed"]);
    assert_ne!(head(&spec), landed, "the forge rewrote the sha");

    let st = ok(&spec, &["status"]);
    assert!(
        st.contains(&format!("freed {} — spec integrated into main", wt.display())),
        "{st}"
    );
    assert!(!wt.exists(), "content proves the arrival, not ancestry");
    let ls = ok(&spec, &["worktree", "ls", "--status", "closed"]);
    assert!(ls.contains("spec solo — closed"), "{ls}");
}

#[test]
fn a_resumed_seat_survives_the_sweep() {
    let (_ws, spec) = protected_repo("resumed");
    ok(&spec, &["worktree", "mint", "one"]);
    ok(&spec, &["worktree", "mint", "two"]);
    let one = spec.parent().unwrap().join("spec-worktrees/one");
    let two = spec.parent().unwrap().join("spec-worktrees/two");
    for (wt, name) in [(&one, "one"), (&two, "two")] {
        fs::write(wt.join(format!("{name}.md")), "work\n").unwrap();
        git(wt, &["add", "-A"]);
        git(wt, &["commit", "-qm", "work"]);
    }
    ok(&spec, &["worktree", "merge", "one", "--to", "feat/one"]);
    ok(&spec, &["worktree", "merge", "two", "--to", "feat/two"]);

    // the review sends the operator back into both seats: one answers with a
    // commit, the other with an edit still in the tree
    fs::write(one.join("answer.md"), "review answered\n").unwrap();
    git(&one, &["add", "-A"]);
    git(&one, &["commit", "-qm", "answer"]);
    fs::write(two.join("two.md"), "still editing\n").unwrap();
    // main takes what it was given
    git(&spec, &["merge", "--no-edit", "feat/one"]);
    git(&spec, &["merge", "--no-edit", "feat/two"]);

    let ls = ok(&spec, &["worktree", "ls"]);
    assert!(!ls.contains("freed"), "a resumed seat is live work: {ls}");
    assert!(one.is_dir(), "a commit on top keeps the seat");
    assert!(two.is_dir(), "an uncommitted edit keeps the seat");
    assert!(!ls.contains("waiting on"), "both read as live work: {ls}");
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
    // The bootstrap worktree maps the real member so the save records its
    // baseline — its own relative `../backend` resolves nowhere.
    worktree_save(&spec, "boot", &[("backend", &backend)], "seed");
    (ws, spec, backend)
}

#[test]
fn a_bound_worktrees_own_members_are_not_rot_to_check() {
    let (ws, spec, backend) = cascade_repo("bindown");
    let bare = ws.join("origin.git");
    git(&ws, &["init", "-q", "--bare", bare.to_str().unwrap()]);
    git(&backend, &["remote", "add", "origin", bare.to_str().unwrap()]);

    ok(&spec, &["worktree", "mint", "own", "--repos", "backend"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/own");

    // Inside the worktree the overlay maps the member to its own worktree —
    // written by the mint, the working map, not a stale row.
    let out = ok(&wt, &["check"]);
    assert!(
        !out.contains("linked worktree"),
        "the worktree's own member worktree graded as rot:\n{out}"
    );
}

#[test]
fn the_cascade_mints_member_worktrees_and_the_overlay() {
    let (ws, spec, backend) = cascade_repo("cascade");
    let bare = ws.join("origin.git");
    git(&ws, &["init", "-q", "--bare", bare.to_str().unwrap()]);
    git(&backend, &["remote", "add", "origin", bare.to_str().unwrap()]);

    let out = ok(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    assert!(out.contains("member backend:"), "{out}");
    let wt = spec.parent().unwrap().join("spec-worktrees/feat");
    let bwt = backend.parent().unwrap().join("backend-worktrees/feat");
    assert!(bwt.is_dir(), "the member worktree cascaded");

    // The overlay points the member at its worktree — resolution inside
    // the worktree sees the cascade, not someone else's checkout.
    let overlay = fs::read_to_string(wt.join("archi/repos.local.toml")).unwrap();
    assert!(overlay.contains("backend-worktrees"), "{overlay}");
    let ls = ok(&wt, &["repo", "ls"]);
    assert!(ls.contains("backend-worktrees"), "{ls}");

    let ls = ok(&spec, &["worktree", "ls"]);
    assert!(ls.contains("member backend:"), "{ls}");
    assert!(ls.contains("(base main) — ok"), "{ls}");

    // Close, the contract's way: the spec saves mid-unit while member code
    // is in flight (the save names the omission), the member commits,
    // the worktree anchors the fresh tip — then member work goes by push, spec by
    // local merge. The spec retires at once; the member seat waits on its base.
    fs::write(bwt.join("src/lib.rs"), "pub fn serve() { /* new */ }\n").unwrap();
    fs::write(
        wt.join("archi/src/model.arch"),
        format!("{MODEL}def node Audit:\n  port log\n"),
    )
    .unwrap();
    ok(&wt, &["version", "save", "-m", "unit spec work"]);
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "spec work"]);
    git(&bwt, &["add", "-A"]);
    git(&bwt, &["commit", "-qm", "member work"]);
    ok(&wt, &["version", "anchor", "--repo", "backend"]);
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "baseline anchored"]);
    let out = ok(&spec, &["worktree", "merge", "feat"]);
    assert!(out.contains("member backend: pushed"), "{out}");
    assert!(out.contains("the member worktree stays until main carries the work"), "{out}");
    assert!(out.contains("merged archi/feat"), "{out}");
    assert!(out.contains("retired"), "{out}");
    assert!(bwt.is_dir(), "a push is not an arrival — the member seat stays");
    assert!(!wt.exists(), "spec worktree retired");
    let heads = Command::new("git")
        .args(["-C", bare.to_str().unwrap(), "branch", "--format=%(refname:short)"])
        .output()
        .unwrap();
    let heads = String::from_utf8_lossy(&heads.stdout).into_owned();
    assert!(heads.contains("archi/feat"), "the member branch reached the remote: {heads}");
    // the listing names the branch the member landed on and the one it waits on
    let ls = ok(&spec, &["worktree", "ls"]);
    assert!(ls.contains("member backend:"), "{ls}");
    assert!(ls.contains("waiting on archi/feat → main"), "{ls}");

    // the forge merges the member's pull request: the next archi command
    // frees the folder and the row closes
    git(&backend, &["merge", "--no-edit", "archi/feat"]);
    let ls = ok(&spec, &["worktree", "ls"]);
    assert!(
        ls.contains(&format!("freed {} — member backend integrated into main", bwt.display())),
        "{ls}"
    );
    assert!(!bwt.exists(), "the member seat frees once its base carries the work");
    assert!(ls.contains("spec feat — closed"), "every side arrived: {ls}");
}

#[test]
fn an_unanchored_member_refuses_the_landing_until_the_worktree_anchors() {
    let (ws, spec, backend) = cascade_repo("anchor-gate");
    let bare = ws.join("origin.git");
    git(&ws, &["init", "-q", "--bare", bare.to_str().unwrap()]);
    git(&backend, &["remote", "add", "origin", bare.to_str().unwrap()]);
    ok(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/feat");
    let bwt = backend.parent().unwrap().join("backend-worktrees/feat");

    // The unit's shape: member code in flight while the spec saves — the
    // fresh version records no backend baseline — then the member commits.
    fs::write(bwt.join("src/lib.rs"), "pub fn serve() { /* grown */ }\n").unwrap();
    fs::write(
        wt.join("archi/src/model.arch"),
        format!("{MODEL}def node Audit:\n  port log\n"),
    )
    .unwrap();
    ok(&wt, &["version", "save", "-m", "unit spec work"]);
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "spec work"]);
    git(&bwt, &["add", "-A"]);
    git(&bwt, &["commit", "-qm", "member work"]);
    let tip = head(&bwt);

    // The landing refuses before anything pushes or merges, naming the
    // member and the repair.
    let (success, _out, err) = run(&spec, &["worktree", "merge", "feat"]);
    assert!(!success, "un-anchored member work never lands");
    assert!(
        err.contains(&format!("member backend: worktree tip {}", &tip[..7])),
        "{err}"
    );
    assert!(err.contains("is past the recorded baseline none"), "{err}");
    assert!(err.contains("archi version anchor --repo backend --project"), "{err}");
    assert!(err.contains(wt.to_str().unwrap()), "the recipe names the worktree: {err}");
    assert!(err.contains("then re-run the merge"), "{err}");
    assert!(bwt.is_dir(), "nothing pushed, nothing retired");

    // The repair: the worktree is bound so anchor passes the guard, and
    // the member resolves through its overlay to the member worktree.
    let out = ok(&wt, &["version", "anchor", "--repo", "backend"]);
    assert!(out.contains(&format!("baseline backend at {tip}")), "{out}");
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "baseline anchored"]);
    let out = ok(&spec, &["worktree", "merge", "feat"]);
    assert!(out.contains("member backend: pushed"), "{out}");
    assert!(out.contains("retired"), "the spec side merged locally: {out}");
    assert!(bwt.is_dir(), "anchored and pushed — the member seat waits on its base");
    let ls = ok(&spec, &["worktree", "ls"]);
    assert!(ls.contains("waiting on archi/feat → main"), "{ls}");
    // the base carries the work: the folder frees itself
    git(&backend, &["merge", "--no-edit", "archi/feat"]);
    ok(&spec, &["worktree", "ls"]);
    assert!(!bwt.exists(), "anchored, landed, arrived, freed");
}

#[test]
fn the_to_landing_runs_the_same_baseline_gate() {
    let (ws, spec, backend) = cascade_repo("to-gate");
    let bare = ws.join("origin.git");
    git(&ws, &["init", "-q", "--bare", bare.to_str().unwrap()]);
    git(&backend, &["remote", "add", "origin", bare.to_str().unwrap()]);
    let recorded = head(&backend);
    ok(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/feat");
    let bwt = backend.parent().unwrap().join("backend-worktrees/feat");
    fs::write(bwt.join("note.txt"), "work\n").unwrap();
    git(&bwt, &["add", "-A"]);
    git(&bwt, &["commit", "-qm", "member work"]);
    let tip = head(&bwt);

    // `--to` protects the same archive: the gate refuses before the
    // sideways landing touches anything.
    let (success, _out, err) = run(&spec, &["worktree", "merge", "feat", "--to", "feat/x"]);
    assert!(!success, "a stale baseline never lands sideways either");
    assert!(err.contains(&format!("worktree tip {}", &tip[..7])), "{err}");
    assert!(
        err.contains(&format!("is past the recorded baseline {}", &recorded[..7])),
        "{err}"
    );
    assert!(err.contains("archi version anchor --repo backend"), "{err}");
    let verify = Command::new("git")
        .args(["-C", spec.to_str().unwrap(), "rev-parse", "--verify", "refs/heads/feat/x"])
        .output()
        .unwrap();
    assert!(!verify.status.success(), "the refusal precedes the landing branch");
    assert!(bwt.is_dir(), "the member stays");

    // The same recipe repairs a recorded-but-stale mark: the moved clean
    // member re-anchors the latest version, saying where the mark stood.
    let out = ok(&wt, &["version", "anchor", "--repo", "backend"]);
    assert!(
        out.contains(&format!("re-anchored v0001: baseline backend at {tip}")),
        "{out}"
    );
    assert!(out.contains(&format!("(was {}; anchor-born", &recorded[..7])), "{out}");
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "baseline re-anchored"]);
    let out = ok(&spec, &["worktree", "merge", "feat", "--to", "feat/x"]);
    assert!(out.contains("member backend: pushed"), "{out}");
    assert!(out.contains("landed archi/feat on new branch feat/x"), "{out}");
    assert!(!out.contains("retired"), "neither side arrived yet: {out}");
    assert!(bwt.is_dir(), "re-anchored and pushed — the member seat stays");
    assert!(wt.is_dir(), "landed sideways — the spec seat stays");
    let ls = ok(&spec, &["worktree", "ls"]);
    assert!(ls.contains("spec feat — waiting on feat/x → main"), "{ls}");
    assert!(ls.contains("waiting on archi/feat → main"), "{ls}");

    // both pull requests merge: the next archi command frees both folders
    // and the row closes
    git(&backend, &["merge", "--no-edit", "archi/feat"]);
    git(&spec, &["merge", "--no-edit", "feat/x"]);
    let ls = ok(&spec, &["worktree", "ls"]);
    assert!(!bwt.exists(), "the member folder freed");
    assert!(!wt.exists(), "the spec folder freed");
    assert!(ls.contains("spec feat — closed"), "{ls}");
}

#[test]
fn a_stale_but_reachable_auto_base_notes_how_far_behind() {
    let (_ws, spec, backend) = cascade_repo("behind");
    let baseline = head(&backend);
    // the member's branch advances past the recorded baseline — still an
    // ancestor: continuing the older pinned version is legitimate
    git(&backend, &["commit", "-qm", "ahead", "--allow-empty"]);

    let out = ok(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    assert!(
        out.contains(&format!("note: member backend: baseline {}", &baseline[..7])),
        "{out}"
    );
    assert!(out.contains("is 1 commit(s) behind `main`"), "{out}");
    assert!(out.contains("archi version anchor --repo backend"), "{out}");
    // the mint proceeded — the note informs, never refuses
    assert!(backend.parent().unwrap().join("backend-worktrees/feat").is_dir());
}

#[test]
fn an_extension_resolves_members_from_the_worktree() {
    // The unit lives in its worktree: member declarations and anchors made
    // there exist on no other branch — the mid-unit extension must read
    // them from the worktree, not the primary checkout.
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
    assert!(swt.is_dir(), "the member worktree cascaded from the worktree");
    let ls = ok(&wt, &["worktree", "ls"]);
    assert!(ls.contains("member svc:"), "{ls}");
    assert!(ls.contains("— ok"), "{ls}");
}

/// A member's overlay row, written directly: plain toml the mint resolves
/// members through — the shape `archi repo map` leaves behind.
fn map_overlay(spec: &Path, name: &str, dir: &Path) {
    fs::write(
        spec.join("archi/repos.local.toml"),
        format!(
            "# machine-local member checkouts — gitignored, never merged\n{name} = \"{}\"\n",
            dir.display()
        ),
    )
    .unwrap();
}

fn has_branch(dir: &Path, name: &str) -> bool {
    Command::new("git")
        .args([
            "-C",
            dir.to_str().unwrap(),
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/heads/{name}"),
        ])
        .output()
        .unwrap()
        .status
        .success()
}

#[test]
fn a_member_mapped_onto_a_linked_worktree_refuses_the_mint() {
    let (ws, spec, backend) = cascade_repo("linked-gate");
    // the field incident's shape: a leftover scratch worktree of the member
    // repo stands on a dead feature branch, and a stale overlay row maps
    // the member there
    let scratch = ws.join("backend-scratch");
    git(&backend, &["worktree", "add", "-q", "-b", "dead/feature", scratch.to_str().unwrap()]);
    map_overlay(&spec, "backend", &scratch);

    let (success, _out, err) = run(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    assert!(!success, "a stale overlay row never seeds a worktree");
    assert!(err.contains("member backend: the mapped checkout"), "{err}");
    assert!(err.contains(scratch.to_str().unwrap()), "{err}");
    assert!(err.contains("is a linked worktree standing on `dead/feature`"), "{err}");
    assert!(err.contains("a stale overlay row"), "{err}");
    assert!(
        err.contains(&format!("archi repo map backend {}", backend.display())),
        "the repair names the main checkout: {err}"
    );
    assert!(err.contains("--base backend="), "{err}");
    // the full-rollback contract: no worktree, no branches, no registry row
    assert!(!spec.parent().unwrap().join("spec-worktrees/feat").exists(), "no worktree");
    assert!(!has_branch(&spec, "archi/feat"), "no spec branch");
    assert!(!has_branch(&backend, "archi/feat"), "no member branch");
    let ls = ok(&spec, &["worktree", "ls"]);
    assert!(!ls.contains("feat"), "no registry row survived: {ls}");
}

#[test]
fn an_explicit_base_skips_the_gate_and_placement_anchors_on_the_main_checkout() {
    let (ws, spec, backend) = cascade_repo("linked-base");
    let scratch = ws.join("backend-scratch");
    git(&backend, &["worktree", "add", "-q", "-b", "dead/feature", scratch.to_str().unwrap()]);
    map_overlay(&spec, "backend", &scratch);

    // the base is named outright: the mapped checkout's identity stops
    // mattering, and the mint proceeds
    let out = ok(&spec, &["worktree", "mint", "feat", "--repos", "backend", "--base", "backend=main"]);
    assert!(out.contains("member backend:"), "{out}");
    assert!(out.contains("(base main)"), "{out}");
    assert!(!out.contains("linked worktree"), "an explicit --base skips the gate: {out}");
    // placement anchors on the MAIN checkout's sibling folder, never on a
    // sibling of the mapped scratch path
    let bwt = ws.join("backend-worktrees/feat");
    assert!(bwt.is_dir(), "the member worktree sits beside the main checkout");
    assert!(
        !ws.join("backend-scratch-worktrees").exists(),
        "nothing lands beside the scratch path"
    );
    // the overlay points the member at the anchored worktree
    let wt = spec.parent().unwrap().join("spec-worktrees/feat");
    let overlay = fs::read_to_string(wt.join("archi/repos.local.toml")).unwrap();
    assert!(overlay.contains("backend-worktrees"), "{overlay}");
    // `main` carries the recorded baseline: the audited line holds, no note
    assert!(!out.contains("off the audited line"), "{out}");
}

#[test]
fn an_explicit_base_off_the_recorded_baseline_notes_the_departure() {
    let (_ws, spec, backend) = cascade_repo("off-line");
    let baseline = head(&backend);
    // a branch with no shared history: it cannot contain the recorded baseline
    git(&backend, &["switch", "-q", "--orphan", "stray"]);
    git(&backend, &["commit", "-qm", "unrelated", "--allow-empty"]);
    git(&backend, &["switch", "-q", "main"]);

    // the escape lane never refuses — the mint proceeds, but the departure
    // from the audited line is said out loud, once
    let out = ok(&spec, &["worktree", "mint", "feat", "--repos", "backend", "--base", "backend=stray"]);
    assert!(
        out.contains(&format!(
            "note: member backend: the named base `stray` does not contain the recorded \
             baseline {} — continuing off the audited line",
            &baseline[..7]
        )),
        "{out}"
    );
    assert!(out.contains("member backend:"), "{out}");
    assert!(out.contains("(base stray)"), "{out}");
    assert!(backend.parent().unwrap().join("backend-worktrees/feat").is_dir(), "the mint proceeded");
}

#[test]
fn a_re_mint_extension_attaches_without_the_gate() {
    let (_ws, spec, backend) = cascade_repo("linked-remint");
    ok(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/feat");
    let bwt = backend.parent().unwrap().join("backend-worktrees/feat");
    assert!(bwt.is_dir());

    // the worktree's own overlay maps the member at its member worktree — a
    // linked checkout by construction; the re-mint attaches in silence,
    // the gate never fires on an already-bound member
    let (success, out, err) = run(&wt, &["worktree", "mint", "feat", "--repos", "backend"]);
    assert!(success, "a re-mint extension attaches: {err}");
    assert!(out.contains("— it carries"), "{out}");
    assert!(!out.contains("linked worktree"), "no gate output: {out}");
    assert!(!err.contains("linked worktree"), "no gate output: {err}");
    assert!(out.contains("member backend:"), "the binding still carries the member: {out}");
    let ls = ok(&wt, &["worktree", "ls"]);
    assert!(ls.contains("member backend:"), "{ls}");
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
    worktree_save(&spec, "boot2", &[("backend", &backend)], "v2 with side baseline");
    let baseline = head(&backend);
    git(&backend, &["switch", "-q", "main"]);

    let (success, _out, err) = run(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    assert!(!success, "an unreachable baseline is a question, not a guess");
    assert!(err.contains("is not on `main`"), "{err}");
    assert!(err.contains("side"), "the candidate branch is named: {err}");
    assert!(err.contains("--base backend="), "{err}");
    let out = ok(&spec, &["worktree", "mint", "feat", "--repos", "backend", "--base", "backend=main"]);
    assert!(out.contains("member backend:"), "{out}");
    assert!(out.contains("(base main)"), "{out}");
    // the behind-by-N gauge belongs to the auto arm alone — the escape lane
    // instead names its departure: `main` does not carry the side baseline
    assert!(!out.contains("behind"), "the auto arm's gauge stays out of the escape lane: {out}");
    assert!(
        out.contains(&format!(
            "note: member backend: the named base `main` does not contain the recorded \
             baseline {} — continuing off the audited line",
            &baseline[..7]
        )),
        "{out}"
    );
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
    worktree_save(&spec, "boot", &[], "seed");
    spec_project(&spec, "[[repo]]\nname = \"backend\"\npath = \"../backend\"\n");
    let (success, _out, err) = run(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    assert!(!success);
    assert!(err.contains("version anchor --repo backend"), "{err}");
    assert!(err.contains("--base backend="), "{err}");
    // the named escape works, and with nothing recorded there is no audited
    // line to depart — the mint stays silent
    let out = ok(&spec, &["worktree", "mint", "feat", "--repos", "backend", "--base", "backend=main"]);
    assert!(out.contains("member backend:"), "{out}");
    assert!(!out.contains("off the audited line"), "{out}");
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
    let wt = spec.parent().unwrap().join("spec-worktrees/feat");
    let bwt = backend.parent().unwrap().join("backend-worktrees/feat");
    // member work lands the contract's way — save mid-flight, commit,
    // anchor in the worktree — so the landing gate passes and the push itself
    // is what refuses below
    fs::write(bwt.join("note.txt"), "work\n").unwrap();
    fs::write(
        wt.join("archi/src/model.arch"),
        format!("{MODEL}def node Extra:\n  port x\n"),
    )
    .unwrap();
    ok(&wt, &["version", "save", "-m", "unit"]);
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "unit"]);
    git(&bwt, &["add", "-A"]);
    git(&bwt, &["commit", "-qm", "work"]);
    ok(&wt, &["version", "anchor", "--repo", "backend"]);
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-qm", "baseline anchored"]);

    // no remote: the push refuses, the member stays bound, nothing retires
    let (success, out, _err) = run(&spec, &["worktree", "merge", "feat"]);
    assert!(!success);
    assert!(out.contains("member backend: kept"), "{out}");
    assert!(bwt.is_dir(), "member worktree stays");

    // repair the remote, re-run: the idempotent close pushes and retires the
    // spec side; the member seat waits on its base
    let bare = ws.join("origin.git");
    git(&ws, &["init", "-q", "--bare", bare.to_str().unwrap()]);
    git(&backend, &["remote", "add", "origin", bare.to_str().unwrap()]);
    let out = ok(&spec, &["worktree", "merge", "feat"]);
    assert!(out.contains("member backend: pushed"), "{out}");
    assert!(out.contains("retired"), "{out}");
    assert!(bwt.is_dir(), "the push is not the arrival");
    let ls = ok(&spec, &["worktree", "ls"]);
    assert!(ls.contains("waiting on archi/feat → main"), "{ls}");
    // the base carries the work: the folder frees itself
    git(&backend, &["merge", "--no-edit", "archi/feat"]);
    ok(&spec, &["worktree", "ls"]);
    assert!(!bwt.exists(), "freed once the base carried it");
}

#[test]
fn close_cascades_over_member_worktrees() {
    let (_ws, spec, backend) = cascade_repo("close-cascade");
    ok(&spec, &["worktree", "mint", "feat", "--repos", "backend"]);
    let bwt = backend.parent().unwrap().join("backend-worktrees/feat");
    assert!(bwt.is_dir());
    let out = ok(&spec, &["worktree", "close", "feat"]);
    assert!(out.contains("closed"), "{out}");
    assert!(!bwt.exists(), "member worktree closed");
    assert!(out.contains("member backend: branch archi/feat stays"), "{out}");
    // the row and its member stay as the record of what this machine carried
    let ls = ok(&spec, &["worktree", "ls", "--spec", "feat"]);
    assert!(ls.contains("spec feat — closed"), "{ls}");
    assert!(ls.contains("member backend:"), "{ls}");
}

#[test]
fn a_doctored_plan_pin_surfaces_as_a_stale_pin_finding() {
    let ws = scratch("stale-plan");
    let spec = ws.join("spec");
    spec_project(&spec, "");
    let spec = util::worktree(&spec);
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
    let spec = util::worktree(&spec);
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
fn a_dirty_spec_outside_a_worktree_fails_check_and_build() {
    let (_ws, spec) = open_repo("verdict");
    // clean unbound tree: both verdicts answer
    ok(&spec, &["check"]);
    ok(&spec, &["build"]);
    // an ungoverned spec edit: both refuse with the worktree recipe
    fs::write(
        spec.join("archi/src/model.arch"),
        format!("{MODEL}def node Rogue:\n  port x\n"),
    )
    .unwrap();
    for command in ["check", "build"] {
        let (success, _out, err) = run(&spec, &[command]);
        assert!(!success, "{command} blessed ungoverned work");
        assert!(err.contains("uncommitted"), "{err}");
        assert!(err.contains("model.arch"), "{err}");
        assert!(err.contains("worktree mint"), "{err}");
    }
    // a non-spec edit alone never trips it
    git(&spec, &["add", "-A"]);
    git(&spec, &["commit", "-qm", "grow"]);
    fs::write(spec.join("notes.md"), "scratch\n").unwrap();
    ok(&spec, &["check"]);
    // the same edit inside a worktree is governed work: check answers
    ok(&spec, &["worktree", "mint", "grow"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/grow");
    fs::write(
        wt.join("archi/src/model.arch"),
        format!("{MODEL}def node Governed:\n  port x\n"),
    )
    .unwrap();
    ok(&wt, &["check"]);
}

#[test]
fn a_hand_removed_worktree_closes_its_row() {
    let (_ws, spec) = protected_repo("heal");
    ok(&spec, &["worktree", "mint", "storm"]);
    let wt = spec.parent().unwrap().join("spec-worktrees/storm");
    git(&spec, &["worktree", "remove", "--force", wt.to_str().unwrap()]);
    // the folder git no longer backs is a finished seat: the row closes and
    // stays — nothing leaves the registry
    let ls = ok(&spec, &["worktree", "ls", "--spec", "storm"]);
    assert!(ls.contains(&format!("{}  archi/storm  spec storm — closed", wt.display())), "{ls}");
    let ls = ok(&spec, &["worktree", "ls", "--spec", "storm", "--status", "active"]);
    assert!(ls.contains("no worktrees match"), "{ls}");
}
