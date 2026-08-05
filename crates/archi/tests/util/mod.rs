//! Shared fixture plumbing: the worktree every e2e mutation runs from.
//!
//! The guard is unconditional — mutating commands run only inside a bound
//! worktree — so a fixture becomes: commit the scaffold, mint the worktree
//! through the binary itself, hand its path to the test. Reads still
//! answer anywhere; only tests that mutate need this.

// Each e2e binary compiles its own copy of this module and uses its own
// slice of it — an unused helper here is sharing, not rot.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// A fresh scratch directory, canonicalized so paths agree with git's own
/// answers on symlinked tmp dirs. `prefix` names the e2e family; pid and a
/// counter keep parallel tests apart.
pub fn scratch(prefix: &str, tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{prefix}-{tag}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::canonicalize(&dir).unwrap()
}

/// Run the built binary against `root`: success flag, stdout, stderr.
pub fn run(root: &Path, args: &[&str]) -> (bool, String, String) {
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

/// [`run`], asserting success; hands back stdout.
pub fn ok(root: &Path, args: &[&str]) -> String {
    let (success, stdout, stderr) = run(root, args);
    assert!(success, "archi {args:?} failed:\n{stdout}\n{stderr}");
    stdout
}

pub fn git(dir: &Path, args: &[&str]) {
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

/// Turn a scaffolded directory into a committed repo and mint its worktree;
/// returns the worktree the test mutates from. The primary checkout
/// stays on `main`, unbound and untouched.
pub fn worktree(fixture: &Path) -> PathBuf {
    git(fixture, &["init", "-q", "-b", "main"]);
    git(fixture, &["config", "user.email", "t@t"]);
    git(fixture, &["config", "user.name", "t"]);
    git(fixture, &["config", "commit.gpgsign", "false"]);
    git(fixture, &["add", "-A"]);
    git(fixture, &["commit", "-qm", "seed"]);
    let out = Command::new(env!("CARGO_BIN_EXE_archi"))
        .args(["worktree", "mint", "wt", "--project", fixture.to_str().unwrap()])
        .output()
        .expect("archi runs");
    assert!(
        out.status.success(),
        "worktree mint: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let name = fixture.file_name().unwrap().to_str().unwrap();
    let wt = fixture
        .parent()
        .unwrap()
        .join(format!("{name}-worktrees"))
        .join("wt");
    std::fs::canonicalize(&wt).unwrap_or(wt)
}
