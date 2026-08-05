//! Git plumbing shared by every module that consults a repository.
//!
//! Two shapes cover every call. [`out`] is the lenient query — absence is a
//! value, `None` on any failure — the shape every read probe rides.
//! [`run`] is the loud mutation — `Err` carrying git's own stderr — the
//! shape every state change answers with. Beside them sit the small facts
//! callers kept restating: canonicalized paths ([`canon`]), seven-char
//! shas ([`sha7`]), and the linked-worktree probe ([`linked_worktree`]).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Lenient query: trimmed stdout on success, `None` on any failure (no
/// git, not a repository, bad rev) — the degrade every probe rides.
pub(crate) fn out(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git").arg("-C").arg(dir).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Loud mutation: trimmed stdout, or `Err` carrying git's stderr verbatim.
pub(crate) fn run(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// Canonicalize with the input as the fallback, so prefix computation and
/// display agree with git's own answers (symlinked tmp dirs included) and
/// a missing path stays displayable.
pub(crate) fn canon(p: &Path) -> PathBuf {
    fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

/// The first seven characters of a sha — length-safe on short input.
pub(crate) fn sha7(sha: &str) -> &str {
    &sha[..sha.len().min(7)]
}

/// A linked worktree seen at some directory: the branch standing in it and
/// the repository's main checkout.
pub(crate) struct LinkedWorktree {
    pub branch: String,
    /// The main checkout as git states it — not canonicalized, so refusal
    /// messages show the path the user knows.
    pub main: PathBuf,
}

/// The linked-worktree probe: in a linked worktree the git dir sits under
/// `<common>/worktrees/<name>`, so git-dir and git-common-dir disagree; in
/// a main checkout they coincide. `None` for a main checkout — and for a
/// path that is no git checkout at all, which is not this probe's business.
pub(crate) fn linked_worktree(dir: &Path) -> Option<LinkedWorktree> {
    // git-dir may come back relative to the queried directory; the common
    // dir is asked absolute. Canonicalized, the two agree exactly when the
    // checkout is the main one (symlinked tmp dirs included).
    let abs = |s: &str| {
        let p = PathBuf::from(s);
        canon(&if p.is_relative() { dir.join(p) } else { p })
    };
    let git_dir = abs(&out(dir, &["rev-parse", "--git-dir"])?);
    let common = abs(&out(dir, &["rev-parse", "--path-format=absolute", "--git-common-dir"])?);
    if git_dir == common {
        return None;
    }
    let branch = out(dir, &["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_else(|| "HEAD".to_string());
    // The first `worktree` entry of the porcelain listing is the main
    // checkout — the ready repair.
    let main = out(dir, &["worktree", "list", "--porcelain"])
        .and_then(|list| {
            list.lines()
                .find_map(|l| l.strip_prefix("worktree ").map(PathBuf::from))
        })
        .unwrap_or_else(|| common.parent().unwrap_or(&common).to_path_buf());
    Some(LinkedWorktree { branch, main })
}
