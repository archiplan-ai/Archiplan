//! The version archive: `archi/versions/` under the project root
//! (`archi/requirements/versioning/`).
//!
//! A version is the compiled model's canonical render
//! ([`Model::render_source`]), identified by the sha256 of its bytes and
//! stored as **keyframes** (full renders, `vNNNN.arch`) and forward
//! **patches** (unified diffs, `vNNNN.arch.patch`) under an append-only
//! `index.toml` manifest. The archive is sealed: every file is pinned by the
//! manifest hashes, reconstruction verifies against them, and `archi check`
//! re-verifies the whole chain. Git history is provenance and the recovery
//! path, never a dependency — everything here works from the files alone.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use modeling_lang::Model;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// How a version's canonical render is stored on disk.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// A keyframe: `vNNNN.arch` holds the whole canonical render.
    Full,
    /// `vNNNN.arch.patch` holds a unified diff from the previous version.
    Patch,
}

impl Kind {
    /// Human name of the encoding: `keyframe` or `patch`.
    pub fn describe(self) -> &'static str {
        match self {
            Kind::Full => "keyframe",
            Kind::Patch => "patch",
        }
    }
}

/// Merkle hashes of one root scope, recorded per version so scope history
/// is a manifest scan (`archi/requirements/versioning/scopes-version-by-hash.md`).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ScopeHashes {
    /// Hash of everything under the node, in canonical order.
    pub full: String,
    /// Hash of the node's define (declared ports) plus its boundary edges.
    pub interface: String,
}

/// One manifest entry. Scalar fields precede `scopes` so the TOML
/// array-of-tables serializes without ambiguity.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Entry {
    /// Dense sequential id, `v0001` onward.
    pub id: String,
    /// The save's mandatory prose note.
    pub note: String,
    /// ISO-8601 UTC timestamp of the save.
    pub created: String,
    /// `sha256:<hex>` of the version's canonical render.
    pub model: String,
    /// The previous version's id; absent on the first entry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Storage encoding of this entry.
    pub kind: Kind,
    /// Name of the preset the model was compiled against.
    pub preset: String,
    /// Git commit the save happened on — recorded only when the working
    /// tree was clean, as provenance; nothing depends on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// Per-member code baselines: where each mapped member's code stood
    /// when this version was agreed. Recorded at save for clean members,
    /// post hoc by `anchor --repo`; provenance like `commit`, never a
    /// dependency. Absent on memberless projects — the entry stays today's.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub commits: BTreeMap<String, Baseline>,
    /// Root-scope hashes, keyed by root node name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub scopes: BTreeMap<String, ScopeHashes>,
}

/// One member's code baseline, remembering how it was born — a save-time
/// recording under the clean-tree guarantee, or a post-hoc anchor whose
/// window the audit must word honestly
/// (`archi/requirements/multi-repo/a-late-baseline-says-so`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Baseline {
    pub sha: String,
    pub born: Born,
}

/// How a baseline came to be recorded.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Born {
    /// At save, the member's tree clean.
    Save,
    /// Post hoc via `archi version anchor --repo` — the span between the
    /// version and the anchor is unaudited, and reports say so.
    Anchor,
}

#[derive(Default, Deserialize)]
struct ManifestFile {
    #[serde(default, rename = "version")]
    versions: Vec<Entry>,
}

#[derive(Serialize)]
struct ManifestOut<'a> {
    version: &'a [Entry],
}

/// The outcome of a save.
pub enum Saved {
    /// The render hashes equal to the latest version: nothing to mint.
    Unchanged {
        /// The version the model is already at.
        latest: String,
    },
    /// A new version was written.
    Written {
        /// The minted id.
        id: String,
        /// How it was stored.
        kind: Kind,
        /// The file written under `archi/versions/`.
        file: PathBuf,
        /// Its size in bytes.
        bytes: usize,
        /// Per-member baseline outcomes, one display line each — recorded
        /// members and named omissions. Empty on memberless projects.
        baseline_notes: Vec<String>,
    },
}

/// Where the live model stands relative to the archive.
pub enum Current {
    /// No archive, or an empty one.
    NoVersions,
    /// The live render matches this saved version.
    At(String),
    /// The live render matches no saved version.
    DirtySince(String),
}

/// The outcome of an anchor.
#[derive(Debug)]
pub enum Anchored {
    /// Commit provenance was recorded on the version.
    Recorded {
        /// The anchored version.
        id: String,
        /// The commit recorded as its provenance.
        commit: String,
    },
    /// The mark already answers: home provenance is a birth fact and never
    /// rewrites; a member baseline stands when the checkout's tip is the
    /// recorded one, or when the version is older history.
    Already {
        /// The version.
        id: String,
        /// Its recorded provenance.
        commit: String,
    },
    /// The latest version's member baseline moved to the member's clean
    /// tip — the explicit command re-marks the latest version only, anchor-born.
    Reanchored {
        /// The re-anchored version.
        id: String,
        /// The freshly recorded commit.
        commit: String,
        /// The sha the mark carried before.
        was: String,
    },
}

/// A loaded version archive.
pub struct Archive {
    dir: PathBuf,
    entries: Vec<Entry>,
}

const GITATTRIBUTES: &str = "# Keyframes are generated renders; patches are the readable change record.\n\
     v*.arch linguist-generated\n";

const INDEX_HEADER: &str = "# Version archive: append-only, sealed by the hashes below.\n\
     # See archi/requirements/versioning/; verified on `archi check`.\n\n";

impl Archive {
    /// The archive directory of a project root.
    pub fn dir_of(root: &Path) -> PathBuf {
        root.join("archi").join("versions")
    }

    /// Load a project's archive; `Ok(None)` when it has none yet.
    pub fn open(root: &Path) -> Result<Option<Archive>, String> {
        let dir = Self::dir_of(root);
        let index = dir.join("index.toml");
        if !index.exists() {
            return Ok(None);
        }
        let text = fs::read_to_string(&index)
            .map_err(|e| format!("cannot read `{}`: {e}", index.display()))?;
        let manifest: ManifestFile = toml::from_str(&text).map_err(|e| {
            if text.contains("<<<<<<<") || text.contains(">>>>>>>") {
                format!(
                    "`{}` holds merge conflict markers — two branches minted the same version \
                     id. Keep the first-landed entry and its patch file (both sides' model and \
                     doc work is already merged), then re-mint the later round onto the lineage: \
                     `archi version remint -m <note> --session <slug>` — run it in the later \
                     round's worktree, re-attached with `archi worktree mint <slug>` if it was \
                     already retired \
                     (archi/requirements/self-hosting/parallel-editing-discipline.md)",
                    index.display()
                )
            } else {
                format!("`{}` does not parse: {e}", index.display())
            }
        })?;
        Ok(Some(Archive {
            dir,
            entries: manifest.versions,
        }))
    }

    fn open_or_empty(root: &Path) -> Result<Archive, String> {
        Ok(Self::open(root)?.unwrap_or(Archive {
            dir: Self::dir_of(root),
            entries: Vec::new(),
        }))
    }

    /// The manifest entries, oldest first.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The entry an id names, when the archive holds it.
    pub fn entry(&self, id: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.id == id)
    }

    fn file_name(e: &Entry) -> String {
        match e.kind {
            Kind::Full => format!("{}.arch", e.id),
            Kind::Patch => format!("{}.arch.patch", e.id),
        }
    }

    fn file_of(&self, e: &Entry) -> PathBuf {
        self.dir.join(Self::file_name(e))
    }

    fn read_file(&self, e: &Entry) -> Result<String, String> {
        let p = self.file_of(e);
        fs::read_to_string(&p).map_err(|err| {
            format!(
                "cannot read `{}`: {err} — the manifest names this file: a save's artifacts \
                 (manifest entry, patch or keyframe, session stamp) travel as one commit; a \
                 missing one is usually a half-committed save — recover it from the save \
                 author's tree or git history (archi/requirements/self-hosting/parallel-editing-discipline.md)",
                p.display()
            )
        })
    }

    /// Reconstruct a version's canonical render: nearest keyframe at or
    /// before it, forward patches applied, the result verified against the
    /// manifest hash.
    pub fn reconstruct(&self, id: &str) -> Result<String, String> {
        let target = self
            .entries
            .iter()
            .position(|e| e.id == id)
            .ok_or_else(|| format!("no version `{id}` in the archive"))?;
        let start = self.entries[..=target]
            .iter()
            .rposition(|e| e.kind == Kind::Full)
            .ok_or_else(|| format!("no keyframe at or before `{id}`; the archive is corrupt"))?;
        let mut text = self.read_file(&self.entries[start])?;
        for e in &self.entries[start + 1..=target] {
            text = apply_patch(&text, &self.read_file(e)?)
                .map_err(|err| format!("`{}`: {err}", e.id))?;
        }
        let entry = &self.entries[target];
        let actual = hash(&text);
        if actual != entry.model {
            return Err(seal_broken(&entry.id, &actual, &entry.model));
        }
        Ok(text)
    }

    /// Verify the whole archive: dense ids, linear parents, an opening
    /// keyframe, every version reconstructing to its sealed hash, and no
    /// stray files. Returns every problem found; empty means intact.
    pub fn verify(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if let Some(first) = self.entries.first()
            && first.kind != Kind::Full
        {
            errors.push(format!(
                "`{}` opens the archive but is not a keyframe",
                first.id
            ));
        }
        for (i, e) in self.entries.iter().enumerate() {
            let want = id_of(i);
            if e.id != want {
                errors.push(format!(
                    "manifest entry {} is `{}`, expected `{want}`: ids are a dense sequence",
                    i + 1,
                    e.id
                ));
            }
            let want_parent = i.checked_sub(1).map(|p| self.entries[p].id.clone());
            if e.parent != want_parent {
                errors.push(format!(
                    "`{}` names parent `{}`, expected `{}`",
                    e.id,
                    e.parent.as_deref().unwrap_or("none"),
                    want_parent.as_deref().unwrap_or("none"),
                ));
            }
        }
        // Walk the chain forward, verifying every entry's seal. A broken
        // link leaves the entries up to the next keyframe unverifiable.
        let mut text: Option<String> = None;
        for e in &self.entries {
            let content = match self.read_file(e) {
                Ok(c) => c,
                Err(err) => {
                    errors.push(err);
                    text = None;
                    continue;
                }
            };
            text = match e.kind {
                Kind::Full => Some(content),
                Kind::Patch => match text.take() {
                    None => {
                        errors.push(format!(
                            "`{}` is unverifiable: no intact base to apply its patch to",
                            e.id
                        ));
                        None
                    }
                    Some(base) => match apply_patch(&base, &content) {
                        Ok(t) => Some(t),
                        Err(err) => {
                            errors.push(format!("`{}`: {err}", e.id));
                            None
                        }
                    },
                },
            };
            if let Some(t) = &text {
                let actual = hash(t);
                if actual != e.model {
                    errors.push(seal_broken(&e.id, &actual, &e.model));
                    text = None;
                }
            }
        }
        // The archive is sealed: files the manifest does not know are errors.
        let expected: Vec<String> = self.entries.iter().map(Self::file_name).collect();
        if let Ok(dir) = fs::read_dir(&self.dir) {
            for f in dir.flatten() {
                let name = f.file_name().to_string_lossy().into_owned();
                if name == "index.toml" || name.starts_with('.') {
                    continue;
                }
                if !expected.contains(&name) {
                    errors.push(format!(
                        "`{name}` is not in the manifest: the archive is append-only through `archi version save`"
                    ));
                }
            }
        }
        errors
    }

    fn write_manifest(&self) -> Result<(), String> {
        let body = toml::to_string_pretty(&ManifestOut {
            version: &self.entries,
        })
        .map_err(|e| format!("manifest does not serialize: {e}"))?;
        let index = self.dir.join("index.toml");
        fs::write(&index, format!("{INDEX_HEADER}{body}"))
            .map_err(|e| format!("cannot write `{}`: {e}", index.display()))
    }
}

/// Verify a project's archive; no archive is vacuously intact.
pub fn verify_at(root: &Path) -> Vec<String> {
    match Archive::open(root) {
        Ok(None) => Vec::new(),
        Ok(Some(a)) => a.verify(),
        Err(e) => vec![e],
    }
}

/// Save the compiled model as a new version. Refuses when the canonical
/// render hashes equal to the latest version — versions mint only on
/// semantic change.
pub fn save(root: &Path, model: &Model, note: &str) -> Result<Saved, String> {
    let mut archive = Archive::open_or_empty(root)?;
    let canonical = model.render_source();
    let model_hash = hash(&canonical);
    if let Some(last) = archive.entries.last()
        && last.model == model_hash
    {
        return Ok(Saved::Unchanged {
            latest: last.id.clone(),
        });
    }
    let (kind, content) = match archive.entries.last() {
        None => (Kind::Full, canonical.clone()),
        Some(last) => {
            let prev = archive.reconstruct(&last.id)?;
            let patch = diffy::create_patch(&prev, &canonical).to_string();
            // Keyframe policy: when the patches since the last keyframe —
            // this one included — outgrow the render itself, write a
            // keyframe. Total archive bytes stay within about twice the
            // keyframe bytes, whatever the churn pattern.
            let since: u64 = archive
                .entries
                .iter()
                .rev()
                .take_while(|e| e.kind == Kind::Patch)
                .filter_map(|e| fs::metadata(archive.file_of(e)).ok())
                .map(|m| m.len())
                .sum();
            if since + patch.len() as u64 > canonical.len() as u64 {
                (Kind::Full, canonical.clone())
            } else {
                (Kind::Patch, patch)
            }
        }
    };
    let (commits, baseline_notes) = member_baselines(root);
    let entry = Entry {
        id: id_of(archive.entries.len()),
        note: note.to_string(),
        created: iso8601_utc(SystemTime::now()),
        model: model_hash,
        parent: archive.entries.last().map(|e| e.id.clone()),
        kind,
        preset: model.preset_name().to_string(),
        commit: provenance(root),
        commits,
        scopes: model
            .scope_sources()
            .into_iter()
            .map(|s| {
                (
                    s.path,
                    ScopeHashes {
                        full: hash(&s.full),
                        interface: hash(&s.interface),
                    },
                )
            })
            .collect(),
    };
    fs::create_dir_all(&archive.dir)
        .map_err(|e| format!("cannot create `{}`: {e}", archive.dir.display()))?;
    let gitattributes = archive.dir.join(".gitattributes");
    if !gitattributes.exists() {
        fs::write(&gitattributes, GITATTRIBUTES)
            .map_err(|e| format!("cannot write `{}`: {e}", gitattributes.display()))?;
    }
    let file = archive.dir.join(Archive::file_name(&entry));
    fs::write(&file, &content).map_err(|e| format!("cannot write `{}`: {e}", file.display()))?;
    let (id, kind) = (entry.id.clone(), entry.kind);
    archive.entries.push(entry);
    archive.write_manifest()?;
    Ok(Saved::Written {
        id,
        kind,
        file,
        bytes: content.len(),
        baseline_notes,
    })
}

/// Each mapped member's baseline at save: recorded when its tree is clean,
/// named as an omission otherwise — the report says what the entry could
/// not, so a missing baseline is a choice the operator sees, not a silent
/// gap (`archi/requirements/multi-repo/provenance-goes-per-member`).
fn member_baselines(root: &Path) -> (BTreeMap<String, Baseline>, Vec<String>) {
    let mut commits = BTreeMap::new();
    let mut notes = Vec::new();
    let set = match crate::members::MemberSet::resolve(root) {
        Ok(s) => s,
        Err(e) => {
            notes.push(format!("member baselines skipped: {e}"));
            return (commits, notes);
        }
    };
    if set.is_single() {
        // Today's project: no member machinery, no notes, the entry as
        // written since v0001.
        return (commits, notes);
    }
    for m in set.declared() {
        let Some(mroot) = &m.root else {
            notes.push(format!(
                "no baseline for `{}`: unreachable here — map it and `archi version anchor --repo {}`",
                m.name, m.name
            ));
            continue;
        };
        let Some(ctx) = crate::members::GitContext::of(mroot) else {
            notes.push(format!(
                "no baseline for `{}`: not a git work tree",
                m.name
            ));
            continue;
        };
        match (ctx.clean(), ctx.head()) {
            (Some(true), Some(sha)) => {
                notes.push(format!("baseline {}: {}", m.name, &sha[..sha.len().min(7)]));
                commits.insert(
                    m.name.clone(),
                    Baseline {
                        sha,
                        born: Born::Save,
                    },
                );
            }
            (Some(false), _) => notes.push(format!(
                "no baseline for `{}`: its tree is dirty — commit it, then `archi version anchor --repo {}`",
                m.name, m.name
            )),
            _ => notes.push(format!(
                "no baseline for `{}`: the repository has no commits yet",
                m.name
            )),
        }
    }
    (commits, notes)
}

/// Which version the live model is at, by hash comparison — "current" is
/// derived, never stored.
pub fn current(root: &Path, model: &Model) -> Result<Current, String> {
    let Some(archive) = Archive::open(root)? else {
        return Ok(Current::NoVersions);
    };
    let Some(last) = archive.entries.last() else {
        return Ok(Current::NoVersions);
    };
    let live = hash(&model.render_source());
    Ok(
        match archive.entries.iter().rev().find(|e| e.model == live) {
            Some(e) => Current::At(e.id.clone()),
            None => Current::DirtySince(last.id.clone()),
        },
    )
}

/// Record commit provenance on the version the live model is at — the
/// post-hoc counterpart of the recording `save` does on a clean tree, for
/// saves minted on a dirty one (adoption: a bootstrap saves before its
/// first commit). The guarantee is the same as at save time: the tree must
/// be clean and its render must hash to the version being anchored, so the
/// recorded commit really contains the sources the render came from.
/// Provenance already on a version is a birth fact and is never rewritten.
pub fn anchor(root: &Path, model: &Model) -> Result<Anchored, String> {
    let mut archive = Archive::open(root)?
        .filter(|a| !a.entries.is_empty())
        .ok_or("no versions saved: nothing to anchor")?;
    let live = hash(&model.render_source());
    let Some(pos) = archive.entries.iter().rposition(|e| e.model == live) else {
        let latest = &archive.entries.last().expect("non-empty").id;
        return Err(format!(
            "the live model matches no saved version (dirty since {latest}): \
             `archi version save` first, commit, then anchor"
        ));
    };
    let entry = &mut archive.entries[pos];
    let id = entry.id.clone();
    if let Some(existing) = &entry.commit {
        return Ok(Anchored::Already {
            id,
            commit: existing.clone(),
        });
    }
    let sha = clean_head(root)?;
    entry.commit = Some(sha.clone());
    archive.write_manifest()?;
    Ok(Anchored::Recorded { id, commit: sha })
}

/// Record a member's baseline post hoc on the version the live model is
/// at — the member-side counterpart of [`anchor`], for members dirty or
/// unreachable at save time. The guarantee is the clean-tree half alone —
/// the strength code provenance ever had — and the baseline is marked
/// anchor-born so the audit words its window honestly. Marks on older
/// versions are history and never move; the latest version's mark follows
/// the member — a clean checkout standing on a different tip re-records
/// it, anchor-born, so a landing never inherits a stale window
/// (`archi/requirements/worktree-parallelism/a-landing-carries-fresh-baselines`).
pub fn anchor_member(root: &Path, model: &Model, member: &str) -> Result<Anchored, String> {
    let mut archive = Archive::open(root)?
        .filter(|a| !a.entries.is_empty())
        .ok_or("no versions saved: nothing to anchor")?;
    let live = hash(&model.render_source());
    let Some(pos) = archive.entries.iter().rposition(|e| e.model == live) else {
        let latest = &archive.entries.last().expect("non-empty").id;
        return Err(format!(
            "the live model matches no saved version (dirty since {latest}): \
             `archi version save` first, then anchor"
        ));
    };
    let set = crate::members::MemberSet::resolve(root)?;
    let Some(m) = set.get(member).filter(|m| m.name != crate::members::HOME) else {
        return Err(format!(
            "`{member}` is not a declared member — add its [[repo]] row to archi.toml; \
             the home anchor is the bare `archi version anchor`"
        ));
    };
    let Some(mroot) = &m.root else {
        return Err(format!(
            "`{member}` is unreachable here: `archi repo map {member} <dir>` first"
        ));
    };
    let is_latest = pos + 1 == archive.entries.len();
    let entry = &mut archive.entries[pos];
    let id = entry.id.clone();
    let existing = entry.commits.get(member).map(|b| b.sha.clone());
    if let Some(prev) = &existing {
        // a mark on an older version is history and never moves; the
        // latest version's mark is judged against the checkout below
        if !is_latest {
            return Ok(Anchored::Already { id, commit: prev.clone() });
        }
    }
    let Some(ctx) = crate::members::GitContext::of(mroot) else {
        return match existing {
            // no readable tip to compare against — the mark stands
            Some(prev) => Ok(Anchored::Already { id, commit: prev }),
            None => Err(format!(
                "`{member}` is not inside a git work tree: nothing to anchor to"
            )),
        };
    };
    if let Some(prev) = &existing {
        match ctx.head() {
            // the checkout stands where the mark says — today's answer
            Some(head) if head == *prev => {
                return Ok(Anchored::Already { id, commit: prev.clone() });
            }
            // no tip to compare — the mark stands
            None => return Ok(Anchored::Already { id, commit: prev.clone() }),
            // a moved tip: the clean gate and the re-record below
            Some(_) => {}
        }
    }
    match ctx.clean() {
        Some(true) => {}
        Some(false) => {
            return Err(format!(
                "`{member}` is dirty: commit it first, so the anchored baseline really \
                 contains the member's code"
            ));
        }
        None => return Err(format!("cannot tell whether `{member}` is clean: `git status` failed")),
    }
    let sha = ctx
        .head()
        .ok_or_else(|| format!("`{member}` has no commits yet: nothing to anchor to"))?;
    entry.commits.insert(
        member.to_string(),
        Baseline {
            sha: sha.clone(),
            born: Born::Anchor,
        },
    );
    archive.write_manifest()?;
    Ok(match existing {
        Some(was) => Anchored::Reanchored { id, commit: sha, was },
        None => Anchored::Recorded { id, commit: sha },
    })
}

fn id_of(index: usize) -> String {
    format!("v{:04}", index + 1)
}

fn seal_broken(id: &str, actual: &str, sealed: &str) -> String {
    format!(
        "`{id}` reconstructs to {actual} but the manifest seals it at {sealed}: \
         the archive was edited; restore it from git history"
    )
}

fn apply_patch(base: &str, patch_text: &str) -> Result<String, String> {
    let patch =
        diffy::Patch::from_str(patch_text).map_err(|e| format!("patch does not parse: {e}"))?;
    diffy::apply(base, &patch).map_err(|e| format!("patch does not apply: {e}"))
}

fn hash(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    let mut out = String::with_capacity(7 + 64);
    out.push_str("sha256:");
    for b in digest {
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// The commit a save happened on — recorded only when git is available and
/// the project's working tree is clean, so the commit really contains the
/// sources the render came from. Code-link birth records share the policy
/// (`archi/requirements/self-hosting/link-truth-is-append-only.md`).
pub(crate) fn provenance(root: &Path) -> Option<String> {
    clean_head(root).ok()
}

/// HEAD's sha when the project's working tree is clean — the provenance
/// precondition, checked silently by `save` and loudly by `anchor`.
fn clean_head(root: &Path) -> Result<String, String> {
    let git = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
    };
    let head = git(&["rev-parse", "HEAD"]).ok_or(
        "no commit to anchor to: git is unavailable, the project is not in a \
         git repository, or the repository has no commits yet",
    )?;
    let sha = String::from_utf8(head.stdout)
        .map_err(|_| "git printed a non-utf8 HEAD sha".to_string())?
        .trim()
        .to_string();
    let status = git(&["status", "--porcelain", "--", "."])
        .ok_or("cannot tell whether the working tree is clean: `git status` failed")?;
    if !status.stdout.is_empty() {
        return Err(
            "the working tree is dirty: commit first, so the anchored commit really \
             contains the model's sources"
                .to_string(),
        );
    }
    Ok(sha)
}

/// Civil-from-days (Howard Hinnant's algorithm): epoch seconds to ISO-8601
/// UTC without a clock dependency.
pub(crate) fn iso8601_utc(t: SystemTime) -> String {
    let secs = t
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (days, rem) = (secs / 86_400, secs % 86_400);
    let (h, min, s) = (rem / 3_600, rem % 3_600 / 60, rem % 60);
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{min:02}:{s:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    /// A throwaway project directory; callers overwrite `archi/src/` to evolve it.
    fn temp_project(model: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-versions-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(dir.join("archi/src")).unwrap();
        fs::write(dir.join("archi.toml"), "[project]\nname = \"t\"\n").unwrap();
        write_model(&dir, model);
        dir
    }

    fn write_model(root: &Path, text: &str) {
        fs::write(root.join("archi/src").join("model.arch"), text).unwrap();
    }

    fn compiled_model(root: &Path) -> modeling_lang::Workspace {
        modeling_lang::source::compile_project(root)
            .unwrap_or_else(|f| panic!("test model failed to compile:\n{}", f.render()))
            .workspace
    }

    fn save_ok(root: &Path, note: &str) -> (String, Kind) {
        let ws = compiled_model(root);
        match save(root, ws.model(), note).unwrap() {
            Saved::Written { id, kind, .. } => (id, kind),
            Saved::Unchanged { latest } => panic!("expected a save, model unchanged at {latest}"),
        }
    }

    /// Big enough that a one-line change patches smaller than the render —
    /// on a handful of lines the keyframe policy correctly always keyframes.
    fn v1() -> String {
        let mut s = String::from("def conn link := * -> *\n");
        for i in 0..8 {
            s.push_str(&format!("def node S{i}:\n  port out\n  port inn\n"));
        }
        for i in 0..7 {
            s.push_str(&format!("S{i}.out link S{}.inn\n", i + 1));
        }
        s.push_str("def node A:\n  port p\ndef node B:\n  port q\nA.p link B.q\n");
        s
    }

    /// Internals-only change: a child inside A; A's ports and edges stand.
    fn v2() -> String {
        format!("{}def node A.Inner\n", v1())
    }

    #[test]
    fn first_save_keyframes_then_patches_and_reconstructs() {
        let root = temp_project(&v1());
        let render1 = compiled_model(&root).model().render_source();

        let (id1, kind1) = save_ok(&root, "first");
        assert_eq!((id1.as_str(), kind1), ("v0001", Kind::Full));

        // Unchanged model: nothing to mint.
        let ws = compiled_model(&root);
        assert!(matches!(
            save(&root, ws.model(), "again").unwrap(),
            Saved::Unchanged { latest } if latest == "v0001"
        ));

        write_model(&root, &v2());
        let render2 = compiled_model(&root).model().render_source();
        let (id2, kind2) = save_ok(&root, "inner detail");
        assert_eq!((id2.as_str(), kind2), ("v0002", Kind::Patch));

        let archive = Archive::open(&root).unwrap().unwrap();
        assert_eq!(archive.reconstruct("v0001").unwrap(), render1);
        assert_eq!(archive.reconstruct("v0002").unwrap(), render2);
        assert_eq!(archive.verify(), Vec::<String>::new());

        // Current is derived by hash: at v0002 now, dirty after an edit.
        let ws = compiled_model(&root);
        assert!(matches!(current(&root, ws.model()).unwrap(), Current::At(id) if id == "v0002"));
        write_model(&root, &format!("{}def node C\n", v2()));
        let ws = compiled_model(&root);
        assert!(matches!(
            current(&root, ws.model()).unwrap(),
            Current::DirtySince(id) if id == "v0002"
        ));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn tampering_breaks_the_seal() {
        let root = temp_project(&v1());
        save_ok(&root, "first");
        write_model(&root, &v2());
        save_ok(&root, "second");

        let patch = Archive::dir_of(&root).join("v0002.arch.patch");
        let doctored = fs::read_to_string(&patch)
            .unwrap()
            .replace("A.Inner", "A.Doctored");
        fs::write(&patch, doctored).unwrap();

        let archive = Archive::open(&root).unwrap().unwrap();
        let errors = archive.verify();
        assert!(
            errors
                .iter()
                .any(|e| e.contains("v0002") && e.contains("seals it at")),
            "{errors:?}"
        );
        assert!(archive.reconstruct("v0002").is_err());
        // The keyframe before the break stays verifiable.
        assert!(archive.reconstruct("v0001").is_ok());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_rewrite_outgrows_its_patch_and_keyframes() {
        let root = temp_project(&v1());
        save_ok(&root, "first");
        // A disjoint model: the patch would replace every line, exceeding
        // the render — the policy writes a keyframe instead.
        write_model(
            &root,
            "def node X:\n  port a\ndef node Y:\n  port b\ndef conn wire := * -> *\nX.a wire Y.b\n",
        );
        let (id, kind) = save_ok(&root, "rewrite");
        assert_eq!((id.as_str(), kind), ("v0002", Kind::Full));
        assert_eq!(
            Archive::open(&root).unwrap().unwrap().verify(),
            Vec::<String>::new()
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn stray_files_are_flagged() {
        let root = temp_project(&v1());
        save_ok(&root, "first");
        fs::write(Archive::dir_of(&root).join("v0002.arch"), "def node Fake\n").unwrap();
        let errors = Archive::open(&root).unwrap().unwrap().verify();
        assert!(
            errors.iter().any(|e| e.contains("v0002.arch")),
            "{errors:?}"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn scope_hashes_split_interface_from_internals() {
        let root = temp_project(&v1());
        save_ok(&root, "first");
        write_model(&root, &v2());
        save_ok(&root, "inner detail");

        let archive = Archive::open(&root).unwrap().unwrap();
        let [e1, e2] = archive.entries() else {
            panic!("two entries");
        };
        let (a1, a2) = (&e1.scopes["A"], &e2.scopes["A"]);
        // The internals-only change moved A's full hash and left its
        // interface hash — the vertical-versioning policy split.
        assert_ne!(a1.full, a2.full);
        assert_eq!(a1.interface, a2.interface);
        // B saw no change at all.
        assert_eq!(e1.scopes["B"], e2.scopes["B"]);

        fs::remove_dir_all(&root).unwrap();
    }

    fn git(root: &Path, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
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

    fn head(root: &Path) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    /// A member checkout: a sibling git repo seeded with one commit.
    fn member_repo(root: &Path, name: &str, dirty: bool) -> PathBuf {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        fs::write(dir.join("code.rs"), "fn seed() {}\n").unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "seed"]);
        if dirty {
            fs::write(dir.join("wip.rs"), "fn wip() {}\n").unwrap();
        }
        dir
    }

    fn declare_members(root: &Path, decls: &str) {
        fs::write(
            root.join("archi.toml"),
            format!("[project]\nname = \"t\"\n{decls}"),
        )
        .unwrap();
    }

    #[test]
    fn save_baselines_clean_members_and_names_omissions() {
        let root = temp_project(&v1());
        member_repo(&root, "backend", false);
        member_repo(&root, "web", true);
        declare_members(
            &root,
            "[[repo]]\nname = \"backend\"\npath = \"backend\"\n\n[[repo]]\nname = \"web\"\npath = \"web\"\n\n[[repo]]\nname = \"gone\"\npath = \"nowhere\"\n",
        );
        let ws = compiled_model(&root);
        let Saved::Written { baseline_notes, .. } = save(&root, ws.model(), "with members").unwrap()
        else {
            panic!("expected a save")
        };
        let archive = Archive::open(&root).unwrap().unwrap();
        let entry = &archive.entries()[0];
        // Exactly the clean member is baselined, save-born.
        assert_eq!(entry.commits.len(), 1, "{:?}", entry.commits);
        assert_eq!(entry.commits["backend"].born, Born::Save);
        // The omissions are named, each with its recovery.
        assert!(baseline_notes.iter().any(|n| n.contains("baseline backend")));
        assert!(
            baseline_notes
                .iter()
                .any(|n| n.contains("`web`") && n.contains("dirty")),
            "{baseline_notes:?}"
        );
        assert!(
            baseline_notes
                .iter()
                .any(|n| n.contains("`gone`") && n.contains("unreachable")),
            "{baseline_notes:?}"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn anchor_member_records_post_hoc_and_the_same_tip_stands() {
        let root = temp_project(&v1());
        let backend = member_repo(&root, "backend", true);
        declare_members(&root, "[[repo]]\nname = \"backend\"\npath = \"backend\"\n");
        let ws = compiled_model(&root);
        save(&root, ws.model(), "dirty member").unwrap();
        let archive = Archive::open(&root).unwrap().unwrap();
        assert!(archive.entries()[0].commits.is_empty(), "dirty at save: no baseline");

        // Undeclared and home names refuse.
        assert!(
            anchor_member(&root, ws.model(), "ghost")
                .unwrap_err()
                .contains("not a declared member")
        );
        // Dirty member refuses; committed it records, anchor-born.
        assert!(anchor_member(&root, ws.model(), "backend").unwrap_err().contains("dirty"));
        git(&backend, &["add", "-A"]);
        git(&backend, &["commit", "-q", "-m", "wip lands"]);
        let sha = head(&backend);
        match anchor_member(&root, ws.model(), "backend").unwrap() {
            Anchored::Recorded { id, commit } => {
                assert_eq!((id.as_str(), commit), ("v0001", sha.clone()))
            }
            other => panic!("first member anchor must record: {other:?}"),
        }
        let archive = Archive::open(&root).unwrap().unwrap();
        let b = &archive.entries()[0].commits["backend"];
        assert_eq!((b.sha.as_str(), b.born), (sha.as_str(), Born::Anchor));
        // The checkout stands where the mark says: nothing to re-record.
        assert!(matches!(
            anchor_member(&root, ws.model(), "backend").unwrap(),
            Anchored::Already { .. }
        ));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_moved_member_re_anchors_the_latest_version_only() {
        let root = temp_project(&v1());
        let backend = member_repo(&root, "backend", false);
        declare_members(&root, "[[repo]]\nname = \"backend\"\npath = \"backend\"\n");
        let ws = compiled_model(&root);
        save(&root, ws.model(), "first").unwrap();
        let b0 = head(&backend);
        write_model(&root, &v2());
        let ws2 = compiled_model(&root);
        save(&root, ws2.model(), "second").unwrap();

        // A dirty tree at the recorded tip keeps today's answer: HEAD is
        // where the mark says, nothing to re-record.
        fs::write(backend.join("more.rs"), "fn more() {}\n").unwrap();
        assert!(matches!(
            anchor_member(&root, ws2.model(), "backend").unwrap(),
            Anchored::Already { id, commit } if id == "v0002" && commit == b0
        ));
        // A moved AND dirty member refuses: the re-record carries the same
        // clean-tree guarantee as any anchor.
        git(&backend, &["add", "-A"]);
        git(&backend, &["commit", "-q", "-m", "moved"]);
        let b1 = head(&backend);
        fs::write(backend.join("wip.rs"), "fn wip() {}\n").unwrap();
        assert!(
            anchor_member(&root, ws2.model(), "backend")
                .unwrap_err()
                .contains("dirty")
        );
        fs::remove_file(backend.join("wip.rs")).unwrap();

        // A clean checkout on a different tip re-records the latest mark,
        // anchor-born; the report carries where the mark stood.
        match anchor_member(&root, ws2.model(), "backend").unwrap() {
            Anchored::Reanchored { id, commit, was } => {
                assert_eq!(
                    (id.as_str(), commit.as_str(), was.as_str()),
                    ("v0002", b1.as_str(), b0.as_str())
                );
            }
            other => panic!("a moved clean member re-records: {other:?}"),
        }
        let archive = Archive::open(&root).unwrap().unwrap();
        let moved = &archive.entries()[1].commits["backend"];
        assert_eq!((moved.sha.as_str(), moved.born), (b1.as_str(), Born::Anchor));
        // Older history stays untouched, save-born as recorded.
        let old = &archive.entries()[0].commits["backend"];
        assert_eq!((old.sha.as_str(), old.born), (b0.as_str(), Born::Save));
        // Baselines live in the manifest alone: the seal survives the move.
        assert_eq!(archive.verify(), Vec::<String>::new());
        // The same tip keeps today's Already answer.
        assert!(matches!(
            anchor_member(&root, ws2.model(), "backend").unwrap(),
            Anchored::Already { id, commit } if id == "v0002" && commit == b1
        ));
        // A live model matching an older version: its mark is history — the
        // command reports it and never moves it.
        write_model(&root, &v1());
        let ws1 = compiled_model(&root);
        assert!(matches!(
            anchor_member(&root, ws1.model(), "backend").unwrap(),
            Anchored::Already { id, commit } if id == "v0001" && commit == b0
        ));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn baselines_stay_outside_identity_and_memberless_entries_stay_todays() {
        // Two identical models, one memberless, one membered: same hash,
        // and the memberless entry serializes without a commits table.
        let plain = temp_project(&v1());
        let ws = compiled_model(&plain);
        save(&plain, ws.model(), "plain").unwrap();
        let manifest = fs::read_to_string(Archive::dir_of(&plain).join("index.toml")).unwrap();
        assert!(!manifest.contains("commits"), "no empty table serialized");
        let plain_hash = Archive::open(&plain).unwrap().unwrap().entries()[0].model.clone();

        let membered = temp_project(&v1());
        member_repo(&membered, "backend", false);
        declare_members(&membered, "[[repo]]\nname = \"backend\"\npath = \"backend\"\n");
        let ws = compiled_model(&membered);
        save(&membered, ws.model(), "membered").unwrap();
        let archive = Archive::open(&membered).unwrap().unwrap();
        assert_eq!(archive.entries()[0].model, plain_hash, "identity is the render alone");
        assert!(!archive.entries()[0].commits.is_empty());
        // The membered no-op save still mints nothing.
        assert!(matches!(
            save(&membered, ws.model(), "again").unwrap(),
            Saved::Unchanged { .. }
        ));

        fs::remove_dir_all(&plain).unwrap();
        fs::remove_dir_all(&membered).unwrap();
    }

    /// The adoption flow of `issues/audit-blind-without-clean-tree-provenance`:
    /// a bootstrap saves on a dirty (here: not even versioned) tree, commits,
    /// then anchors the save post hoc.
    #[test]
    fn anchor_records_provenance_post_hoc() {
        let root = temp_project(&v1());
        save_ok(&root, "bootstrap");
        let ws = compiled_model(&root);

        // No repository yet: nothing to anchor to.
        assert!(anchor(&root, ws.model()).unwrap_err().contains("no commit"));

        git(&root, &["init", "-q"]);
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-q", "-m", "bootstrap"]);

        // An untracked file keeps the tree dirty: anchor refuses.
        fs::write(root.join("junk.txt"), "x").unwrap();
        assert!(anchor(&root, ws.model()).unwrap_err().contains("dirty"));
        fs::remove_file(root.join("junk.txt")).unwrap();

        // Clean tree whose render matches v0001: provenance lands, and a
        // second anchor is a no-op — even though recording it just dirtied
        // the manifest.
        let first = head(&root);
        match anchor(&root, ws.model()).unwrap() {
            Anchored::Recorded { id, commit } => {
                assert_eq!((id.as_str(), commit), ("v0001", first.clone()))
            }
            other => panic!("first anchor must record: {other:?}"),
        }
        let archive = Archive::open(&root).unwrap().unwrap();
        assert_eq!(archive.entries()[0].commit.as_deref(), Some(first.as_str()));
        assert!(matches!(
            anchor(&root, ws.model()).unwrap(),
            Anchored::Already { id, .. } if id == "v0001"
        ));

        // A live model matching no saved version cannot anchor.
        write_model(&root, &v2());
        let ws = compiled_model(&root);
        assert!(
            anchor(&root, ws.model())
                .unwrap_err()
                .contains("matches no saved version")
        );

        // Recorded provenance is a birth fact: after HEAD moves on, anchor
        // still reports the recorded commit rather than rewriting it.
        save_ok(&root, "second");
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-q", "-m", "second"]);
        let second = head(&root);
        assert!(matches!(
            anchor(&root, ws.model()).unwrap(),
            Anchored::Recorded { id, commit } if id == "v0002" && commit == second
        ));
        fs::write(root.join("later.txt"), "y").unwrap();
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-q", "-m", "later"]);
        assert!(matches!(
            anchor(&root, ws.model()).unwrap(),
            Anchored::Already { id, commit } if id == "v0002" && commit == second
        ));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn timestamps_render_civil_utc() {
        let at = |secs: u64| iso8601_utc(UNIX_EPOCH + Duration::from_secs(secs));
        assert_eq!(at(0), "1970-01-01T00:00:00Z");
        assert_eq!(at(86_400), "1970-01-02T00:00:00Z");
        assert_eq!(at(1_600_000_000), "2020-09-13T12:26:40Z");
        assert_eq!(at(1_751_846_400), "2025-07-07T00:00:00Z");
    }
}
