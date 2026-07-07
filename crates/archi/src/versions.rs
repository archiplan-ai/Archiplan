//! The version archive: `archi/versions/` under the project root
//! (`requirements/versioning.md`).
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
/// is a manifest scan (`requirements/versioning.md#versioning--scopes`).
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
    /// Root-scope hashes, keyed by root node name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub scopes: BTreeMap<String, ScopeHashes>,
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

/// A loaded version archive.
pub struct Archive {
    dir: PathBuf,
    entries: Vec<Entry>,
}

const GITATTRIBUTES: &str = "# Keyframes are generated renders; patches are the readable change record.\n\
     v*.arch linguist-generated\n";

const INDEX_HEADER: &str = "# Version archive: append-only, sealed by the hashes below.\n\
     # See requirements/versioning.md; verified on `archi check`.\n\n";

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
        let manifest: ManifestFile = toml::from_str(&text)
            .map_err(|e| format!("`{}` does not parse: {e}", index.display()))?;
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
        fs::read_to_string(&p).map_err(|err| format!("cannot read `{}`: {err}", p.display()))
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
    let entry = Entry {
        id: id_of(archive.entries.len()),
        note: note.to_string(),
        created: iso8601_utc(SystemTime::now()),
        model: model_hash,
        parent: archive.entries.last().map(|e| e.id.clone()),
        kind,
        preset: model.preset_name().to_string(),
        commit: provenance(root),
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
    })
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
/// (`requirements/code-link.md#stored-as-files`).
pub(crate) fn provenance(root: &Path) -> Option<String> {
    let git = |args: &[&str]| {
        Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
    };
    let head = git(&["rev-parse", "HEAD"])?;
    let sha = String::from_utf8(head.stdout).ok()?.trim().to_string();
    let status = git(&["status", "--porcelain", "--", "."])?;
    status.stdout.is_empty().then_some(sha)
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

    /// A throwaway project directory; callers overwrite `src/` to evolve it.
    fn temp_project(model: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-versions-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("archi.toml"), "[project]\nname = \"t\"\n").unwrap();
        write_model(&dir, model);
        dir
    }

    fn write_model(root: &Path, text: &str) {
        fs::write(root.join("src").join("model.arch"), text).unwrap();
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

    #[test]
    fn timestamps_render_civil_utc() {
        let at = |secs: u64| iso8601_utc(UNIX_EPOCH + Duration::from_secs(secs));
        assert_eq!(at(0), "1970-01-01T00:00:00Z");
        assert_eq!(at(86_400), "1970-01-02T00:00:00Z");
        assert_eq!(at(1_600_000_000), "2020-09-13T12:26:40Z");
        assert_eq!(at(1_751_846_400), "2025-07-07T00:00:00Z");
    }
}
