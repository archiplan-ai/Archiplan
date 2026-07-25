//! Shared fixture plumbing: the seat every e2e mutation runs from.
//!
//! The guard is unconditional — mutating verbs run only inside a seated
//! worktree — so a fixture becomes: commit the scaffold, mint the seat
//! through the binary itself, hand the seat path to the test. Reads still
//! answer anywhere; only tests that mutate need this.

use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Turn a scaffolded directory into a committed repo and mint its seat;
/// returns the seat worktree the test mutates from. The primary checkout
/// stays on `main`, unbound and untouched.
pub fn seat(fixture: &Path) -> PathBuf {
    git(fixture, &["init", "-q", "-b", "main"]);
    git(fixture, &["config", "user.email", "t@t"]);
    git(fixture, &["config", "user.name", "t"]);
    git(fixture, &["config", "commit.gpgsign", "false"]);
    git(fixture, &["add", "-A"]);
    git(fixture, &["commit", "-qm", "seed"]);
    let out = Command::new(env!("CARGO_BIN_EXE_archi"))
        .args(["worktree", "mint", "seat", "--project", fixture.to_str().unwrap()])
        .output()
        .expect("archi runs");
    assert!(
        out.status.success(),
        "worktree mint: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let name = fixture.file_name().unwrap().to_str().unwrap();
    let seat = fixture
        .parent()
        .unwrap()
        .join(format!("{name}-worktrees"))
        .join("seat");
    std::fs::canonicalize(&seat).unwrap_or(seat)
}
