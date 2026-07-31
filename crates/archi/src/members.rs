//! The member registry: declared repositories resolved to local roots.
//!
//! Identity is committed — `[[repo]]` rows in the manifest, read through the
//! compiler's one reader (`modeling_lang::source::manifest_repos`) — while
//! where a checkout sits is a machine fact: an uncommitted overlay,
//! `archi/repos.local.toml`, maps members per machine and wins over the
//! manifest's committed path convention. Home — the project's own
//! repository — is the implicit member `""` and is always reachable.
//!
//! Resolution never errors on absence: an unmapped member is a value every
//! consumer must branch on, so a half checkout — the normal state of a
//! multi-repo team — reads as narrowed scope, not as loss
//! (`archi/requirements/multi-repo/absence-is-not-drift`).
//!
//! Every git consultation is framed by a [`GitContext`]: the repository's
//! actual top level plus the member root's prefix below it, so git's
//! top-relative paths are rebased into the member's frame before any
//! comparison (`archi/requirements/multi-repo/git-speaks-from-its-own-root`).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The implicit home member's name. Renders as `home` in reports; the empty
/// string keeps unqualified refs and pre-member journal events meaning home
/// without a migration.
pub const HOME: &str = "";

/// The overlay file, project-root-relative: machine-local, gitignored.
pub const OVERLAY: &str = "archi/repos.local.toml";

/// One member, declared or home, with its resolution on this machine.
#[derive(Clone, Debug)]
pub struct Member {
    /// `""` for home.
    pub name: String,
    pub url: Option<String>,
    /// The manifest's committed convention, project-root-relative.
    pub declared_path: Option<String>,
    /// This machine's overlay row, when present (wins over `declared_path`).
    pub mapped_path: Option<String>,
    /// The resolved checkout root; `None` = unreachable on this machine.
    pub root: Option<PathBuf>,
}

impl Member {
    /// The name as reports print it.
    pub fn display_name(&self) -> &str {
        if self.name == HOME { "home" } else { &self.name }
    }

    /// The path the member would resolve through, for reporting.
    pub fn stated_path(&self) -> Option<&str> {
        self.mapped_path.as_deref().or(self.declared_path.as_deref())
    }
}

/// Every member exactly once — home first, then declaration order — so an
/// iterating scan cannot silently skip one.
#[derive(Clone, Debug)]
pub struct MemberSet {
    pub project_root: PathBuf,
    pub members: Vec<Member>,
}

impl MemberSet {
    /// Read declarations and the overlay, resolve each member. Pure read:
    /// nothing on disk changes. Errors are real input faults — an unparsable
    /// manifest or overlay, an overlay row naming an undeclared member —
    /// never absence.
    pub fn resolve(project_root: &Path) -> Result<MemberSet, String> {
        let decls = modeling_lang::source::manifest_repos(project_root)
            .map_err(|d| format!("{}: {}", d.code, d.message))?;
        let overlay = read_overlay(project_root)?;
        for name in overlay.keys() {
            if !decls.iter().any(|d| &d.name == name) {
                return Err(format!(
                    "{OVERLAY} maps `{name}`, which archi.toml does not declare — add its \
                     [[repo]] row or drop the mapping"
                ));
            }
        }
        let mut members = vec![Member {
            name: HOME.to_string(),
            url: None,
            declared_path: None,
            mapped_path: None,
            root: Some(project_root.to_path_buf()),
        }];
        for d in &decls {
            let mapped = overlay.get(&d.name).cloned();
            let stated = mapped.clone().or_else(|| d.path.clone());
            let root = stated.as_deref().and_then(|p| {
                let joined = project_root.join(p);
                joined.is_dir().then(|| {
                    // canonicalized so prefix computation and display agree
                    // with git's own top-level answers (symlinked tmp dirs)
                    fs::canonicalize(&joined).unwrap_or(joined)
                })
            });
            members.push(Member {
                name: d.name.clone(),
                url: d.url.clone(),
                declared_path: d.path.clone(),
                mapped_path: mapped,
                root,
            });
        }
        Ok(MemberSet {
            project_root: project_root.to_path_buf(),
            members,
        })
    }

    pub fn get(&self, name: &str) -> Option<&Member> {
        self.members.iter().find(|m| m.name == name)
    }

    /// True when nothing beyond home is declared — today's project.
    pub fn is_single(&self) -> bool {
        self.members.len() == 1
    }

    /// The declared members (home excluded).
    pub fn declared(&self) -> &[Member] {
        &self.members[1..]
    }
}

fn read_overlay(project_root: &Path) -> Result<BTreeMap<String, String>, String> {
    let path = project_root.join(OVERLAY);
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(BTreeMap::new());
    };
    let value: toml::Value =
        toml::from_str(&text).map_err(|e| format!("{OVERLAY}: {e}"))?;
    let toml::Value::Table(table) = value else {
        return Err(format!("{OVERLAY}: expected a flat `member = \"dir\"` table"));
    };
    let mut out = BTreeMap::new();
    for (k, v) in table {
        match v {
            toml::Value::String(s) => {
                out.insert(k, s);
            }
            _ => {
                return Err(format!(
                    "{OVERLAY}: `{k}` must map to a directory string"
                ));
            }
        }
    }
    Ok(out)
}

/// Map a member to a directory on this machine: writes the overlay row.
/// The member must be declared — the overlay never invents identity — and
/// the directory must be the repo's MAIN checkout: a linked worktree is
/// refused before anything is written.
pub fn map_member(project_root: &Path, name: &str, dir: &str) -> Result<Member, String> {
    let decls = modeling_lang::source::manifest_repos(project_root)
        .map_err(|d| format!("{}: {}", d.code, d.message))?;
    if !decls.iter().any(|d| d.name == name) {
        return Err(format!(
            "`{name}` is not declared — add its [[repo]] row to archi.toml first; \
             the overlay maps identity, it never mints it"
        ));
    }
    if let Some(refusal) = linked_worktree_refusal(dir, &project_root.join(dir), name) {
        return Err(refusal);
    }
    let mut overlay = read_overlay(project_root)?;
    overlay.insert(name.to_string(), dir.to_string());
    let mut text = String::from(
        "# machine-local member checkouts — gitignored, never merged\n",
    );
    for (k, v) in &overlay {
        text.push_str(&format!("{k} = {}\n", toml_string(v)));
    }
    let path = project_root.join(OVERLAY);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    let set = MemberSet::resolve(project_root)?;
    Ok(set.get(name).expect("just mapped").clone())
}

/// The write-time gate on `repo map`: a mapping must name the repo's main
/// checkout. A linked worktree's row outlives the worktree — the checkout
/// retires, the row stands, and a later `worktree mint` bases seats on
/// whatever dead branch stood there. Detection is git's own frame: in a
/// linked worktree the git-dir sits under `<common>/worktrees/<name>`, so
/// git-dir and git-common-dir disagree; in a main checkout they coincide.
/// A path that is no git checkout at all is not this gate's business —
/// `None`, today's behavior untouched.
fn linked_worktree_refusal(dir: &str, target: &Path, member: &str) -> Option<String> {
    let git = |args: &[&str]| -> Option<String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(target)
            .args(args)
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    // git-dir may come back relative to the queried directory; the common
    // dir is asked absolute. Canonicalized, the two agree exactly when the
    // checkout is the main one (symlinked tmp dirs included).
    let canon = |s: &str| {
        let p = PathBuf::from(s);
        let p = if p.is_relative() { target.join(p) } else { p };
        fs::canonicalize(&p).unwrap_or(p)
    };
    let git_dir = canon(&git(&["rev-parse", "--git-dir"])?);
    let common = canon(&git(&["rev-parse", "--path-format=absolute", "--git-common-dir"])?);
    if git_dir == common {
        return None;
    }
    let branch =
        git(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "HEAD".to_string());
    // The first `worktree` entry of the porcelain listing is the main
    // checkout — the ready repair.
    let main = git(&["worktree", "list", "--porcelain"])
        .and_then(|list| {
            list.lines()
                .find_map(|l| l.strip_prefix("worktree ").map(str::to_string))
        })
        .unwrap_or_else(|| common.parent().unwrap_or(&common).display().to_string());
    Some(format!(
        "{dir} is a linked worktree of the repo, standing on `{branch}` — a mapping \
         outlives the worktree and poisons future mints. The main checkout is {main}: \
         `archi repo map {member} {main}`"
    ))
}

pub(crate) fn toml_string(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

// ---------------------------------------------------------------------------
// Git, framed per member

/// A member's git frame: the repository's actual top level and the member
/// root's prefix below it (`""` when they coincide). Git speaks paths
/// relative to `top`; archi speaks paths relative to the member root — every
/// comparison crosses through [`GitContext::rebase`].
#[derive(Clone, Debug)]
pub struct GitContext {
    pub top: PathBuf,
    pub prefix: String,
}

impl GitContext {
    /// The context of the repository containing `dir`, or `None` when `dir`
    /// is not inside a git work tree (or git is unavailable).
    pub fn of(dir: &Path) -> Option<GitContext> {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let top = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
        let top = fs::canonicalize(&top).unwrap_or(top);
        let dir_canon = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        let prefix = dir_canon
            .strip_prefix(&top)
            .ok()?
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        Some(GitContext { top, prefix })
    }

    /// Rebase a top-relative git path into the member's frame. `None` means
    /// the path lives outside the frame — callers drop it with a note, never
    /// compare it raw.
    pub fn rebase(&self, top_relative: &str) -> Option<String> {
        if self.prefix.is_empty() {
            return Some(top_relative.to_string());
        }
        top_relative
            .strip_prefix(&self.prefix)
            .and_then(|rest| rest.strip_prefix('/'))
            .map(str::to_string)
    }

    /// HEAD's sha, when the repository has one.
    pub fn head(&self) -> Option<String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.top)
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()?;
        out.status.success().then(|| {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        })
    }

    /// Whether the member's subtree is clean — scoped to the frame, so a
    /// dirty sibling in a shared repository never dirties this member.
    pub fn clean(&self) -> Option<bool> {
        let scope = if self.prefix.is_empty() {
            ".".to_string()
        } else {
            self.prefix.clone()
        };
        let out = Command::new("git")
            .arg("-C")
            .arg(&self.top)
            .args(["status", "--porcelain", "--"])
            .arg(&scope)
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| out.stdout.iter().all(|b| b.is_ascii_whitespace()))
    }
}

// ---------------------------------------------------------------------------
// Survey — the doctor's read

/// One member's row in `repo ls`.
#[derive(Clone, Debug)]
pub struct MemberStatus {
    pub name: String,
    pub display_name: String,
    pub url: Option<String>,
    pub stated_path: Option<String>,
    pub root: Option<PathBuf>,
    pub clean: Option<bool>,
    pub head: Option<String>,
    pub baseline: Option<String>,
}

/// Survey every member: resolution, git state, and the baseline column read
/// verbatim from the latest version entry — shown as recorded, never
/// recomputed.
pub fn survey(set: &MemberSet) -> Vec<MemberStatus> {
    let baselines = latest_baselines(&set.project_root);
    set.members
        .iter()
        .map(|m| {
            let git = m.root.as_deref().and_then(GitContext::of);
            MemberStatus {
                name: m.name.clone(),
                display_name: m.display_name().to_string(),
                url: m.url.clone(),
                stated_path: m.stated_path().map(str::to_string),
                root: m.root.clone(),
                clean: git.as_ref().and_then(GitContext::clean),
                head: git.as_ref().and_then(GitContext::head),
                baseline: baselines.get(&m.name).cloned(),
            }
        })
        .collect()
}

/// The latest version entry's provenance, per member: home's `commit` field
/// and the `commits` table, read leniently — the archive stays versions.rs's
/// to write.
fn latest_baselines(project_root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let path = project_root.join("archi").join("versions").join("index.toml");
    let Ok(text) = fs::read_to_string(&path) else {
        return out;
    };
    let Ok(value) = toml::from_str::<toml::Value>(&text) else {
        return out;
    };
    let Some(last) = value
        .get("version")
        .and_then(|v| v.as_array())
        .and_then(|a| a.last())
    else {
        return out;
    };
    if let Some(c) = last.get("commit").and_then(|v| v.as_str()) {
        out.insert(HOME.to_string(), format!("{c} (save)"));
    }
    if let Some(commits) = last.get("commits").and_then(|v| v.as_table()) {
        for (name, entry) in commits {
            let rendered = match entry {
                toml::Value::String(sha) => Some(format!("{sha} (save)")),
                toml::Value::Table(t) => t.get("sha").and_then(|v| v.as_str()).map(|sha| {
                    match t.get("born").and_then(|v| v.as_str()) {
                        Some(born) => format!("{sha} ({born})"),
                        None => format!("{sha} (save)"),
                    }
                }),
                _ => None,
            };
            if let Some(r) = rendered {
                out.insert(name.clone(), r);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn scratch() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-members-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::canonicalize(&dir).unwrap()
    }

    fn write_manifest(root: &Path, extra: &str) {
        fs::write(
            root.join("archi.toml"),
            format!("[project]\nname = \"t\"\n{extra}"),
        )
        .unwrap();
    }

    fn git(dir: &Path, args: &[&str]) {
        let out = Command::new("git").arg("-C").arg(dir).args(args).output().unwrap();
        assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    }

    fn git_init(dir: &Path) {
        git(dir, &["init", "-q"]);
        git(dir, &["config", "user.email", "t@t"]);
        git(dir, &["config", "user.name", "t"]);
    }

    #[test]
    fn no_declarations_resolve_to_home_alone() {
        let root = scratch();
        write_manifest(&root, "");
        let set = MemberSet::resolve(&root).unwrap();
        assert!(set.is_single());
        assert_eq!(set.members.len(), 1);
        assert_eq!(set.members[0].name, HOME);
        assert_eq!(set.members[0].root.as_deref(), Some(root.as_path()));
    }

    #[test]
    fn every_declared_member_appears_exactly_once_mapped_or_not() {
        let root = scratch();
        fs::create_dir_all(root.join("checkout")).unwrap();
        write_manifest(
            &root,
            "[[repo]]\nname = \"here\"\npath = \"checkout\"\n\n[[repo]]\nname = \"gone\"\npath = \"nowhere\"\n",
        );
        let set = MemberSet::resolve(&root).unwrap();
        let names: Vec<_> = set.members.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, ["", "here", "gone"]);
        assert!(set.get("here").unwrap().root.is_some());
        assert!(set.get("gone").unwrap().root.is_none(), "absence is a value, not an error");
    }

    #[test]
    fn the_overlay_overrides_the_manifest_path() {
        let root = scratch();
        fs::create_dir_all(root.join("actual")).unwrap();
        fs::create_dir_all(root.join("archi")).unwrap();
        write_manifest(&root, "[[repo]]\nname = \"backend\"\npath = \"declared\"\n");
        fs::write(root.join(OVERLAY), "backend = \"actual\"\n").unwrap();
        let set = MemberSet::resolve(&root).unwrap();
        let m = set.get("backend").unwrap();
        assert_eq!(m.mapped_path.as_deref(), Some("actual"));
        assert!(m.root.as_ref().unwrap().ends_with("actual"));
    }

    #[test]
    fn an_overlay_row_for_an_undeclared_member_is_loud() {
        let root = scratch();
        fs::create_dir_all(root.join("archi")).unwrap();
        write_manifest(&root, "");
        fs::write(root.join(OVERLAY), "ghost = \"somewhere\"\n").unwrap();
        let e = MemberSet::resolve(&root).unwrap_err();
        assert!(e.contains("ghost"), "{e}");
        assert!(e.contains("does not declare"), "{e}");
    }

    #[test]
    fn map_member_writes_the_row_and_refuses_undeclared_names() {
        let root = scratch();
        fs::create_dir_all(root.join("co")).unwrap();
        write_manifest(&root, "[[repo]]\nname = \"backend\"\n");
        let e = map_member(&root, "ghost", "co").unwrap_err();
        assert!(e.contains("not declared"), "{e}");
        let m = map_member(&root, "backend", "co").unwrap();
        assert!(m.root.is_some());
        // re-map updates the same row
        fs::create_dir_all(root.join("co2")).unwrap();
        let m = map_member(&root, "backend", "co2").unwrap();
        assert_eq!(m.mapped_path.as_deref(), Some("co2"));
        let text = fs::read_to_string(root.join(OVERLAY)).unwrap();
        assert_eq!(text.matches("backend").count(), 1);
    }

    #[test]
    fn map_member_refuses_a_linked_worktree_and_names_the_repair() {
        let root = scratch();
        write_manifest(&root, "[[repo]]\nname = \"backend\"\n");
        let repo = scratch();
        git_init(&repo);
        fs::write(repo.join("f.txt"), "x").unwrap();
        git(&repo, &["add", "-A"]);
        git(&repo, &["commit", "-qm", "seed"]);
        let wt = scratch().join("wt");
        git(&repo, &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "feature/dead"]);
        let e = map_member(&root, "backend", wt.to_str().unwrap()).unwrap_err();
        assert!(e.contains("linked worktree"), "{e}");
        assert!(e.contains("`feature/dead`"), "the standing branch is named: {e}");
        assert!(e.contains(repo.to_str().unwrap()), "the main checkout is the repair: {e}");
        assert!(e.contains("archi repo map backend"), "{e}");
        assert!(!root.join(OVERLAY).exists(), "no row written on refusal");
        // The named repair goes through as before.
        let m = map_member(&root, "backend", repo.to_str().unwrap()).unwrap();
        assert_eq!(m.root.as_deref(), Some(repo.as_path()));
    }

    #[test]
    fn resolution_is_a_pure_read() {
        let root = scratch();
        write_manifest(&root, "[[repo]]\nname = \"backend\"\npath = \"co\"\n");
        let before: Vec<_> = fs::read_dir(&root).unwrap().map(|e| e.unwrap().file_name()).collect();
        MemberSet::resolve(&root).unwrap();
        let after: Vec<_> = fs::read_dir(&root).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(before, after);
    }

    #[test]
    fn a_nested_member_rebases_git_paths_into_its_frame() {
        let repo = scratch();
        git_init(&repo);
        let member = repo.join("services").join("backend");
        fs::create_dir_all(&member).unwrap();
        let ctx = GitContext::of(&member).unwrap();
        assert_eq!(ctx.prefix, "services/backend");
        assert_eq!(
            ctx.rebase("services/backend/src/main.rs").as_deref(),
            Some("src/main.rs")
        );
        assert_eq!(ctx.rebase("services/other/src/lib.rs"), None, "outside the frame");
        assert_eq!(ctx.rebase("services/backendish/x.rs"), None, "prefix match is per segment");
    }

    #[test]
    fn an_unnested_context_rebases_identity() {
        let repo = scratch();
        git_init(&repo);
        let ctx = GitContext::of(&repo).unwrap();
        assert_eq!(ctx.prefix, "");
        assert_eq!(ctx.rebase("src/main.rs").as_deref(), Some("src/main.rs"));
    }

    #[test]
    fn cleanliness_is_scoped_to_the_member_frame() {
        let repo = scratch();
        git_init(&repo);
        let member = repo.join("backend");
        fs::create_dir_all(&member).unwrap();
        fs::write(member.join("kept.txt"), "x").unwrap();
        git(&repo, &["add", "."]);
        git(&repo, &["commit", "-qm", "seed"]);
        fs::write(repo.join("elsewhere.txt"), "dirt outside the frame").unwrap();
        let ctx = GitContext::of(&member).unwrap();
        assert_eq!(ctx.clean(), Some(true), "a dirty sibling never dirties this member");
        fs::write(member.join("new.txt"), "dirt inside").unwrap();
        assert_eq!(ctx.clean(), Some(false));
    }

    #[test]
    fn survey_reads_baselines_verbatim_from_the_latest_entry() {
        let root = scratch();
        write_manifest(&root, "[[repo]]\nname = \"backend\"\n");
        let vdir = root.join("archi").join("versions");
        fs::create_dir_all(&vdir).unwrap();
        fs::write(
            vdir.join("index.toml"),
            "[[version]]\nid = \"v0001\"\ncommit = \"aaa111\"\n\n[[version]]\nid = \"v0002\"\ncommit = \"bbb222\"\n\n[version.commits]\nbackend = { sha = \"ccc333\", born = \"anchor\" }\n",
        )
        .unwrap();
        let set = MemberSet::resolve(&root).unwrap();
        let rows = survey(&set);
        assert_eq!(rows[0].baseline.as_deref(), Some("bbb222 (save)"));
        assert_eq!(rows[1].baseline.as_deref(), Some("ccc333 (anchor)"));
    }
}
