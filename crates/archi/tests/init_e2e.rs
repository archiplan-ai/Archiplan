//! End to end through the real binary: one verb stands a project up
//! (`archi/requirements/cold-start/one-verb-stands-a-project-up`), a second
//! run changes no bytes, the manifest routes the starter and aborts the
//! broken run, the briefing lands verbatim, and the verbs around init keep
//! their contracts.

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// This binary's embedded briefing sources, for byte-equality checks.
const SKILL_ARCHI: &str = include_str!("../../../skills/archi.md");
const SKILL_MERGE: &str = include_str!("../../../skills/archi-merge.md");
const SKILL_MIGRATE: &str = include_str!("../../../skills/archi-migrate-fractal.md");

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-init-e2e-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Run the real binary with `dir` as the working directory; exit code,
/// stdout, stderr.
fn run_in(dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_archi"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("archi runs");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn ok_in(dir: &Path, args: &[&str]) -> String {
    let (code, stdout, stderr) = run_in(dir, args);
    assert_eq!(code, Some(0), "archi {args:?} failed:\n{stdout}\n{stderr}");
    stdout
}

/// Every file under `dir` with its bytes, path-sorted.
fn snapshot(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        for e in fs::read_dir(dir).unwrap().flatten() {
            let path = e.path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                let bytes = fs::read(&path).unwrap();
                out.push((path, bytes));
            }
        }
    }
    let mut out = Vec::new();
    walk(dir, &mut out);
    out.sort();
    out
}

#[test]
fn a_fresh_init_stands_up_a_building_project() {
    let root = temp_dir();
    let out = ok_in(&root, &["init", "proj"]);

    // The report: every artifact created, the manifest on the last created
    // line, the verdict naming the project.
    let created: Vec<&str> = out.lines().filter(|l| l.starts_with("created")).collect();
    assert_eq!(created.len(), 11, "{out}");
    assert!(created.last().unwrap().contains("archi.toml"), "{out}");
    assert!(out.contains("initialized `proj`"), "{out}");

    // Seat artifacts are ignored from birth — machine-local, never merged.
    let ignore = fs::read_to_string(root.join("proj/.gitignore")).unwrap();
    assert!(ignore.contains("archi/*.local.toml"), "{ignore}");
    assert!(ignore.contains("archi/plans/.current"), "{ignore}");

    // The seat discipline is declared from birth; deleting the line opts out.
    let manifest = fs::read_to_string(root.join("proj/archi.toml")).unwrap();
    assert!(manifest.contains("protected = [\"main\"]"), "{manifest}");

    // The scaffolded tree is a passing, empty project.
    let build = ok_in(&root, &["build", "--project", "proj"]);
    assert!(build.contains("ok: 0 statements"), "{build}");
    let check = ok_in(&root, &["check", "--project", "proj"]);
    assert!(check.contains("no findings"), "{check}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_second_init_changes_no_bytes_and_extra_args_are_usage_errors() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let before = snapshot(&root);

    let out = ok_in(&root, &["init", "."]);
    assert!(out.contains("already initialized"), "{out}");
    assert!(!out.contains("created"), "{out}");
    assert_eq!(before, snapshot(&root));

    // A second directory and a --project are both malformed invocations.
    let (code, _, _) = run_in(&root, &["init", "a", "b"]);
    assert_eq!(code, Some(2));
    let (code, _, _) = run_in(&root, &["init", "--project", "a"]);
    assert_eq!(code, Some(2));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn the_manifest_routes_the_starter_and_a_broken_one_aborts() {
    let root = temp_dir();
    fs::write(
        root.join("archi.toml"),
        "[project]\nname = \"t\"\nsrc = \"spec\"\npreset = \"default\"\n",
    )
    .unwrap();
    ok_in(&root, &["init", "."]);
    assert!(root.join("spec/model.arch").is_file());
    assert!(!root.join("archi/src").exists());
    let build = ok_in(&root, &["build", "--project", "."]);
    assert!(build.contains("ok: 0 statements"), "{build}");

    // A manifest that fails to parse stops the run before a byte lands.
    let broken = temp_dir();
    fs::write(broken.join("archi.toml"), "not toml at all [").unwrap();
    let (code, _, stderr) = run_in(&broken, &["init", "."]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("archi.toml"), "{stderr}");
    assert!(!broken.join(".claude").exists());
    assert!(!broken.join("CLAUDE.md").exists());

    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(&broken).unwrap();
}

#[test]
fn the_briefing_lands_verbatim_and_the_fence_appends_once() {
    let root = temp_dir();
    fs::write(root.join("CLAUDE.md"), "# House rules\n\nTabs are love.\n").unwrap();
    fs::write(root.join(".gitignore"), "target/\narchi/*.local.toml\n").unwrap();
    ok_in(&root, &["init", "."]);

    // A partial .gitignore gains only its missing line, exactly once.
    let ignore = fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(ignore.starts_with("target/\n"), "{ignore}");
    assert_eq!(ignore.matches("archi/*.local.toml").count(), 1, "{ignore}");
    assert_eq!(ignore.matches("archi/plans/.current").count(), 1, "{ignore}");

    for (skill, text) in [
        ("archi", SKILL_ARCHI),
        ("archi-merge", SKILL_MERGE),
        ("archi-migrate-fractal", SKILL_MIGRATE),
    ] {
        let installed =
            fs::read_to_string(root.join(".claude/skills").join(skill).join("SKILL.md")).unwrap();
        assert_eq!(installed, text, "{skill} drifted on install");
    }

    let claude = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert!(claude.starts_with("# House rules\n\nTabs are love.\n"), "{claude}");
    assert_eq!(claude.matches("<!-- archi:begin -->").count(), 1, "{claude}");

    ok_in(&root, &["init", "."]);
    let claude = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert_eq!(claude.matches("<!-- archi:begin -->").count(), 1, "{claude}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn init_owes_nothing_to_the_compiler_and_advisory_state_never_blocks() {
    // A model that does not compile: init still equips the tree, exit zero.
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    fs::write(root.join("archi/src/model.arch"), "Ghost.out wire Phantom.inn\n").unwrap();
    fs::remove_dir_all(root.join(".claude")).unwrap();
    let (code, _, _) = run_in(&root, &["check"]);
    assert_eq!(code, Some(1));
    let out = ok_in(&root, &["init", "."]);
    assert!(out.contains("created  .claude/skills/archi/SKILL.md"), "{out}");

    // A KB carrying advisory findings inits exactly as a clean one.
    fs::write(root.join("archi/src/model.arch"), "def node Lone\n").unwrap();
    fs::create_dir_all(root.join("archi/requirements/an-intent")).unwrap();
    fs::write(
        root.join("archi/requirements/an-intent/an-intent.md"),
        "# An intent\n\nA problem worth modeling.\n",
    )
    .unwrap();
    fs::write(
        root.join("archi/requirements/an-intent/still-open.md"),
        "---\nkind: functional\norigin: intent\nsatisfied-by: []\ndeferred:\n---\n\n\
         # Still open\n\nAn unsatisfied claim.\n\n## System Context\n\n## Satisfy\n",
    )
    .unwrap();
    let check = ok_in(&root, &["check"]);
    assert!(check.contains("unsatisfied requirement"), "{check}");
    let out = ok_in(&root, &["init", "."]);
    assert!(out.contains("already initialized"), "{out}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn the_verbs_around_init_keep_their_contracts() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);

    // Search runs on the fresh project — the dispatch gained a verb and
    // lost none.
    let (code, _, _) = run_in(&root, &["search", "anything", "at", "all"]);
    assert_eq!(code, Some(0));

    // Gitless mutation is a full stop naming the repair, never a silent
    // bare run — and the discipline is unconditional: deleting the
    // `protected` line opts nothing out.
    let (code, _, stderr) = run_in(&root, &["version", "save", "-m", "first"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("git init"), "{stderr}");
    let manifest = fs::read_to_string(root.join("archi.toml")).unwrap();
    fs::write(root.join("archi.toml"), manifest.replace("protected = [\"main\"]\n", "")).unwrap();
    let (code, _, stderr) = run_in(&root, &["version", "save", "-m", "first"]);
    assert_eq!(code, Some(1), "no opt-out: {stderr}");
    assert!(stderr.contains("git init"), "{stderr}");

    // Seated, the flow runs: a save, an init, a save — init minted nothing
    // the second save could notice.
    let seat = util::seat(&root);
    ok_in(&seat, &["version", "save", "-m", "first"]);
    let out = ok_in(&seat, &["init", "."]);
    assert!(out.contains("already initialized"), "{out}");
    let (_, stdout, stderr) = run_in(&seat, &["version", "save", "-m", "again"]);
    assert!(
        (stdout.clone() + &stderr).contains("unchanged since v0001"),
        "{stdout}\n{stderr}"
    );

    let _ = fs::remove_dir_all(root.parent().unwrap().join(format!(
        "{}-worktrees",
        root.file_name().unwrap().to_str().unwrap()
    )));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_nested_init_names_the_enclosing_root() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let out = ok_in(&root, &["init", "services/billing"]);
    assert!(out.contains("enclosing project"), "{out}");
    assert!(root.join("services/billing/archi.toml").is_file());
    fs::remove_dir_all(&root).unwrap();
}
