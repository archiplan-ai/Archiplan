//! Doc sources: intents, requirements, stress sessions and stressors
//! (`requirements/requirements.md`, `requirements/stressing.md`,
//! `requirements/intent.md`) — structured markdown under
//! `archi/requirements/` and `archi/stress/`, compiled and integrity-checked
//! against the model on every `archi check`.
//!
//! Errors carry the shared doc catalog (`E_DOC`, `E_SLUG`, `E_DOC_REF`,
//! `E_MODEL_REF`, `E_PLACEMENT`, plus `E_AFFECTS_EMPTY` and `E_SESSION`) as
//! `file:line:col`-located diagnostics; advisory states — open requirements,
//! deferrals, unanswered breaking stressors — are findings, never blocking.
//! Stressor affects validate against the *pinned* version of their session,
//! reconstructed from the archive; `satisfied-by` validates against the live
//! model.

mod md;
mod schema;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use modeling_lang::source::{compile_sources, project_preset};
use modeling_lang::{Model, Preset, Workspace};
use serde::Serialize;

use crate::versions;
use md::FieldValue;
use schema::{Intent, Origin, Outcome, Requirement, Session, Stressor};

/// A doc-source compile error, located like a model compile diagnostic.
#[derive(Serialize)]
pub struct DocDiagnostic {
    /// Stable code of the shared doc catalog.
    pub code: &'static str,
    /// Human-readable one-liner.
    pub message: String,
    /// Project-relative path of the offending file.
    pub file: String,
    /// 1-based line.
    pub line: usize,
    /// Secondary location ("first defined here", …).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<DocNote>,
}

/// A diagnostic's secondary location.
#[derive(Serialize)]
pub struct DocNote {
    /// What the location is.
    pub message: String,
    /// Project-relative path.
    pub file: String,
    /// 1-based line.
    pub line: usize,
}

impl DocDiagnostic {
    fn new(code: &'static str, message: impl Into<String>, file: &str, line: usize) -> Self {
        DocDiagnostic {
            code,
            message: message.into(),
            file: file.to_string(),
            line,
            note: None,
        }
    }

    fn with_note(mut self, message: impl Into<String>, file: &str, line: usize) -> Self {
        self.note = Some(DocNote {
            message: message.into(),
            file: file.to_string(),
            line,
        });
        self
    }
}

impl fmt::Display for DocDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:1: {}: {}",
            self.file, self.line, self.code, self.message
        )?;
        if let Some(n) = &self.note {
            write!(f, "\n  {}:{}: {}", n.file, n.line, n.message)?;
        }
        Ok(())
    }
}

/// An advisory doc-layer finding (`requirements/requirements.md#compile`,
/// `requirements/stressing.md#compile`). Kinds are append-only.
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DocFinding {
    /// No satisfaction record, not deferred, no satisfied ancestor.
    UnsatisfiedRequirement {
        /// The requirement's slug.
        requirement: String,
    },
    /// A deferral in force, with its reason.
    DeferredRequirement {
        /// The requirement's slug.
        requirement: String,
        /// Why it is out of the current architecture's scope.
        reason: String,
    },
    /// A satisfaction record with no verification entries.
    UnverifiedSatisfaction {
        /// The requirement's slug.
        requirement: String,
    },
    /// A closed session holds a stressor with no outcome.
    PendingStressor {
        /// The stressor's slug.
        stressor: String,
        /// Its session's slug.
        session: String,
    },
    /// A breaking stressor no requirement records as its origin.
    BreakingUnanswered {
        /// The stressor's slug.
        stressor: String,
        /// Its session's slug.
        session: String,
    },
    /// A session with no stressors.
    EmptySession {
        /// The session's slug.
        session: String,
    },
}

impl fmt::Display for DocFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocFinding::UnsatisfiedRequirement { requirement } => {
                write!(f, "unsatisfied requirement: {requirement}")
            }
            DocFinding::DeferredRequirement {
                requirement,
                reason,
            } => write!(f, "deferred requirement: {requirement} — {reason}"),
            DocFinding::UnverifiedSatisfaction { requirement } => {
                write!(f, "satisfaction without verification: {requirement}")
            }
            DocFinding::PendingStressor { stressor, session } => {
                write!(
                    f,
                    "pending stressor in closed session `{session}`: {stressor}"
                )
            }
            DocFinding::BreakingUnanswered { stressor, .. } => write!(
                f,
                "breaking stressor unanswered: {stressor} — no requirement records it as origin"
            ),
            DocFinding::EmptySession { session } => write!(f, "empty session: {session}"),
        }
    }
}

/// The outcome of compiling the doc sources.
pub struct DocReport {
    /// Errors — the sources are ill-formed or contradict the model.
    pub diagnostics: Vec<DocDiagnostic>,
    /// Advisory findings.
    pub findings: Vec<DocFinding>,
}

/// Everything the doc trees hold, parsed best-effort.
#[derive(Default)]
struct Tree {
    intents: Vec<Intent>,
    requirements: Vec<Requirement>,
    sessions: Vec<Session>,
    stressors: Vec<Stressor>,
}

/// Compile and cross-check the doc sources of a project against its
/// compiled model.
pub fn check(root: &Path, model: &Model) -> DocReport {
    let mut diags = Vec::new();
    let tree = discover(root, &mut diags);
    let findings = cross_check(root, model, &tree, &mut diags);
    diags.sort_by(|a, b| (a.file.as_str(), a.line).cmp(&(b.file.as_str(), b.line)));
    DocReport {
        diagnostics: diags,
        findings,
    }
}

// ---- discovery -------------------------------------------------------------

fn sorted_entries(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut v: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
    v.sort();
    v
}

fn is_md(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "md")
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn read_doc(
    root: &Path,
    path: &Path,
    diags: &mut Vec<DocDiagnostic>,
) -> Option<(String, md::MdDoc)> {
    let file = rel(root, path);
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                format!("cannot read the file: {e}"),
                &file,
                1,
            ));
            return None;
        }
    };
    match md::parse(&text) {
        Ok(doc) => Some((file, doc)),
        Err(e) => {
            diags.push(DocDiagnostic::new("E_DOC", e.message, &file, e.line));
            None
        }
    }
}

fn discover(root: &Path, diags: &mut Vec<DocDiagnostic>) -> Tree {
    let mut tree = Tree::default();

    let base = root.join("archi").join("requirements");
    for path in sorted_entries(&base) {
        if path.is_dir() {
            let islug = file_name(&path);
            let anchor = path.join(format!("{islug}.md"));
            if anchor.is_file() {
                if let Some((file, doc)) = read_doc(root, &anchor, diags) {
                    tree.intents
                        .push(schema::intent(&doc, &file, &islug, diags));
                }
            } else {
                diags.push(DocDiagnostic::new(
                    "E_PLACEMENT",
                    format!(
                        "intent folder `{islug}/` has no `{islug}.md` — the intent anchors its area"
                    ),
                    &rel(root, &path),
                    1,
                ));
            }
            walk_requirements(root, &path, &islug, None, true, &mut tree, diags);
        } else if is_md(&path) {
            diags.push(DocDiagnostic::new(
                "E_PLACEMENT",
                format!(
                    "`{}` sits outside any intent folder — requirements live under archi/requirements/<intent>/",
                    file_name(&path)
                ),
                &rel(root, &path),
                1,
            ));
        }
    }

    let base = root.join("archi").join("stress");
    for path in sorted_entries(&base) {
        if path.is_dir() {
            let sslug = file_name(&path);
            let anchor = path.join(format!("{sslug}.md"));
            if anchor.is_file() {
                if let Some((file, doc)) = read_doc(root, &anchor, diags) {
                    tree.sessions
                        .push(schema::session(&doc, &file, &sslug, diags));
                }
            } else {
                diags.push(DocDiagnostic::new(
                    "E_PLACEMENT",
                    format!("session folder `{sslug}/` has no `{sslug}.md` — the session anchors its round"),
                    &rel(root, &path),
                    1,
                ));
            }
            for inner in sorted_entries(&path) {
                if inner.is_dir() {
                    diags.push(DocDiagnostic::new(
                        "E_PLACEMENT",
                        "sessions are flat — a session folder holds stressor files only",
                        &rel(root, &inner),
                        1,
                    ));
                } else if is_md(&inner)
                    && stem(&inner) != sslug
                    && let Some((file, doc)) = read_doc(root, &inner, diags)
                {
                    tree.stressors.push(schema::stressor(
                        &doc,
                        &file,
                        &stem(&inner),
                        &sslug,
                        diags,
                    ));
                }
            }
        } else if is_md(&path) {
            diags.push(DocDiagnostic::new(
                "E_PLACEMENT",
                format!(
                    "`{}` sits outside any session folder — stressors live under archi/stress/<session>/",
                    file_name(&path)
                ),
                &rel(root, &path),
                1,
            ));
        }
    }

    tree
}

/// Walk one requirement folder: files are requirements, folders are epics —
/// containment is the hierarchy.
fn walk_requirements(
    root: &Path,
    dir: &Path,
    folder_slug: &str,
    parent: Option<&str>,
    at_intent_root: bool,
    tree: &mut Tree,
    diags: &mut Vec<DocDiagnostic>,
) {
    for path in sorted_entries(dir) {
        if path.is_dir() {
            let eslug = file_name(&path);
            let anchor = path.join(format!("{eslug}.md"));
            if anchor.is_file() {
                if let Some((file, doc)) = read_doc(root, &anchor, diags) {
                    let (req, sections) = schema::requirement_file(
                        &doc,
                        &file,
                        &eslug,
                        parent.map(str::to_string),
                        at_intent_root,
                        diags,
                    );
                    tree.requirements.push(req);
                    tree.requirements.extend(sections);
                }
            } else {
                diags.push(DocDiagnostic::new(
                    "E_PLACEMENT",
                    format!("requirement folder `{eslug}/` has no `{eslug}.md` — the folder is a promoted requirement"),
                    &rel(root, &path),
                    1,
                ));
            }
            walk_requirements(root, &path, &eslug, Some(&eslug), false, tree, diags);
        } else if is_md(&path)
            && stem(&path) != folder_slug
            && let Some((file, doc)) = read_doc(root, &path, diags)
        {
            let (req, sections) = schema::requirement_file(
                &doc,
                &file,
                &stem(&path),
                parent.map(str::to_string),
                at_intent_root,
                diags,
            );
            tree.requirements.push(req);
            tree.requirements.extend(sections);
        }
    }
}

// ---- cross-checks ----------------------------------------------------------

fn cross_check(
    root: &Path,
    model: &Model,
    tree: &Tree,
    diags: &mut Vec<DocDiagnostic>,
) -> Vec<DocFinding> {
    // Slugs are the reference currency: unique project-wide across
    // archiplan primitives (requirements/requirements.md#slugs).
    let mut seen: BTreeMap<&str, (&str, usize, &'static str)> = BTreeMap::new();
    let everything = tree
        .intents
        .iter()
        .map(|i| (i.slug.as_str(), i.file.as_str(), i.line, "intent"))
        .chain(
            tree.requirements
                .iter()
                .map(|r| (r.slug.as_str(), r.file.as_str(), r.line, "requirement")),
        )
        .chain(
            tree.sessions
                .iter()
                .map(|s| (s.slug.as_str(), s.file.as_str(), s.line, "session")),
        )
        .chain(
            tree.stressors
                .iter()
                .map(|s| (s.slug.as_str(), s.file.as_str(), s.line, "stressor")),
        );
    for (slug, file, line, what) in everything {
        if slug.is_empty() {
            diags.push(DocDiagnostic::new(
                "E_SLUG",
                "the name derives to an empty slug",
                file,
                line,
            ));
            continue;
        }
        match seen.get(slug) {
            Some((f0, l0, w0)) => diags.push(
                DocDiagnostic::new(
                    "E_SLUG",
                    format!("slug `{slug}` collides with the {w0} of the same name"),
                    file,
                    line,
                )
                .with_note("first defined here", f0, *l0),
            ),
            None => {
                seen.insert(slug, (file, line, what));
            }
        }
    }

    let req_slugs: BTreeSet<&str> = tree.requirements.iter().map(|r| r.slug.as_str()).collect();
    let stressor_slugs: BTreeSet<&str> = tree.stressors.iter().map(|s| s.slug.as_str()).collect();

    // Origin placement and references; satisfied-by against the live model.
    for r in &tree.requirements {
        let Some(f) = &r.fields else { continue };
        if let Some((origin, line)) = &f.origin {
            match origin {
                Origin::Intent if !r.at_intent_root => diags.push(DocDiagnostic::new(
                    "E_PLACEMENT",
                    "`origin: intent` is legal only at the root of an intent folder",
                    &r.file,
                    *line,
                )),
                Origin::Parent if r.parent.is_none() => diags.push(DocDiagnostic::new(
                    "E_PLACEMENT",
                    "`origin: parent` needs a parent requirement — at an intent folder's root the origin is the intent",
                    &r.file,
                    *line,
                )),
                Origin::Stressors(slugs) => {
                    for s in slugs {
                        if !stressor_slugs.contains(s.as_str()) {
                            diags.push(DocDiagnostic::new(
                                "E_DOC_REF",
                                format!("origin names no stressor `{s}`"),
                                &r.file,
                                *line,
                            ));
                        }
                    }
                }
                Origin::Fusion(slugs) => {
                    for s in slugs {
                        if s == &r.slug {
                            diags.push(DocDiagnostic::new(
                                "E_DOC_REF",
                                "a requirement cannot fuse from itself",
                                &r.file,
                                *line,
                            ));
                        } else if !req_slugs.contains(s.as_str()) {
                            diags.push(DocDiagnostic::new(
                                "E_DOC_REF",
                                format!("origin fuses from no requirement `{s}`"),
                                &r.file,
                                *line,
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some((entries, line)) = &f.satisfied_by {
            for p in entries {
                if !model.has_node(p) {
                    diags.push(DocDiagnostic::new(
                        "E_MODEL_REF",
                        format!("satisfied-by names no element `{p}` of the current model"),
                        &r.file,
                        *line,
                    ));
                }
            }
        }
    }

    // Sessions: pinned versions exist; at most one session is open. An
    // unreadable archive is E_ARCHIVE territory, reported elsewhere.
    let archive = versions::Archive::open(root).ok().flatten();
    let ids: BTreeSet<&str> = archive
        .iter()
        .flat_map(|a| a.entries())
        .map(|e| e.id.as_str())
        .collect();
    for s in &tree.sessions {
        if let Some((v, line)) = &s.version {
            if v.is_empty() {
                diags.push(DocDiagnostic::new(
                    "E_SESSION",
                    "the session pins no version — `version` names the version it presses on",
                    &s.file,
                    *line,
                ));
            } else if !ids.contains(v.as_str()) {
                diags.push(DocDiagnostic::new(
                    "E_SESSION",
                    format!("`{v}` names no archived version"),
                    &s.file,
                    *line,
                ));
            }
        }
        if let Some((c, line)) = &s.closed
            && !c.is_empty()
            && !ids.contains(c.as_str())
        {
            diags.push(DocDiagnostic::new(
                "E_SESSION",
                format!("`{c}` names no archived version"),
                &s.file,
                *line,
            ));
        }
    }
    let open: Vec<&Session> = tree.sessions.iter().filter(|s| s.open()).collect();
    for later in open.iter().skip(1) {
        let first = open[0];
        diags.push(
            DocDiagnostic::new(
                "E_SESSION",
                format!(
                    "sessions `{}` and `{}` are both open — at most one stress session is open",
                    first.slug, later.slug
                ),
                &later.file,
                later.line,
            )
            .with_note("the other open session", &first.file, first.line),
        );
    }

    // Affects of open sessions validate against their pinned version,
    // reconstructed and compiled (requirements/stressing.md#compile).
    if let Some(archive) = &archive {
        for s in open {
            let Some((v, line)) = &s.version else {
                continue;
            };
            if !ids.contains(v.as_str()) {
                continue; // already reported
            }
            match compile_version(root, archive, v) {
                Err(e) => diags.push(DocDiagnostic::new(
                    "E_SESSION",
                    format!(
                        "version `{v}` of session `{}` cannot be validated: {e}",
                        s.slug
                    ),
                    &s.file,
                    *line,
                )),
                Ok(ws) => {
                    for st in tree.stressors.iter().filter(|st| st.session == s.slug) {
                        let Some((entries, aline)) = &st.affects else {
                            continue;
                        };
                        for p in entries {
                            if !ws.model().has_node(p) {
                                diags.push(DocDiagnostic::new(
                                    "E_MODEL_REF",
                                    format!("affects names no element `{p}` of version `{v}`"),
                                    &st.file,
                                    *aline,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    findings(tree)
}

/// Reconstruct an archived version and compile it against the preset it was
/// saved with.
fn compile_version(
    root: &Path,
    archive: &versions::Archive,
    id: &str,
) -> Result<Workspace, String> {
    let text = archive.reconstruct(id)?;
    let entry = archive
        .entries()
        .iter()
        .find(|e| e.id == id)
        .ok_or_else(|| format!("no version `{id}` in the archive"))?;
    let name = entry.preset.as_str();
    let preset = project_preset(root)
        .ok()
        .filter(|p| p.name() == name)
        .or_else(|| match name {
            "core" => Some(Preset::core()),
            "default" => Some(Preset::default_ontology()),
            _ => None,
        })
        .ok_or_else(|| format!("preset `{name}` is not available"))?;
    compile_sources(&preset, &[("model", &text)])
        .map(|c| c.workspace)
        .map_err(|f| match f.diagnostics.first() {
            Some(d) => format!("its render does not compile: {d}"),
            None => "its render does not compile".to_string(),
        })
}

fn findings(tree: &Tree) -> Vec<DocFinding> {
    let mut out = Vec::new();
    let by_slug: BTreeMap<&str, &Requirement> = tree
        .requirements
        .iter()
        .map(|r| (r.slug.as_str(), r))
        .collect();
    // Walk the parent chain — containment is the hierarchy, so chains are
    // finite; a bound guards the degenerate case of colliding slugs.
    let ancestor = |r: &Requirement, test: &dyn Fn(&Requirement) -> bool| -> bool {
        let mut cur = r.parent.as_deref();
        for _ in 0..1_000 {
            let Some(p) = cur.and_then(|p| by_slug.get(p)) else {
                return false;
            };
            if test(p) {
                return true;
            }
            cur = p.parent.as_deref();
        }
        false
    };
    let satisfied = |r: &Requirement| r.fields.as_ref().is_some_and(|f| f.satisfied());
    let deferred = |r: &Requirement| r.fields.as_ref().is_some_and(|f| f.deferred());

    for r in &tree.requirements {
        let Some(f) = &r.fields else { continue };
        // Unparsed fields were already errors; findings need sound state.
        let (Some(_), Some(_)) = (&f.satisfied_by, &f.deferred) else {
            continue;
        };
        if f.satisfied() {
            if f.verifications == 0 {
                out.push(DocFinding::UnverifiedSatisfaction {
                    requirement: r.slug.clone(),
                });
            }
        } else if ancestor(r, &satisfied) {
            // Transitivity: nothing to report under a satisfied ancestor.
        } else if f.deferred() {
            out.push(DocFinding::DeferredRequirement {
                requirement: r.slug.clone(),
                reason: f.deferred.clone().unwrap_or_default(),
            });
        } else if ancestor(r, &deferred) {
            // The deferral of an ancestor puts the subtree out of scope.
        } else {
            out.push(DocFinding::UnsatisfiedRequirement {
                requirement: r.slug.clone(),
            });
        }
    }

    let closed: BTreeSet<&str> = tree
        .sessions
        .iter()
        .filter(|s| s.closed.as_ref().is_some_and(|(v, _)| !v.is_empty()))
        .map(|s| s.slug.as_str())
        .collect();
    let answered: BTreeSet<&str> = tree
        .requirements
        .iter()
        .filter_map(|r| r.fields.as_ref())
        .filter_map(|f| f.origin.as_ref())
        .filter_map(|(o, _)| match o {
            Origin::Stressors(slugs) => Some(slugs.iter().map(String::as_str)),
            _ => None,
        })
        .flatten()
        .collect();
    for st in &tree.stressors {
        let Some(outcome) = st.outcome else { continue };
        if closed.contains(st.session.as_str()) && outcome == Outcome::Pending {
            out.push(DocFinding::PendingStressor {
                stressor: st.slug.clone(),
                session: st.session.clone(),
            });
        }
        if outcome == Outcome::Breaking && !answered.contains(st.slug.as_str()) {
            out.push(DocFinding::BreakingUnanswered {
                stressor: st.slug.clone(),
                session: st.session.clone(),
            });
        }
    }
    for s in &tree.sessions {
        if !tree.stressors.iter().any(|st| st.session == s.slug) {
            out.push(DocFinding::EmptySession {
                session: s.slug.clone(),
            });
        }
    }
    out
}

// ---- session closing on version save ---------------------------------------

/// Stamp the open session's `closed:` field with the just-minted version id
/// — `archi version save` closes the active stress session
/// (`requirements/versioning.md#versioning--stressing`). Returns the closed
/// session's slug, or `None` when no session was open.
pub fn close_open_session(root: &Path, version_id: &str) -> Result<Option<String>, String> {
    let base = root.join("archi").join("stress");
    let mut open: Vec<(String, PathBuf)> = Vec::new();
    for dir in sorted_entries(&base) {
        if !dir.is_dir() {
            continue;
        }
        let slug = file_name(&dir);
        let anchor = dir.join(format!("{slug}.md"));
        if !anchor.is_file() {
            continue; // a placement error, reported by `check`
        }
        let text = fs::read_to_string(&anchor)
            .map_err(|e| format!("cannot read `{}`: {e}", rel(root, &anchor)))?;
        let doc = md::parse(&text).map_err(|e| {
            format!(
                "session `{slug}` does not parse ({}:{}: {})",
                rel(root, &anchor),
                e.line,
                e.message
            )
        })?;
        let is_open = doc
            .frontmatter
            .as_deref()
            .and_then(|fm| fm.iter().find(|f| f.key == "closed"))
            .is_some_and(|f| matches!(&f.value, FieldValue::Scalar(s) if s.is_empty()));
        if is_open {
            open.push((slug, anchor));
        }
    }
    match open.as_mut_slice() {
        [] => Ok(None),
        [(slug, path)] => {
            stamp_closed(path, version_id)?;
            Ok(Some(std::mem::take(slug)))
        }
        more => Err(format!(
            "sessions {} are all open — at most one stress session is open",
            more.iter()
                .map(|(s, _)| format!("`{s}`"))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

fn stamp_closed(path: &Path, id: &str) -> Result<(), String> {
    let text =
        fs::read_to_string(path).map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
    let mut out = String::with_capacity(text.len() + id.len() + 1);
    let mut in_frontmatter = false;
    for (i, line) in text.lines().enumerate() {
        if line.trim_end() == "---" {
            in_frontmatter = i == 0;
            out.push_str(line);
        } else if in_frontmatter
            && line
                .split_once(':')
                .is_some_and(|(k, _)| k.trim() == "closed")
        {
            out.push_str("closed: ");
            out.push_str(id);
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    fs::write(path, out).map_err(|e| format!("cannot write `{}`: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    const MODEL: &str = "def node AuthService:\n  port handle_login\ndef node CredStore\nService type_of AuthService\n";

    fn temp_project() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-docs-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(
            dir.join("archi.toml"),
            "[project]\nname = \"t\"\npreset = \"default\"\n",
        )
        .unwrap();
        fs::write(dir.join("src").join("model.arch"), MODEL).unwrap();
        dir
    }

    fn put(root: &Path, rel_path: &str, text: &str) {
        let path = root.join(rel_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn compiled(root: &Path) -> Workspace {
        modeling_lang::source::compile_project(root)
            .unwrap_or_else(|f| panic!("test model failed to compile:\n{}", f.render()))
            .workspace
    }

    fn save_version(root: &Path, note: &str) -> String {
        let ws = compiled(root);
        match versions::save(root, ws.model(), note).unwrap() {
            versions::Saved::Written { id, .. } => id,
            versions::Saved::Unchanged { latest } => panic!("unchanged at {latest}"),
        }
    }

    fn check_at(root: &Path) -> DocReport {
        let ws = compiled(root);
        check(root, ws.model())
    }

    fn codes(report: &DocReport) -> Vec<&'static str> {
        report.diagnostics.iter().map(|d| d.code).collect()
    }

    fn rendered_findings(report: &DocReport) -> Vec<String> {
        let mut v: Vec<String> = report.findings.iter().map(|f| f.to_string()).collect();
        v.sort();
        v
    }

    const INTENT: &str = "# Secure auth\n\nPassword authentication that leaks nothing.\n";

    fn requirement(origin: &str, satisfied_by: &str, deferred: &str, satisfy: &str) -> String {
        format!(
            "---\nkind: non-functional\norigin: {origin}\nsatisfied-by: [{satisfied_by}]\ndeferred: {deferred}\n---\n\n# NAME\n\nSummary paragraph.\n\n## System Context\n\n## Satisfy\n{satisfy}\n"
        )
    }

    fn named(template: &str, name: &str) -> String {
        template.replace("NAME", name)
    }

    /// The worked tree: an intent, a verified requirement with an inline
    /// subrequirement, a satisfied-but-unverified epic with an open child
    /// (suppressed by transitivity), a deferred requirement, an open
    /// stressor-derived requirement answering the breaking stressor of the
    /// open session pinned to v0001.
    fn full_tree(root: &Path) -> String {
        let v1 = save_version(root, "first");
        put(
            root,
            "archi/requirements/secure-auth/secure-auth.md",
            INTENT,
        );
        put(
            root,
            "archi/requirements/secure-auth/no-plaintext-credentials.md",
            &named(
                &requirement(
                    "intent",
                    "CredStore",
                    "",
                    "\n`CredStore` holds only salted hashes.\n\n- test — register, then scan the store for the raw credential\n\n## Applies to backups\n\nBackups hold the same bytes.\n",
                ),
                "No plaintext credentials",
            ),
        );
        put(
            root,
            "archi/requirements/secure-auth/session-revocation/session-revocation.md",
            &named(
                &requirement(
                    "intent",
                    "AuthService",
                    "",
                    "\nRevocation rides the login path.\n",
                ),
                "Session revocation",
            ),
        );
        put(
            root,
            "archi/requirements/secure-auth/session-revocation/revoke-on-breach.md",
            &named(&requirement("parent", "", "", ""), "Revoke on breach"),
        );
        put(
            root,
            "archi/requirements/secure-auth/token-rotation.md",
            &named(
                &requirement("intent", "", "postponed to the v2 key hierarchy", ""),
                "Token rotation",
            ),
        );
        put(
            root,
            "archi/requirements/secure-auth/rate-limit-logins.md",
            &named(
                &requirement("stressor(credential-stuffing)", "", "", ""),
                "Rate limit logins",
            ),
        );
        put(
            root,
            "archi/stress/auth-hardening/auth-hardening.md",
            &format!(
                "---\nversion: {v1}\nclosed:\n---\n\n# Auth hardening\n\nFirst adversarial round.\n"
            ),
        );
        put(
            root,
            "archi/stress/auth-hardening/credential-stuffing.md",
            "---\naffects: [AuthService, Service]\noutcome: breaking\n---\n\n# Credential stuffing\n\nBots replay leaked pairs at 100x the organic rate.\n\n## Attractor\n\nThe login path saturates on hash verification.\n\n## Resolution\n\nRate limiting takes the burst off the hot path: derived rate-limit-logins.\n",
        );
        v1
    }

    #[test]
    fn the_worked_tree_checks_out() {
        let root = temp_project();
        full_tree(&root);
        let report = check_at(&root);
        assert_eq!(
            report
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            Vec::<String>::new()
        );
        // Open work is findings, never errors; transitivity suppresses the
        // children of the satisfied epic and the deferral names its reason.
        assert_eq!(
            rendered_findings(&report),
            [
                "deferred requirement: token-rotation — postponed to the v2 key hierarchy",
                "satisfaction without verification: session-revocation",
                "unsatisfied requirement: rate-limit-logins",
            ]
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn affects_pin_to_the_sessions_version_while_satisfied_by_tracks_the_live_model() {
        let root = temp_project();
        full_tree(&root);
        // The live model grows a node the pinned v0001 does not have.
        fs::write(
            root.join("src").join("model.arch"),
            format!("{MODEL}def node RateLimiter\n"),
        )
        .unwrap();
        // Satisfaction may claim it: satisfied-by reads the live model.
        put(
            root.join("archi/requirements/secure-auth").as_path(),
            "rate-limit-logins.md",
            &named(
                &requirement(
                    "stressor(credential-stuffing)",
                    "RateLimiter",
                    "",
                    "\n`RateLimiter` sheds the burst before hashing.\n\n- test — replay a leaked-pair burst, organic logins stay under 100ms\n",
                ),
                "Rate limit logins",
            ),
        );
        // A stressor may not: affects read the pinned version.
        put(
            &root,
            "archi/stress/auth-hardening/limiter-bypass.md",
            "---\naffects: [RateLimiter]\noutcome: pending\n---\n\n# Limiter bypass\n\nDistributed bots stay under the per-ip threshold.\n\n## Attractor\n\nThe limiter sees no single hot key.\n\n## Resolution\n",
        );
        let report = check_at(&root);
        let model_refs: Vec<&DocDiagnostic> = report
            .diagnostics
            .iter()
            .filter(|d| d.code == "E_MODEL_REF")
            .collect();
        assert_eq!(model_refs.len(), 1, "{:?}", codes(&report));
        assert!(model_refs[0].message.contains("RateLimiter"));
        assert!(model_refs[0].message.contains("v0001"));
        assert!(model_refs[0].file.contains("limiter-bypass"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn schema_violations_are_e_doc() {
        let root = temp_project();
        save_version(&root, "first");
        put(&root, "archi/requirements/a/a.md", "# A\n\nArea.\n");
        // Missing `deferred`, satisfaction half-present, misordered sections.
        put(
            &root,
            "archi/requirements/a/half.md",
            "---\nkind: functional\norigin: intent\nsatisfied-by: [CredStore]\n---\n\n# Half\n\nSummary.\n\n## System Context\n\n## Satisfy\n",
        );
        put(
            &root,
            "archi/requirements/a/misordered.md",
            &named(&requirement("intent", "", "", ""), "Misordered").replace(
                "## System Context\n\n## Satisfy",
                "## Satisfy\n\n## System Context",
            ),
        );
        // An intent with frontmatter.
        put(
            &root,
            "archi/requirements/b/b.md",
            "---\nkind: functional\n---\n# B\n\nArea.\n",
        );
        // A decided outcome with an empty Resolution.
        put(
            &root,
            "archi/stress/s/s.md",
            "---\nversion: v0001\nclosed:\n---\n\n# S\n\nRound.\n",
        );
        put(
            &root,
            "archi/stress/s/undecided.md",
            "---\naffects: [AuthService]\noutcome: surviving\n---\n\n# Undecided\n\nPress.\n\n## Attractor\n\n## Resolution\n",
        );
        let report = check_at(&root);
        // half.md errs twice: the missing `deferred` field and the
        // half-present satisfaction record.
        let e_doc = codes(&report).iter().filter(|c| **c == "E_DOC").count();
        assert_eq!(
            e_doc,
            5,
            "{:#?}",
            report
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn slugs_and_references_hold_project_wide() {
        let root = temp_project();
        save_version(&root, "first");
        put(&root, "archi/requirements/a/a.md", "# A\n\nArea.\n");
        // The H1 does not derive to the filename.
        put(
            &root,
            "archi/requirements/a/wrong.md",
            &named(&requirement("intent", "", "", ""), "Right"),
        );
        // A requirement colliding with a stressor's slug.
        put(
            &root,
            "archi/requirements/a/clash.md",
            &named(&requirement("intent", "", "", ""), "Clash"),
        );
        // Origins referencing nothing.
        put(
            &root,
            "archi/requirements/a/dangling.md",
            &named(&requirement("stressor(ghost)", "", "", ""), "Dangling"),
        );
        // satisfied-by referencing no live element.
        put(
            &root,
            "archi/requirements/a/phantom.md",
            &named(
                &requirement("intent", "Phantom.Node", "", "\nProse.\n"),
                "Phantom",
            ),
        );
        put(
            &root,
            "archi/stress/s/s.md",
            "---\nversion: v0001\nclosed:\n---\n\n# S\n\nRound.\n",
        );
        put(
            &root,
            "archi/stress/s/clash.md",
            "---\naffects: [AuthService]\noutcome: pending\n---\n\n# Clash\n\nPress.\n\n## Attractor\n\n## Resolution\n",
        );
        let report = check_at(&root);
        let by_code = |c: &str| codes(&report).iter().filter(|x| **x == c).count();
        assert_eq!(
            by_code("E_SLUG"),
            2,
            "{:#?}",
            report
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        assert_eq!(by_code("E_DOC_REF"), 1);
        assert_eq!(by_code("E_MODEL_REF"), 1);
        let collision = report
            .diagnostics
            .iter()
            .find(|d| d.message.contains("collides"))
            .unwrap();
        assert!(collision.note.is_some());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn placement_is_meaning() {
        let root = temp_project();
        save_version(&root, "first");
        put(
            &root,
            "archi/requirements/loose.md",
            &named(&requirement("intent", "", "", ""), "Loose"),
        );
        put(&root, "archi/requirements/a/a.md", "# A\n\nArea.\n");
        // `origin: parent` at the intent root; `origin: intent` inside an epic.
        put(
            &root,
            "archi/requirements/a/orphan.md",
            &named(&requirement("parent", "", "", ""), "Orphan"),
        );
        put(
            &root,
            "archi/requirements/a/epic/epic.md",
            &named(&requirement("intent", "", "", ""), "Epic"),
        );
        put(
            &root,
            "archi/requirements/a/epic/rooted.md",
            &named(&requirement("intent", "", "", ""), "Rooted"),
        );
        // A folder without its anchor; a loose stressor.
        fs::create_dir_all(root.join("archi/requirements/a/hollow")).unwrap();
        put(
            &root,
            "archi/stress/loose-stress.md",
            "---\naffects: [AuthService]\noutcome: pending\n---\n\n# Loose stress\n\nPress.\n\n## Attractor\n\n## Resolution\n",
        );
        let report = check_at(&root);
        assert_eq!(
            codes(&report)
                .iter()
                .filter(|c| **c == "E_PLACEMENT")
                .count(),
            5,
            "{:#?}",
            report
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn session_discipline() {
        let root = temp_project();
        save_version(&root, "first");
        // Two open sessions, one pinning an unknown version; a closed
        // session with a pending stressor; an empty affects list.
        put(
            &root,
            "archi/stress/one/one.md",
            "---\nversion: v0001\nclosed:\n---\n\n# One\n\nRound.\n",
        );
        put(
            &root,
            "archi/stress/two/two.md",
            "---\nversion: v9999\nclosed:\n---\n\n# Two\n\nRound.\n",
        );
        put(
            &root,
            "archi/stress/one/hollow.md",
            "---\naffects:\noutcome: pending\n---\n\n# Hollow\n\nPress.\n\n## Attractor\n\n## Resolution\n",
        );
        fs::write(
            root.join("src/model.arch"),
            format!("{MODEL}def node Extra\n"),
        )
        .unwrap();
        let v2 = save_version(&root, "second");
        put(
            &root,
            "archi/stress/done/done.md",
            &format!("---\nversion: v0001\nclosed: {v2}\n---\n\n# Done\n\nClosed round.\n"),
        );
        put(
            &root,
            "archi/stress/done/unfinished.md",
            "---\naffects: [AuthService]\noutcome: pending\n---\n\n# Unfinished\n\nPress.\n\n## Attractor\n\n## Resolution\n",
        );
        let report = check_at(&root);
        let by_code = |c: &str| codes(&report).iter().filter(|x| **x == c).count();
        assert_eq!(
            by_code("E_SESSION"),
            2,
            "{:#?}",
            report
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        assert_eq!(by_code("E_AFFECTS_EMPTY"), 1);
        let findings = rendered_findings(&report);
        assert!(
            findings
                .iter()
                .any(|f| f.contains("pending stressor in closed session `done`")),
            "{findings:?}"
        );
        assert!(
            findings.iter().any(|f| f == "empty session: two"),
            "{findings:?}"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_save_stamps_the_open_session_closed() {
        let root = temp_project();
        full_tree(&root);
        fs::write(
            root.join("src/model.arch"),
            format!("{MODEL}def node RateLimiter\n"),
        )
        .unwrap();
        let v2 = save_version(&root, "answer the round");
        assert_eq!(
            close_open_session(&root, &v2).unwrap().as_deref(),
            Some("auth-hardening")
        );
        let text =
            fs::read_to_string(root.join("archi/stress/auth-hardening/auth-hardening.md")).unwrap();
        assert!(text.contains(&format!("closed: {v2}")), "{text}");
        // The session is closed now: nothing further to stamp, and the tree
        // still checks out — the closed session validates structurally.
        assert_eq!(close_open_session(&root, "v0003").unwrap(), None);
        let report = check_at(&root);
        assert!(
            report.diagnostics.is_empty(),
            "{:#?}",
            report
                .diagnostics
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        fs::remove_dir_all(&root).unwrap();
    }
}
