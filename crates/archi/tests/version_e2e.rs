//! End to end through the real binary: `version save` mints on semantic
//! change only, and the round ceremony — stamping the open stress session
//! closed, firing the incidence report — finishes whether or not a
//! version minted; the bare no-op is a success and genuine failures stay
//! loud (`archi/requirements/self-hosting/unchanged-saves-close-rounds.md`).

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const MODEL: &str = "def conn wire := * -> *\n\
                     def node Gate:\n  port out\n\
                     def node Auth:\n  port inn\n\
                     Gate.out wire Auth.inn\n";

fn temp_project() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-version-e2e-{}-{}",
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
    util::seat(&dir)
}

/// An open stress session pinned to `version`, with one stressor so the
/// round is schema-shaped like a real one.
fn open_session(root: &Path, slug: &str, title: &str, version: &str) {
    let dir = root.join("archi/stress").join(slug);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("{slug}.md")),
        format!("---\nversion: {version}\nclosed:\n---\n\n# {title}\n\nThe round.\n"),
    )
    .unwrap();
    fs::write(
        dir.join("push.md"),
        "---\naffects: [Gate]\noutcome: surviving\n---\n\n# Push\n\nPressure on the gate.\n\n\
         ## Attractor\n\nDrift.\n\n## Resolution\n\nHolds.\n",
    )
    .unwrap();
}

/// Run the binary; return (success, stdout, stderr).
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

fn version_count(root: &Path) -> usize {
    fs::read_to_string(root.join("archi/versions/index.toml"))
        .unwrap()
        .matches("[[version]]")
        .count()
}

#[test]
fn unchanged_save_closes_the_open_session() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    open_session(&root, "round-one", "Round one", "v0001");
    let out = ok(&root, &["version", "save", "-m", "behavior-only round"]);
    assert!(
        out.contains("nothing to mint: the model is unchanged since v0001"),
        "{out}"
    );
    assert!(
        out.contains("closed stress session `round-one` at v0001"),
        "{out}"
    );
    assert!(out.contains("incidence — session `round-one`"), "{out}");
    let session = fs::read_to_string(root.join("archi/stress/round-one/round-one.md")).unwrap();
    assert!(session.contains("closed: v0001"), "{session}");
    assert_eq!(version_count(&root), 1, "the close mints nothing");
}

#[test]
fn unchanged_save_with_no_open_session_is_a_benign_noop() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    let out = ok(&root, &["version", "save", "-m", "again"]);
    assert!(
        out.contains("nothing to save: the model is unchanged since v0001 and no session is open"),
        "{out}"
    );
    assert_eq!(version_count(&root), 1);
}

#[test]
fn changed_save_still_mints_and_closes_at_the_minted_id() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    open_session(&root, "round-one", "Round one", "v0001");
    fs::write(
        root.join("archi/src/model.arch"),
        format!("{MODEL}def node Store:\n  port inn\n"),
    )
    .unwrap();
    let out = ok(&root, &["version", "save", "-m", "grew"]);
    assert!(out.contains("saved v0002"), "{out}");
    assert!(out.contains("closed stress session `round-one`"), "{out}");
    let session = fs::read_to_string(root.join("archi/stress/round-one/round-one.md")).unwrap();
    assert!(session.contains("closed: v0002"), "{session}");
    assert_eq!(version_count(&root), 2);
}

#[test]
fn two_open_sessions_keep_failing_the_save() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    open_session(&root, "round-one", "Round one", "v0001");
    open_session(&root, "round-two", "Round two", "v0001");
    let (success, _, stderr) = run(&root, &["version", "save", "-m", "jam"]);
    assert!(!success, "two open sessions must fail the save");
    assert!(stderr.contains("are all open"), "{stderr}");
    assert_eq!(version_count(&root), 1);
}
