//! Worktree seats: the machine-local binding of work to checkouts.
//!
//! Parallel agent sessions each live in their own git worktree; which
//! worktree carries which plan is a machine fact, not shared truth. The
//! registry lives under the repository's common git dir — the one place
//! every worktree shares, tracked by none — and moves only through verbs:
//! mint writes entries, merge clears them, `worktree ls`/`drop` read and
//! repair (`archi/requirements/worktree-parallelism/`).
//!
//! Git queries are lenient (`Option`, absence is a value); git mutations are
//! loud (`Result` carrying git's own stderr). No verb here ever changes the
//! caller's working directory — minting prints the path, entering it is the
//! caller's move.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

/// The registry file, relative to the common git dir.
const REGISTRY: &str = "archi/worktrees.toml";

/// The branch a slug's work lives on.
pub fn branch_of(slug: &str) -> String {
    format!("archi/{slug}")
}

// ---------------------------------------------------------------------------
// Git plumbing

fn canon(p: &Path) -> PathBuf {
    fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

/// Lenient query: `None` on any failure (no git, not a repo, bad rev).
fn git_out(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git").arg("-C").arg(dir).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Loud mutation: `Err` carries git's stderr verbatim.
fn git_run(dir: &Path, args: &[&str]) -> Result<String, String> {
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

pub fn branch_exists(repo: &Path, name: &str) -> bool {
    let refname = format!("refs/heads/{name}");
    git_out(repo, &["rev-parse", "--verify", "--quiet", &refname]).is_some()
}

fn worktree_add(
    repo: &Path,
    path: &Path,
    branch: &str,
    create: bool,
    base: Option<&str>,
) -> Result<(), String> {
    let path_s = path.to_string_lossy().into_owned();
    let mut args: Vec<&str> = if create {
        vec!["worktree", "add", "-b", branch, &path_s]
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

/// Keep archi's machine-local seat artifacts out of git without touching
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
    text.push_str("# archi seat artifacts — machine-local, never committed\n");
    for p in missing {
        text.push_str(p);
        text.push('\n');
    }
    let _ = fs::write(&path, text);
}

/// Remove the seat artifacts archi itself wrote into a worktree — the
/// member overlay and the active-plan marker — so a retire's non-force
/// `worktree remove` only ever refuses over the *user's* uncommitted work.
pub fn scrub_seat(wt_project: &Path) {
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

/// One member's cascaded seat within a binding.
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
}

/// What one worktree carries.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Binding {
    pub branch: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub members: BTreeMap<String, MemberBinding>,
}

impl Binding {
    /// The slug this binding answers to: the plan, or the spec effort.
    pub fn slug(&self) -> Option<&str> {
        self.plan.as_deref().or(self.effort.as_deref())
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
    /// registry — it appears at the first write, no init step. Entries whose
    /// worktree git no longer lists are dropped (written back only when
    /// something dropped); a member seat is dropped only when its own repo
    /// confirms the worktree gone, an unreachable member repo keeps it.
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
        reg.entries.retain(|k, _| {
            let keep = live.iter().any(|p| p.as_path() == Path::new(k));
            healed |= !keep;
            keep
        });
        for b in reg.entries.values_mut() {
            b.members.retain(|_, m| {
                let repo = if m.checkout.is_dir() { &m.checkout } else { &m.path };
                let listed = list_worktrees(repo);
                // an unreachable repo lists nothing — keep the seat
                let keep = listed.is_empty()
                    || listed.iter().any(|w| w.path == m.path);
                healed |= !keep;
                keep
            });
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
            format!("# machine-local worktree bindings — operated by archi verbs, never merged\n{body}");
        fs::write(&self.path, text).map_err(|e| format!("cannot write {}: {e}", self.path.display()))
    }

    pub fn entries(&self) -> impl Iterator<Item = (&str, &Binding)> {
        self.entries.iter().map(|(k, v)| (k.as_str(), v))
    }

    pub fn binding_of(&self, worktree: &Path) -> Option<&Binding> {
        self.entries.get(&canon(worktree).to_string_lossy().into_owned())
    }

    /// The worktree that carries `plan`, when one does.
    pub fn owner_of_plan(&self, plan: &str) -> Option<(&str, &Binding)> {
        self.entries
            .iter()
            .find(|(_, b)| b.plan.as_deref() == Some(plan))
            .map(|(k, b)| (k.as_str(), b))
    }

    pub fn bind(&mut self, worktree: &Path, binding: Binding) {
        self.entries
            .insert(canon(worktree).to_string_lossy().into_owned(), binding);
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Binding> {
        self.entries.get_mut(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Binding> {
        self.entries.remove(key)
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
// Mint

/// Where a slug's worktree goes: a sibling folder of the checkout.
pub fn default_worktree_dir(top: &Path, slug: &str) -> PathBuf {
    let name = top
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "repo".to_string());
    top.parent().unwrap_or(top).join(format!("{name}-worktrees")).join(slug)
}

#[derive(Debug)]
pub struct Minted {
    pub path: PathBuf,
    pub branch: String,
    /// True when the branch already existed and the worktree attached to it.
    pub attached: bool,
    /// True when the caller already sits in the slug's worktree and the
    /// binding was merely extended.
    pub seated: bool,
}

/// Mint the seat for `slug`: the branch (created, or attached when it
/// already exists), the worktree, the member cascade, the overlay, the
/// registry entry — entry last, and a partial cascade rolls back whole, so
/// no failure leaves a dangling binding. Re-minting from inside the slug's
/// own worktree extends the binding (new members, plan/effort upserts)
/// instead of creating anything anew.
pub fn mint(
    root: &Path,
    slug: &str,
    plan: Option<&str>,
    effort: Option<&str>,
    repos: &[String],
    bases: &BTreeMap<String, String>,
) -> Result<Minted, String> {
    let root = canon(root);
    let top =
        toplevel(&root).ok_or_else(|| "not a git repository — worktrees need git".to_string())?;
    let branch = branch_of(slug);
    let mut reg = Registry::load(&root)?.expect("toplevel resolved, so common dir does");
    ensure_excludes(&top);
    // where the project sits inside the tree — a monorepo spec below the
    // git root keeps that offset inside its seat too
    let rel = root.strip_prefix(&top).unwrap_or(Path::new("")).to_path_buf();
    let worktrees = list_worktrees(&top);
    let seated = worktrees
        .iter()
        .find(|w| w.branch.as_deref() == Some(branch.as_str()))
        .map(|w| w.path.clone());
    if let Some(seat) = &seated {
        if *seat != top {
            return Err(format!(
                "`{branch}` is already checked out at {} — continue there (cd {}); \
                 if that checkout is gone, run `git worktree prune` and retry",
                seat.display(),
                seat.display()
            ));
        }
    }
    let existing = match &seated {
        Some(_) => reg.binding_of(&top).cloned(),
        None => None,
    };
    let known: Vec<String> = existing
        .as_ref()
        .map(|b| b.members.keys().cloned().collect())
        .unwrap_or_default();
    // Members resolve against the invoked project root — the checkout that
    // carries the unit's manifest, overlay and archive: the primary on a
    // first cascade, the seat itself on a mid-unit extension (declarations
    // and anchors made in the seat land on the primary only at the final
    // merge). Already-cascaded members are excluded above, so the seat's
    // overlay rows pointing at member worktrees never feed the cascade;
    // a fresh member unresolvable from here refuses toward `repo map` —
    // member locations are machine-local truth, the overlay carries them.
    let fresh_repos: Vec<String> =
        repos.iter().filter(|r| !known.contains(r)).cloned().collect();
    let targets = if fresh_repos.is_empty() {
        Vec::new()
    } else {
        plan_cascade(&root, slug, plan, &fresh_repos, bases)?
    };

    // create: spec worktree (unless seated), then member worktrees — any
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
    let (wt_path, attached) = match &seated {
        Some(seat) => (seat.clone(), true),
        None => {
            let path = default_worktree_dir(&top, slug);
            if path.exists() {
                return Err(format!(
                    "{} already exists but is not a worktree of this repository — move it aside",
                    path.display()
                ));
            }
            let attached = branch_exists(&top, &branch);
            worktree_add(&top, &path, &branch, !attached, None)?;
            let path = canon(&path);
            created.push((top.clone(), path.clone(), !attached));
            (path, attached)
        }
    };
    let mut members: BTreeMap<String, MemberBinding> =
        existing.as_ref().map(|b| b.members.clone()).unwrap_or_default();
    for t in &targets {
        let base = (!t.attach).then_some(t.base.as_str());
        if let Err(e) = worktree_add(&t.repo_top, &t.target, &branch, !t.attach, base) {
            rollback(&created);
            return Err(format!("{}: {e}", t.carries[0].0));
        }
        created.push((t.repo_top.clone(), canon(&t.target), !t.attach));
        let target = canon(&t.target);
        for (name, checkout) in &t.carries {
            members.insert(
                name.clone(),
                MemberBinding {
                    path: target.clone(),
                    branch: branch.clone(),
                    base: t.base.clone(),
                    checkout: checkout.clone(),
                },
            );
        }
    }
    // the overlay the seat resolves members through — every cascaded member,
    // old and new, points at its member worktree
    if !members.is_empty() {
        let rows: Vec<(String, PathBuf)> = members
            .iter()
            .map(|(name, m)| (name.clone(), member_row(&m.path, &m.checkout)))
            .collect();
        if let Err(e) = write_worktree_overlay(&wt_path.join(&rel), &rows) {
            rollback(&created);
            return Err(e);
        }
    }
    let binding = Binding {
        branch: branch.clone(),
        plan: plan.map(str::to_string).or(existing.as_ref().and_then(|b| b.plan.clone())),
        effort: effort.map(str::to_string).or(existing.as_ref().and_then(|b| b.effort.clone())),
        members,
    };
    reg.bind(&wt_path, binding);
    reg.save()?;
    Ok(Minted { path: wt_path, branch, attached, seated: seated.is_some() })
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
                    &sha[..sha.len().min(7)],
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
                    &sha[..sha.len().min(7)]
                ));
                continue;
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
        targets.push(CascadeTarget {
            repo_top,
            target,
            base,
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
        String::from("# member worktrees of this seat — written by archi worktree mint\n");
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
// Merge — the closing verb

/// One repository's outcome inside a merge.
#[derive(Debug)]
pub enum RepoOutcome {
    /// Spec: merged into the receiving branch (or already up to date).
    Merged,
    /// Spec: landed on a new branch without merging (`--to`).
    Landed { branch: String },
    /// Spec: the merge stopped on conflicts; nothing retired.
    Conflict { detail: String },
    /// Member: its branch went to the remote.
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
    /// True when the worktree and its binding are gone.
    pub retired: bool,
}

/// Close a worktree: merge its branch into the current branch of this
/// checkout — or land it on a new branch with `to` — push each member's
/// branch, then remove the worktree and clear its binding in the same move.
/// A conflict (or a refused member) stops short of retiring; re-running
/// after the repair is idempotent. `to` keys: `""` = spec, member name =
/// that member's remote branch.
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
    // A seat lands only after its plan closes: work mid-wave never merges.
    // Spec-only seats (no plan) land freely.
    if let Some(plan_name) = &binding.plan {
        let rel = root.strip_prefix(&top).unwrap_or(Path::new(""));
        if let Ok(plans) = crate::plans::all_plans(&wt_path.join(rel)) {
            if let Some(p) = plans.iter().find(|p| &p.name == plan_name) {
                if p.state != crate::plans::PlanState::Completed {
                    return Err(format!(
                        "plan `{plan_name}` is {} — a seat lands only after its plan \
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

    // Members first: push is independent of the spec merge, and a refused
    // member must not block the spec's landing (or vice versa).
    let mut members: Vec<(String, RepoOutcome)> = Vec::new();
    for (name, m) in &binding.members {
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
                if m.path.is_dir() && m.path != repo {
                    if let Err(e) = worktree_remove(&repo, &m.path, false) {
                        members.push((name.clone(), RepoOutcome::Refused {
                            detail: format!("pushed, but the worktree stays: {e}"),
                        }));
                        continue;
                    }
                }
                let _ = git_run(&repo, &["branch", "-D", &m.branch]);
                members.push((name.clone(), RepoOutcome::Pushed { remote_branch }));
            }
            Err(e) => {
                members.push((name.clone(), RepoOutcome::Refused { detail: e }));
            }
        }
    }
    let b = reg.get_mut(&key).expect("resolved key");
    for (name, outcome) in &members {
        if matches!(outcome, RepoOutcome::Pushed { .. }) {
            b.members.remove(name);
        }
    }
    let members_clear = b.members.is_empty();
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
        RepoOutcome::Landed { branch: new_branch.clone() }
    } else {
        match git_run(&top, &["merge", "--no-edit", &branch]) {
            Ok(_) => RepoOutcome::Merged,
            Err(detail) => RepoOutcome::Conflict { detail },
        }
    };

    // Retire — worktree first, then the binding, so no failure leaves a
    // dangling entry pointing at nothing.
    let mut retired = false;
    if !matches!(spec, RepoOutcome::Conflict { .. }) && members_clear {
        let rel = root.strip_prefix(&top).unwrap_or(Path::new(""));
        scrub_seat(&wt_path.join(rel));
        worktree_remove(&top, &wt_path, false).map_err(|e| {
            format!("{e}\nthe worktree keeps its binding; commit or clean it, then re-run")
        })?;
        if matches!(spec, RepoOutcome::Merged) {
            let _ = git_run(&top, &["branch", "-d", &branch]);
        }
        let mut reg = Registry::load(&root)?.expect("still a repository");
        reg.remove(&key);
        reg.save()?;
        retired = true;
    }
    Ok(MergeReport { worktree: wt_path, branch, spec, members, retired })
}

// ---------------------------------------------------------------------------
// The guard

/// The three-outcome gate every mutating route passes — wired once, at the
/// router in `main`, never inside verb bodies: bound here — proceed; bound
/// elsewhere — refuse naming the owner; unbound — refuse toward a seat.
/// The discipline is unconditional: the binding, not the branch, is the
/// license to mutate — an unbound checkout (the primary included) never
/// mutates, and gitless refuses loudly: the seat model (isolation,
/// branches, merge) needs a repository. `protected` in archi.toml keeps a
/// single meaning — branches that never receive a local merge.
pub fn guard_mutation(root: &Path, work: Option<&str>) -> Result<(), String> {
    let root = canon(root);
    let Some(top) = toplevel(&root) else {
        return Err(
            "not a git repository — archi mutations run only inside a seated worktree, \
             and the seat model (isolation, branches, merge) needs one. Ask the user: \
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
                    "plan `{slug}` is seated at {owner} — continue there (cd {owner}); \
                     if that checkout is gone, `archi worktree drop {slug}`"
                ));
            }
        }
    }
    if reg.binding_of(&top).is_some() {
        return Ok(());
    }
    // One seat carries the whole unit — spec, plan, code. When seats exist,
    // continuation belongs to one of them: list, never mint over them; the
    // CLI cannot know which spec a new plan serves, the caller can.
    let seats: Vec<String> = reg
        .entries()
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
        Some(slug) if seats.is_empty() => {
            let minted = mint(&root, slug, Some(slug), None, &[], &BTreeMap::new())?;
            Err(format!(
                "this checkout is unbound — mutating verbs run only inside a seated \
                 worktree; minted worktree {} on branch {}; cd {} and re-run this verb; \
                 the CLI never changes your directory",
                minted.path.display(),
                minted.branch,
                minted.path.display()
            ))
        }
        Some(slug) => Err(format!(
            "this checkout is unbound — mutating verbs run only inside a seated \
             worktree; existing seats:\n{}\nif `{slug}` continues one of them, work \
             there (cd its path); only new, unrelated work mints its own: \
             `archi worktree mint {slug}`",
            seats.join("\n")
        )),
        None if seats.is_empty() => Err(
            "this checkout is unbound — mutating verbs run only inside a seated \
             worktree; name the work first: `archi plan use <name>` mints a plan \
             worktree, `archi worktree mint <slug>` seats spec work without a plan"
                .to_string(),
        ),
        None => Err(format!(
            "this checkout is unbound — mutating verbs run only inside a seated \
             worktree; existing seats:\n{}\ncontinue in one of them (cd its path), or \
             seat new work: `archi worktree mint <slug>`",
            seats.join("\n")
        )),
    }
}

/// The verdict gate `check` and `build` pass at the router: reads answer
/// anywhere, but a verdict on ungoverned work is a lie — an unbound
/// checkout whose spec carries uncommitted edits refuses with the seat
/// recipe instead of blessing them. A bound seat never trips it; a clean
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
        if reg.binding_of(&top).is_some() {
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
        "the spec carries uncommitted edits outside a seated worktree:\n{}{tail}\n\
         a passing report here would bless ungoverned work — continue in an \
         existing seat (`archi worktree ls`) or mint one (`archi worktree \
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
            plan: Some(plan.to_string()),
            effort: None,
            members: BTreeMap::new(),
        },
    };
    reg.bind(&top, binding);
    let _ = reg.save();
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn manifest(root: &Path, extra: &str) {
        fs::write(root.join("archi.toml"), format!("[project]\nname = \"t\"\n{extra}")).unwrap();
    }

    fn mint_plain(root: &Path, slug: &str, plan: Option<&str>, effort: Option<&str>) -> Result<Minted, String> {
        mint(root, slug, plan, effort, &[], &BTreeMap::new())
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
    fn mint_attaches_to_an_existing_branch_and_refuses_a_seated_one() {
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
        assert!(m2.seated);
        let reg = Registry::load(&spec).unwrap().unwrap();
        let b = reg.binding_of(&minted.path).unwrap();
        assert_eq!(b.plan.as_deref(), Some("auth"));
        assert_eq!(b.effort.as_deref(), Some("auth-spec"));
    }

    #[test]
    fn mint_without_a_plan_seats_an_effort() {
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
        assert!(e.contains("seated worktree"), "{e}");
    }

    #[test]
    fn the_guard_mints_for_an_unbound_checkout_and_names_the_seat() {
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
        assert!(e.contains("seated at"), "{e}");
        // unrelated new work with seats standing: candidates listed, no mint
        let e = guard_mutation(&spec, Some("billing")).unwrap_err();
        assert!(e.contains("existing seats"), "{e}");
        assert!(e.contains("plan auth"), "{e}");
        assert!(e.contains("worktree mint billing"), "{e}");
        assert!(
            !default_worktree_dir(&spec, "billing").exists(),
            "continuation is the default — nothing minted over standing seats"
        );
        // a verb with no work to name gets the same candidates
        let e = guard_mutation(&spec, None).unwrap_err();
        assert!(e.contains("existing seats"), "{e}");
        assert!(e.contains("worktree mint"), "{e}");
    }

    #[test]
    fn a_seatless_registry_gets_both_recipes_for_nameless_work() {
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
    fn the_verdict_gate_refuses_only_a_dirty_spec_outside_a_seat() {
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
        // an uncommitted spec edit outside a seat refuses with the recipe
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
        // the same edit inside a bound seat never trips the gate
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
