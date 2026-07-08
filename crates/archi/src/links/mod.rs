//! Code-links: spec ↔ code traceability (`requirements/code-link.md`).
//!
//! A link ties a **`SpecRef`** — a node path or a typed edge, at a version
//! slot — to an **anchor** in the code tree: a file, optionally a symbol.
//! It carries two layers with opposite mutability: the immutable **birth
//! record** (the spans that realized the spec element, content-pinned) and
//! the **projection** (where that code lives now — anchor plus the
//! interface/body hash pair), recomputed by `verify` and rewritten only by
//! an explicit `repin`.
//!
//! Storage is an append-only journal, `archi/links/journal.jsonl` — events
//! `add`, `confirm`, `repin`, `retire`; the live link set is its fold. A
//! commit sha in a birth record is provenance, never a dependency, exactly
//! as in the version archive (`requirements/versioning.md#why-this-shape`).

pub(crate) mod capture;
pub(crate) mod code;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use modeling_lang::{Definition, Model, Statement, Workspace};
use serde::{Deserialize, Serialize};

use crate::docs;
use crate::versions::{self, Archive};

// ---- the link model --------------------------------------------------------

/// Which hash the link watches (`requirements/code-link.md#kinds-and-standing`).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkKind {
    /// The exact body realizes the spec element — body-hash drift is signal.
    Literal,
    /// The symbol's role realizes it — only interface-hash drift is signal.
    Indirect,
}

impl LinkKind {
    /// Parse the CLI spelling.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "literal" => Some(LinkKind::Literal),
            "indirect" => Some(LinkKind::Indirect),
            _ => None,
        }
    }

    fn describe(self) -> &'static str {
        match self {
            LinkKind::Literal => "literal",
            LinkKind::Indirect => "indirect",
        }
    }
}

/// What the link may do: asserted links gate, evidence links inform.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Standing {
    /// A claim: participates in gates, verifies strictly.
    Asserted,
    /// Accreted by capture; never fails a verify, retires when decayed.
    Evidence,
}

impl Standing {
    fn describe(self) -> &'static str {
        match self {
            Standing::Asserted => "asserted",
            Standing::Evidence => "evidence",
        }
    }
}

/// Provenance, orthogonal to standing.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Origin {
    /// Minted by `archi link add` — asserted by construction.
    Authored,
    /// Minted by task-close capture — lands as evidence.
    Captured {
        /// The task whose delta produced it.
        task: String,
    },
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Origin::Authored => write!(f, "authored"),
            Origin::Captured { task } => write!(f, "captured({task})"),
        }
    }
}

/// A spec element reference: a node path (`AuthService.Storage`) or a typed
/// edge in its canonical surface form (`A.p link B.q`), optionally pinned
/// to a version slot (`@v0003`; absent = Working, the live tree).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SpecRef {
    /// The element: a dot path, or canonical edge text (contains spaces).
    #[serde(rename = "ref")]
    pub path: String,
    /// Pinned version slot; `None` is Working.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl SpecRef {
    /// Parse `<element>[@vNNNN]`. Names never contain `@`, so the split is
    /// unambiguous.
    pub fn parse(text: &str) -> Result<SpecRef, String> {
        let text = text.trim();
        if text.is_empty() {
            return Err("the spec ref is empty".into());
        }
        match text.rsplit_once('@') {
            None => Ok(SpecRef {
                path: normalize_ref(text),
                version: None,
            }),
            Some((path, slot)) => {
                if path.is_empty() || slot.is_empty() {
                    return Err(format!("`{text}` is not `<element>[@version]`"));
                }
                Ok(SpecRef {
                    path: normalize_ref(path),
                    version: Some(slot.to_string()),
                })
            }
        }
    }

    fn is_edge(&self) -> bool {
        self.path.contains(' ')
    }
}

impl fmt::Display for SpecRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.version {
            None => write!(f, "{}", self.path),
            Some(v) => write!(f, "{}@{v}", self.path),
        }
    }
}

/// Where the code lives: a project-relative file, optionally a symbol path
/// inside it (`crates/auth/src/store.rs#Store::persist`).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Anchor {
    /// Project-relative path, `/`-separated.
    pub file: String,
    /// `::`-joined item path; `None` anchors the whole file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

impl Anchor {
    /// Parse `<file>[#<symbol>]`.
    pub fn parse(text: &str) -> Result<Anchor, String> {
        let text = text.trim();
        if text.is_empty() {
            return Err("the code ref is empty".into());
        }
        let (file, symbol) = match text.split_once('#') {
            None => (text, None),
            Some((f, s)) if !f.is_empty() && !s.is_empty() => (f, Some(s.to_string())),
            Some(_) => return Err(format!("`{text}` is not `<file>[#<symbol>]`")),
        };
        Ok(Anchor {
            file: file.replace('\\', "/"),
            symbol,
        })
    }
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.symbol {
            None => write!(f, "{}", self.file),
            Some(s) => write!(f, "{}#{s}", self.file),
        }
    }
}

/// One span of the birth record: the lines that were born, content-pinned
/// by their raw bytes.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Span {
    /// Project-relative file the span was born in.
    pub file: String,
    /// 1-based inclusive line range at birth.
    pub start: usize,
    /// End line, inclusive.
    pub end: usize,
    /// `sha256:` of the raw span bytes — the bytes actually born.
    pub hash: String,
}

/// The immutable provenance fact: what was written, when, under what.
/// Never rewritten — `repin` touches only the projection.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Birth {
    /// ISO-8601 UTC timestamp of the mint.
    pub created: String,
    /// Commit provenance — recorded only on a clean tree, never depended on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// The content-pinned spans.
    pub spans: Vec<Span>,
}

/// The projection's hash pair, under a pinned canonicalizer.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Pins {
    /// The canonicalizer that produced the hashes (`rust-tok-v1`, `text-v1`).
    pub canonicalizer: String,
    /// Hash of the anchored item's signature (its declared shape).
    pub interface: String,
    /// Hash of the whole anchored item's canonical tokens.
    pub body: String,
}

/// One code-link, as journaled and as folded.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Link {
    /// Dense id, `l0001` onward. Two branches minting the same id collide
    /// as an ordinary git merge conflict in the journal — surfaced, not
    /// silent, like colliding version ids.
    pub id: String,
    /// The spec element this code realizes.
    pub spec: SpecRef,
    /// Where the code lives now.
    pub anchor: Anchor,
    /// Which hash is watched.
    pub kind: LinkKind,
    /// Asserted or evidence.
    pub standing: Standing,
    /// Where the link came from.
    pub origin: Origin,
    /// The immutable birth record.
    pub birth: Birth,
    /// The projection's hashes.
    pub pins: Pins,
    /// Tasks whose captures re-encountered this link — evidence confidence
    /// accrues. Folded from `touch` events, one entry per task.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub touches: Vec<String>,
    /// Tasks whose captures saw the anchored item change without carrying
    /// the spec_ref — confidence decays. Folded from `decay` events.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decays: Vec<String>,
}

// ---- the journal -----------------------------------------------------------

/// One journal event. The journal is append-only; the live link set is the
/// fold of its events in order.
#[derive(Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "lowercase")]
enum Event {
    Add {
        link: Link,
    },
    Confirm {
        id: String,
        at: String,
    },
    Repin {
        id: String,
        at: String,
        anchor: Anchor,
        pins: Pins,
    },
    Retire {
        id: String,
        at: String,
    },
    Touch {
        id: String,
        task: String,
        at: String,
    },
    Decay {
        id: String,
        task: String,
        at: String,
    },
}

/// The folded journal: live links in add order, plus mint bookkeeping.
struct Folded {
    live: Vec<Link>,
    /// Retired links, folded state at retirement — capture's dedup memory:
    /// a subtracted candidate must stay subtracted across re-runs.
    retired: Vec<Link>,
    /// Adds ever journaled — dense ids mint past retirements.
    adds: usize,
}

impl Folded {
    fn get(&self, id: &str) -> Option<&Link> {
        self.live.iter().find(|l| l.id == id)
    }

    fn next_id(&self) -> String {
        format!("l{:04}", self.adds + 1)
    }
}

fn journal_path(root: &Path) -> PathBuf {
    root.join("archi").join("links").join("journal.jsonl")
}

fn read_journal(root: &Path) -> Result<Vec<Event>, String> {
    let path = journal_path(root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text =
        fs::read_to_string(&path).map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
    let mut events = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event: Event = serde_json::from_str(line).map_err(|e| {
            format!(
                "`{}` line {}: not a journal event: {e}",
                path.display(),
                i + 1
            )
        })?;
        events.push(event);
    }
    Ok(events)
}

fn fold(events: Vec<Event>) -> Result<Folded, String> {
    let mut live: Vec<Link> = Vec::new();
    let mut retired: Vec<Link> = Vec::new();
    let mut adds = 0;
    let corrupt = |id: &str, what: &str| {
        format!("journal corrupt: `{what}` names `{id}`, which is not a live link")
    };
    for event in events {
        match event {
            Event::Add { link } => {
                if live.iter().chain(&retired).any(|l| l.id == link.id) {
                    return Err(format!("journal corrupt: `{}` is added twice", link.id));
                }
                live.push(link);
                adds += 1;
            }
            Event::Confirm { id, .. } => match live.iter_mut().find(|l| l.id == id) {
                Some(l) => l.standing = Standing::Asserted,
                None => return Err(corrupt(&id, "confirm")),
            },
            Event::Repin {
                id, anchor, pins, ..
            } => match live.iter_mut().find(|l| l.id == id) {
                Some(l) => {
                    l.anchor = anchor;
                    l.pins = pins;
                }
                None => return Err(corrupt(&id, "repin")),
            },
            Event::Retire { id, .. } => {
                let Some(at) = live.iter().position(|l| l.id == id) else {
                    return Err(corrupt(&id, "retire"));
                };
                retired.push(live.remove(at));
            }
            Event::Touch { id, task, .. } => match live.iter_mut().find(|l| l.id == id) {
                Some(l) => {
                    if !l.touches.contains(&task) {
                        l.touches.push(task);
                    }
                }
                None => return Err(corrupt(&id, "touch")),
            },
            Event::Decay { id, task, .. } => match live.iter_mut().find(|l| l.id == id) {
                Some(l) => {
                    if !l.decays.contains(&task) {
                        l.decays.push(task);
                    }
                }
                None => return Err(corrupt(&id, "decay")),
            },
        }
    }
    Ok(Folded {
        live,
        retired,
        adds,
    })
}

fn load(root: &Path) -> Result<Folded, String> {
    fold(read_journal(root)?)
}

fn append(root: &Path, events: &[Event]) -> Result<(), String> {
    let path = journal_path(root);
    let dir = path.parent().expect("journal has a directory");
    fs::create_dir_all(dir).map_err(|e| format!("cannot create `{}`: {e}", dir.display()))?;
    let mut out = String::new();
    for e in events {
        out.push_str(&serde_json::to_string(e).map_err(|e| format!("event serializes: {e}"))?);
        out.push('\n');
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut f| f.write_all(out.as_bytes()))
        .map_err(|e| format!("cannot append to `{}`: {e}", path.display()))
}

fn now() -> String {
    versions::iso8601_utc(SystemTime::now())
}

// ---- spec-side resolution --------------------------------------------------

/// Compiled pinned versions, opened lazily: most verifies never touch the
/// archive.
struct Slots<'a> {
    root: &'a Path,
    archive: Option<Option<Archive>>,
    compiled: BTreeMap<String, Workspace>,
}

impl<'a> Slots<'a> {
    fn new(root: &'a Path) -> Self {
        Slots {
            root,
            archive: None,
            compiled: BTreeMap::new(),
        }
    }

    /// Whether the ref resolves in the model of its pinned slot.
    fn resolves_pinned(&mut self, spec: &SpecRef) -> Result<bool, String> {
        let slot = spec.version.as_deref().expect("caller checked the slot");
        if !self.compiled.contains_key(slot) {
            if self.archive.is_none() {
                self.archive = Some(Archive::open(self.root)?);
            }
            let archive = self
                .archive
                .as_ref()
                .expect("just opened")
                .as_ref()
                .ok_or_else(|| format!("`@{slot}`: the project has no version archive"))?;
            let ws = docs::compile_version(self.root, archive, slot)?;
            self.compiled.insert(slot.to_string(), ws);
        }
        let model = self.compiled[slot].model();
        Ok(resolves_in(model, spec))
    }
}

/// Whether a ref names an element of a model: a node by path, an edge by
/// its canonical surface text (views stripped, whitespace collapsed).
pub(crate) fn resolves_in(model: &Model, spec: &SpecRef) -> bool {
    if spec.is_edge() {
        edge_pseudos(model).contains(&spec.path)
    } else {
        model.has_node(&spec.path)
    }
}

pub(crate) fn normalize_ref(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The canonical surface text of an edge statement — views stripped,
/// whitespace collapsed; `None` for non-edges. The one vocabulary edge
/// SpecRefs and task spec_ref seeding speak.
pub(crate) fn edge_pseudo(s: &Statement) -> Option<String> {
    let stripped = match s {
        Statement::RelEdge {
            rel,
            source,
            target,
            ..
        } => Statement::RelEdge {
            rel: rel.clone(),
            source: source.clone(),
            target: target.clone(),
            views: Vec::new(),
        },
        Statement::ConnEdge {
            conn,
            source,
            carrier,
            rev_carrier,
            target,
            ..
        } => Statement::ConnEdge {
            conn: conn.clone(),
            source: source.clone(),
            carrier: carrier.clone(),
            rev_carrier: rev_carrier.clone(),
            target: target.clone(),
            views: Vec::new(),
        },
        _ => return None,
    };
    Some(normalize_ref(&stripped.pseudo()))
}

/// Every typed edge of the model in canonical surface form — the vocabulary
/// edge SpecRefs are checked against. Views are presentation, not identity.
fn edge_pseudos(model: &Model) -> BTreeSet<String> {
    model.dump().iter().filter_map(edge_pseudo).collect()
}

/// Every node path of the model, for audit's coverage sweep. Dumps exclude
/// preset elements, so the sweep sees only user spec.
fn node_paths(model: &Model) -> Vec<String> {
    model
        .dump()
        .into_iter()
        .filter_map(|s| match s {
            Statement::Define(Definition::Node { path, .. }) => Some(path),
            _ => None,
        })
        .collect()
}

// ---- anchoring -------------------------------------------------------------

/// A freshly resolved anchor: its pins and the span it occupies right now.
struct Resolved {
    pins: Pins,
    span: Span,
}

/// Resolve an anchor against the working tree: read, canonicalize, index.
fn resolve_anchor(root: &Path, anchor: &Anchor) -> Result<Resolved, String> {
    let path = root.join(&anchor.file);
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("cannot read `{}`: {e}", anchor.file))?;
    let canonical = code::canonicalize(&anchor.file, &text);
    match &anchor.symbol {
        None => {
            let hash = canonical.file_hash();
            let lines = text.lines().count().max(1);
            Ok(Resolved {
                pins: Pins {
                    canonicalizer: canonical.canonicalizer.to_string(),
                    interface: hash.clone(),
                    body: hash,
                },
                span: Span {
                    file: anchor.file.clone(),
                    start: 1,
                    end: lines,
                    hash: code::hash_bytes(text.as_bytes()),
                },
            })
        }
        Some(symbol) => {
            if canonical.canonicalizer != code::RUST_CANON {
                return Err(format!(
                    "`{}`: symbol anchors need a Rust file ({}); anchor the whole file",
                    anchor.file,
                    code::RUST_CANON
                ));
            }
            let found = canonical.find(symbol);
            let item = match found.as_slice() {
                [] => {
                    return Err(format!(
                        "`{}` names no item `{symbol}`",
                        anchor.file
                    ));
                }
                [one] => one,
                many => {
                    let lines: Vec<String> =
                        many.iter().map(|i| i.start_line.to_string()).collect();
                    return Err(format!(
                        "`{}#{symbol}` is ambiguous: items at lines {}; qualify the symbol",
                        anchor.file,
                        lines.join(", ")
                    ));
                }
            };
            Ok(Resolved {
                pins: Pins {
                    canonicalizer: canonical.canonicalizer.to_string(),
                    interface: item.interface.clone(),
                    body: item.body.clone(),
                },
                span: Span {
                    file: anchor.file.clone(),
                    start: item.start_line,
                    end: item.end_line,
                    hash: code::hash_bytes(line_bytes(&text, item.start_line, item.end_line)),
                },
            })
        }
    }
}

/// The raw bytes of an inclusive 1-based line range.
fn line_bytes(text: &str, start: usize, end: usize) -> &[u8] {
    let mut from = None;
    let mut to = text.len();
    let mut line = 1;
    let mut offset = 0;
    for l in text.split_inclusive('\n') {
        if line == start {
            from = Some(offset);
        }
        offset += l.len();
        if line == end {
            to = offset;
            break;
        }
        line += 1;
    }
    &text.as_bytes()[from.unwrap_or(0)..to]
}

// ---- operations ------------------------------------------------------------

/// `archi link add`: an authored, asserted link — spec ref resolved at its
/// slot, anchor resolved and pinned, birth record minted from the current
/// tree.
pub fn add(
    root: &Path,
    model: &Model,
    spec_text: &str,
    code_text: &str,
    kind: LinkKind,
) -> Result<Link, String> {
    let spec = SpecRef::parse(spec_text)?;
    let resolves = match &spec.version {
        None => resolves_in(model, &spec),
        Some(_) => Slots::new(root).resolves_pinned(&spec)?,
    };
    if !resolves {
        let slot = spec.version.as_deref().unwrap_or("the live model");
        return Err(format!(
            "`{}` names no element of {slot} (E_MODEL_REF)",
            spec.path
        ));
    }
    let anchor = Anchor::parse(code_text)?;
    let resolved = resolve_anchor(root, &anchor)?;
    let folded = load(root)?;
    let link = Link {
        id: folded.next_id(),
        spec,
        anchor,
        kind,
        standing: Standing::Asserted,
        origin: Origin::Authored,
        birth: Birth {
            created: now(),
            commit: versions::provenance(root),
            spans: vec![resolved.span],
        },
        pins: resolved.pins,
        touches: Vec::new(),
        decays: Vec::new(),
    };
    append(root, &[Event::Add { link: link.clone() }])?;
    Ok(link)
}

/// `archi link ls`: the live links, optionally filtered.
pub fn ls(
    root: &Path,
    spec: Option<&str>,
    evidence_only: bool,
) -> Result<Vec<Link>, String> {
    let folded = load(root)?;
    let filter = spec.map(SpecRef::parse).transpose()?;
    Ok(folded
        .live
        .into_iter()
        .filter(|l| {
            filter.as_ref().is_none_or(|f| {
                l.spec.path == f.path
                    && (f.version.is_none() || l.spec.version == f.version)
            })
        })
        .filter(|l| !evidence_only || l.standing == Standing::Evidence)
        .collect())
}

/// `archi link confirm`: raise an evidence link to asserted — a decision,
/// recorded.
pub fn confirm(root: &Path, id: &str) -> Result<Link, String> {
    let folded = load(root)?;
    let link = folded
        .get(id)
        .ok_or_else(|| format!("no live link `{id}`"))?;
    if link.standing == Standing::Asserted {
        return Err(format!("`{id}` is already asserted"));
    }
    append(
        root,
        &[Event::Confirm {
            id: id.to_string(),
            at: now(),
        }],
    )?;
    let mut confirmed = link.clone();
    confirmed.standing = Standing::Asserted;
    Ok(confirmed)
}

/// `archi link rm`: retire links by id.
pub fn retire(root: &Path, ids: &[String]) -> Result<(), String> {
    let folded = load(root)?;
    for id in ids {
        if folded.get(id).is_none() {
            return Err(format!("no live link `{id}`"));
        }
    }
    let at = now();
    let events: Vec<Event> = ids
        .iter()
        .map(|id| Event::Retire {
            id: id.clone(),
            at: at.clone(),
        })
        .collect();
    append(root, &events)
}

/// `archi link rm --spec … --yes`: retire every live link on a spec ref.
pub fn retire_spec(root: &Path, spec: &str) -> Result<Vec<String>, String> {
    let ids: Vec<String> = ls(root, Some(spec), false)?
        .into_iter()
        .map(|l| l.id)
        .collect();
    if ids.is_empty() {
        return Err(format!("no live links on `{spec}`"));
    }
    retire(root, &ids)?;
    Ok(ids)
}

/// `archi link repin`: rewrite the projection — accept drift at the current
/// anchor, or follow a move to a new one. The birth record is untouched.
pub fn repin(root: &Path, id: &str, to: Option<&str>) -> Result<Link, String> {
    let folded = load(root)?;
    let link = folded
        .get(id)
        .ok_or_else(|| format!("no live link `{id}`"))?;
    let anchor = match to {
        None => link.anchor.clone(),
        Some(t) => Anchor::parse(t)?,
    };
    let resolved = resolve_anchor(root, &anchor)?;
    append(
        root,
        &[Event::Repin {
            id: id.to_string(),
            at: now(),
            anchor: anchor.clone(),
            pins: resolved.pins.clone(),
        }],
    )?;
    let mut repinned = link.clone();
    repinned.anchor = anchor;
    repinned.pins = resolved.pins;
    Ok(repinned)
}

// ---- verify ----------------------------------------------------------------

/// A projection's graded state (`requirements/code-link.md#verify-and-drift`).
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum State {
    /// Anchor resolves; the watched hash matches.
    Clean,
    /// Anchor resolves; the watched hash moved.
    Drifted,
    /// Anchor gone; a heuristic candidate exists elsewhere.
    Moved {
        /// Candidate file.
        file: String,
        /// Candidate symbol, for symbol anchors.
        #[serde(skip_serializing_if = "Option::is_none")]
        symbol: Option<String>,
        /// The candidate's body hashes equal to the pinned body — the code
        /// moved verbatim.
        exact: bool,
    },
    /// Nothing resolves.
    Missing,
    /// The stored canonicalizer is unknown to this verifier.
    CanonicalizerMismatch,
    /// The spec side moved: the ref no longer resolves at Working.
    SpecDrifted,
}

impl State {
    fn describe(&self) -> &'static str {
        match self {
            State::Clean => "clean",
            State::Drifted => "drifted",
            State::Moved { .. } => "moved",
            State::Missing => "missing",
            State::CanonicalizerMismatch => "canonicalizer-mismatch",
            State::SpecDrifted => "spec-drifted",
        }
    }
}

/// The floor below which evidence reads as decayed — confirm or retire
/// (`requirements/code-link.md#audit--dark-deltas-dark-spec`).
pub const CONFIDENCE_FLOOR: f64 = 0.25;

/// Derived confidence of an evidence link — never stored. Born at 0.5;
/// each task whose capture re-encountered it accrues, each task that
/// rewrote the anchored item without carrying the spec_ref erodes, any
/// projection drift erodes once, and a dead anchor zeroes.
pub fn confidence(link: &Link, state: &State) -> f64 {
    if matches!(state, State::Missing) {
        return 0.0;
    }
    let drift = if matches!(state, State::Clean) { 0.0 } else { -0.25 };
    let accrued = 0.15 * link.touches.len() as f64;
    let eroded = 0.25 * link.decays.len() as f64;
    (0.5 + accrued - eroded + drift).clamp(0.0, 1.0)
}

/// One verified link.
#[derive(Serialize)]
pub struct Checked {
    /// The link, as folded.
    pub link: Link,
    /// Its graded state.
    #[serde(flatten)]
    pub state: State,
    /// Whether this state fails the verify: asserted links only — evidence
    /// never fails. `Missing`, `CanonicalizerMismatch` and a Working-slot
    /// `SpecDrifted` always fail; `Drifted` fails literal links only.
    pub failing: bool,
    /// Human context: what moved, what held.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// The outcome of `archi link verify`.
#[derive(Serialize)]
pub struct VerifyReport {
    /// Every link checked, in journal order.
    pub checked: Vec<Checked>,
    /// Links skipped by `--since` — their anchor files did not change.
    pub skipped: usize,
}

impl VerifyReport {
    /// Whether any checked link fails the gate.
    pub fn failing(&self) -> bool {
        self.checked.iter().any(|c| c.failing)
    }
}

/// Options of `archi link verify`.
#[derive(Default)]
pub struct VerifyOptions {
    /// Check only links on this spec ref.
    pub spec: Option<String>,
    /// Check only links whose anchor file changed since this git rev.
    pub since: Option<String>,
}

/// Verify the live links: recompute every projection in scope, grade it.
pub fn verify(root: &Path, model: &Model, opts: &VerifyOptions) -> Result<VerifyReport, String> {
    let folded = load(root)?;
    let filter = opts.spec.as_deref().map(SpecRef::parse).transpose()?;
    let changed = opts
        .since
        .as_deref()
        .map(|rev| changed_files(root, rev))
        .transpose()?;
    let mut slots = Slots::new(root);
    let mut checked = Vec::new();
    let mut skipped = 0;
    for link in folded.live {
        if let Some(f) = &filter
            && (link.spec.path != f.path
                || (f.version.is_some() && link.spec.version != f.version))
        {
            continue;
        }
        if let Some(changed) = &changed
            && !changed.contains(&link.anchor.file)
        {
            skipped += 1;
            continue;
        }
        checked.push(check_link(root, model, &mut slots, link)?);
    }
    Ok(VerifyReport { checked, skipped })
}

/// Grade one link: spec side first, then the projection.
fn check_link(
    root: &Path,
    model: &Model,
    slots: &mut Slots,
    link: Link,
) -> Result<Checked, String> {
    // Spec side. A pinned ref resolves by construction — the archive is
    // sealed — so a pinned link reports Working drift as a note, not a
    // state; a Working-slot ref that stopped resolving is SpecDrifted.
    let at_working = resolves_in(model, &link.spec);
    if link.spec.version.is_some() && !slots.resolves_pinned(&link.spec)? {
        let state = State::SpecDrifted;
        return Ok(Checked {
            failing: link.standing == Standing::Asserted,
            note: Some(format!(
                "`{}` is not an element of {} — the journal disagrees with the sealed archive",
                link.spec.path,
                link.spec.version.as_deref().unwrap_or(""),
            )),
            link,
            state,
        });
    }
    if link.spec.version.is_none() && !at_working {
        let state = State::SpecDrifted;
        return Ok(Checked {
            failing: link.standing == Standing::Asserted,
            note: Some(
                "the spec element is gone from the live model; the version chain locates the \
                 rename or removal"
                    .to_string(),
            ),
            link,
            state,
        });
    }
    let working_note = (link.spec.version.is_some() && !at_working)
        .then(|| "spec ref no longer resolves at Working".to_string());

    // The canonicalizer must be known before its hashes mean anything.
    if !code::knows(&link.pins.canonicalizer) {
        let state = State::CanonicalizerMismatch;
        return Ok(Checked {
            failing: link.standing == Standing::Asserted,
            note: Some(format!(
                "stored canonicalizer `{}` is unknown to this verifier; rehash with `link repin`",
                link.pins.canonicalizer
            )),
            link,
            state,
        });
    }

    // The projection.
    let path = root.join(&link.anchor.file);
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => {
            let (state, note) = scan_for_candidate(root, &link);
            return Ok(Checked {
                failing: link.standing == Standing::Asserted
                    && matches!(state, State::Missing),
                note: note.or(working_note),
                link,
                state,
            });
        }
    };
    let canonical = code::canonicalize(&link.anchor.file, &text);
    if canonical.canonicalizer != link.pins.canonicalizer {
        let state = State::CanonicalizerMismatch;
        return Ok(Checked {
            failing: link.standing == Standing::Asserted,
            note: Some(format!(
                "`{}` now canonicalizes as `{}`, pinned under `{}`",
                link.anchor.file, canonical.canonicalizer, link.pins.canonicalizer
            )),
            link,
            state,
        });
    }
    let (interface, body) = match &link.anchor.symbol {
        None => {
            let h = canonical.file_hash();
            (h.clone(), h)
        }
        Some(symbol) => match canonical.find(symbol).as_slice() {
            [] => {
                let (state, note) = scan_for_candidate(root, &link);
                return Ok(Checked {
                    failing: link.standing == Standing::Asserted
                        && matches!(state, State::Missing),
                    note: note.or(working_note),
                    link,
                    state,
                });
            }
            [one] => (one.interface.clone(), one.body.clone()),
            many => {
                let lines: Vec<String> = many.iter().map(|i| i.start_line.to_string()).collect();
                let state = State::Missing;
                return Ok(Checked {
                    failing: link.standing == Standing::Asserted,
                    note: Some(format!(
                        "`{symbol}` is ambiguous in `{}` (lines {}); repin a qualified symbol",
                        link.anchor.file,
                        lines.join(", ")
                    )),
                    link,
                    state,
                });
            }
        },
    };
    let watched_holds = match link.kind {
        LinkKind::Literal => body == link.pins.body,
        LinkKind::Indirect => interface == link.pins.interface,
    };
    let state = if watched_holds {
        State::Clean
    } else {
        State::Drifted
    };
    let note = if watched_holds && body != link.pins.body {
        Some("body moved; the watched interface holds".to_string())
    } else if !watched_holds && link.kind == LinkKind::Indirect && link.anchor.symbol.is_some() {
        Some("the declared shape moved".to_string())
    } else {
        working_note
    };
    Ok(Checked {
        failing: link.standing == Standing::Asserted
            && link.kind == LinkKind::Literal
            && state == State::Drifted,
        note,
        link,
        state,
    })
}

/// The anchor is gone: sweep the tree for a candidate — an item with the
/// same symbol (or a file with the same canonical hash), body-equality
/// ranking exact moves first.
fn scan_for_candidate(root: &Path, link: &Link) -> (State, Option<String>) {
    let extension = Path::new(&link.anchor.file)
        .extension()
        .map(|e| e.to_string_lossy().into_owned());
    let mut candidate: Option<(String, Option<String>, bool)> = None;
    for file in code_files(root) {
        if file == link.anchor.file
            || Path::new(&file).extension().map(|e| e.to_string_lossy().into_owned()) != extension
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(root.join(&file)) else {
            continue;
        };
        let canonical = code::canonicalize(&file, &text);
        match &link.anchor.symbol {
            None => {
                if canonical.file_hash() == link.pins.body {
                    candidate = Some((file, None, true));
                    break;
                }
            }
            Some(symbol) => {
                for item in canonical.find(symbol) {
                    let exact = item.body == link.pins.body;
                    if exact {
                        candidate = Some((file.clone(), Some(item.symbol.clone()), true));
                    } else if candidate.is_none() {
                        candidate = Some((file.clone(), Some(item.symbol.clone()), false));
                    }
                }
                if candidate.as_ref().is_some_and(|c| c.2) {
                    break;
                }
            }
        }
    }
    match candidate {
        Some((file, symbol, exact)) => {
            let note = format!(
                "candidate: `{file}{}`{} — confirm with `link repin <id> --to`",
                symbol.as_deref().map(|s| format!("#{s}")).unwrap_or_default(),
                if exact { " (verbatim move)" } else { "" },
            );
            (
                State::Moved {
                    file,
                    symbol,
                    exact,
                },
                Some(note),
            )
        }
        None => (State::Missing, None),
    }
}

/// The project's scan-exclusion patterns: `[audit] exclude` in
/// `archi.toml`. Read leniently — the manifest parser owns loud
/// validation — and consulted by every tree scan (`code_files`,
/// `delta_hunks`), never by verify or the fold: exclusion governs what
/// the scans volunteer, not what links may claim.
fn scan_exclusions(root: &Path) -> Vec<String> {
    #[derive(serde::Deserialize)]
    struct ScanConfig {
        audit: Option<AuditConfig>,
    }
    #[derive(serde::Deserialize)]
    struct AuditConfig {
        #[serde(default)]
        exclude: Vec<String>,
    }
    fs::read_to_string(root.join("archi.toml"))
        .ok()
        .and_then(|t| toml::from_str::<ScanConfig>(&t).ok())
        .and_then(|c| c.audit)
        .map(|a| a.exclude)
        .unwrap_or_default()
}

/// One root-relative path against the exclusion patterns: a trailing `/`
/// is a directory prefix, a leading `*` a suffix, anything else an exact
/// path.
fn excluded(file: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| {
        if p.ends_with('/') {
            file.starts_with(p.as_str())
        } else if let Some(suffix) = p.strip_prefix('*') {
            file.ends_with(suffix)
        } else {
            file == p
        }
    })
}

/// Every code file of the project, root-relative: the tree minus VCS and
/// build dirs, the `archi/` tree (model, docs, archive, journal),
/// `.arch` sources — code is what the model is *about*, not the model —
/// and whatever the project's `[audit] exclude` patterns mute.
fn code_files(root: &Path) -> Vec<String> {
    let patterns = scan_exclusions(root);
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else {
            continue;
        };
        let mut entries: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
        entries.sort();
        for path in entries {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            if path.is_dir() {
                if !(dir == *root && name == "archi") {
                    stack.push(path);
                }
            } else if !name.ends_with(".arch")
                && name != "archi.toml"
                && fs::metadata(&path).is_ok_and(|m| m.len() < 1_048_576)
            {
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                if !excluded(&rel, &patterns) {
                    out.push(rel);
                }
            }
        }
    }
    out.sort();
    out
}

/// The files git says changed since a rev — the `--since` fast path.
fn changed_files(root: &Path, rev: &str) -> Result<BTreeSet<String>, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["diff", "--name-only", rev, "--", "."])
        .output()
        .map_err(|e| format!("--since needs git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git diff --name-only {rev}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

// ---- audit -----------------------------------------------------------------

/// An advisory audit finding (`requirements/code-link.md#audit--dark-deltas-dark-spec`).
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuditFinding {
    /// A delta hunk no link claims — code motion with no architectural
    /// account.
    UnaccountedDelta {
        /// The changed file.
        file: String,
        /// 1-based new-side line range of the hunk.
        start: usize,
        /// End line, inclusive.
        end: usize,
        /// The enclosing item, when one resolves.
        #[serde(skip_serializing_if = "Option::is_none")]
        symbol: Option<String>,
    },
    /// A spec element in the audited scope with no asserted link.
    UnlinkedSpecRef {
        /// The node path.
        path: String,
    },
    /// An evidence link whose derived confidence fell below the floor.
    DecayedEvidence {
        /// The link id.
        id: String,
        /// Its spec ref.
        spec: String,
        /// Its anchor.
        anchor: String,
        /// The derived confidence, below [`CONFIDENCE_FLOOR`].
        confidence: f64,
    },
}

impl fmt::Display for AuditFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditFinding::UnaccountedDelta {
                file,
                start,
                end,
                symbol,
            } => {
                write!(f, "unaccounted delta: {file}:{start}-{end}")?;
                if let Some(s) = symbol {
                    write!(f, " (in `{s}`)")?;
                }
                write!(f, " — no link claims it")
            }
            AuditFinding::UnlinkedSpecRef { path } => {
                write!(f, "unlinked spec element: {path} — no asserted code-link")
            }
            AuditFinding::DecayedEvidence {
                id,
                spec,
                anchor,
                confidence,
            } => {
                write!(
                    f,
                    "decayed evidence: {id} ({spec} ← {anchor}) — confidence {confidence:.2} \
                     is below the floor; confirm or retire"
                )
            }
        }
    }
}

/// Options of `archi link audit`.
#[derive(Default)]
pub struct AuditOptions {
    /// Delta source override; defaults to the latest version's commit
    /// provenance.
    pub since: Option<String>,
    /// Sweep this scope's nodes for asserted-link coverage.
    pub scope: Option<String>,
    /// Retire decayed evidence instead of only reporting it.
    pub prune: bool,
}

/// The outcome of `archi link audit`.
#[derive(Serialize)]
pub struct AuditReport {
    /// Live links.
    pub live: usize,
    /// Of them, asserted.
    pub asserted: usize,
    /// Of them, evidence.
    pub evidence: usize,
    /// Advisory findings — visible until lifted, never blocking.
    pub findings: Vec<AuditFinding>,
    /// What the audit could not cover, and why.
    pub notes: Vec<String>,
    /// Evidence links `--prune` retired.
    pub pruned: Vec<String>,
}

/// Aggregate hygiene: dark deltas, dark spec, decayed evidence.
pub fn audit(root: &Path, model: &Model, opts: &AuditOptions) -> Result<AuditReport, String> {
    let folded = load(root)?;
    let mut report = AuditReport {
        live: folded.live.len(),
        asserted: folded
            .live
            .iter()
            .filter(|l| l.standing == Standing::Asserted)
            .count(),
        evidence: folded
            .live
            .iter()
            .filter(|l| l.standing == Standing::Evidence)
            .count(),
        findings: Vec::new(),
        notes: Vec::new(),
        pruned: Vec::new(),
    };

    // Dark deltas: every hunk since the delta source either lands on a
    // linked span or is unaccounted for.
    let rev = match &opts.since {
        Some(r) => Some(r.clone()),
        None => latest_version_commit(root)?,
    };
    match rev {
        None => report.notes.push(
            "no delta source: commit the tree and run `archi version anchor` so the latest \
             version gains commit provenance, or pass --since <rev>"
                .to_string(),
        ),
        Some(rev) => {
            for (file, start, end) in delta_hunks(root, &rev)? {
                if let Some(finding) = unaccounted(root, &folded.live, &file, start, end) {
                    report.findings.push(finding);
                }
            }
        }
    }

    // Dark spec: elements of the audited scope with no asserted link and
    // no live evidence. The scope is `--scope`'s subtree — or, by default,
    // the active plan's task spec_refs.
    let dark = |path: &str| {
        !folded
            .live
            .iter()
            .any(|l| l.spec.version.is_none() && l.spec.path == path)
    };
    if let Some(scope) = &opts.scope {
        if !model.has_node(scope) {
            return Err(format!("--scope `{scope}` names no element of the live model"));
        }
        let prefix = format!("{scope}.");
        for path in node_paths(model) {
            if (path == *scope || path.starts_with(&prefix)) && dark(&path) {
                report.findings.push(AuditFinding::UnlinkedSpecRef { path });
            }
        }
    } else if crate::plans::active_name(root)?.is_some() {
        match crate::plans::load_active(root) {
            Ok(plan) => {
                let refs: BTreeSet<&String> =
                    plan.tasks.iter().flat_map(|t| t.spec_refs.iter()).collect();
                for r in refs {
                    if dark(r) {
                        report
                            .findings
                            .push(AuditFinding::UnlinkedSpecRef { path: r.clone() });
                    }
                }
            }
            Err(e) => report
                .notes
                .push(format!("the unlinked-spec-ref sweep skipped the active plan: {e}")),
        }
    }

    // Decayed evidence: derived confidence below the floor.
    let mut slots = Slots::new(root);
    let mut decayed = Vec::new();
    for link in folded.live.iter().filter(|l| l.standing == Standing::Evidence) {
        let checked = check_link(root, model, &mut slots, link.clone())?;
        let confidence = confidence(link, &checked.state);
        if confidence < CONFIDENCE_FLOOR {
            decayed.push(link.id.clone());
            report.findings.push(AuditFinding::DecayedEvidence {
                id: link.id.clone(),
                spec: link.spec.to_string(),
                anchor: link.anchor.to_string(),
                confidence,
            });
        }
    }
    if opts.prune && !decayed.is_empty() {
        retire(root, &decayed)?;
        report.pruned = decayed;
    }
    Ok(report)
}

/// The latest archived version's commit provenance, when recorded.
fn latest_version_commit(root: &Path) -> Result<Option<String>, String> {
    Ok(Archive::open(root)?
        .and_then(|a| a.entries().last().and_then(|e| e.commit.clone())))
}

/// New-side hunks of the code delta since a rev: `git diff` plus untracked
/// files — `archi/`, `.arch` sources and the project's `[audit] exclude`
/// patterns muted, the same boundary `code_files` walks.
fn delta_hunks(root: &Path, rev: &str) -> Result<Vec<(String, usize, usize)>, String> {
    let git = |args: &[&str]| -> Result<String, String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .map_err(|e| format!("the delta source needs git: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "git {}: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    };
    let patterns = scan_exclusions(root);
    let is_code = |file: &str| {
        !file.starts_with("archi/")
            && !file.ends_with(".arch")
            && file != "archi.toml"
            && !excluded(file, &patterns)
    };
    let mut hunks = Vec::new();
    let diff = git(&["diff", "--unified=0", "--no-color", rev, "--", "."])?;
    let mut current: Option<String> = None;
    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            current = Some(path.to_string());
        } else if line.starts_with("+++ ") {
            current = None; // deletion: no new side
        } else if let Some(rest) = line.strip_prefix("@@ ")
            && let Some(file) = &current
            && is_code(file)
            && let Some(plus) = rest.split(' ').find(|s| s.starts_with('+'))
        {
            let mut parts = plus[1..].splitn(2, ',');
            let start: usize = parts.next().unwrap_or("0").parse().unwrap_or(0);
            let count: usize = parts.next().map_or(1, |c| c.parse().unwrap_or(1));
            if count > 0 && start > 0 {
                hunks.push((file.clone(), start, start + count - 1));
            }
        }
    }
    let status = git(&["status", "--porcelain=v1", "--untracked-files=all"])?;
    for line in status.lines() {
        if let Some(file) = line.strip_prefix("?? ")
            && is_code(file)
            && let Ok(text) = fs::read_to_string(root.join(file))
        {
            hunks.push((file.to_string(), 1, text.lines().count().max(1)));
        }
    }
    Ok(hunks)
}

/// Whether a hunk is claimed by some live link: a file anchor claims the
/// whole file; a symbol anchor claims its item's current span.
fn unaccounted(
    root: &Path,
    live: &[Link],
    file: &str,
    start: usize,
    end: usize,
) -> Option<AuditFinding> {
    let on_file: Vec<&Link> = live.iter().filter(|l| l.anchor.file == file).collect();
    if on_file.iter().any(|l| l.anchor.symbol.is_none()) {
        return None;
    }
    let canonical = fs::read_to_string(root.join(file))
        .ok()
        .map(|text| code::canonicalize(file, &text));
    if let Some(canonical) = &canonical {
        for link in &on_file {
            let symbol = link.anchor.symbol.as_deref().expect("file anchors returned");
            for item in canonical.find(symbol) {
                if item.start_line <= end && start <= item.end_line {
                    return None;
                }
            }
        }
    }
    let symbol = canonical.as_ref().and_then(|c| {
        c.items
            .iter()
            .filter(|i| i.start_line <= start && end <= i.end_line)
            .min_by_key(|i| i.end_line - i.start_line)
            .map(|i| i.symbol.clone())
    });
    Some(AuditFinding::UnaccountedDelta {
        file: file.to_string(),
        start,
        end,
        symbol,
    })
}

// ---- rendering -------------------------------------------------------------

/// One link as a human line: id, kind/standing, spec ← anchor.
pub fn render_link(l: &Link) -> String {
    format!(
        "{}  {:8} {:8} {:10} {} ← {}",
        l.id,
        l.kind.describe(),
        l.standing.describe(),
        l.origin.to_string(),
        l.spec,
        l.anchor
    )
}

/// The verify report as human lines: one per link, then the tally.
pub fn render_verify(report: &VerifyReport) -> String {
    let mut out = String::new();
    for c in &report.checked {
        out.push_str(&format!(
            "{:22} {}{}\n",
            c.state.describe(),
            render_link(&c.link),
            if c.failing { "  [failing]" } else { "" }
        ));
        if let Some(n) = &c.note {
            out.push_str(&format!("{:22}   {n}\n", ""));
        }
    }
    let failing = report.checked.iter().filter(|c| c.failing).count();
    let clean = report
        .checked
        .iter()
        .filter(|c| c.state == State::Clean)
        .count();
    out.push_str(&format!(
        "{} checked: {clean} clean, {failing} failing",
        report.checked.len()
    ));
    if report.skipped > 0 {
        out.push_str(&format!(", {} skipped (unchanged since rev)", report.skipped));
    }
    out.push('\n');
    out
}

/// The audit report as human lines.
pub fn render_audit(report: &AuditReport) -> String {
    let mut out = format!(
        "links: {} live ({} asserted, {} evidence)\n",
        report.live, report.asserted, report.evidence
    );
    if report.findings.is_empty() {
        out.push_str("no findings\n");
    }
    for f in &report.findings {
        out.push_str(&format!("{f}\n"));
    }
    for n in &report.notes {
        out.push_str(&format!("note: {n}\n"));
    }
    for id in &report.pruned {
        out.push_str(&format!("pruned {id}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn temp_project() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-links-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("code")).unwrap();
        fs::write(dir.join("archi.toml"), "[project]\nname = \"t\"\n").unwrap();
        fs::write(dir.join("src").join("model.arch"), MODEL).unwrap();
        fs::write(dir.join("code").join("auth.rs"), AUTH_RS).unwrap();
        dir
    }

    const MODEL: &str = "def conn wire := * -> *\n\
                         def node Auth:\n  port store\n\
                         def node Vault:\n  port inn\n\
                         Auth.store wire Vault.inn\n";

    const AUTH_RS: &str = "pub struct Vault {\n    salted: Vec<u8>,\n}\n\n\
                           impl Vault {\n    pub fn persist(&mut self, hash: &[u8]) {\n        self.salted.extend(hash);\n    }\n}\n";

    fn model_of(root: &Path) -> Workspace {
        modeling_lang::source::compile_project(root)
            .unwrap_or_else(|f| panic!("test model failed to compile:\n{}", f.render()))
            .workspace
    }

    fn state_of(root: &Path, ws: &Workspace, id: &str) -> (State, bool) {
        let report = verify(root, ws.model(), &VerifyOptions::default()).unwrap();
        let c = report
            .checked
            .iter()
            .find(|c| c.link.id == id)
            .unwrap_or_else(|| panic!("no `{id}` in the report"));
        (c.state.clone(), c.failing)
    }

    #[test]
    fn add_verify_and_the_drift_grades() {
        let root = temp_project();
        let ws = model_of(&root);
        let lit = add(
            &root,
            ws.model(),
            "Vault",
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();
        let ind = add(
            &root,
            ws.model(),
            "Auth",
            "code/auth.rs#Vault::persist",
            LinkKind::Indirect,
        )
        .unwrap();
        assert_eq!((lit.id.as_str(), ind.id.as_str()), ("l0001", "l0002"));
        assert_eq!(lit.standing, Standing::Asserted);
        assert_eq!(lit.birth.spans[0].file, "code/auth.rs");

        assert_eq!(state_of(&root, &ws, "l0001"), (State::Clean, false));

        // Reformatting is not drift: canonical tokens are the identity.
        fs::write(
            root.join("code/auth.rs"),
            AUTH_RS.replace(
                "pub fn persist(&mut self, hash: &[u8]) {\n        self.salted.extend(hash);\n    }",
                "// persists a salted hash\n    pub fn persist(&mut self,\n                   hash: &[u8]) {\n        self.salted.extend(hash);\n    }",
            ),
        )
        .unwrap();
        assert_eq!(state_of(&root, &ws, "l0001"), (State::Clean, false));

        // A body edit drifts the literal link and fails it; the indirect
        // link's watched interface holds.
        fs::write(
            root.join("code/auth.rs"),
            AUTH_RS.replace("self.salted.extend(hash);", "self.salted = hash.to_vec();"),
        )
        .unwrap();
        assert_eq!(state_of(&root, &ws, "l0001"), (State::Drifted, true));
        assert_eq!(state_of(&root, &ws, "l0002"), (State::Clean, false));

        // A signature edit drifts the indirect link too.
        fs::write(
            root.join("code/auth.rs"),
            AUTH_RS.replace("persist(&mut self, hash: &[u8])", "persist(&mut self, hash: Vec<u8>)"),
        )
        .unwrap();
        assert_eq!(state_of(&root, &ws, "l0002"), (State::Drifted, false));

        // Repin accepts the drift: the projection rewrites, birth stands.
        let before = ls(&root, None, false).unwrap()[0].birth.clone();
        repin(&root, "l0001", None).unwrap();
        assert_eq!(state_of(&root, &ws, "l0001"), (State::Clean, false));
        assert_eq!(ls(&root, None, false).unwrap()[0].birth, before);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn moves_are_candidates_and_deletions_are_missing() {
        let root = temp_project();
        let ws = model_of(&root);
        add(
            &root,
            ws.model(),
            "Vault",
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();

        // The whole impl moves verbatim to another file.
        let (structs, rest) = AUTH_RS.split_once("\n\n").unwrap();
        fs::write(root.join("code/auth.rs"), structs).unwrap();
        fs::write(
            root.join("code/store.rs"),
            format!("use super::Vault;\n\n{rest}"),
        )
        .unwrap();
        let (state, failing) = state_of(&root, &ws, "l0001");
        assert!(
            matches!(&state, State::Moved { file, exact: true, .. } if file == "code/store.rs"),
            "{state:?}"
        );
        assert!(!failing, "moved has a candidate; missing is what fails");

        repin(&root, "l0001", Some("code/store.rs#Vault::persist")).unwrap();
        assert_eq!(state_of(&root, &ws, "l0001"), (State::Clean, false));

        fs::remove_dir_all(root.join("code").join("store.rs")).ok();
        fs::remove_file(root.join("code").join("store.rs")).ok();
        let (state, failing) = state_of(&root, &ws, "l0001");
        assert_eq!(state, State::Missing);
        assert!(failing);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn spec_refs_resolve_nodes_edges_and_slots() {
        let root = temp_project();
        let ws = model_of(&root);

        // A ref that names nothing refuses to mint.
        let err = add(&root, ws.model(), "Nope", "code/auth.rs", LinkKind::Literal).unwrap_err();
        assert!(err.contains("E_MODEL_REF"), "{err}");

        // An edge ref is the canonical surface text.
        add(
            &root,
            ws.model(),
            "Auth.store wire Vault.inn",
            "code/auth.rs#Vault::persist",
            LinkKind::Indirect,
        )
        .unwrap();

        // Pin a version, then rename the node in the live model: the pinned
        // ref still resolves (note only), the Working ref spec-drifts.
        versions::save(&root, ws.model(), "first").unwrap();
        add(&root, ws.model(), "Vault@v0001", "code/auth.rs", LinkKind::Indirect).unwrap();
        add(&root, ws.model(), "Vault", "code/auth.rs", LinkKind::Indirect).unwrap();
        fs::write(
            root.join("src").join("model.arch"),
            MODEL.replace("Vault", "Safe"),
        )
        .unwrap();
        let ws2 = model_of(&root);
        let report = verify(&root, ws2.model(), &VerifyOptions::default()).unwrap();
        let by_id = |id: &str| report.checked.iter().find(|c| c.link.id == id).unwrap();
        assert_eq!(by_id("l0002").state, State::Clean, "pinned slot holds");
        assert_eq!(by_id("l0003").state, State::SpecDrifted);
        assert!(by_id("l0003").failing);
        // The edge ref names Vault too: drifted with it.
        assert_eq!(by_id("l0001").state, State::SpecDrifted);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn evidence_confirms_decays_and_prunes() {
        let root = temp_project();
        let ws = model_of(&root);
        // A captured link, as task-close capture will mint it.
        let resolved = resolve_anchor(
            &root,
            &Anchor::parse("code/auth.rs#Vault::persist").unwrap(),
        )
        .unwrap();
        append(
            &root,
            &[Event::Add {
                link: Link {
                    id: "l0001".into(),
                    spec: SpecRef::parse("Vault").unwrap(),
                    anchor: Anchor::parse("code/auth.rs#Vault::persist").unwrap(),
                    kind: LinkKind::Indirect,
                    standing: Standing::Evidence,
                    origin: Origin::Captured { task: "t1".into() },
                    birth: Birth {
                        created: now(),
                        commit: None,
                        spans: vec![resolved.span.clone()],
                    },
                    pins: resolved.pins.clone(),
                    touches: Vec::new(),
                    decays: Vec::new(),
                },
            }],
        )
        .unwrap();

        // Evidence never fails a verify, even drifted.
        fs::write(
            root.join("code/auth.rs"),
            AUTH_RS.replace("persist(&mut self, hash: &[u8])", "persist(&mut self, h: u8)"),
        )
        .unwrap();
        let (state, failing) = state_of(&root, &ws, "l0001");
        assert_eq!(state, State::Drifted);
        assert!(!failing);

        // Confirm records the decision.
        let confirmed = confirm(&root, "l0001").unwrap();
        assert_eq!(confirmed.standing, Standing::Asserted);
        assert!(confirm(&root, "l0001").is_err(), "already asserted");

        // A second evidence link whose anchor dies decays; --prune retires.
        append(
            &root,
            &[Event::Add {
                link: Link {
                    id: "l0002".into(),
                    spec: SpecRef::parse("Auth").unwrap(),
                    anchor: Anchor::parse("code/auth.rs#Vault::gone").unwrap(),
                    kind: LinkKind::Indirect,
                    standing: Standing::Evidence,
                    origin: Origin::Captured { task: "t1".into() },
                    birth: Birth {
                        created: now(),
                        commit: None,
                        spans: Vec::new(),
                    },
                    pins: resolved.pins,
                    touches: Vec::new(),
                    decays: Vec::new(),
                },
            }],
        )
        .unwrap();
        let report = audit(
            &root,
            ws.model(),
            &AuditOptions {
                prune: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            report
                .findings
                .iter()
                .any(|f| matches!(f, AuditFinding::DecayedEvidence { id, .. } if id == "l0002")),
        );
        assert_eq!(report.pruned, vec!["l0002".to_string()]);
        assert!(ls(&root, None, false).unwrap().iter().all(|l| l.id != "l0002"));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn confidence_accrues_by_touch_and_erodes_by_decay() {
        let root = temp_project();
        let ws = model_of(&root);
        let resolved = resolve_anchor(
            &root,
            &Anchor::parse("code/auth.rs#Vault::persist").unwrap(),
        )
        .unwrap();
        append(
            &root,
            &[Event::Add {
                link: Link {
                    id: "l0001".into(),
                    spec: SpecRef::parse("Vault").unwrap(),
                    anchor: Anchor::parse("code/auth.rs#Vault::persist").unwrap(),
                    kind: LinkKind::Indirect,
                    standing: Standing::Evidence,
                    origin: Origin::Captured { task: "t1".into() },
                    birth: Birth {
                        created: now(),
                        commit: None,
                        spans: vec![resolved.span],
                    },
                    pins: resolved.pins,
                    touches: Vec::new(),
                    decays: Vec::new(),
                },
            }],
        )
        .unwrap();

        // Born clean at 0.5 — above the floor, no finding.
        let live = ls(&root, None, false).unwrap();
        assert!((confidence(&live[0], &State::Clean) - 0.5).abs() < 1e-9);
        let report = audit(&root, ws.model(), &AuditOptions::default()).unwrap();
        assert!(
            !report
                .findings
                .iter()
                .any(|f| matches!(f, AuditFinding::DecayedEvidence { .. })),
        );

        // A touch accrues once per task; decays erode harder. The fold
        // dedups replayed events.
        let at = now();
        let touch = |task: &str| Event::Touch {
            id: "l0001".into(),
            task: task.into(),
            at: at.clone(),
        };
        let decay = |task: &str| Event::Decay {
            id: "l0001".into(),
            task: task.into(),
            at: at.clone(),
        };
        append(&root, &[touch("t2"), touch("t2"), decay("t3"), decay("t4")]).unwrap();
        let live = ls(&root, None, false).unwrap();
        assert_eq!(live[0].touches, vec!["t2".to_string()]);
        assert_eq!(
            live[0].decays,
            vec!["t3".to_string(), "t4".to_string()]
        );
        // 0.5 + 0.15 − 2·0.25 = 0.15: below the floor — flagged, prunable.
        assert!((confidence(&live[0], &State::Clean) - 0.15).abs() < 1e-9);
        let report = audit(
            &root,
            ws.model(),
            &AuditOptions {
                prune: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            report.findings.iter().any(|f| matches!(
                f,
                AuditFinding::DecayedEvidence { id, confidence, .. }
                    if id == "l0001" && (confidence - 0.15).abs() < 1e-9
            )),
            "{}",
            render_audit(&report)
        );
        assert_eq!(report.pruned, vec!["l0001".to_string()]);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn audit_scopes_unlinked_refs_from_the_active_plan() {
        let root = temp_project();
        let ws = model_of(&root);
        versions::save(&root, ws.model(), "planned").unwrap();
        crate::plans::use_plan(&root, ws.model(), "mvp").unwrap();
        crate::plans::task_add(&root, "Vault", None).unwrap();

        // No --scope: the sweep reads the active plan's task spec_refs —
        // the node and its incoming edge, both dark.
        let unlinked = |report: &AuditReport| -> Vec<String> {
            report
                .findings
                .iter()
                .filter_map(|f| match f {
                    AuditFinding::UnlinkedSpecRef { path } => Some(path.clone()),
                    _ => None,
                })
                .collect()
        };
        let report = audit(&root, ws.model(), &AuditOptions::default()).unwrap();
        assert_eq!(
            unlinked(&report),
            vec!["Auth.store wire Vault.inn".to_string(), "Vault".to_string()]
        );

        // An asserted link lifts the node; live evidence lifts the edge —
        // dark means no asserted link *and* no live evidence.
        add(
            &root,
            ws.model(),
            "Vault",
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();
        let resolved = resolve_anchor(
            &root,
            &Anchor::parse("code/auth.rs#Vault::persist").unwrap(),
        )
        .unwrap();
        append(
            &root,
            &[Event::Add {
                link: Link {
                    id: "l0002".into(),
                    spec: SpecRef::parse("Auth.store wire Vault.inn").unwrap(),
                    anchor: Anchor::parse("code/auth.rs#Vault::persist").unwrap(),
                    kind: LinkKind::Indirect,
                    standing: Standing::Evidence,
                    origin: Origin::Captured { task: "t1".into() },
                    birth: Birth {
                        created: now(),
                        commit: None,
                        spans: vec![resolved.span],
                    },
                    pins: resolved.pins,
                    touches: Vec::new(),
                    decays: Vec::new(),
                },
            }],
        )
        .unwrap();
        let report = audit(&root, ws.model(), &AuditOptions::default()).unwrap();
        assert_eq!(unlinked(&report), Vec::<String>::new());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn audit_sweeps_scope_coverage_and_dark_deltas() {
        let root = temp_project();
        let git = |args: &[&str]| {
            Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(args)
                .output()
                .ok()
                .filter(|o| o.status.success())
        };
        if git(&["init", "-q"]).is_none() {
            return; // no git in this environment: the delta source is optional
        }
        git(&["add", "."]).unwrap();
        if git(&[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-qm",
            "base",
        ])
        .is_none()
        {
            return;
        }
        let ws = model_of(&root);
        add(
            &root,
            ws.model(),
            "Vault",
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();

        // An edit inside the linked item is claimed; a new unlinked file is
        // a dark delta.
        fs::write(
            root.join("code/auth.rs"),
            AUTH_RS.replace("self.salted.extend(hash);", "self.salted = hash.to_vec();"),
        )
        .unwrap();
        fs::write(root.join("code/rogue.rs"), "pub fn rogue() -> u8 { 42 }\n").unwrap();
        let report = audit(
            &root,
            ws.model(),
            &AuditOptions {
                since: Some("HEAD".into()),
                scope: Some("Auth".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let dark: Vec<&AuditFinding> = report
            .findings
            .iter()
            .filter(|f| matches!(f, AuditFinding::UnaccountedDelta { .. }))
            .collect();
        assert_eq!(dark.len(), 1, "{}", render_audit(&report));
        assert!(
            matches!(dark[0], AuditFinding::UnaccountedDelta { file, symbol: Some(s), .. }
                if file == "code/rogue.rs" && s == "rogue")
        );
        // Auth has no asserted link: dark spec.
        assert!(
            report
                .findings
                .iter()
                .any(|f| matches!(f, AuditFinding::UnlinkedSpecRef { path } if path == "Auth"))
        );

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn rm_retires_and_the_journal_stays_dense() {
        let root = temp_project();
        let ws = model_of(&root);
        add(&root, ws.model(), "Vault", "code/auth.rs", LinkKind::Literal).unwrap();
        add(&root, ws.model(), "Auth", "code/auth.rs", LinkKind::Literal).unwrap();
        retire(&root, &["l0001".to_string()]).unwrap();
        assert!(retire(&root, &["l0001".to_string()]).is_err(), "already retired");
        let live = ls(&root, None, false).unwrap();
        assert_eq!(live.len(), 1);
        // Ids mint past retirements: dense over adds, never reused.
        let third = add(&root, ws.model(), "Vault", "code/auth.rs", LinkKind::Indirect).unwrap();
        assert_eq!(third.id, "l0003");
        // Bulk by spec.
        let retired = retire_spec(&root, "Vault").unwrap();
        assert_eq!(retired, vec!["l0003".to_string()]);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn scans_honor_audit_exclusions_and_code_stays_dark() {
        let root = temp_project();
        fs::write(
            root.join("archi.toml"),
            "[project]\nname = \"t\"\n\n[audit]\nexclude = [\"*.md\", \"notes/\", \"code/vendored.rs\"]\n",
        )
        .unwrap();
        let git = |args: &[&str]| {
            Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(args)
                .output()
                .ok()
                .filter(|o| o.status.success())
        };
        if git(&["init", "-q"]).is_none() {
            return; // no git in this environment: the delta source is optional
        }
        git(&["add", "."]).unwrap();
        if git(&[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-qm",
            "base",
        ])
        .is_none()
        {
            return;
        }
        // Prose, an excluded directory, an exactly excluded file — and one
        // genuinely unclaimed code file beside them.
        fs::write(root.join("README.md"), "# t\n\nprose\n").unwrap();
        fs::create_dir_all(root.join("notes")).unwrap();
        fs::write(root.join("notes/n.txt"), "scratch\n").unwrap();
        fs::write(root.join("code/vendored.rs"), "pub fn vendored() {}\n").unwrap();
        fs::write(root.join("code/rogue.rs"), "pub fn rogue() -> u8 { 42 }\n").unwrap();

        let files = code_files(&root);
        assert!(files.contains(&"code/rogue.rs".to_string()), "{files:?}");
        assert!(files.contains(&"code/auth.rs".to_string()), "{files:?}");
        assert!(!files.contains(&"README.md".to_string()), "{files:?}");
        assert!(!files.contains(&"notes/n.txt".to_string()), "{files:?}");
        assert!(!files.contains(&"code/vendored.rs".to_string()), "{files:?}");

        let ws = model_of(&root);
        let report = audit(
            &root,
            ws.model(),
            &AuditOptions {
                since: Some("HEAD".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let dark: Vec<String> = report
            .findings
            .iter()
            .filter_map(|f| match f {
                AuditFinding::UnaccountedDelta { file, .. } => Some(file.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            dark,
            vec!["code/rogue.rs".to_string()],
            "{}",
            render_audit(&report)
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn excluded_files_keep_their_links() {
        let root = temp_project();
        fs::write(
            root.join("archi.toml"),
            "[project]\nname = \"t\"\n\n[audit]\nexclude = [\"*.md\"]\n",
        )
        .unwrap();
        fs::write(root.join("README.md"), "# t\n\nAuth is the gate.\n").unwrap();
        let ws = model_of(&root);
        let link = add(&root, ws.model(), "Auth", "README.md", LinkKind::Indirect).unwrap();
        let (state, failing) = state_of(&root, &ws, &link.id);
        assert_eq!(state, State::Clean);
        assert!(!failing, "exclusion scopes the scans, not the claims");
        fs::remove_dir_all(&root).unwrap();
    }
}
