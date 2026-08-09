//! Worktree bindings: the machine-local registry that binds work to checkouts.
//!
//! Parallel agent sessions each live in their own git worktree; which
//! worktree carries which plan is a machine fact, not shared truth. The
//! registry lives under the repository's common git dir — the one place
//! every worktree shares, tracked by none — and moves only through commands:
//! mint writes rows, the landing marks them, `worktree ls`/`close` read and
//! repair (`archi/requirements/worktree-parallelism/`).
//!
//! A row outlives its folder. It carries `active` or `closed`, never
//! disappears, and a side that landed carries where the work went. The
//! folder is a disposable derivative: the sweep frees it as soon as the
//! receiving branch carries its content, and the row closes when every
//! side arrived.
//!
//! Git queries are lenient (`Option`, absence is a value); git mutations are
//! loud (`Result` carrying git's own stderr). No command here ever changes the
//! caller's working directory — minting prints the path, entering it is the
//! caller's move.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::gitcmd::{self, canon, out as git_out, run as git_run};

/// The registry file, relative to the common git dir.
const REGISTRY: &str = "archi/worktrees.toml";

/// The branch a slug's work lives on.
pub fn branch_of(slug: &str) -> String {
    format!("archi/{slug}")
}

// ---------------------------------------------------------------------------
// Git plumbing — the shapes live in [`crate::gitcmd`]; here only the
// derivations this module owns.

/// The work tree containing `dir`, canonicalized.
pub fn toplevel(dir: &Path) -> Option<PathBuf> {
    git_out(dir, &["rev-parse", "--show-toplevel"]).map(|s| canon(Path::new(&s)))
}

/// The common git dir every worktree of the repository shares.
pub fn common_dir(dir: &Path) -> Option<PathBuf> {
    git_out(dir, &["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .map(|s| canon(Path::new(&s)))
}

/// The checked-out branch; `None` when detached (or no git).
pub fn current_branch(dir: &Path) -> Option<String> {
    let b = git_out(dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    (b != "HEAD").then_some(b)
}

/// One live worktree as git lists it.
pub struct Wt {
    pub path: PathBuf,
    pub branch: Option<String>,
}

/// Every worktree of the repository containing `dir`, canonicalized.
pub fn list_worktrees(dir: &Path) -> Vec<Wt> {
    let Some(text) = git_out(dir, &["worktree", "list", "--porcelain"]) else {
        return Vec::new();
    };
    let mut out: Vec<Wt> = Vec::new();
    for line in text.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            out.push(Wt { path: canon(Path::new(p)), branch: None });
        } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
            if let Some(w) = out.last_mut() {
                w.branch = Some(b.to_string());
            }
        }
    }
    out
}

/// Whether `refs/heads/<name>` stands here — the same verified read the
/// integration probe rides, asked for a yes or no.
pub fn branch_exists(repo: &Path, name: &str) -> bool {
    commit_of(repo, &format!("refs/heads/{name}")).is_some()
}

fn worktree_add(
    repo: &Path,
    path: &Path,
    branch: &str,
    create: bool,
    base: Option<&str>,
) -> Result<(), String> {
    let path_s = path.to_string_lossy().into_owned();
    // `--no-track`: a base of `origin/<branch>` would otherwise make the
    // remote branch this seat's upstream, so a bare `git push` in the seat
    // would aim at someone else's branch. A seat's work travels by the
    // landing's explicit refspec, never by an inherited upstream.
    let mut args: Vec<&str> = if create {
        vec!["worktree", "add", "--no-track", "-b", branch, &path_s]
    } else {
        vec!["worktree", "add", &path_s, branch]
    };
    if create {
        // a new branch grows from the chosen base's tip, not from HEAD
        if let Some(b) = base {
            args.push(b);
        }
    }
    git_run(repo, &args).map(|_| ())
}

/// Keep archi's machine-local worktree artifacts out of git without touching
/// any committed file: the repo-local exclude (`$GIT_COMMON_DIR/info/
/// exclude`, shared by every worktree, never committed) gains the overlay
/// and marker patterns, so an agent's `git add -A` cannot leak machine
/// paths into a branch (branches-stay-transport). Best effort, idempotent.
fn ensure_excludes(top: &Path) {
    let Some(common) = common_dir(top) else {
        return;
    };
    let path = common.join("info").join("exclude");
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let missing: Vec<&str> = ["**/archi/*.local.toml", "**/archi/plans/.current"]
        .into_iter()
        .filter(|p| !existing.lines().any(|l| l.trim() == *p))
        .collect();
    if missing.is_empty() {
        return;
    }
    let Some(parent) = path.parent() else { return };
    let _ = fs::create_dir_all(parent);
    let mut text = existing;
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str("# archi worktree artifacts — machine-local, never committed\n");
    for p in missing {
        text.push_str(p);
        text.push('\n');
    }
    let _ = fs::write(&path, text);
}

/// Remove the worktree artifacts archi itself wrote into a worktree — the
/// member overlay and the active-plan marker — so a retire's non-force
/// `worktree remove` only ever refuses over the *user's* uncommitted work.
pub fn scrub_worktree(wt_project: &Path) {
    let _ = fs::remove_file(wt_project.join(crate::members::OVERLAY));
    let _ = fs::remove_file(wt_project.join("archi").join("plans").join(".current"));
}

pub fn worktree_remove(repo: &Path, path: &Path, force: bool) -> Result<(), String> {
    let path_s = path.to_string_lossy().into_owned();
    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(&path_s);
    git_run(repo, &args).map(|_| ())
}

// ---------------------------------------------------------------------------
// The registry

/// Whether a row still stands, or is history. Absent in a file written
/// before the status existed, which reads as `active` — every registry
/// ever written keeps loading.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Live work: the row binds, the guard licenses, the folder stands.
    #[default]
    Active,
    /// History: the work arrived (or the operator closed the seat). The row
    /// stays as the record of what this machine carried and licenses nothing.
    Closed,
}

impl Status {
    pub fn is_active(self) -> bool {
        self == Status::Active
    }
}

/// Where one side's work was put, and where it must arrive.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Landing {
    /// The branch the work was put on: the `--to` target for the spec,
    /// `archi/<slug>` for a member.
    pub branch: String,
    /// The branch the work must reach: the receiving checkout's branch for
    /// the spec, the recorded base for a member.
    pub receiving: String,
    /// The side's head at landing time.
    pub sha: String,
}

/// One member's cascaded worktree within a binding.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemberBinding {
    /// The member worktree (or the shared worktree, for members sharing a
    /// repository), canonicalized.
    pub path: PathBuf,
    pub branch: String,
    /// The branch the work was based on — the default receiving branch.
    pub base: String,
    /// The member's main checkout at mint time: the self-heal fallback when
    /// the overlay no longer resolves.
    pub checkout: PathBuf,
    #[serde(default)]
    pub status: Status,
    /// Written by the push, cleared by nothing: where this member's work
    /// went and where it must arrive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub landed: Option<Landing>,
}

/// What one worktree carries.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    pub branch: String,
    #[serde(default)]
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    /// Written by the sideways landing: where the spec work went and where
    /// it must arrive. The local merge needs none — it arrives at once.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub landed: Option<Landing>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub members: BTreeMap<String, MemberBinding>,
}

impl Binding {
    /// The slug this binding answers to: the plan, or the spec effort.
    pub fn slug(&self) -> Option<&str> {
        self.plan.as_deref().or(self.effort.as_deref())
    }

    /// The members still standing — the only ones a cascade, a push or a
    /// sweep ever touches. A closed member is history: it carries no work
    /// to land and no folder to free.
    pub fn active_members(&self) -> impl Iterator<Item = (&String, &MemberBinding)> {
        self.members.iter().filter(|(_, m)| m.status.is_active())
    }

    /// True when no member still stands — the members' half of the row's
    /// close condition, which the spec side completes.
    pub fn members_done(&self) -> bool {
        self.active_members().next().is_none()
    }
}

#[derive(Default, Serialize, Deserialize)]
struct RegistryFile {
    #[serde(default)]
    worktree: BTreeMap<String, Binding>,
}

/// The registry, loaded and self-healed. Keys are canonicalized worktree
/// paths as strings.
pub struct Registry {
    path: PathBuf,
    entries: BTreeMap<String, Binding>,
}

impl Registry {
    /// Load the registry of the repository containing `root`; `None` when
    /// `root` is not inside a git repository. A missing file is the empty
    /// registry — it appears at the first write, no init step. Rows whose
    /// worktree git no longer lists close (written back only when something
    /// moved) — nothing leaves the registry; a row whose members still wait
    /// on their receiving branches stays open, so the sweep can still free
    /// their folders. A member closes only when its own repo confirms the
    /// worktree gone, an unreachable member repo keeps it standing.
    pub fn load(root: &Path) -> Result<Option<Registry>, String> {
        let Some(common) = common_dir(root) else {
            return Ok(None);
        };
        let path = common.join(REGISTRY);
        let entries = match fs::read_to_string(&path) {
            Ok(text) => {
                let file: RegistryFile = toml::from_str(&text)
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                file.worktree
            }
            Err(_) => BTreeMap::new(),
        };
        let mut reg = Registry { path, entries };
        let live: Vec<PathBuf> = list_worktrees(root).into_iter().map(|w| w.path).collect();
        let mut healed = false;
        for (k, b) in reg.entries.iter_mut() {
            for m in b.members.values_mut() {
                if !m.status.is_active() {
                    continue;
                }
                let repo = if m.checkout.is_dir() { &m.checkout } else { &m.path };
                let listed = list_worktrees(repo);
                // an unreachable repo lists nothing — keep the member standing
                if !listed.is_empty() && !listed.iter().any(|w| w.path == m.path) {
                    m.status = Status::Closed;
                    healed = true;
                }
            }
            // A folder git no longer backs is a finished seat — unless a
            // member of it still waits: the row stays open until the sweep
            // frees that folder too.
            let standing = live.iter().any(|p| p.as_path() == Path::new(k));
            let members_done = b.members_done();
            if b.status.is_active() && !standing && members_done {
                b.status = Status::Closed;
                healed = true;
            }
        }
        if healed {
            reg.save()?;
        }
        Ok(Some(reg))
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        let file = RegistryFile { worktree: self.entries.clone() };
        let body = toml::to_string(&file).map_err(|e| format!("{}: {e}", self.path.display()))?;
        let text =
            format!("# machine-local worktree bindings — operated by archi commands, never merged\n{body}");
        fs::write(&self.path, text).map_err(|e| format!("cannot write {}: {e}", self.path.display()))
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &Binding)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// The row this checkout carries, whatever its status — the reading
    /// lookup: a listing renders history, it does not act on it.
    pub fn binding_of(&self, worktree: &Path) -> Option<&Binding> {
        self.entries.get(&canon(worktree).to_string_lossy().into_owned())
    }

    /// The row this checkout carries while it stands — the licensing lookup.
    /// A closed row is history and grants nothing
    /// (`archi/requirements/worktree-parallelism/only-an-active-row-binds.md`).
    pub fn active_binding_of(&self, worktree: &Path) -> Option<&Binding> {
        self.binding_of(worktree).filter(|b| b.status.is_active())
    }

    /// The rows still standing — the seats this machine can continue in, and
    /// the only ones a licensing question ever consults.
    pub fn active_entries(&self) -> impl Iterator<Item = (&str, &Binding)> {
        self.entries().filter(|(_, b)| b.status.is_active())
    }

    /// The standing worktree that carries `plan`, when one does. A closed
    /// row owns nothing: its plan is free to be carried again.
    pub fn owner_of_plan(&self, plan: &str) -> Option<(&str, &Binding)> {
        self.active_entries().find(|(_, b)| b.plan.as_deref() == Some(plan))
    }

    pub fn bind(&mut self, worktree: &Path, binding: Binding) {
        self.entries
            .insert(canon(worktree).to_string_lossy().into_owned(), binding);
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Binding> {
        self.entries.get_mut(key)
    }

    /// Resolve a user-supplied handle — a path, a plan/effort slug, or a
    /// worktree directory name — to a registry key.
    pub fn resolve_key(&self, arg: &str) -> Option<String> {
        if self.entries.contains_key(arg) {
            return Some(arg.to_string());
        }
        let as_path = canon(Path::new(arg)).to_string_lossy().into_owned();
        if self.entries.contains_key(&as_path) {
            return Some(as_path);
        }
        self.entries
            .iter()
            .find(|(k, b)| {
                b.slug() == Some(arg)
                    || Path::new(k).file_name().is_some_and(|n| n.to_string_lossy() == arg)
            })
            .map(|(k, _)| k.clone())
    }
}

// ---------------------------------------------------------------------------
// The branch point — the refresh, the divergence, the ref a seat grows from

/// The ref a fresh branch grew from, and what decided it.
#[derive(Clone, Debug)]
pub struct BranchPoint {
    /// The ref as the report names it: `main`, or `origin/main`.
    pub from: String,
    /// Its commit when the seat was cut.
    pub sha: String,
    /// The divergence, the unpushed work, or the refresh that did not run —
    /// `None` when the local ref simply was the whole answer.
    pub reason: Option<String>,
}

impl BranchPoint {
    /// `from origin/main 1a2b3c4 (local main was 3 behind)` — the phrase a
    /// mint report appends to the line it already prints.
    pub fn describe(&self) -> String {
        match &self.reason {
            Some(r) => format!("from {} {} ({r})", self.from, gitcmd::sha7(&self.sha)),
            None => format!("from {} {}", self.from, gitcmd::sha7(&self.sha)),
        }
    }
}

/// The remote that answers for `branch`: its configured one, else `origin`
/// when the repository has it — and only while a counterpart is on record,
/// which is either an upstream the branch declares or a remote-tracking ref
/// some earlier fetch or push left behind. `None` — nothing to refresh from,
/// and the silence is the whole answer: a repository with no remote, and a
/// branch that has never been on one (a seat's own `archi/<slug>`, forked
/// from inside another seat, among them) have no counterpart to compare
/// against, so neither one is worth a network call or a word of report.
fn remote_of(repo: &Path, branch: &str) -> Option<String> {
    let configured = git_out(repo, &["config", "--get", &format!("branch.{branch}.remote")])
        .filter(|r| !r.is_empty() && r != ".");
    let remote = configured.or_else(|| {
        let remotes = git_out(repo, &["remote"])?;
        remotes.lines().any(|l| l.trim() == "origin").then(|| "origin".to_string())
    })?;
    let declared =
        git_out(repo, &["config", "--get", &format!("branch.{branch}.merge")]).is_some();
    let known = commit_of(repo, &format!("refs/remotes/{remote}/{branch}")).is_some();
    (declared || known).then_some(remote)
}

/// How far the local branch stands from its remote counterpart, as
/// (ahead, behind). `None` when either side does not resolve — no remote
/// ref yet, no comparison, no verdict.
fn divergence(repo: &Path, branch: &str, remote: &str) -> Option<(u64, u64)> {
    let range = format!("refs/heads/{branch}...refs/remotes/{remote}/{branch}");
    let counts = git_out(repo, &["rev-list", "--left-right", "--count", &range])?;
    let mut fields = counts.split_whitespace();
    let ahead = fields.next()?.parse().ok()?;
    let behind = fields.next()?.parse().ok()?;
    Some((ahead, behind))
}

/// Git's failure in one line: the first thing it said, without its severity
/// word, short enough to ride inside a report line.
fn brief(err: &str) -> String {
    let line = err.lines().find(|l| !l.trim().is_empty()).unwrap_or("").trim();
    let line = line.strip_prefix("fatal: ").or_else(|| line.strip_prefix("error: ")).unwrap_or(line);
    if line.chars().count() > 72 {
        format!("{}…", line.chars().take(72).collect::<String>())
    } else {
        line.to_string()
    }
}

/// Unpushed commits, counted the way a sentence counts them.
fn unpushed(n: u64) -> String {
    if n == 1 {
        "1 unpushed commit rides with the seat".to_string()
    } else {
        format!("{n} unpushed commits ride with the seat")
    }
}

/// Refresh `branch` in `repo`, then read where a fresh branch should grow
/// from. The fetch carries one branch and writes remote-tracking refs alone
/// — never a pull, so no checked-out branch moves and no working tree is
/// touched — and it is never a gate: no remote, no network or a refusal all
/// degrade to the local ref with the reason in hand
/// (`archi/requirements/worktree-parallelism/the-refresh-never-blocks-the-mint.md`).
/// The choice that follows keeps unpushed work: behind the remote takes the
/// remote ref, ahead of it or diverged from it takes the local branch
/// (`…/the-branch-point-follows-the-divergence.md`). `hint` names the flag
/// that overrides the choice on this side, where one exists. `None` — the
/// branch does not resolve here, and the caller keeps its implicit base.
fn branch_point(
    repo: &Path,
    branch: &str,
    refresh: bool,
    hint: Option<&str>,
) -> Option<BranchPoint> {
    let sha = commit_of(repo, &format!("refs/heads/{branch}"))?;
    let local = |reason: Option<String>| {
        Some(BranchPoint { from: branch.to_string(), sha: sha.clone(), reason })
    };
    // no remote to ask: the local ref is the world, and says so silently
    let Some(remote) = remote_of(repo, branch) else {
        return local(None);
    };
    if !refresh {
        return local(Some("no fetch".to_string()));
    }
    let args = ["fetch", "--quiet", "--no-tags", remote.as_str(), branch];
    if let Err(e) = gitcmd::run_offline_safe(repo, &args) {
        let prefix = format!("git {}: ", args.join(" "));
        let said = e.strip_prefix(&prefix).unwrap_or(&e);
        return local(Some(format!("no fetch: {}", brief(said))));
    }
    let Some((ahead, behind)) = divergence(repo, branch, &remote) else {
        return local(None);
    };
    let tracking = format!("{remote}/{branch}");
    match (ahead, behind) {
        (0, 0) => local(None),
        // behind only: the remote carries everything local does, and more
        (0, behind) => match commit_of(repo, &format!("refs/remotes/{tracking}")) {
            Some(sha) => Some(BranchPoint {
                from: tracking,
                sha,
                reason: Some(format!("local {branch} was {behind} behind")),
            }),
            None => local(None),
        },
        // unpushed commits are work: they ride with the seat
        (ahead, 0) => local(Some(unpushed(ahead))),
        (ahead, behind) => {
            let mut reason = format!("{ahead} ahead, {behind} behind {tracking}");
            if let Some(h) = hint {
                reason.push_str(&format!(" — `{h}` overrides"));
            }
            local(Some(reason))
        }
    }
}

// ---------------------------------------------------------------------------
// Mint

/// Where a slug's worktree goes: a sibling folder of the checkout.
pub fn default_worktree_dir(top: &Path, slug: &str) -> PathBuf {
    let name = top
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "repo".to_string());
    top.parent().unwrap_or(top).join(format!("{name}-worktrees")).join(slug)
}

/// The checkout a seat folder anchors on: the repository's main checkout,
/// whichever of its worktrees the mint runs from. Minting from inside a seat
/// would otherwise nest the new folder under its parent, one level deeper
/// every time — the anchor the member cascade already uses
/// (`archi/requirements/worktree-parallelism/a-seat-sits-beside-the-main-checkout.md`).
fn folder_anchor(top: &Path) -> PathBuf {
    gitcmd::linked_worktree(top)
        .map(|w| canon(&w.main))
        .unwrap_or_else(|| top.to_path_buf())
}

#[derive(Debug)]
pub struct Minted {
    pub path: PathBuf,
    pub branch: String,
    /// True when the branch already existed and the worktree attached to it.
    pub attached: bool,
    /// True when the caller already sits in the slug's worktree and the
    /// binding was merely extended.
    pub extended: bool,
    /// Where the fresh branch grew from. `None` when nothing grew: an
    /// attach continues a branch, an extension creates nothing at all.
    pub point: Option<BranchPoint>,
    /// The same, per member the cascade branched in this run.
    pub member_points: BTreeMap<String, BranchPoint>,
}

/// Mint the worktree for `slug`: the branch (created, or attached when it
/// already exists), the worktree, the member cascade, the overlay, the
/// registry entry — entry last, and a partial cascade rolls back whole, so
/// no failure leaves a dangling binding. Re-minting from inside the slug's
/// own worktree extends the binding (new members, plan/effort upserts)
/// instead of creating anything anew, and a slug whose row closed re-opens
/// that row — its folder comes back, its record continues.
///
/// `refresh` fetches each base branch before the branch points resolve;
/// `false` is `--no-fetch`, and a failed fetch is never a gate.
pub fn mint(
    root: &Path,
    slug: &str,
    plan: Option<&str>,
    effort: Option<&str>,
    repos: &[String],
    bases: &BTreeMap<String, String>,
    refresh: bool,
) -> Result<Minted, String> {
    let root = canon(root);
    let top =
        toplevel(&root).ok_or_else(|| "not a git repository — worktrees need git".to_string())?;
    let branch = branch_of(slug);
    let mut reg = Registry::load(&root)?.expect("toplevel resolved, so common dir does");
    ensure_excludes(&top);
    // where the project sits inside the tree — a monorepo spec below the
    // git root keeps that offset inside its worktree too
    let rel = root.strip_prefix(&top).unwrap_or(Path::new("")).to_path_buf();
    let worktrees = list_worktrees(&top);
    let extended = worktrees
        .iter()
        .find(|w| w.branch.as_deref() == Some(branch.as_str()))
        .map(|w| w.path.clone());
    if let Some(wt) = &extended {
        if *wt != top {
            return Err(format!(
                "`{branch}` is already checked out at {} — continue there (cd {}); \
                 if that checkout is gone, run `git worktree prune` and retry",
                wt.display(),
                wt.display()
            ));
        }
    }
    // A closed row for this slug re-opens on its own key: nothing leaves the
    // registry, so a returning seat continues the row it left instead of
    // writing a second one. Only a row whose folder is really gone re-opens —
    // a standing checkout is never re-created under it.
    let reopen: Option<(PathBuf, Binding)> = extended.is_none().then(|| {
        reg.entries
            .iter()
            .find(|(k, b)| {
                !b.status.is_active()
                    && b.slug() == Some(slug)
                    && !worktrees.iter().any(|w| w.path.as_path() == Path::new(k))
                    && !Path::new(k).exists()
            })
            .map(|(k, b)| (PathBuf::from(k), b.clone()))
    })
    .flatten();
    let existing = match (&extended, &reopen) {
        (Some(_), _) => reg.binding_of(&top).cloned(),
        (None, Some((_, b))) => Some(b.clone()),
        (None, None) => None,
    };
    // Only a standing member counts as already cascaded — a closed one
    // cascades again like a fresh name.
    let known: Vec<String> = existing
        .as_ref()
        .map(|b| b.active_members().map(|(n, _)| n.clone()).collect())
        .unwrap_or_default();
    // Members resolve against the invoked project root — the checkout that
    // carries the unit's manifest, overlay and archive: the primary on a
    // first cascade, the worktree itself on a mid-unit extension (declarations
    // and anchors made in the worktree land on the primary only at the final
    // merge). Already-cascaded members are excluded above, so the worktree's
    // overlay rows pointing at member worktrees never feed the cascade;
    // a fresh member unresolvable from here refuses toward `repo map` —
    // member locations are machine-local truth, the overlay carries them.
    let fresh_repos: Vec<String> =
        repos.iter().filter(|r| !known.contains(r)).cloned().collect();
    let targets = if fresh_repos.is_empty() {
        Vec::new()
    } else {
        plan_cascade(&root, slug, plan, &fresh_repos, bases, refresh)?
    };

    // create: spec worktree (unless it exists), then member worktrees — any
    // failure rolls the run's creations back
    let mut created: Vec<(PathBuf, PathBuf, bool)> = Vec::new();
    let rollback = |created: &[(PathBuf, PathBuf, bool)]| {
        for (repo, wt, branch_created) in created.iter().rev() {
            let _ = worktree_remove(repo, wt, true);
            if *branch_created {
                let _ = git_run(repo, &["branch", "-D", &branch_of(slug)]);
            }
        }
    };
    let mut point: Option<BranchPoint> = None;
    let (wt_path, attached) = match &extended {
        Some(wt) => (wt.clone(), true),
        None => {
            let path = match &reopen {
                Some((p, _)) => p.clone(),
                None => default_worktree_dir(&folder_anchor(&top), slug),
            };
            if path.exists() {
                return Err(format!(
                    "{} already exists but is not a worktree of this repository — move it aside",
                    path.display()
                ));
            }
            let attached = branch_exists(&top, &branch);
            // A fresh branch grows from the branch this checkout stands on —
            // refreshed first, and taken from whichever side of the remote
            // carries everything. An attach continues a branch that already
            // exists: nothing to choose, nothing to fetch.
            if !attached {
                point = current_branch(&top)
                    .and_then(|b| branch_point(&top, &b, refresh, None));
            }
            let from = point.as_ref().map(|p| p.from.as_str());
            worktree_add(&top, &path, &branch, !attached, from)?;
            let path = canon(&path);
            created.push((top.clone(), path.clone(), !attached));
            (path, attached)
        }
    };
    let mut members: BTreeMap<String, MemberBinding> =
        existing.as_ref().map(|b| b.members.clone()).unwrap_or_default();
    let mut member_points: BTreeMap<String, BranchPoint> = BTreeMap::new();
    for t in &targets {
        // The branch point the cascade chose; the recorded base stays the
        // local branch this member lands back on.
        let from = (!t.attach)
            .then(|| t.point.as_ref().map_or(t.base.as_str(), |p| p.from.as_str()));
        if let Err(e) = worktree_add(&t.repo_top, &t.target, &branch, !t.attach, from) {
            rollback(&created);
            return Err(format!("{}: {e}", t.carries[0].0));
        }
        created.push((t.repo_top.clone(), canon(&t.target), !t.attach));
        let target = canon(&t.target);
        for (name, checkout) in &t.carries {
            if let Some(p) = &t.point {
                member_points.insert(name.clone(), p.clone());
            }
            members.insert(
                name.clone(),
                MemberBinding {
                    path: target.clone(),
                    branch: branch.clone(),
                    base: t.base.clone(),
                    checkout: checkout.clone(),
                    status: Status::Active,
                    landed: None,
                },
            );
        }
    }
    // the overlay the worktree resolves members through — every cascaded member,
    // old and new, points at its member worktree
    let rows: Vec<(String, PathBuf)> = members
        .iter()
        .filter(|(_, m)| m.status.is_active())
        .map(|(name, m)| (name.clone(), member_row(&m.path, &m.checkout)))
        .collect();
    if !rows.is_empty()
        && let Err(e) = write_worktree_overlay(&wt_path.join(&rel), &rows)
    {
        rollback(&created);
        return Err(e);
    }
    // The seat is live work again: re-opened or extended, it stands on no
    // landing of its own.
    let binding = Binding {
        branch: branch.clone(),
        status: Status::Active,
        plan: plan.map(str::to_string).or(existing.as_ref().and_then(|b| b.plan.clone())),
        effort: effort.map(str::to_string).or(existing.as_ref().and_then(|b| b.effort.clone())),
        landed: None,
        members,
    };
    reg.bind(&wt_path, binding);
    reg.save()?;
    Ok(Minted {
        path: wt_path,
        branch,
        attached,
        extended: extended.is_some(),
        point,
        member_points,
    })
}

// ---------------------------------------------------------------------------
// The cascade: member repositories follow the plan

/// One member repository's part of a cascade, validated but not yet created.
struct CascadeTarget {
    /// The member repo's primary checkout top (git worktree list runs here).
    repo_top: PathBuf,
    /// The member worktree to create.
    target: PathBuf,
    /// The branch the worktree is based on — the default receiving branch.
    base: String,
    /// Where the branch grows from, once the refresh spoke. `None` on the
    /// attach path and whenever the caller named the base itself.
    point: Option<BranchPoint>,
    /// True when `archi/<slug>` already exists in this repo.
    attach: bool,
    /// The members this repo carries: (name, checkout root).
    carries: Vec<(String, PathBuf)>,
}

/// The overlay row a member resolves through inside the spec worktree: the
/// member's root within the minted worktree, derived from its prefix below
/// the repo top.
fn member_row(wt: &Path, checkout: &Path) -> PathBuf {
    match toplevel(checkout).and_then(|top| checkout.strip_prefix(top).map(Path::to_path_buf).ok())
    {
        Some(prefix) if prefix.as_os_str().is_empty() => wt.to_path_buf(),
        Some(prefix) => wt.join(prefix),
        None => wt.to_path_buf(),
    }
}

/// The per-member baselines anchoring the cascade: the plan's pinned
/// version when the plan exists, else the archive tip. Empty when neither.
fn cascade_baselines(project: &Path, plan: Option<&str>) -> BTreeMap<String, String> {
    let Ok(Some(archive)) = crate::versions::Archive::open(project) else {
        return BTreeMap::new();
    };
    let pinned = plan
        .and_then(|name| crate::plans::all_plans(project).ok()?.into_iter().find(|p| p.name == name))
        .map(|p| p.version);
    let entry = match &pinned {
        Some(v) => archive.entry(v),
        None => archive.entries().last(),
    };
    entry
        .map(|e| e.commits.iter().map(|(k, b)| (k.clone(), b.sha.clone())).collect())
        .unwrap_or_default()
}

/// Validate the whole cascade before creating anything. Every refusal across
/// every repo lands in one message — one round-trip for the caller.
fn plan_cascade(
    project: &Path,
    slug: &str,
    plan: Option<&str>,
    repos: &[String],
    bases: &BTreeMap<String, String>,
    refresh: bool,
) -> Result<Vec<CascadeTarget>, String> {
    let set = crate::members::MemberSet::resolve(project)?;
    let branch = branch_of(slug);
    let baselines = cascade_baselines(project, plan);
    let mut refusals: Vec<String> = Vec::new();
    // one target per underlying repository — members sharing a repo share it
    let mut targets: Vec<CascadeTarget> = Vec::new();
    for name in repos {
        let Some(member) = set.get(name).filter(|m| m.name != crate::members::HOME) else {
            refusals.push(format!(
                "`{name}` is not a declared member — archi.toml declares them"
            ));
            continue;
        };
        let Some(checkout) = member.root.clone() else {
            refusals.push(format!(
                "`{name}` is unmapped on this machine — `archi repo map {name} <dir>`"
            ));
            continue;
        };
        let Some(repo_top) = toplevel(&checkout) else {
            refusals.push(format!(
                "`{name}` at {} is not a git repository",
                checkout.display()
            ));
            continue;
        };
        // The cascade anchors on the repository's MAIN checkout — placement,
        // branch surgery, shared-repo dedup — never on the mapped path,
        // which a stale overlay row can leave standing in a scratch
        // worktree. For an ordinary mapping the two coincide.
        let linked = gitcmd::linked_worktree(&checkout);
        let repo_top = linked.as_ref().map(|w| canon(&w.main)).unwrap_or(repo_top);
        // A member mapped onto a linked worktree is a stale overlay row:
        // without an explicit `--base` the worktree would branch from whatever
        // dead branch stands there — foreign commits from birth. A named
        // base skips the check (the branch is chosen; the checkout's
        // identity stops mattering), and an already-bound member never
        // reaches the cascade — a re-mint extension attaches, gate-free.
        if !bases.contains_key(name) && let Some(wt) = linked {
            let standing = wt.branch;
            refusals.push(format!(
                "member {name}: the mapped checkout {} is a linked worktree standing on \
                 `{standing}` — a stale overlay row. Re-map: `archi repo map {name} {}`, \
                 or name the base outright: `--base {name}=<branch>`",
                checkout.display(),
                repo_top.display()
            ));
            continue;
        }
        if let Some(t) = targets.iter_mut().find(|t| t.repo_top == repo_top) {
            if let Some(b) = bases.get(name) {
                if *b != t.base {
                    refusals.push(format!(
                        "`{name}` shares a repository with `{}` — their bases must agree \
                         (`{}` vs `{b}`)",
                        t.carries[0].0, t.base
                    ));
                    continue;
                }
            }
            t.carries.push((name.clone(), checkout));
            continue;
        }
        let attach = branch_exists(&repo_top, &branch);
        let base = if attach {
            if bases.contains_key(name) {
                refusals.push(format!(
                    "`{name}`: branch {branch} already exists and attaches as-is — \
                     `--base` cannot rebase it; delete the branch to restart"
                ));
                continue;
            }
            // the receiving default stays the checkout's own branch
            match current_branch(&checkout) {
                Some(b) => b,
                None => {
                    refusals.push(format!(
                        "`{name}` at {} is on a detached HEAD — check out a branch there",
                        checkout.display()
                    ));
                    continue;
                }
            }
        } else if let Some(b) = bases.get(name) {
            if !branch_exists(&repo_top, b) {
                refusals.push(format!("`{name}`: base branch `{b}` does not exist"));
                continue;
            }
            // The escape lane never refuses — the branch is the caller's own
            // choice — but when the pinned version records a baseline the
            // named branch does not carry (or the object is missing here),
            // it says so once and continues: eyes open, off the audited line.
            if let Some(sha) = baselines.get(name.as_str()) {
                let refname = format!("refs/heads/{b}");
                if git_out(&repo_top, &["merge-base", "--is-ancestor", sha, &refname]).is_none() {
                    println!(
                        "note: member {name}: the named base `{b}` does not contain the \
                         recorded baseline {} — continuing off the audited line",
                        gitcmd::sha7(sha)
                    );
                }
            }
            b.clone()
        } else {
            // auto: the recorded baseline must be reachable from the
            // checkout's own branch, else the caller chooses explicitly
            let Some(b) = current_branch(&checkout) else {
                refusals.push(format!(
                    "`{name}` at {} is on a detached HEAD — check out a branch there, \
                     or pass `--base {name}=<branch>`",
                    checkout.display()
                ));
                continue;
            };
            let Some(sha) = baselines.get(name.as_str()).cloned() else {
                refusals.push(format!(
                    "no recorded baseline for `{name}` — `archi version anchor --repo {name}` \
                     records one, or pass `--base {name}=<branch>`"
                ));
                continue;
            };
            let probe = format!("{sha}^{{commit}}");
            if git_out(&repo_top, &["cat-file", "-e", &probe]).is_none() {
                refusals.push(format!(
                    "baseline {} for `{name}` is not in {} — fetch there, or pass \
                     `--base {name}=<branch>`",
                    gitcmd::sha7(&sha),
                    repo_top.display()
                ));
                continue;
            }
            let refname = format!("refs/heads/{b}");
            if git_out(&repo_top, &["merge-base", "--is-ancestor", &sha, &refname]).is_none() {
                let candidates = git_out(
                    &repo_top,
                    &["branch", "--format=%(refname:short)", "--contains", &sha],
                )
                .unwrap_or_default();
                let candidates = if candidates.is_empty() {
                    "no local branch contains it".to_string()
                } else {
                    format!("branches containing it: {}", candidates.replace('\n', ", "))
                };
                refusals.push(format!(
                    "baseline {} for `{name}` is not on `{b}` — {candidates}; \
                     choose with `--base {name}=<branch>`",
                    gitcmd::sha7(&sha)
                ));
                continue;
            }
            // A reachable baseline behind the branch proceeds — continuing
            // an older pinned version is legitimate — but says how far, so
            // fresh work anchors first instead of inheriting a foreign
            // delta window. Only this auto arm speaks: an explicit `--base`
            // and the attach path are the caller's own choice.
            if let Some(n) = git_out(&repo_top, &["rev-list", "--count", &format!("{sha}..{refname}")])
                && n.parse::<u64>().unwrap_or(0) > 0
            {
                println!(
                    "note: member {name}: baseline {} is {n} commit(s) behind `{b}` — \
                     continuing an older pinned version; fresh work anchors first \
                     (`archi version anchor --repo {name}`)",
                    gitcmd::sha7(&sha)
                );
            }
            b
        };
        let target = default_worktree_dir(&repo_top, slug);
        if !attach && target.exists() {
            refusals.push(format!(
                "`{name}`: {} already exists but is not this repo's worktree — move it aside",
                target.display()
            ));
            continue;
        }
        // The refresh and the divergence read run on the auto arm alone: a
        // named base is the caller's own choice, so it is taken as given —
        // no fetch, no second-guessing.
        let point = (!attach && !bases.contains_key(name))
            .then(|| {
                branch_point(&repo_top, &base, refresh, Some(&format!("--base {name}=<branch>")))
            })
            .flatten();
        targets.push(CascadeTarget {
            repo_top,
            target,
            base,
            point,
            attach,
            carries: vec![(name.clone(), checkout)],
        });
    }
    if refusals.is_empty() {
        Ok(targets)
    } else {
        Err(refusals.join("\n"))
    }
}

/// Write the overlay the spec worktree resolves members through: every
/// cascaded member points at its member worktree; members outside the
/// cascade get no row — narrowed scope, never someone else's checkout.
fn write_worktree_overlay(
    wt_project: &Path,
    rows: &[(String, PathBuf)],
) -> Result<(), String> {
    let path = wt_project.join(crate::members::OVERLAY);
    let mut text =
        String::from("# member worktrees bound to this checkout — written by archi worktree mint\n");
    for (name, dir) in rows {
        text.push_str(&format!(
            "{name} = {}\n",
            crate::members::toml_string(&dir.to_string_lossy())
        ));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// The integration probe — one read-only look, two proofs

/// What a look at a landed side says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The receiving branch carries the work — by ancestry, or by content.
    Integrated,
    /// The receiving branch does not carry it yet.
    NotYet,
    /// No verdict: a ref, or the repository itself, is out of reach.
    Unknown,
}

/// A revision as a commit sha, or `None` when it does not resolve here.
fn commit_of(repo: &Path, rev: &str) -> Option<String> {
    git_out(repo, &["rev-parse", "--verify", "--quiet", &format!("{rev}^{{commit}}")])
}

/// A branch name as a commit sha — the ref namespace first, the bare name
/// as the fallback.
fn branch_commit(repo: &Path, name: &str) -> Option<String> {
    commit_of(repo, &format!("refs/heads/{name}")).or_else(|| commit_of(repo, name))
}

/// `git diff --quiet` as three answers, not two: `Some(true)` — git found no
/// difference, `Some(false)` — git found one (its exit code 1), `None` — git
/// could not answer at all. [`git_out`] cannot serve here: it folds "they
/// differ" and "git failed" into the same `None`, and a probe that reads a
/// failure as a match would free a seat whose work never arrived.
fn diff_quiet(repo: &Path, args: &[&str]) -> Option<bool> {
    let out = std::process::Command::new("git").arg("-C").arg(repo).args(args).output().ok()?;
    match out.status.code() {
        Some(0) => Some(true),
        Some(1) => Some(false),
        _ => None,
    }
}

/// How many bytes of pathspec one `git diff` call carries. A landing may
/// touch more paths than a command line holds, so the content proof runs in
/// batches; every batch must come back quiet, which is the same claim one
/// call would make.
const PATHSPEC_BUDGET: usize = 60_000;

/// The paths cut into command-line-sized batches — every path once, in
/// order, and never an empty batch, so one path longer than the whole
/// budget still travels (alone).
fn pathspec_batches<'a>(paths: &[&'a str], budget: usize) -> Vec<Vec<&'a str>> {
    let mut batches = Vec::new();
    let mut start = 0;
    while start < paths.len() {
        let (mut end, mut bytes) = (start, 0);
        while end < paths.len() && (end == start || bytes + paths[end].len() < budget) {
            bytes += paths[end].len() + 1;
            end += 1;
        }
        batches.push(paths[start..end].to_vec());
        start = end;
    }
    batches
}

/// The one integration probe, shared by the landing and the sweep: does
/// `receiving` carry what `landing` put on its branch? Ancestry is the cheap
/// first pass. Content answers the squash, where the forge rewrote every
/// sha — and it asks one-directionally, over the paths the landing itself
/// touched (the landing against its merge base with `receiving`), never over
/// the whole tree. A shared receiving branch carries work of its own, and a
/// whole-tree comparison would read that work as "not yet" forever, on
/// exactly the squash this pass exists to see. The one error left is a
/// receiving branch that rewrote those very paths differently, which reads
/// as `NotYet` and keeps the seat standing — the safe direction. Both reads
/// only — no merge ever runs to find out — and anything git cannot answer
/// yields no verdict at all, never a false arrival
/// (`archi/requirements/worktree-parallelism/integration-is-proven-by-content.md`).
pub(crate) fn integration(repo: &Path, landing: &Landing) -> Verdict {
    let Some(sha) = commit_of(repo, &landing.sha) else {
        return Verdict::Unknown;
    };
    let Some(receiving) = branch_commit(repo, &landing.receiving) else {
        return Verdict::Unknown;
    };
    if git_out(repo, &["merge-base", "--is-ancestor", &sha, &receiving]).is_some() {
        return Verdict::Integrated;
    }
    // where the two sides parted: no common history, no content question
    let Some(base) = git_out(repo, &["merge-base", &sha, &receiving]) else {
        return Verdict::Unknown;
    };
    // the paths the landing itself touched. `-z` keeps odd names verbatim
    // (quoting would break the pathspec); `--no-renames` names both sides of
    // a rename, so a source the receiving branch still holds is not missed.
    let Some(touched) = git_out(repo, &["diff", "--no-renames", "--name-only", "-z", &base, &sha])
    else {
        return Verdict::Unknown;
    };
    let paths: Vec<&str> = touched.split('\0').filter(|p| !p.is_empty()).collect();
    if paths.is_empty() {
        // the landing put nothing on top of the merge base: nothing to carry
        return Verdict::Integrated;
    }
    // every batch quiet, or the claim is not made
    for batch in pathspec_batches(&paths, PATHSPEC_BUDGET) {
        // `--literal-pathspecs`: a path is a path, never a glob
        let mut args =
            vec!["--literal-pathspecs", "diff", "--quiet", sha.as_str(), receiving.as_str(), "--"];
        args.extend_from_slice(&batch);
        match diff_quiet(repo, &args) {
            Some(true) => {}
            Some(false) => return Verdict::NotYet,
            None => return Verdict::Unknown,
        }
    }
    Verdict::Integrated
}

/// `git status --porcelain` silent: tracked files unmodified and every
/// untracked file covered by an ignore rule. Ignored build output never
/// speaks here — it is reproducible, and it is the reason the folder is
/// worth freeing
/// (`archi/requirements/worktree-parallelism/ignored-files-never-veto-a-cleanup.md`).
pub fn tree_clean(worktree: &Path) -> bool {
    git_out(worktree, &["status", "--porcelain"]).is_some_and(|s| s.trim().is_empty())
}

/// A landing record counts only while the seat stands exactly where it
/// landed: the same head, and a tree git reports clean. A commit on top or
/// an uncommitted edit makes the seat live work again — derived on every
/// read, never written back
/// (`archi/requirements/worktree-parallelism/a-resumed-seat-is-live-again.md`).
pub fn landing_stands(worktree: &Path, landing: &Landing) -> bool {
    git_out(worktree, &["rev-parse", "HEAD"]).as_deref() == Some(landing.sha.as_str())
        && tree_clean(worktree)
}

// ---------------------------------------------------------------------------
// Merge — the closing command

/// One repository's outcome inside a merge.
#[derive(Debug)]
pub enum RepoOutcome {
    /// Spec: merged into the receiving branch (or already up to date).
    Merged,
    /// Spec: landed on a new branch without merging (`--to`). The seat keeps
    /// standing until the receiving branch carries the work.
    Landed { branch: String },
    /// Spec: the merge stopped on conflicts; nothing retired.
    Conflict { detail: String },
    /// Member: its branch went to the remote. The member worktree keeps
    /// standing until its base carries the work.
    Pushed { remote_branch: String },
    /// Member: kept in the binding, with the reason.
    Refused { detail: String },
}

#[derive(Debug)]
pub struct MergeReport {
    pub worktree: PathBuf,
    pub branch: String,
    pub spec: RepoOutcome,
    pub members: Vec<(String, RepoOutcome)>,
    /// True when the local merge removed the worktree in this run.
    pub retired: bool,
}

/// Close a worktree: merge its branch into the current branch of this
/// checkout — or land it on a new branch with `to` — and push each member's
/// branch. Retirement follows integration, never the push: the local merge
/// puts the work in the receiving branch at that moment, so it removes the
/// worktree and marks the row in the same move; the sideways landing and
/// every member push record where the work went and keep their folders,
/// which the sweep frees once the receiving branch carries the content. A
/// conflict (or a refused member) stops short of retiring; re-running after
/// the repair is idempotent. `to` keys: `""` = spec, member name = that
/// member's remote branch.
pub fn merge(
    root: &Path,
    handle: &str,
    to: &BTreeMap<String, String>,
) -> Result<MergeReport, String> {
    let root = canon(root);
    let top =
        toplevel(&root).ok_or_else(|| "not a git repository — worktrees need git".to_string())?;
    let mut reg = Registry::load(&root)?.expect("toplevel resolved, so common dir does");
    let Some(key) = reg.resolve_key(handle) else {
        return Err(format!(
            "`{handle}` matches no registry entry — `archi worktree ls` shows them"
        ));
    };
    let binding = reg.get_mut(&key).expect("resolved key").clone();
    let wt_path = PathBuf::from(&key);
    if wt_path == top {
        return Err(format!(
            "merge runs from the receiving checkout, not from the worktree being merged — \
             cd out of {} first",
            wt_path.display()
        ));
    }
    let branch = binding.branch.clone();
    // The project's offset below the git top — the worktree keeps it, so its
    // project root is the worktree plus the same offset.
    let rel = root.strip_prefix(&top).unwrap_or(Path::new(""));
    let wt_project =
        if rel.as_os_str().is_empty() { wt_path.clone() } else { wt_path.join(rel) };
    // A worktree lands only after its plan closes: work mid-wave never merges.
    // A worktree with no plan lands freely.
    if let Some(plan_name) = &binding.plan {
        if let Ok(plans) = crate::plans::all_plans(&wt_project) {
            if let Some(p) = plans.iter().find(|p| &p.name == plan_name) {
                if p.state != crate::plans::PlanState::Completed {
                    return Err(format!(
                        "plan `{plan_name}` is {} — a worktree lands only after its plan \
                         closes: finish the waves (`archi plan next`) or `archi plan \
                         close`, then re-run the merge",
                        p.state.describe()
                    ));
                }
            }
        }
    }
    // A protected branch never receives a local merge — landing there is a
    // push/PR ceremony; `--to` still lands sideways.
    if to.get("").is_none() {
        let protected = modeling_lang::source::manifest_protected(&root)
            .map_err(|d| format!("{}: {}", d.code, d.message))?;
        if let Some(receiving) = current_branch(&top) {
            if protected.iter().any(|p| p == &receiving) {
                return Err(format!(
                    "`{receiving}` is protected — it never receives a local merge; push \
                     {branch} and open a PR, or land sideways: \
                     `archi worktree merge {handle} --to <branch>`"
                ));
            }
        }
    }

    // The landed archive is what every future unit inherits, so the gate
    // runs on both paths — the local merge and `--to` alike: while any
    // member's worktree tip is not the baseline the worktree's latest version
    // records, refuse before anything pushes or merges, every stale member
    // batched into the one message with its repair
    // (archi/requirements/worktree-parallelism/a-landing-carries-fresh-baselines.md).
    if !binding.members.is_empty() {
        let baselines: BTreeMap<String, String> = crate::versions::Archive::open(&wt_project)?
            .and_then(|a| {
                a.entries().last().map(|e| {
                    e.commits.iter().map(|(k, b)| (k.clone(), b.sha.clone())).collect()
                })
            })
            .unwrap_or_default();
        let stale: Vec<String> = binding
            .members
            .iter()
            .filter_map(|(name, m)| {
                // an unreadable member worktree is the push loop's refusal
                // to make, not a staleness verdict
                let tip = git_out(&m.path, &["rev-parse", "HEAD"])?;
                let recorded = baselines.get(name.as_str());
                (recorded.map(String::as_str) != Some(tip.as_str())).then(|| {
                    format!(
                        "member {name}: worktree tip {} is past the recorded baseline {} — \
                         `archi version anchor --repo {name} --project {}`, then re-run the merge",
                        gitcmd::sha7(&tip),
                        recorded.map_or("none", |s| gitcmd::sha7(s)),
                        wt_project.display()
                    )
                })
            })
            .collect();
        if !stale.is_empty() {
            return Err(stale.join("\n"));
        }
    }

    // Members first: push is independent of the spec merge, and a refused
    // member must not block the spec's landing (or vice versa). A member
    // already closed carries no work to land — it is history.
    let mut members: Vec<(String, RepoOutcome)> = Vec::new();
    let mut landings: BTreeMap<String, Landing> = BTreeMap::new();
    for (name, m) in binding.active_members() {
        let repo = if m.checkout.is_dir() {
            m.checkout.clone()
        } else if m.path.is_dir() {
            m.path.clone()
        } else {
            members.push((
                name.clone(),
                RepoOutcome::Refused {
                    detail: format!(
                        "checkout unresolved ({}) — `archi repo map {name} <dir>`",
                        m.checkout.display()
                    ),
                },
            ));
            continue;
        };
        let remote_branch = to.get(name).cloned().unwrap_or_else(|| m.branch.clone());
        let refspec = format!("{}:refs/heads/{}", m.branch, remote_branch);
        match git_run(&repo, &["push", "origin", &refspec]) {
            Ok(_) => {
                // The push is not the arrival: the branch waits on a pull
                // request that merges hours or days later, and the member
                // keeps its worktree and its branch until the base carries
                // the work (a-seat-lives-until-its-work-lands).
                if let Some(sha) = git_out(&m.path, &["rev-parse", "HEAD"])
                    .or_else(|| branch_commit(&repo, &m.branch))
                {
                    landings.insert(
                        name.clone(),
                        Landing {
                            branch: m.branch.clone(),
                            receiving: m.base.clone(),
                            sha,
                        },
                    );
                }
                members.push((name.clone(), RepoOutcome::Pushed { remote_branch }));
            }
            Err(e) => {
                members.push((name.clone(), RepoOutcome::Refused { detail: e }));
            }
        }
    }
    let b = reg.get_mut(&key).expect("resolved key");
    for (name, landing) in landings {
        if let Some(m) = b.members.get_mut(&name) {
            m.landed = Some(landing);
        }
    }
    // Today's retire gate, stated the way it always meant: every member that
    // had work to push pushed it. A refused member still stops the seat.
    let members_landed =
        members.iter().all(|(_, o)| !matches!(o, RepoOutcome::Refused { .. }));
    reg.save()?;

    // The spec repo: land on a new branch, or merge into the current one.
    let spec = if let Some(new_branch) = to.get("") {
        let sha = git_out(&wt_path, &["rev-parse", "HEAD"])
            .ok_or_else(|| format!("cannot read HEAD of {}", wt_path.display()))?;
        if branch_exists(&top, new_branch) {
            let refname = format!("refs/heads/{new_branch}");
            // a re-run after a repaired retire finds its own branch — done
            if git_out(&top, &["rev-parse", &refname]).as_deref() != Some(sha.as_str()) {
                return Err(format!(
                    "branch `{new_branch}` already exists — `--to` lands work on a new branch; \
                     merge into an existing one from its own checkout"
                ));
            }
        } else {
            git_run(&top, &["branch", new_branch, &sha])?;
        }
        // The sideways landing is a departure, not an arrival: the branch
        // waits on its pull request. Record where the work went and keep the
        // seat standing; the sweep frees the folder once the receiving branch
        // carries the content.
        if let Some(receiving) = current_branch(&top)
            && let Some(b) = reg.get_mut(&key)
        {
            b.landed = Some(Landing { branch: new_branch.clone(), receiving, sha });
            reg.save()?;
        }
        RepoOutcome::Landed { branch: new_branch.clone() }
    } else {
        match git_run(&top, &["merge", "--no-edit", &branch]) {
            Ok(_) => RepoOutcome::Merged,
            Err(detail) => RepoOutcome::Conflict { detail },
        }
    };

    // Retire — the local merge alone. The work is in the receiving branch at
    // this moment, so the folder goes now: worktree first, then the row, so
    // no failure leaves a row pointing at nothing. The row is marked, never
    // removed — it closes once every member arrived too
    // (nothing-leaves-the-registry).
    let mut retired = false;
    if matches!(spec, RepoOutcome::Merged) && members_landed {
        scrub_worktree(&wt_project);
        worktree_remove(&top, &wt_path, false).map_err(|e| {
            format!("{e}\nthe worktree keeps its binding; commit or clean it, then re-run")
        })?;
        let _ = git_run(&top, &["branch", "-d", &branch]);
        let mut reg = Registry::load(&root)?.expect("still a repository");
        if let Some(b) = reg.get_mut(&key) {
            b.landed = None;
            if b.members_done() {
                b.status = Status::Closed;
            }
        }
        reg.save()?;
        retired = true;
    }
    Ok(MergeReport { worktree: wt_path, branch, spec, members, retired })
}

// ---------------------------------------------------------------------------
// The sweep — an integrated folder frees itself

/// Which side of a row a sweep freed.
#[derive(Clone, Debug, PartialEq)]
pub enum Side {
    /// The seat's own checkout.
    Spec,
    /// The member checkouts sharing one folder.
    Members(Vec<String>),
}

/// One folder the sweep freed.
#[derive(Debug)]
pub struct Freed {
    pub path: PathBuf,
    pub side: Side,
    /// The branch that carries the work now.
    pub receiving: String,
}

/// What one sweep did — empty on the ordinary run, one line for the caller
/// otherwise.
#[derive(Debug, Default)]
pub struct SweepReport {
    pub freed: Vec<Freed>,
    /// The registry keys whose row closed in this pass.
    pub closed: Vec<String>,
}

/// Free every folder whose work arrived. For each standing row, each side
/// that carries a live landing record is probed read-only; a side whose
/// receiving branch carries the content, and whose tree git reports clean,
/// loses its folder (forced — ignored build output never vetoes a cleanup)
/// and is marked done. The row closes when the spec side and every member
/// arrived. Silent by construction: an unreachable member, an unresolvable
/// ref or a refusing removal simply leaves that folder standing, and the
/// checkout the caller stands in is never freed under its own feet.
// The registry-reading commands — `worktree ls`, `worktree mint`, `status` —
// run it before they render and print what it freed.
pub fn sweep(root: &Path) -> SweepReport {
    let mut report = SweepReport::default();
    let root = canon(root);
    let Some(top) = toplevel(&root) else {
        return report;
    };
    let Ok(Some(mut reg)) = Registry::load(&root) else {
        return report;
    };
    let live: Vec<PathBuf> = list_worktrees(&top).into_iter().map(|w| w.path).collect();
    let mut moved = false;
    for (key, b) in reg.entries.iter_mut() {
        if !b.status.is_active() {
            continue;
        }
        let wt = PathBuf::from(key);
        let mut spec_standing = live.iter().any(|p| p.as_path() == wt.as_path());
        // The spec side: its landing record names the branch it waits on.
        if let Some(landing) = b.landed.clone()
            && spec_standing
            && !root.starts_with(&wt)
            && landing_stands(&wt, &landing)
            && integration(&top, &landing) == Verdict::Integrated
            && worktree_remove(&top, &wt, true).is_ok()
        {
            b.landed = None;
            spec_standing = false;
            moved = true;
            report.freed.push(Freed {
                path: wt.clone(),
                side: Side::Spec,
                receiving: landing.receiving,
            });
        }
        // The members: those sharing one physical repository share one
        // folder — the grouping the cascade gave them — so they share one
        // verdict and one removal.
        let mut groups: BTreeMap<PathBuf, Vec<String>> = BTreeMap::new();
        for (name, m) in b.active_members() {
            if m.landed.is_some() {
                groups.entry(m.path.clone()).or_default().push(name.clone());
            }
        }
        for (path, names) in groups {
            let Some(m) = b.members.get(&names[0]) else { continue };
            let Some(landing) = m.landed.clone() else { continue };
            let repo = if m.checkout.is_dir() { m.checkout.clone() } else { path.clone() };
            if root.starts_with(&path)
                || !landing_stands(&path, &landing)
                || integration(&path, &landing) != Verdict::Integrated
                || worktree_remove(&repo, &path, true).is_err()
            {
                continue;
            }
            for name in &names {
                if let Some(m) = b.members.get_mut(name) {
                    m.status = Status::Closed;
                }
            }
            moved = true;
            report.freed.push(Freed {
                path,
                side: Side::Members(names),
                receiving: landing.receiving,
            });
        }
        // Every side arrived: the row becomes the record of what this
        // machine carried.
        let members_done = b.members_done();
        if !spec_standing && b.landed.is_none() && members_done {
            b.status = Status::Closed;
            moved = true;
            report.closed.push(key.clone());
        }
    }
    if moved {
        let _ = reg.save();
    }
    report
}

// ---------------------------------------------------------------------------
// The guard

/// The three-outcome gate every mutating route passes — wired once, at the
/// router in `main`, never inside command bodies: bound here — proceed; bound
/// elsewhere — refuse naming the owner; unbound — refuse toward a worktree.
/// The discipline is unconditional: the binding, not the branch, is the
/// license to mutate — an unbound checkout (the primary included) never
/// mutates, and gitless refuses loudly: the worktree model (isolation,
/// branches, merge) needs a repository. `protected` in archi.toml keeps a
/// single meaning — branches that never receive a local merge.
pub fn guard_mutation(root: &Path, work: Option<&str>) -> Result<(), String> {
    let root = canon(root);
    let Some(top) = toplevel(&root) else {
        return Err(
            "not a git repository — archi mutations run only inside a bound worktree, \
             and the worktree model (isolation, branches, merge) needs one. Ask the user: \
             create it (`git init` and a seed commit), or cancel the work — never \
             proceed bare."
                .to_string(),
        );
    };
    let reg = Registry::load(&root)?.expect("a repository — toplevel resolved above");
    if let Some(slug) = work {
        if let Some((owner, _)) = reg.owner_of_plan(slug) {
            if Path::new(owner) != top.as_path() {
                return Err(format!(
                    "plan `{slug}` is bound to {owner} — continue there (cd {owner}); \
                     if that checkout is gone, `archi worktree close {slug}`"
                ));
            }
        }
    }
    if reg.active_binding_of(&top).is_some() {
        return Ok(());
    }
    // One worktree carries the whole unit — spec, plan, code. When some
    // exist,
    // continuation belongs to one of them: list, never mint over them; the
    // CLI cannot know which spec a new plan serves, the caller can. A closed
    // row is no place to continue, so only standing rows are offered.
    let standing: Vec<String> = reg
        .active_entries()
        .map(|(k, b)| {
            let mut parts = Vec::new();
            if let Some(s) = &b.effort {
                parts.push(format!("spec {s}"));
            }
            if let Some(p) = &b.plan {
                parts.push(format!("plan {p}"));
            }
            let what = if parts.is_empty() { "bound".to_string() } else { parts.join(", ") };
            format!("  {what} — {k}")
        })
        .collect();
    match work {
        Some(slug) if standing.is_empty() => {
            let minted = mint(&root, slug, Some(slug), None, &[], &BTreeMap::new(), true)?;
            // a seat born here names its branch point too — it is as fresh
            // as one the operator minted by hand
            let grew =
                minted.point.as_ref().map(|p| format!(" {}", p.describe())).unwrap_or_default();
            Err(format!(
                "this checkout is unbound — mutating commands run only inside a bound \
                 worktree; minted worktree {} on branch {}{grew}; cd {} and re-run this \
                 command; the CLI never changes your directory",
                minted.path.display(),
                minted.branch,
                minted.path.display()
            ))
        }
        Some(slug) => Err(format!(
            "this checkout is unbound — mutating commands run only inside a bound \
             worktree; existing worktrees:\n{}\nif `{slug}` continues one of them, work \
             there (cd its path); only new, unrelated work mints its own: \
             `archi worktree mint {slug}`",
            standing.join("\n")
        )),
        None if standing.is_empty() => Err(
            "this checkout is unbound — mutating commands run only inside a bound \
             worktree; name the work first: `archi plan use <name>` mints a plan \
             worktree, `archi worktree mint <slug>` binds spec work without a plan"
                .to_string(),
        ),
        None => Err(format!(
            "this checkout is unbound — mutating commands run only inside a bound \
             worktree; existing worktrees:\n{}\ncontinue in one of them (cd its path), or \
             bind new work: `archi worktree mint <slug>`",
            standing.join("\n")
        )),
    }
}

/// The verdict gate `check` and `build` pass at the router: reads answer
/// anywhere, but a verdict on ungoverned work is a lie — an unbound
/// checkout whose spec carries uncommitted edits refuses with the worktree
/// recipe instead of blessing them. A bound worktree never trips it; a clean
/// tree passes (CI, the receiving checkout after a landing); a tree
/// mid-merge is exempt — the join triage (`archi-merge`) needs `check`
/// exactly while `archi/` is conflicted. Gitless stays free: branch
/// governance is the mutation guard's and the skill's full stop, and the
/// post-init smoke (`archi build`) predates the repository.
pub fn guard_verdict(root: &Path) -> Result<(), String> {
    let root = canon(root);
    let Some(top) = toplevel(&root) else {
        return Ok(());
    };
    if let Some(reg) = Registry::load(&root)? {
        if reg.active_binding_of(&top).is_some() {
            return Ok(());
        }
    }
    if let Some(p) = git_out(&top, &["rev-parse", "--git-path", "MERGE_HEAD"]) {
        let p = PathBuf::from(p);
        let p = if p.is_absolute() { p } else { top.join(p) };
        if p.exists() {
            return Ok(());
        }
    }
    let dirty = dirty_spec(&root, &top);
    if dirty.is_empty() {
        return Ok(());
    }
    let shown: Vec<String> = dirty.iter().take(8).map(|f| format!("  {f}")).collect();
    let more = dirty.len().saturating_sub(8);
    let tail = if more > 0 { format!("\n  …and {more} more") } else { String::new() };
    Err(format!(
        "the spec carries uncommitted edits outside a bound worktree:\n{}{tail}\n\
         a passing report here would bless ungoverned work — continue in an \
         existing worktree (`archi worktree ls`) or mint one (`archi worktree \
         mint <slug>`), carry the edits there, and re-run",
        shown.join("\n")
    ))
}

/// Uncommitted paths under the governed spec surface — `archi/`, the
/// manifest, and the model source dir when it lives elsewhere — relative
/// to the repository top. Machine-local files are gitignored and never
/// appear; a broken manifest falls back to the default layout (the real
/// diagnostic belongs to the compile that follows).
fn dirty_spec(root: &Path, top: &Path) -> Vec<String> {
    let rel = root.strip_prefix(top).unwrap_or(Path::new(""));
    let src = modeling_lang::source::manifest_src(root)
        .unwrap_or_else(|_| "archi/src".to_string());
    let mut specs = vec![
        rel.join("archi").display().to_string(),
        rel.join("archi.toml").display().to_string(),
    ];
    let src_rel = rel.join(&src);
    if !src_rel.starts_with(rel.join("archi")) {
        specs.push(src_rel.display().to_string());
    }
    let mut args = vec!["status", "--porcelain", "--"];
    args.extend(specs.iter().map(String::as_str));
    let Some(out) = git_out(top, &args) else {
        return Vec::new();
    };
    out.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.get(3..).unwrap_or(l).to_string())
        .collect()
}

/// Record that this checkout carries `plan`. Lenient: no git, no registry —
/// no record; ownership questions then have nothing to refuse on, which is
/// exactly the single-checkout workflow.
pub fn bind_plan(root: &Path, plan: &str) {
    let root = canon(root);
    let Some(top) = toplevel(&root) else {
        return;
    };
    let Ok(Some(mut reg)) = Registry::load(&root) else {
        return;
    };
    let branch = current_branch(&top).unwrap_or_default();
    let binding = match reg.binding_of(&top).cloned() {
        Some(b) => Binding { plan: Some(plan.to_string()), ..b },
        None => Binding {
            branch,
            status: Status::Active,
            plan: Some(plan.to_string()),
            effort: None,
            landed: None,
            members: BTreeMap::new(),
        },
    };
    reg.bind(&top, binding);
    let _ = reg.save();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn scratch() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-worktrees-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::canonicalize(&dir).unwrap()
    }

    fn git(dir: &Path, args: &[&str]) {
        let out = Command::new("git").arg("-C").arg(dir).args(args).output().unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn repo(parent: &Path, name: &str) -> PathBuf {
        let dir = parent.join(name);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q", "-b", "main"]);
        git(&dir, &["config", "user.email", "t@t"]);
        git(&dir, &["config", "user.name", "t"]);
        git(&dir, &["config", "commit.gpgsign", "false"]);
        fs::write(dir.join("seed.txt"), "seed").unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-qm", "seed"]);
        fs::canonicalize(&dir).unwrap()
    }

    fn head(dir: &Path) -> String {
        git_out(dir, &["rev-parse", "HEAD"]).expect("a head")
    }

    fn manifest(root: &Path, extra: &str) {
        fs::write(root.join("archi.toml"), format!("[project]\nname = \"t\"\n{extra}")).unwrap();
    }

    fn mint_plain(root: &Path, slug: &str, plan: Option<&str>, effort: Option<&str>) -> Result<Minted, String> {
        mint(root, slug, plan, effort, &[], &BTreeMap::new(), true)
    }

    #[test]
    fn the_registry_appears_at_first_write_and_self_heals() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        let reg_path = common_dir(&spec).unwrap().join(REGISTRY);
        let reg = Registry::load(&spec).unwrap().unwrap();
        assert!(!reg_path.exists(), "load alone creates nothing");
        drop(reg);
        let minted = mint_plain(&spec, "auth", Some("auth"), None).unwrap();
        assert!(reg_path.exists());
        assert!(minted.path.is_dir());
        assert!(!minted.attached);
        // hand-removing the worktree heals the entry on the next load
        git(&spec, &["worktree", "remove", "--force", minted.path.to_str().unwrap()]);
        let reg = Registry::load(&spec).unwrap().unwrap();
        assert!(reg.owner_of_plan("auth").is_none(), "gone from git, gone from the registry");
    }

    #[test]
    fn mint_attaches_to_an_existing_branch_and_refuses_a_checked_out_one() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        git(&spec, &["branch", "archi/auth"]);
        let minted = mint_plain(&spec, "auth", Some("auth"), None).unwrap();
        assert!(minted.attached, "existing branch is attached, not duplicated");
        // the branch is now checked out — a second mint refuses with the path
        let e = mint_plain(&spec, "auth", Some("auth"), None).unwrap_err();
        assert!(e.contains("already checked out"), "{e}");
        assert!(e.contains(&minted.path.to_string_lossy().into_owned()), "{e}");
        // ...but from inside its own worktree, mint extends instead
        let m2 = mint_plain(&minted.path, "auth", None, Some("auth-spec")).unwrap();
        assert!(m2.extended);
        let reg = Registry::load(&spec).unwrap().unwrap();
        let b = reg.binding_of(&minted.path).unwrap();
        assert_eq!(b.plan.as_deref(), Some("auth"));
        assert_eq!(b.effort.as_deref(), Some("auth-spec"));
    }

    #[test]
    fn mint_without_a_plan_binds_an_effort() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        let minted = mint_plain(&spec, "storm", None, Some("storm")).unwrap();
        let reg = Registry::load(&spec).unwrap().unwrap();
        let b = reg.binding_of(&minted.path).unwrap();
        assert_eq!(b.plan, None);
        assert_eq!(b.slug(), Some("storm"));
    }

    #[test]
    fn the_guard_is_unconditional() {
        // gitless refuses loudly, naming the repair — with or without a
        // protected list; the discipline never evaporates
        let plain = scratch();
        manifest(&plain, "");
        let e = guard_mutation(&plain, Some("x")).unwrap_err();
        assert!(e.contains("not a git repository"), "{e}");
        assert!(e.contains("git init"), "{e}");
        assert!(e.contains("or cancel"), "{e}");
        // the discipline is checkout-conditional, not branch-conditional:
        // an unbound checkout refuses on any branch, `protected` or not —
        // the list keeps only its merge meaning
        let outer = scratch();
        let spec = repo(&outer, "spec");
        manifest(&spec, "");
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "manifest"]);
        git(&spec, &["switch", "-qc", "feature"]);
        let e = guard_mutation(&spec, Some("x")).unwrap_err();
        assert!(e.contains("unbound"), "{e}");
        assert!(e.contains("bound worktree"), "{e}");
    }

    #[test]
    fn the_guard_mints_for_an_unbound_checkout_and_names_the_worktree() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        manifest(&spec, "protected = [\"main\"]\n");
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "manifest"]);
        let e = guard_mutation(&spec, Some("auth")).unwrap_err();
        assert!(e.contains("unbound"), "{e}");
        assert!(e.contains("archi/auth"), "{e}");
        let reg = Registry::load(&spec).unwrap().unwrap();
        let (owner, b) = reg.owner_of_plan("auth").unwrap();
        assert_eq!(b.branch, "archi/auth");
        // main checkout still on main, untouched
        assert_eq!(current_branch(&spec).as_deref(), Some("main"));
        // the minted worktree passes the guard
        assert!(guard_mutation(Path::new(owner), Some("auth")).is_ok());
        // any other checkout refuses with the owner's path
        let e = guard_mutation(&spec, Some("auth")).unwrap_err();
        assert!(e.contains("is bound to"), "{e}");
        // unrelated new work with worktrees standing: candidates listed, no mint
        let e = guard_mutation(&spec, Some("billing")).unwrap_err();
        assert!(e.contains("existing worktrees"), "{e}");
        assert!(e.contains("plan auth"), "{e}");
        assert!(e.contains("worktree mint billing"), "{e}");
        assert!(
            !default_worktree_dir(&spec, "billing").exists(),
            "continuation is the default — nothing minted over standing worktrees"
        );
        // a command with no work to name gets the same candidates
        let e = guard_mutation(&spec, None).unwrap_err();
        assert!(e.contains("existing worktrees"), "{e}");
        assert!(e.contains("worktree mint"), "{e}");
    }

    #[test]
    fn an_empty_registry_gets_both_recipes_for_nameless_work() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        manifest(&spec, "protected = [\"main\"]\n");
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "manifest"]);
        let e = guard_mutation(&spec, None).unwrap_err();
        assert!(e.contains("plan use"), "{e}");
        assert!(e.contains("worktree mint"), "{e}");
    }

    #[test]
    fn the_verdict_gate_refuses_only_a_dirty_spec_outside_a_worktree() {
        // gitless: free — the post-init smoke predates the repository
        let plain = scratch();
        manifest(&plain, "");
        assert!(guard_verdict(&plain).is_ok());

        let outer = scratch();
        let spec = repo(&outer, "spec");
        manifest(&spec, "");
        fs::create_dir_all(spec.join("archi/src")).unwrap();
        fs::write(spec.join("archi/src/model.arch"), "def node A\n").unwrap();
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "spec"]);
        // clean unbound tree: passes (CI, the receiving checkout)
        assert!(guard_verdict(&spec).is_ok());
        // a non-spec edit does not trip it
        fs::write(spec.join("notes.md"), "scratch\n").unwrap();
        assert!(guard_verdict(&spec).is_ok());
        // an uncommitted spec edit outside a worktree refuses with the recipe
        fs::write(spec.join("archi/src/model.arch"), "def node A\ndef node B\n").unwrap();
        let e = guard_verdict(&spec).unwrap_err();
        assert!(e.contains("uncommitted"), "{e}");
        assert!(e.contains("model.arch"), "{e}");
        assert!(e.contains("worktree mint"), "{e}");
        // mid-merge the triage is exempt
        let merge_head = spec.join(".git/MERGE_HEAD");
        fs::write(&merge_head, "0000000000000000000000000000000000000000\n").unwrap();
        assert!(guard_verdict(&spec).is_ok());
        fs::remove_file(&merge_head).unwrap();
        // committed, it passes again
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "grow"]);
        assert!(guard_verdict(&spec).is_ok());
        // the same edit inside a bound worktree never trips the gate
        let minted = mint_plain(&spec, "work", None, Some("work")).unwrap();
        fs::write(minted.path.join("archi/src/model.arch"), "def node C\n").unwrap();
        assert!(guard_verdict(&minted.path).is_ok());
    }

    #[test]
    fn bind_plan_upserts_this_checkouts_entry() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        bind_plan(&spec, "auth");
        let reg = Registry::load(&spec).unwrap().unwrap();
        let b = reg.binding_of(&spec).unwrap();
        assert_eq!(b.plan.as_deref(), Some("auth"));
        assert_eq!(b.branch, "main");
        bind_plan(&spec, "search");
        let reg = Registry::load(&spec).unwrap().unwrap();
        assert_eq!(reg.binding_of(&spec).unwrap().plan.as_deref(), Some("search"));
    }

    #[test]
    fn the_probe_answers_ancestry_content_and_absence() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        git(&spec, &["switch", "-qc", "work"]);
        fs::write(spec.join("a.txt"), "a\n").unwrap();
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "work"]);
        let sha = head(&spec);
        git(&spec, &["switch", "-q", "main"]);
        let landing =
            Landing { branch: "work".to_string(), receiving: "main".to_string(), sha: sha.clone() };

        // main carries neither the commit nor the content
        assert_eq!(integration(&spec, &landing), Verdict::NotYet);
        // a ref out of reach is no verdict at all — never a "not yet"
        let nowhere = Landing { receiving: "nowhere".to_string(), ..landing.clone() };
        assert_eq!(integration(&spec, &nowhere), Verdict::Unknown);
        let gone = Landing { sha: "0".repeat(40), ..landing.clone() };
        assert_eq!(integration(&spec, &gone), Verdict::Unknown);

        // the squash: main takes the content under a sha of its own, so
        // ancestry answers no and the tree diff answers yes
        git(&spec, &["merge", "--squash", "work"]);
        git(&spec, &["commit", "-qm", "squashed"]);
        assert_ne!(head(&spec), sha, "the forge rewrote the sha");
        assert!(
            git_out(&spec, &["merge-base", "--is-ancestor", &sha, "refs/heads/main"]).is_none(),
            "ancestry alone would answer no"
        );
        assert_eq!(integration(&spec, &landing), Verdict::Integrated);

        // ancestry: a merged branch stays integrated once main moves on and
        // the trees part
        git(&spec, &["switch", "-qc", "more"]);
        fs::write(spec.join("b.txt"), "b\n").unwrap();
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "more"]);
        let more = head(&spec);
        git(&spec, &["switch", "-q", "main"]);
        git(&spec, &["merge", "--no-edit", "more"]);
        fs::write(spec.join("c.txt"), "c\n").unwrap();
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "main moves on"]);
        let landing = Landing { branch: "more".to_string(), receiving: "main".to_string(), sha: more };
        assert_eq!(integration(&spec, &landing), Verdict::Integrated);
    }

    #[test]
    fn the_pathspec_batches_carry_every_path_once_and_in_order() {
        let paths = ["aa", "bb", "cc", "dd"];
        assert_eq!(
            pathspec_batches(&paths, 100),
            vec![vec!["aa", "bb", "cc", "dd"]],
            "one call while the budget holds"
        );
        assert_eq!(pathspec_batches(&paths, 7), vec![vec!["aa", "bb"], vec!["cc", "dd"]]);
        // a path wider than the whole budget still travels, alone
        let wide = ["x".repeat(20), "y".to_string()];
        let refs: Vec<&str> = wide.iter().map(String::as_str).collect();
        let cut = pathspec_batches(&refs, 5);
        assert_eq!(cut.len(), 2);
        assert_eq!(cut.concat(), refs, "every path once, in order — the claim stays whole");
        let none: [&str; 0] = [];
        assert!(pathspec_batches(&none, 5).is_empty());
    }

    #[test]
    fn the_probe_reads_a_squash_onto_a_branch_that_moved_on() {
        // the live shape: the receiving branch is shared, so it carries work
        // of its own beside the squashed landing
        let outer = scratch();
        let spec = repo(&outer, "spec");
        git(&spec, &["switch", "-qc", "feat"]);
        fs::write(spec.join("feat.txt"), "feat\n").unwrap();
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "feat"]);
        let feat = head(&spec);
        git(&spec, &["switch", "-q", "main"]);
        fs::write(spec.join("other.txt"), "someone else's work\n").unwrap();
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "unrelated"]);
        let landing =
            Landing { branch: "feat".to_string(), receiving: "main".to_string(), sha: feat };

        // unrelated work alone is not the landing
        assert_eq!(integration(&spec, &landing), Verdict::NotYet);

        // the forge squashes the pull request in beside it
        fs::write(spec.join("feat.txt"), "feat\n").unwrap();
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "squashed"]);
        assert!(
            git_out(&spec, &["merge-base", "--is-ancestor", &landing.sha, "refs/heads/main"])
                .is_none(),
            "ancestry alone would answer no"
        );
        assert!(
            git_out(&spec, &["diff", "--quiet", "refs/heads/main", "refs/heads/feat"]).is_none(),
            "the trees differ over other.txt — a whole-tree test would answer no forever"
        );
        assert_eq!(integration(&spec, &landing), Verdict::Integrated);
    }

    #[test]
    fn the_probe_keeps_the_seat_when_the_receiving_branch_rewrote_the_landings_paths() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        git(&spec, &["switch", "-qc", "feat"]);
        fs::write(spec.join("feat.txt"), "as the seat wrote it\n").unwrap();
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "feat"]);
        let feat = head(&spec);
        git(&spec, &["switch", "-q", "main"]);
        fs::write(spec.join("feat.txt"), "as main wrote it\n").unwrap();
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "main's own take"]);
        let landing =
            Landing { branch: "feat".to_string(), receiving: "main".to_string(), sha: feat };
        assert_eq!(
            integration(&spec, &landing),
            Verdict::NotYet,
            "one of the landing's own paths differs — the seat stands"
        );

        // a landing that touched nothing has nothing to carry
        git(&spec, &["switch", "-qc", "hollow"]);
        git(&spec, &["commit", "-q", "--allow-empty", "-m", "nothing"]);
        let hollow = head(&spec);
        git(&spec, &["switch", "-q", "main"]);
        fs::write(spec.join("main.txt"), "main moves on\n").unwrap();
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "onward"]);
        let landing =
            Landing { branch: "hollow".to_string(), receiving: "main".to_string(), sha: hollow };
        assert_eq!(integration(&spec, &landing), Verdict::Integrated);
    }

    #[test]
    fn a_moved_head_or_a_dirty_tree_stops_a_landing_record_counting() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        let minted = mint_plain(&spec, "feat", None, Some("feat")).unwrap();
        let landing = Landing {
            branch: "feat/x".to_string(),
            receiving: "main".to_string(),
            sha: head(&minted.path),
        };
        assert!(landing_stands(&minted.path, &landing));

        // the review sends the operator back into the seat
        fs::write(minted.path.join("seed.txt"), "answering the review\n").unwrap();
        assert!(!landing_stands(&minted.path, &landing), "an uncommitted edit is live work");
        git(&minted.path, &["checkout", "--", "seed.txt"]);
        assert!(
            landing_stands(&minted.path, &landing),
            "the state derives from head and tree — nothing was written to say so"
        );

        // a commit on top: the record describes a head that moved
        fs::write(minted.path.join("more.txt"), "more\n").unwrap();
        git(&minted.path, &["add", "-A"]);
        git(&minted.path, &["commit", "-qm", "resumed"]);
        assert!(!landing_stands(&minted.path, &landing));
    }

    #[test]
    fn the_sweep_frees_an_integrated_folder_and_leaves_live_work_alone() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        fs::write(spec.join(".gitignore"), "junk/\n").unwrap();
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "ignore the build output"]);
        let one = mint_plain(&spec, "one", None, Some("one")).unwrap().path;
        let two = mint_plain(&spec, "two", None, Some("two")).unwrap().path;
        for (wt, name) in [(&one, "one"), (&two, "two")] {
            fs::write(wt.join(format!("{name}.txt")), "work\n").unwrap();
            git(wt, &["add", "-A"]);
            git(wt, &["commit", "-qm", "work"]);
        }
        let to = |branch: &str| BTreeMap::from([(String::new(), branch.to_string())]);
        merge(&spec, "one", &to("feat/one")).unwrap();
        merge(&spec, "two", &to("feat/two")).unwrap();

        // the sideways landing kept both seats and recorded where they went
        assert!(one.is_dir() && two.is_dir(), "a landing is not an arrival");
        let reg = Registry::load(&spec).unwrap().unwrap();
        let landed = reg.binding_of(&one).unwrap().landed.clone().unwrap();
        assert_eq!(landed.branch, "feat/one");
        assert_eq!(landed.receiving, "main");
        assert!(sweep(&spec).freed.is_empty(), "nothing arrived yet");

        // main takes both branches; one seat holds ignored build output, the
        // other holds an untracked file no rule covers
        git(&spec, &["merge", "--no-edit", "feat/one"]);
        git(&spec, &["merge", "--no-edit", "feat/two"]);
        fs::create_dir_all(one.join("junk")).unwrap();
        fs::write(one.join("junk/build.bin"), "tens of gigabytes\n").unwrap();
        fs::write(two.join("stray.txt"), "unfinished\n").unwrap();

        let report = sweep(&spec);
        assert_eq!(report.freed.len(), 1, "one folder freed, one left standing");
        assert_eq!(report.freed[0].path, one);
        assert_eq!(report.freed[0].receiving, "main");
        assert_eq!(report.freed[0].side, Side::Spec);
        assert!(!one.exists(), "ignored files never veto a cleanup");
        assert!(two.is_dir(), "an untracked file keeps its folder");
        let reg = Registry::load(&spec).unwrap().unwrap();
        assert_eq!(reg.binding_of(&one).unwrap().status, Status::Closed);
        assert_eq!(report.closed, vec![one.to_string_lossy().into_owned()]);
        let standing = reg.binding_of(&two).unwrap();
        assert_eq!(standing.status, Status::Active);
        assert!(standing.landed.is_some(), "the record stays — it is read, never rewritten");
    }

    #[test]
    fn the_sweep_gives_members_of_one_repository_one_folder_and_one_verdict() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        let backend = repo(&outer, "backend");
        let minted = mint_plain(&spec, "feat", None, Some("feat")).unwrap();
        // the shape the cascade leaves behind: two members carried by one
        // repository, so one folder and one branch answer for both
        let bwt = outer.join("backend-worktrees").join("feat");
        git(&backend, &["worktree", "add", "-q", "-b", "archi/feat", bwt.to_str().unwrap()]);
        let bwt = fs::canonicalize(&bwt).unwrap();
        fs::write(bwt.join("lib.txt"), "member work\n").unwrap();
        git(&bwt, &["add", "-A"]);
        git(&bwt, &["commit", "-qm", "member work"]);
        let sha = head(&bwt);
        let carried = |path: &Path, checkout: &Path| MemberBinding {
            path: path.to_path_buf(),
            branch: "archi/feat".to_string(),
            base: "main".to_string(),
            checkout: checkout.to_path_buf(),
            status: Status::Active,
            landed: Some(Landing {
                branch: "archi/feat".to_string(),
                receiving: "main".to_string(),
                sha: sha.clone(),
            }),
        };
        let mut reg = Registry::load(&spec).unwrap().unwrap();
        let key = reg.resolve_key("feat").unwrap();
        let b = reg.get_mut(&key).unwrap();
        b.members.insert("api".to_string(), carried(&bwt, &backend));
        b.members.insert("web".to_string(), carried(&bwt, &backend));
        // a member no repository answers for: no verdict, no action, no error
        let gone = outer.join("gone");
        b.members.insert("ghost".to_string(), carried(&gone, &gone));
        reg.save().unwrap();

        assert!(sweep(&spec).freed.is_empty(), "the base does not carry it yet");
        assert!(bwt.is_dir());

        // the forge squashes the branch into the base
        git(&backend, &["merge", "--squash", "archi/feat"]);
        git(&backend, &["commit", "-qm", "squashed"]);
        let report = sweep(&spec);
        assert_eq!(report.freed.len(), 1, "one folder, one removal");
        assert_eq!(
            report.freed[0].side,
            Side::Members(vec!["api".to_string(), "web".to_string()])
        );
        assert!(!bwt.exists());
        let reg = Registry::load(&spec).unwrap().unwrap();
        let b = reg.binding_of(&minted.path).unwrap();
        assert_eq!(b.members["api"].status, Status::Closed);
        assert_eq!(b.members["web"].status, Status::Closed);
        assert_eq!(b.members["ghost"].status, Status::Active, "unreachable is not gone");
        assert_eq!(b.status, Status::Active, "the spec side still stands");
    }

    #[test]
    fn a_hand_removed_worktree_closes_its_row_and_keeps_it() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        let minted = mint_plain(&spec, "auth", Some("auth"), None).unwrap();
        git(&spec, &["worktree", "remove", "--force", minted.path.to_str().unwrap()]);
        let reg = Registry::load(&spec).unwrap().unwrap();
        assert_eq!(reg.entries().count(), 1, "nothing leaves the registry");
        assert_eq!(reg.binding_of(&minted.path).unwrap().status, Status::Closed);
        assert!(reg.active_binding_of(&minted.path).is_none());
        assert!(reg.owner_of_plan("auth").is_none(), "history owns no plan");
        let text = fs::read_to_string(common_dir(&spec).unwrap().join(REGISTRY)).unwrap();
        assert!(text.contains("status = \"closed\""), "the heal was written back: {text}");
    }

    #[test]
    fn a_closed_row_licenses_nothing() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        manifest(&spec, "");
        fs::create_dir_all(spec.join("archi/src")).unwrap();
        git(&spec, &["add", "-A"]);
        git(&spec, &["commit", "-qm", "manifest"]);
        bind_plan(&spec, "auth");
        // the row stands: this checkout mutates, and its edits are governed
        assert!(guard_mutation(&spec, Some("auth")).is_ok());
        fs::write(spec.join("archi/src/model.arch"), "def node A\n").unwrap();
        assert!(guard_verdict(&spec).is_ok());

        let mut reg = Registry::load(&spec).unwrap().unwrap();
        let key = reg.resolve_key("auth").unwrap();
        reg.get_mut(&key).unwrap().status = Status::Closed;
        reg.save().unwrap();

        let reg = Registry::load(&spec).unwrap().unwrap();
        assert!(reg.binding_of(&spec).is_some(), "the row stays as the record");
        assert!(reg.active_binding_of(&spec).is_none(), "history binds nothing");
        assert!(reg.owner_of_plan("auth").is_none());
        let e = guard_verdict(&spec).unwrap_err();
        assert!(e.contains("uncommitted"), "{e}");
        let e = guard_mutation(&spec, Some("auth")).unwrap_err();
        assert!(e.contains("unbound"), "{e}");
        assert!(e.contains("minted worktree"), "the recipe mints a live seat: {e}");
    }

    #[test]
    fn a_mint_of_a_closed_slug_re_opens_the_same_row() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        let first = mint_plain(&spec, "auth", Some("auth"), None).unwrap();
        git(&spec, &["worktree", "remove", "--force", first.path.to_str().unwrap()]);
        assert_eq!(
            Registry::load(&spec).unwrap().unwrap().binding_of(&first.path).unwrap().status,
            Status::Closed
        );

        let second = mint_plain(&spec, "auth", None, Some("auth-spec")).unwrap();
        assert_eq!(second.path, first.path, "the seat comes back on its own key");
        assert!(second.attached, "the branch it left is the branch it returns to");
        let reg = Registry::load(&spec).unwrap().unwrap();
        assert_eq!(reg.entries().count(), 1, "one row, re-opened — never a second");
        let b = reg.binding_of(&first.path).unwrap();
        assert_eq!(b.status, Status::Active);
        assert_eq!(b.plan.as_deref(), Some("auth"), "the row keeps what it carried");
        assert_eq!(b.effort.as_deref(), Some("auth-spec"));
        assert!(reg.owner_of_plan("auth").is_some());
    }

    #[test]
    fn resolve_key_answers_paths_slugs_and_dir_names() {
        let outer = scratch();
        let spec = repo(&outer, "spec");
        let minted = mint_plain(&spec, "auth", Some("auth"), None).unwrap();
        let reg = Registry::load(&spec).unwrap().unwrap();
        let key = minted.path.to_string_lossy().into_owned();
        assert_eq!(reg.resolve_key(&key).as_deref(), Some(key.as_str()));
        assert_eq!(reg.resolve_key("auth").as_deref(), Some(key.as_str()));
        assert_eq!(reg.resolve_key("nope"), None);
    }
}
