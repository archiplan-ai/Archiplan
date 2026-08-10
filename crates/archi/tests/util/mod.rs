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

/// One world fact on disk, in the shape `archi world add` mints and a person
/// fills (`archi/requirements/world-facts/`): the three frontmatter lists, the
/// name, the conditioning paragraph, what would make the fact false, and the
/// `Scenarios` block. Every e2e family fills the same skeleton with its own
/// words, so the skeleton lives here once.
pub struct Fact<'a> {
    /// The `covers:` entries, already comma-joined.
    pub covers: &'a str,
    /// The `sources:` entries, already comma-joined; empty is the recorded
    /// hypothesis state.
    pub sources: &'a str,
    /// The `uses:` entries, already comma-joined.
    pub uses: &'a str,
    /// The conditioning paragraph, under the name.
    pub condition: &'a str,
    /// What would make the fact false, under `## What kills this`.
    pub killer: &'a str,
    /// The `Scenarios` block, its `Feature` line and all.
    pub scenarios: &'a str,
}

impl Fact<'_> {
    /// Write the fact as `archi/world/<slug>.md`. The folder arrives with the
    /// file, exactly as the mint makes it.
    pub fn write(&self, root: &Path, slug: &str, title: &str) {
        let dir = root.join("archi/world");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{slug}.md")),
            format!(
                "---\ncovers: [{}]\nsources: [{}]\nuses: [{}]\n---\n\n\
                 # {title}\n\n{}\n\n## What kills this\n\n{}\n\n## Scenarios\n\n{}",
                self.covers, self.sources, self.uses, self.condition, self.killer, self.scenarios
            ),
        )
        .unwrap();
    }
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
