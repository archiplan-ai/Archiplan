//! Shared fixture plumbing: the worktree every e2e mutation runs from.
//!
//! The guard is unconditional — mutating commands run only inside a bound
//! worktree — so a fixture becomes: commit the scaffold, mint the worktree
//! through the binary itself, hand its path to the test. Reads still
//! answer anywhere; only tests that mutate need this.

// Each e2e binary compiles its own copy of this module and uses its own
// slice of it — an unused helper here is sharing, not rot.
#![allow(dead_code)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The planning skill as the binary carries it — the source `sync-skills`
/// installs from. Two families read the same bytes: `init` asks whether the
/// installed copy drifted from it, and `plan` asks what rule it still carries.
pub const SKILL_PLAN: &str = include_str!("../../../../skills/archi-plan.md");

/// One text on one line. Skill prose and the binary's own message strings are
/// both hard-wrapped, so a sentence is read over its line breaks: what a text
/// says must not depend on where a line ends. Two families read it — `init`
/// over the installed skills, `plan` over a refusal — so it stands here.
pub fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

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

/// A directory whose `archi` is the built binary bound to `root`. A printed
/// continuation is a line a person pastes into a shell, so a test pastes it
/// into one: the quoting, the `&&` and the trailing comment all meet a real
/// `sh`, not a parser written to agree with them. `prefix` names the e2e
/// family, as [`scratch`] takes it.
pub fn shim(root: &Path, prefix: &str) -> PathBuf {
    let dir = scratch(prefix, "bin");
    let path = dir.join("archi");
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nexec \"{}\" \"$@\" --project \"{}\"\n",
            env!("CARGO_BIN_EXE_archi"),
            root.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    dir
}

/// Run one line through `sh`, exactly as it was printed, with [`shim`]'s
/// directory ahead of the real `PATH`.
pub fn shell(bin: &Path, line: &str) -> (Option<i32>, String, String) {
    let out = Command::new("sh")
        .arg("-c")
        .arg(line)
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .output()
        .expect("sh runs");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
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
/// name, the conditioning paragraph, what people do instead while the condition
/// holds, and the `Scenarios` block. Every e2e family fills the same skeleton
/// with its own words, so the skeleton lives here once.
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
    /// The workaround and what it costs, under `## What people do instead`.
    pub workaround: &'a str,
    /// The `Scenarios` block: a `### ` heading per scenario, and its step
    /// lines under it.
    pub scenarios: &'a str,
}

impl Fact<'_> {
    /// Write the fact as `archi/world/facts/<slug>.md` — the layer of the
    /// strict record (`archi/requirements/world-facts/the-world-holds-four-layers.md`).
    /// The layer and the folder over it arrive with the file, exactly as the
    /// mint makes them.
    pub fn write(&self, root: &Path, slug: &str, title: &str) {
        let dir = root.join("archi/world/facts");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{slug}.md")),
            format!(
                "---\ncovers: [{}]\nsources: [{}]\nuses: [{}]\n---\n\n\
                 # {title}\n\n{}\n\n## What people do instead\n\n{}\n\n## Scenarios\n\n{}",
                self.covers,
                self.sources,
                self.uses,
                self.condition,
                self.workaround,
                self.scenarios
            ),
        )
        .unwrap();
    }
}

/// The nodes of a fixture's model that no fact of it reaches, declared
/// internal beside the world so `version save` mints
/// (`archi/requirements/world-facts/the-save-refuses-an-unconditioned-element.md`).
///
/// A test writes the one fact its point needs, and the gate on the save asks
/// about the whole model. The nodes left over are named here with the reason
/// the gate wants, which is its second exit; nothing under test moves, because
/// `covers` is still exactly what the test wrote. A test that is about the gate
/// itself writes its own file, so the declaration it exercises is the one a
/// person would write.
pub fn declare_internal(root: &Path, nodes: &[&str]) {
    let dir = root.join("archi/world");
    std::fs::create_dir_all(&dir).unwrap();
    let mut text = String::new();
    for n in nodes {
        text.push_str(&format!(
            "{n} — the fixture's own plumbing: this family conditions the node its test is \
             about, and nothing outside the tool reaches this one\n"
        ));
    }
    std::fs::write(dir.join(".worldignore"), text).unwrap();
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
