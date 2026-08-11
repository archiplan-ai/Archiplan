//! Code-links: spec ↔ code traceability (`archi/requirements/code-link/`).
//!
//! A link ties a **`SpecRef`** — a node path or a typed edge at a version
//! slot, or a scenario of a world fact — to an **anchor** in the code tree:
//! a file, optionally a symbol.
//! It carries two layers with opposite mutability: the immutable **birth
//! record** (the spans that realized the spec element, content-pinned) and
//! the **projection** (where that code lives now — anchor plus the
//! interface/body hash pair), recomputed by `verify` and rewritten only by
//! an explicit `repin`.
//!
//! A link onto a scenario witnesses both sides: the projection carries the
//! digest of the story too, and `verify` names which side moved
//! (`archi/requirements/world-facts/a-scenario-link-binds-two-hashes.md`).
//!
//! Storage is an append-only journal, `archi/links/journal.jsonl` — events
//! `add`, `confirm`, `repin`, `retire`; the live link set is its fold. A
//! commit sha in a birth record is provenance, never a dependency, exactly
//! as in the version archive (`archi/requirements/versioning/keyframes-bound-the-archive.md`).

pub(crate) mod capture;
pub(crate) mod code;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use modeling_lang::{Definition, Model, Statement, Workspace};
use serde::{Deserialize, Serialize};

use crate::docs;
use crate::versions::{self, Archive};

// ---- the link model --------------------------------------------------------

/// Which hash the link watches (`archi/requirements/self-hosting/drift-graded-per-kind.md`).
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

/// Which rule produced a row
/// (`archi/requirements/code-link/the-journal-says-which-rule-made-a-row.md`).
///
/// The journal is append-only, so a rule that stops governing does not retire
/// the rows it made: they are marked, they keep standing and they keep
/// grading. Marking is the whole answer — a record that can be rewritten when
/// it becomes inconvenient is not a record.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rule {
    /// Matched from terms the ref and the code share — the tool's guess.
    Inferred,
    /// Minted from a declaration a writer wrote.
    Declared,
    /// Minted by hand through `archi link add`.
    Authored,
}

impl Rule {
    /// The rule a row records when it names none — every row journaled
    /// before the field existed. The provenance already says it: a row
    /// `link add` minted is a claim a person made by hand, and a row capture
    /// minted under the shared-term rule is the tool's guess. Reading it
    /// from the origin keeps those rows exactly what they were, and no
    /// migration rewrites append-only truth to say so.
    fn of(origin: &Origin) -> Rule {
        match origin {
            Origin::Authored => Rule::Authored,
            Origin::Captured { .. } => Rule::Inferred,
        }
    }

    fn describe(self) -> &'static str {
        match self {
            Rule::Inferred => "inferred",
            Rule::Declared => "declared",
            Rule::Authored => "authored",
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.describe())
    }
}

/// A spec element reference: a node path (`AuthService.Storage`), a typed
/// edge in its canonical surface form (`A.p link B.q`), optionally pinned
/// to a version slot (`@v0003`; absent = Working, the live tree) — a
/// scenario of a world fact, `<fact-slug>#<scenario name>`
/// (`archi/requirements/world-facts/the-scenario-is-the-address-not-the-step.md`)
/// — or a requirement, `req:<slug>`
/// (`archi/requirements/code-link/a-requirement-is-addressable-in-the-journal.md`).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SpecRef {
    /// The element: a dot path, canonical edge text (contains spaces), or a
    /// fact slug and a scenario name joined by `#`.
    #[serde(rename = "ref")]
    pub path: String,
    /// Pinned version slot; `None` is Working.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// The prefix that addresses a requirement. An addressing scheme that says
/// what it is can be joined later by another that says what it is, where a
/// bare slug could not
/// (`archi/requirements/code-link/a-requirement-is-addressable-in-the-journal.md`).
pub const REQ: &str = "req:";

impl SpecRef {
    /// Parse `<element>[@vNNNN]`, the world form `<fact-slug>#<scenario
    /// name>`, or the requirement form `req:<slug>`. The prefix is read
    /// first, before either split: a requirement slug holds no `#` and no
    /// `@`, so it would fall to the element branch and resolve against no
    /// element. Names never contain `@`, so the slot split is unambiguous;
    /// a `#` decides the world form before it, because a scenario name is
    /// prose that may hold anything and is taken verbatim — a fact stands in
    /// one slot, what the tree holds now.
    pub fn parse(text: &str) -> Result<SpecRef, String> {
        let text = text.trim();
        if text.is_empty() {
            return Err("the spec ref is empty".into());
        }
        if let Some(slug) = text.strip_prefix(REQ) {
            let slug = slug.trim();
            if slug.is_empty() {
                return Err(format!("`{text}` names no requirement — `{REQ}<slug>` addresses one"));
            }
            // A requirement slug is stable across versions, by the rule that
            // already governs plan ownership, so it takes no slot at all.
            if slug.contains('@') {
                return Err(format!(
                    "`{text}`: a requirement carries no version pin — its slug is stable across \
                     versions, as plan ownership already reads it"
                ));
            }
            return Ok(SpecRef {
                path: format!("{REQ}{}", normalize_ref(slug)),
                version: None,
            });
        }
        if let Some((slug, name)) = text.split_once('#') {
            let (slug, name) = (slug.trim(), normalize_ref(name));
            if slug.is_empty() || name.is_empty() {
                return Err(format!("`{text}` is not `<fact-slug>#<scenario name>`"));
            }
            return Ok(SpecRef {
                path: format!("{slug}#{name}"),
                version: None,
            });
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

    /// The world form's parts — the fact's slug and the scenario name inside
    /// it — or `None` for an element ref: element paths hold no `#`. The
    /// requirement form answers `None` too: its prefix is read first, and
    /// whatever a slug holds is part of the slug.
    pub fn scenario(&self) -> Option<(&str, &str)> {
        if self.requirement().is_some() {
            return None;
        }
        self.path.split_once('#')
    }

    /// The requirement form's slug, or `None` for every other shape:
    /// element paths and fact slugs carry no `req:` prefix.
    pub fn requirement(&self) -> Option<&str> {
        self.path.strip_prefix(REQ)
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

/// Where the code lives: a member-relative file, optionally a symbol path
/// inside it (`crates/auth/src/store.rs#Store::persist`), optionally
/// qualified by the member repository the file lives in
/// (`backend//src/api.rs#serve`). Unqualified means home — every ref and
/// journal event written before members keeps its meaning unchanged
/// (`archi/requirements/multi-repo/refs-carry-their-repo`).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Anchor {
    /// The declared member holding the file; `None` = home.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Member-root-relative path, `/`-separated.
    pub file: String,
    /// `::`-joined item path; `None` anchors the whole file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

impl Anchor {
    /// Parse `[<member>//]<file>[#<symbol>]`.
    pub fn parse(text: &str) -> Result<Anchor, String> {
        let text = text.trim();
        if text.is_empty() {
            return Err("the code ref is empty".into());
        }
        let (repo, rest) = match text.split_once("//") {
            Some((m, r)) if !m.is_empty() && !r.is_empty() => (Some(m.to_string()), r),
            Some(_) => {
                return Err(format!(
                    "`{text}` is not `[<member>//]<file>[#<symbol>]`"
                ));
            }
            None => (None, text),
        };
        let (file, symbol) = match rest.split_once('#') {
            None => (rest, None),
            Some((f, s)) if !f.is_empty() && !s.is_empty() => (f, Some(s.to_string())),
            Some(_) => return Err(format!("`{text}` is not `[<member>//]<file>[#<symbol>]`")),
        };
        Ok(Anchor {
            repo,
            file: file.replace('\\', "/"),
            symbol,
        })
    }

    /// The anchor's file as a scan key: `member//file`, bare for home.
    pub fn qualified_file(&self) -> String {
        qualify(self.repo.as_deref(), &self.file)
    }
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(m) = &self.repo {
            write!(f, "{m}//")?;
        }
        match &self.symbol {
            None => write!(f, "{}", self.file),
            Some(s) => write!(f, "{}#{s}", self.file),
        }
    }
}

/// Join a member and a bare path into a scan key; home stays bare, so
/// memberless projects' keys — and their stored indexes — are unchanged.
pub(crate) fn qualify(member: Option<&str>, file: &str) -> String {
    match member {
        None | Some("") => file.to_string(),
        Some(m) => format!("{m}//{file}"),
    }
}

/// Split a scan key into its member and bare path.
pub(crate) fn split_qualified(key: &str) -> (Option<&str>, &str) {
    match key.split_once("//") {
        Some((m, rest)) if !m.is_empty() => (Some(m), rest),
        _ => (None, key),
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

/// The projection's hash pair, under a pinned canonicalizer — and, where the
/// spec ref names a scenario, the digest of the story the pair was bound to.
/// The witness of both sides sits in one record, so `repin` binds the pair in
/// one event (`archi/requirements/world-facts/a-scenario-link-binds-two-hashes.md`).
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Pins {
    /// The canonicalizer that produced the hashes (`rust-tok-v1`, `text-v1`).
    pub canonicalizer: String,
    /// Hash of the anchored item's signature (its declared shape).
    pub interface: String,
    /// Hash of the whole anchored item's canonical tokens.
    pub body: String,
    /// The spec side: the fingerprint of the fact's scenarios as the grammar
    /// parsed them. Absent on every link over an element path — an element
    /// path has no story to witness — and on every event journaled before the
    /// pair was witnessed, which replays unchanged. Absence means nothing was
    /// bound, never that something moved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,
}

/// One code-link, as journaled and as folded.
///
/// Reading goes through [`Row`], because one field of a row written before
/// this shape existed is read from another: a row that names no producing
/// rule takes it from its provenance
/// (`archi/requirements/code-link/the-journal-says-which-rule-made-a-row.md`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(from = "Row")]
pub struct Link {
    /// Readable sequence plus a content suffix, `l0042-9f3ab1`. The suffix
    /// hashes the link's content and mint moment, so two branches minting
    /// in parallel cannot collide and their journals union-merge cleanly
    /// (archi/requirements/self-hosting/parallel-editing-discipline.md). Bootstrap-era ids are bare `l0001`
    /// onward — the fold treats ids as opaque.
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
    /// Which rule produced the row. Every row minted from here on says it;
    /// a row journaled before the field existed names none, and [`Rule::of`]
    /// reads it from the origin, so those rows keep saying exactly what they
    /// always were and no migration runs
    /// (`archi/requirements/code-link/the-journal-says-which-rule-made-a-row.md`).
    pub rule: Rule,
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

/// A journaled row as it is read: [`Link`]'s shape, with the producing rule
/// optional. It is the whole of the backward compatibility — a row written
/// before the field existed names no rule, and the conversion reads one from
/// the origin rather than defaulting every such row to one word. Nothing
/// else in the shape moved, so every other field replays exactly as before.
#[derive(Deserialize)]
struct Row {
    id: String,
    #[serde(rename = "spec")]
    spec: SpecRef,
    anchor: Anchor,
    kind: LinkKind,
    standing: Standing,
    origin: Origin,
    #[serde(default)]
    rule: Option<Rule>,
    birth: Birth,
    pins: Pins,
    #[serde(default)]
    touches: Vec<String>,
    #[serde(default)]
    decays: Vec<String>,
}

impl From<Row> for Link {
    fn from(r: Row) -> Link {
        Link {
            rule: r.rule.unwrap_or_else(|| Rule::of(&r.origin)),
            id: r.id,
            spec: r.spec,
            anchor: r.anchor,
            kind: r.kind,
            standing: r.standing,
            origin: r.origin,
            birth: r.birth,
            pins: r.pins,
            touches: r.touches,
            decays: r.decays,
        }
    }
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
        /// The spec side, when the repin moved it onto a renamed scenario.
        /// Absent in an anchor-only repin — and in every event written
        /// before the world form, which replays unchanged.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        spec: Option<SpecRef>,
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
    /// Adds ever journaled — the id sequence counts past retirements.
    adds: usize,
    /// Events the fold absorbed instead of applying — identical replayed
    /// lines and events landing on tombstones, the residue of merging
    /// concurrent branch histories. Surfaced by verify and audit, never
    /// silent, never corruption (archi/requirements/self-hosting/parallel-editing-discipline.md).
    absorbed: Vec<String>,
}

impl Folded {
    fn get(&self, id: &str) -> Option<&Link> {
        self.live.iter().find(|l| l.id == id)
    }

    fn next_id(&self, salt: &str) -> String {
        mint_id(self.adds + 1, salt)
    }
}

/// Mint a link id: a readable dense sequence plus a six-hex content suffix,
/// so ids minted on parallel branches cannot collide when the journals
/// union-merge (archi/requirements/self-hosting/parallel-editing-discipline.md).
pub(crate) fn mint_id(seq: usize, salt: &str) -> String {
    let mut h = Sha256::new();
    h.update(salt.as_bytes());
    if let Ok(t) = SystemTime::now().duration_since(UNIX_EPOCH) {
        h.update(t.as_nanos().to_le_bytes());
    }
    h.update(std::process::id().to_le_bytes());
    let hex = format!("{:x}", h.finalize());
    format!("l{seq:04}-{}", &hex[..6])
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

/// Fold the event stream into the live set. Concurrent branch histories
/// union-merge into interleaves the writers never saw; the fold stays
/// order-independent over them by absorbing what a sequential history
/// would forbid — an identical replayed line, an event landing on a
/// tombstone — and surfacing every absorption as a note. Only an event
/// naming an id the journal never minted, or two different links under one
/// id, remains corruption (archi/requirements/self-hosting/parallel-editing-discipline.md).
fn fold(events: Vec<Event>) -> Result<Folded, String> {
    let mut live: Vec<Link> = Vec::new();
    let mut retired: Vec<Link> = Vec::new();
    let mut adds = 0;
    let mut absorbed: Vec<String> = Vec::new();
    let corrupt = |id: &str, what: &str| {
        format!("journal corrupt: `{what}` names `{id}`, which was never minted")
    };
    for event in events {
        match event {
            Event::Add { link } => {
                if let Some(existing) = live.iter().chain(&retired).find(|l| l.id == link.id) {
                    let same = serde_json::to_value(existing).ok()
                        == serde_json::to_value(&link).ok();
                    if same {
                        absorbed.push(format!(
                            "add `{}` replayed identically — absorbed",
                            link.id
                        ));
                        continue;
                    }
                    return Err(format!("journal corrupt: `{}` is added twice", link.id));
                }
                live.push(link);
                adds += 1;
            }
            Event::Confirm { id, .. } => match live.iter_mut().find(|l| l.id == id) {
                Some(l) => l.standing = Standing::Asserted,
                None if retired.iter().any(|l| l.id == id) => {
                    absorbed.push(format!("`confirm` on retired `{id}` — absorbed"));
                }
                None => return Err(corrupt(&id, "confirm")),
            },
            Event::Repin {
                id,
                anchor,
                pins,
                spec,
                ..
            } => match live.iter_mut().find(|l| l.id == id) {
                Some(l) => {
                    l.anchor = anchor;
                    l.pins = pins;
                    if let Some(s) = spec {
                        l.spec = s;
                    }
                }
                None if retired.iter().any(|l| l.id == id) => {
                    absorbed.push(format!("`repin` on retired `{id}` — absorbed"));
                }
                None => return Err(corrupt(&id, "repin")),
            },
            Event::Retire { id, .. } => match live.iter().position(|l| l.id == id) {
                Some(at) => {
                    retired.push(live.remove(at));
                }
                None if retired.iter().any(|l| l.id == id) => {
                    absorbed.push(format!("`retire` on retired `{id}` — absorbed"));
                }
                None => return Err(corrupt(&id, "retire")),
            },
            Event::Touch { id, task, .. } => match live.iter_mut().find(|l| l.id == id) {
                Some(l) => {
                    if !l.touches.contains(&task) {
                        l.touches.push(task);
                    }
                }
                None if retired.iter().any(|l| l.id == id) => {
                    absorbed.push(format!("`touch` on retired `{id}` — absorbed"));
                }
                None => return Err(corrupt(&id, "touch")),
            },
            Event::Decay { id, task, .. } => match live.iter_mut().find(|l| l.id == id) {
                Some(l) => {
                    if !l.decays.contains(&task) {
                        l.decays.push(task);
                    }
                }
                None if retired.iter().any(|l| l.id == id) => {
                    absorbed.push(format!("`decay` on retired `{id}` — absorbed"));
                }
                None => return Err(corrupt(&id, "decay")),
            },
        }
    }
    Ok(Folded {
        live,
        retired,
        adds,
        absorbed,
    })
}

fn load(root: &Path) -> Result<Folded, String> {
    fold(read_journal(root)?)
}

const JOURNAL_GITATTRIBUTES: &str = "\
# The journal is append-only and its fold absorbs concurrent histories:
# branch merges concatenate instead of conflicting (archi/requirements/self-hosting/parallel-editing-discipline.md).
journal.jsonl merge=union
";

fn append(root: &Path, events: &[Event]) -> Result<(), String> {
    let path = journal_path(root);
    let dir = path.parent().expect("journal has a directory");
    fs::create_dir_all(dir).map_err(|e| format!("cannot create `{}`: {e}", dir.display()))?;
    let gitattributes = dir.join(".gitattributes");
    if !gitattributes.exists() {
        fs::write(&gitattributes, JOURNAL_GITATTRIBUTES)
            .map_err(|e| format!("cannot write `{}`: {e}", gitattributes.display()))?;
    }
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

/// What a command's spec-side resolution reads more than once, opened
/// lazily: the compiled pinned versions — most verifies never touch the
/// archive — and the requirement set, one walk of the doc tree however many
/// rows address a requirement.
struct Slots<'a> {
    root: &'a Path,
    archive: Option<Option<Archive>>,
    compiled: BTreeMap<String, Workspace>,
    requirements: Option<BTreeSet<String>>,
}

impl<'a> Slots<'a> {
    fn new(root: &'a Path) -> Self {
        Slots {
            root,
            archive: None,
            compiled: BTreeMap::new(),
            requirements: None,
        }
    }

    /// Whether the tree holds a requirement of that slug. The set is the doc
    /// tree's own, read through the discovery pass `plan verify` reads for
    /// `owns:`, so a link and a plan can never disagree about what a
    /// requirement is.
    fn resolves_requirement(&mut self, slug: &str) -> bool {
        self.requirements
            .get_or_insert_with(|| {
                docs::discover_tree(self.root)
                    .requirements
                    .into_iter()
                    .map(|r| r.slug)
                    .collect()
            })
            .contains(slug)
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

/// Whether a ref names an element of a model — a node by path, a port by
/// `Node.port`, or an edge by its canonical surface text. A link and a
/// requirement's `satisfied-by` consult the one shared resolver, so an element
/// means the same thing whether the journal or a doc names it
/// (`archi/requirements/element-addressing/satisfaction-names-the-interface.md`).
pub(crate) fn resolves_in(model: &Model, spec: &SpecRef) -> bool {
    model.resolve_element(&spec.path).is_some()
}

/// Whether a ref resolves against what stands now: a world ref against the
/// facts in the tree, a requirement ref against the requirement set, every
/// other ref against the live model. A fact is prose a person edits and no
/// version render holds a copy of it, so the world form knows one slot —
/// this one; a requirement slug is stable across versions and knows the same.
fn resolves_now(
    root: &Path,
    model: &Model,
    slots: &mut Slots,
    spec: &SpecRef,
) -> Result<bool, String> {
    if let Some(slug) = spec.requirement() {
        return Ok(slots.resolves_requirement(slug));
    }
    match spec.scenario() {
        Some((slug, name)) => resolves_scenario(root, slug, name),
        None => Ok(resolves_in(model, spec)),
    }
}

/// Why a requirement ref resolved to nothing: the slug names no file the
/// requirement discovery found.
fn requirement_refusal(slug: &str) -> String {
    format!(
        "`{slug}` names no requirement — no file under `archi/requirements/` holds it (E_MODEL_REF)"
    )
}

/// Whether a fact holds exactly one scenario of that name. Two scenarios
/// sharing one name inside one fact is a located error, because the name is
/// the address and an ambiguous address names nothing
/// (`archi/requirements/world-facts/the-scenario-is-the-address-not-the-step.md`).
fn resolves_scenario(root: &Path, slug: &str, name: &str) -> Result<bool, String> {
    let Some(scenarios) = fact_scenarios(root, slug) else {
        return Ok(false);
    };
    let lines: Vec<usize> = scenarios
        .iter()
        .filter(|(n, _)| n == name)
        .map(|(_, line)| *line)
        .collect();
    match lines.as_slice() {
        [] => Ok(false),
        [_] => Ok(true),
        [first, second, ..] => Err(format!(
            "{}:{second}: two scenarios are named `{name}` — the scenario name is the address, \
             and one fact holds it once (the first is at line {first})",
            fact_file(slug)
        )),
    }
}

/// Why a world ref resolved to nothing: the fact, or the name inside it.
fn scenario_refusal(root: &Path, slug: &str, path: &str) -> String {
    match fact_scenarios(root, slug) {
        None => format!(
            "`{slug}` names no world fact — `{}` is not in the tree (E_MODEL_REF)",
            fact_file(slug)
        ),
        Some(_) => format!(
            "`{path}` names no scenario of `{}` (E_MODEL_REF)",
            fact_file(slug)
        ),
    }
}

/// A fact's project-relative file. The folder decides what a file is, and a
/// fact is what stands under `facts/`
/// (`archi/requirements/world-facts/the-world-holds-four-layers.md`).
fn fact_file(slug: &str) -> String {
    format!("{}facts/{slug}.md", docs::world::WORLD)
}

/// The scenario names one world fact holds, each with the 1-based file line
/// it sits on; `None` when the tree holds no such fact. The record is read
/// through the docs pass — the same reader `check` runs — so a link and a
/// check can never disagree about what a fact holds.
fn fact_scenarios(root: &Path, slug: &str) -> Option<Vec<(String, usize)>> {
    let file = fact_file(slug);
    let text = fs::read_to_string(root.join(&file)).ok()?;
    // A record the reader cannot parse holds no addressable scenario; its
    // shape is `archi check`'s to report, never a link's.
    Some(
        docs::md::parse(&text)
            .ok()
            .and_then(|doc| docs::world::parse(&doc, &file, slug, root, &mut Vec::new()).scenarios)
            .map(|block| scenario_names(&block))
            .unwrap_or_default(),
    )
}

/// The `### ` headings of a `Scenarios` block and the names on them. The
/// smallest reader an address needs: the heading text is the scenario's name,
/// and the name is the address. What opens a heading is the grammar's own rule
/// ([`docs::gherkin::heading`]), read from there and not written again, so a
/// deeper heading opens no scenario here either.
///
/// [`docs::gherkin::parse`] reads the same block whole, and it does not
/// replace this reader: the grammar reports form, and a link resolves an
/// address. The grammar drops every scenario any refusal touched — a `But`
/// step, a heading over no step, a `Scenario:` line of the shape this one
/// replaced — and keeps a name with its inner whitespace as written, so a
/// normalized ref matches it only after the same collapse. Reading an address
/// through it would report a doc-grammar error as a lost reference, and would
/// unresolve a link on a malformed fact that `archi check` already reports.
/// The digest beside it does read through the grammar ([`fact_digest`]),
/// because a witness is of the story the grammar accepted, and it collapses
/// the name the same way to find the scenario a ref names; the two readers
/// answer two questions, and only `check` gates on form
/// (`archi/requirements/world-facts/the-grammar-is-a-named-subset.md`,
/// `archi/requirements/world-facts/the-scenario-is-the-address-not-the-step.md`).
fn scenario_names(block: &docs::world::Block) -> Vec<(String, usize)> {
    block
        .text
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let name = docs::gherkin::heading(line)?;
            Some((normalize_ref(name), block.line + i))
        })
        .filter(|(name, _)| !name.is_empty())
        .collect()
}

/// The fingerprint of one fact's scenarios, as the grammar parsed them —
/// [`docs::world_check::scenario_digest`], the one function the plan and the
/// link both read, so a plan and a link can never disagree about whether the
/// story moved. `scenario` is the grain: the name a ref addresses, or `None`
/// for the whole block. `None` for the result when no file holds the fact.
///
/// The file is read through [`docs::world_check::read_fact`], the reader the
/// wing walks its own folder with, so a link and a check hold one fact one
/// way. A block the grammar refuses digests as an empty story, and `archi
/// check` reports that form error where form errors belong.
fn fact_digest(root: &Path, slug: &str, scenario: Option<&str>) -> Option<String> {
    let fact = docs::world_check::read_fact(
        root,
        &root.join(fact_file(slug)),
        // The form of a fact is `archi check`'s to report, never a link's.
        &mut Vec::new(),
    )?;
    Some(docs::world_check::scenario_digest(&fact, scenario))
}

/// The spec side of a link as it stands now: the fingerprint of the one
/// scenario the ref addresses, nothing for an element path. A link witnesses
/// the scenario it names, so a sibling in the same fact is another link's
/// business
/// (`archi/requirements/world-facts/the-digest-witnesses-one-scenario.md`).
fn bind_scenario(root: &Path, spec: &SpecRef) -> Option<String> {
    let (slug, name) = spec.scenario()?;
    fact_digest(root, slug, Some(name))
}

/// Whether the scenario side of a witnessed pair parted from what the link
/// recorded. A link that recorded no digest — every link over an element
/// path, and every scenario link journaled before the pair was witnessed —
/// has nothing to compare against, and nothing never moved.
///
/// A digest journaled while the grain was the whole block is read at the
/// grain it was written: if it is the fact's whole story as that story stands
/// now, nothing moved. Narrowing the grain is not motion, so no link
/// false-fails on the day it narrows; the first `repin` writes the scenario's
/// own fingerprint, and the narrow grain rules the link from there
/// (`archi/requirements/world-facts/the-digest-witnesses-one-scenario.md`).
fn scenario_moved(root: &Path, link: &Link) -> bool {
    let Some(recorded) = &link.pins.scenario else {
        return false;
    };
    let Some((slug, _)) = link.spec.scenario() else {
        return false;
    };
    if bind_scenario(root, &link.spec).is_none_or(|now| now == *recorded) {
        return false;
    }
    fact_digest(root, slug, None).is_none_or(|whole| whole != *recorded)
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

/// Where an anchor's member stands on this machine. Absence is a value,
/// never a panic path (`archi/requirements/multi-repo/absence-is-not-drift`).
pub(crate) enum MemberRoot {
    /// The member resolves; anchors under it read from this root.
    At(PathBuf),
    /// Declared, but no checkout here.
    Unmapped(String),
    /// The journal names a member the manifest does not declare — the
    /// renamed-member trap; the recovery is restoring the declaration.
    Undeclared(String),
}

/// The member resolution every link operation reads through: home plus the
/// declared members, resolved once per command.
pub(crate) struct Roots {
    set: crate::members::MemberSet,
}

impl Roots {
    pub(crate) fn resolve(root: &Path) -> Result<Roots, String> {
        Ok(Roots {
            set: crate::members::MemberSet::resolve(root)?,
        })
    }

    pub(crate) fn set(&self) -> &crate::members::MemberSet {
        &self.set
    }

    /// The root an anchor's files resolve under.
    pub(crate) fn of(&self, repo: &Option<String>) -> MemberRoot {
        let name = repo.as_deref().unwrap_or(crate::members::HOME);
        match self.set.get(name) {
            None => MemberRoot::Undeclared(name.to_string()),
            Some(m) => match &m.root {
                Some(r) => MemberRoot::At(r.clone()),
                None => MemberRoot::Unmapped(name.to_string()),
            },
        }
    }

    /// The root, or a loud error naming the recovery — for commands that
    /// cannot proceed on absence (`link add`, `repin --to`).
    pub(crate) fn require(&self, repo: &Option<String>) -> Result<PathBuf, String> {
        match self.of(repo) {
            MemberRoot::At(r) => Ok(r),
            MemberRoot::Unmapped(m) => Err(format!(
                "member `{m}` is unreachable here: `archi repo map {m} <dir>` first"
            )),
            MemberRoot::Undeclared(m) => Err(format!(
                "`{m}` is not a declared member — add its [[repo]] row to archi.toml"
            )),
        }
    }
}

/// A freshly resolved anchor: its pins and the span it occupies right now.
struct Resolved {
    pins: Pins,
    span: Span,
}

/// Resolve an anchor against its member's working tree: read, canonicalize,
/// index. `member_root` is the anchor's member root, already resolved.
fn resolve_anchor(member_root: &Path, anchor: &Anchor) -> Result<Resolved, String> {
    let path = member_root.join(&anchor.file);
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
                    scenario: None,
                },
                span: Span {
                    file: anchor.qualified_file(),
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
                    scenario: None,
                },
                span: Span {
                    file: anchor.qualified_file(),
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
    mint(
        root,
        model,
        spec_text,
        code_text,
        kind,
        Rule::Authored,
        Origin::Authored,
        Standing::Asserted,
    )
}

/// Mint one row: resolve the spec side at its slot, resolve and pin the
/// anchor, journal the add. The producing rule, the provenance and the
/// standing are the caller's — `link add` stamps a hand-authored claim, and
/// the reader of a writer's declaration stamps `declared` through this same
/// path, so a declared row is minted exactly as an authored one and differs
/// only in what it says of itself
/// (`archi/requirements/code-link/the-journal-says-which-rule-made-a-row.md`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn mint(
    root: &Path,
    model: &Model,
    spec_text: &str,
    code_text: &str,
    kind: LinkKind,
    rule: Rule,
    origin: Origin,
    standing: Standing,
) -> Result<Link, String> {
    let spec = SpecRef::parse(spec_text)?;
    let mut slots = Slots::new(root);
    let resolves = match &spec.version {
        None => resolves_now(root, model, &mut slots, &spec)?,
        Some(_) => slots.resolves_pinned(&spec)?,
    };
    if !resolves {
        return Err(match (spec.requirement(), spec.scenario()) {
            (Some(slug), _) => requirement_refusal(slug),
            (None, Some((slug, _))) => scenario_refusal(root, slug, &spec.path),
            (None, None) => {
                let slot = spec.version.as_deref().unwrap_or("the live model");
                format!("`{}` names no element of {slot} (E_MODEL_REF)", spec.path)
            }
        });
    }
    let anchor = Anchor::parse(code_text)?;
    let roots = Roots::resolve(root)?;
    let member_root = roots.require(&anchor.repo)?;
    let resolved = resolve_anchor(&member_root, &anchor)?;
    // The pair is bound at birth: the scenario says what must happen, the
    // code answers it, and both sides are witnessed together.
    let mut pins = resolved.pins;
    pins.scenario = bind_scenario(root, &spec);
    let folded = load(root)?;
    let link = Link {
        id: folded.next_id(&format!("{spec_text}{code_text}")),
        spec,
        anchor,
        kind,
        standing,
        origin,
        rule,
        birth: Birth {
            created: now(),
            commit: versions::provenance(root),
            spans: vec![resolved.span],
        },
        pins,
        touches: Vec::new(),
        decays: Vec::new(),
    };
    append(root, &[Event::Add { link: link.clone() }])?;
    Ok(link)
}

/// The live rows a spec ref names: the path exactly, and the slot when the
/// filter pins one. The one place `ls` and a bulk retire agree on what a ref
/// covers.
fn on_ref(live: Vec<Link>, filter: Option<&SpecRef>) -> Vec<Link> {
    live.into_iter()
        .filter(|l| {
            filter.is_none_or(|f| {
                l.spec.path == f.path
                    && (f.version.is_none() || l.spec.version == f.version)
            })
        })
        .collect()
}

/// `archi link ls`: the live links, optionally filtered.
///
/// `--spec req:<slug>` is the reverse view — what answers this requirement —
/// and it answers from the rows a person stood behind: declared and
/// authored. The inferred rows keep standing and keep grading; they are
/// simply not evidence that anybody claimed this code answers this
/// requirement (`archi/requirements/code-link/the-journal-says-which-rule-made-a-row.md`).
pub fn ls(
    root: &Path,
    spec: Option<&str>,
    evidence_only: bool,
) -> Result<Vec<Link>, String> {
    let folded = load(root)?;
    let filter = spec.map(SpecRef::parse).transpose()?;
    let reverse = filter.as_ref().is_some_and(|f| f.requirement().is_some());
    Ok(on_ref(folded.live, filter.as_ref())
        .into_iter()
        .filter(|l| !reverse || l.rule != Rule::Inferred)
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
/// The selector is the ref and only the ref — a retire never reads the
/// producing rule, because no verb retires rows in bulk by the rule that
/// made them (`archi/requirements/code-link/the-journal-says-which-rule-made-a-row.md`).
pub fn retire_spec(root: &Path, spec: &str) -> Result<Vec<String>, String> {
    let folded = load(root)?;
    let filter = SpecRef::parse(spec)?;
    let ids: Vec<String> = on_ref(folded.live, Some(&filter))
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
/// anchor, or follow a move to a new one. On a scenario link it binds the
/// pair again, both sides at once: the operator looked, and what the link
/// witnesses now is what stands. The birth record is untouched.
pub fn repin(root: &Path, id: &str, to: Option<&str>) -> Result<Link, String> {
    let folded = load(root)?;
    let link = folded
        .get(id)
        .ok_or_else(|| format!("no live link `{id}`"))?;
    let anchor = match to {
        None => link.anchor.clone(),
        Some(t) => Anchor::parse(t)?,
    };
    let roots = Roots::resolve(root)?;
    let member_root = roots.require(&anchor.repo)?;
    let resolved = resolve_anchor(&member_root, &anchor)?;
    let mut pins = resolved.pins;
    pins.scenario = bind_scenario(root, &link.spec);
    append(
        root,
        &[Event::Repin {
            id: id.to_string(),
            at: now(),
            anchor: anchor.clone(),
            pins: pins.clone(),
            spec: None,
        }],
    )?;
    let mut repinned = link.clone();
    repinned.anchor = anchor;
    repinned.pins = pins;
    Ok(repinned)
}

/// `archi link repin --spec`: move the link onto a renamed scenario. The
/// code side is untouched — the name moved, the code did not — and so is the
/// birth record; the scenario side binds to the fact under its new name,
/// because the name is part of the story the digest witnesses. The spec ref
/// moves for this one reason: an element rename is located by the version
/// chain instead, so this repair needs no model to resolve its target
/// (`archi/requirements/world-facts/the-scenario-is-the-address-not-the-step.md`).
pub fn repin_spec(root: &Path, id: &str, spec_text: &str) -> Result<Link, String> {
    let folded = load(root)?;
    let link = folded
        .get(id)
        .ok_or_else(|| format!("no live link `{id}`"))?;
    let target = SpecRef::parse(spec_text)?;
    let Some((slug, name)) = target.scenario() else {
        return Err(format!(
            "`{}` is not `<fact-slug>#<scenario name>` — `repin --spec` moves a link onto a \
             renamed scenario; an element rename is located by the version chain",
            target.path
        ));
    };
    if !resolves_scenario(root, slug, name)? {
        return Err(scenario_refusal(root, slug, &target.path));
    }
    let mut pins = link.pins.clone();
    pins.scenario = bind_scenario(root, &target);
    append(
        root,
        &[Event::Repin {
            id: id.to_string(),
            at: now(),
            anchor: link.anchor.clone(),
            pins: pins.clone(),
            spec: Some(target.clone()),
        }],
    )?;
    let mut repinned = link.clone();
    repinned.spec = target;
    repinned.pins = pins;
    Ok(repinned)
}

// ---- verify ----------------------------------------------------------------

/// A projection's graded state (`archi/requirements/code-link/verify-grades-every-claim.md`).
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
    /// The anchor's member has no checkout here — a state of its own,
    /// upstream of Missing: the code is not gone, this machine cannot see
    /// it. No observation, no decay, no prune
    /// (`archi/requirements/multi-repo/absence-is-not-drift`).
    Unreachable {
        /// The member with no local root.
        member: String,
    },
    /// The stored canonicalizer is unknown to this verifier.
    CanonicalizerMismatch,
    /// The spec side moved: the ref no longer resolves at Working.
    SpecDrifted,
    /// The witnessed pair parted: the scenario the link recorded is not the
    /// scenario the grammar reads now. A ref that stopped resolving is
    /// `SpecDrifted` and reads as the rename it is; this one is a story that
    /// moved under an address that still stands
    /// (`archi/requirements/world-facts/a-scenario-link-binds-two-hashes.md`).
    ScenarioDrifted {
        /// The watched code hash moved under the same verify — both sides
        /// parted, and the report says both.
        code: bool,
    },
}

impl State {
    fn describe(&self) -> &'static str {
        match self {
            State::Clean => "clean",
            State::Drifted => "drifted",
            State::Moved { .. } => "moved",
            State::Missing => "missing",
            State::Unreachable { .. } => "unreachable",
            State::CanonicalizerMismatch => "canonicalizer-mismatch",
            State::SpecDrifted => "spec-drifted",
            State::ScenarioDrifted { code: false } => "scenario-drifted",
            State::ScenarioDrifted { code: true } => "pair-drifted",
        }
    }
}

/// The floor below which evidence reads as decayed — confirm or retire
/// (`archi/requirements/code-link/the-audit-inverts-coverage.md`).
pub const CONFIDENCE_FLOOR: f64 = 0.25;

/// Derived confidence of an evidence link — never stored. Born at 0.5;
/// each task whose capture re-encountered it accrues, each task that
/// rewrote the anchored item without carrying the spec_ref erodes, any
/// projection drift erodes once, and a dead anchor zeroes.
pub fn confidence(link: &Link, state: &State) -> f64 {
    if matches!(state, State::Missing) {
        return 0.0;
    }
    // An unreachable member is no observation at all: confidence holds
    // exactly where the last actual read left it.
    let unread = matches!(state, State::Unreachable { .. });
    let drift = if matches!(state, State::Clean) || unread { 0.0 } else { -0.25 };
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
    /// never fails. `Missing`, `CanonicalizerMismatch`, a Working-slot
    /// `SpecDrifted` and `ScenarioDrifted` always fail; `Drifted` fails
    /// literal links only. The spec side knows no literal/indirect split: a
    /// witness is one thing, and half of it moving moves all of it.
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
    /// Events the fold absorbed — merged-history residue, surfaced.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub absorbed: Vec<String>,
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
    /// Check only links whose anchor file changed since this git rev —
    /// `[<member>=]<rev>`, bare rev meaning home.
    pub since: Option<String>,
    /// Check only links into this member (`home` for the project's own
    /// repository). Inside this explicit scope, absence is the error it is:
    /// an unreachable member fails instead of reporting.
    pub repo: Option<String>,
}

/// A `[<member>=]<rev>` delta-source override, bare rev meaning home.
fn parse_since(text: &str) -> (String, String) {
    match text.split_once('=') {
        Some((m, r)) if !m.is_empty() && !r.is_empty() => (m.to_string(), r.to_string()),
        _ => (crate::members::HOME.to_string(), text.to_string()),
    }
}

/// A `--repo` value as the member name it scopes to: `home` (or the empty
/// string) is the project's own repository.
fn scope_member(text: &str) -> &str {
    if text == "home" { crate::members::HOME } else { text }
}

/// Verify the live links: recompute every projection in scope, grade it.
pub fn verify(root: &Path, model: &Model, opts: &VerifyOptions) -> Result<VerifyReport, String> {
    let folded = load(root)?;
    let filter = opts.spec.as_deref().map(SpecRef::parse).transpose()?;
    let roots = Roots::resolve(root)?;
    let scope = opts.repo.as_deref().map(scope_member);
    if let Some(member) = scope {
        // The explicit ask: the member must exist here to be verified.
        roots.require(&Some(member.to_string()).filter(|m| !m.is_empty()))?;
    }
    let changed = match opts.since.as_deref().map(parse_since) {
        None => None,
        Some((member, rev)) => {
            let mroot = roots.require(&Some(member.clone()).filter(|m| !m.is_empty()))?;
            let ctx = crate::members::GitContext::of(&mroot)
                .ok_or_else(|| format!("--since needs git: no work tree at `{}`", mroot.display()))?;
            Some((member, changed_files(&ctx, &rev)?))
        }
    };
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
        if let Some(member) = scope
            && link.anchor.repo.as_deref().unwrap_or(crate::members::HOME) != member
        {
            skipped += 1;
            continue;
        }
        if let Some((member, changed)) = &changed {
            let link_member = link.anchor.repo.as_deref().unwrap_or(crate::members::HOME);
            if link_member != member || !changed.contains(&link.anchor.file) {
                skipped += 1;
                continue;
            }
        }
        checked.push(check_link(root, model, &roots, &mut slots, link)?);
    }
    Ok(VerifyReport {
        checked,
        skipped,
        absorbed: folded.absorbed,
    })
}

/// Grade one link: spec side first, then reachability, then the projection.
fn check_link(
    root: &Path,
    model: &Model,
    roots: &Roots,
    slots: &mut Slots,
    link: Link,
) -> Result<Checked, String> {
    // Spec side. A pinned ref resolves by construction — the archive is
    // sealed — so a pinned link reports Working drift as a note, not a
    // state; a Working-slot ref that stopped resolving is SpecDrifted.
    let at_working = resolves_now(root, model, slots, &link.spec)?;
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
            // A renamed scenario has no version chain to locate it — the
            // repair is naming the new name. A retired requirement has none
            // either: the slug is the identity, and it went.
            note: Some(match (link.spec.requirement(), link.spec.scenario()) {
                (Some(slug), _) => format!(
                    "`{slug}` names no requirement of the tree — the requirement retired or was \
                     renamed; restore the file, or `link rm {}`",
                    link.id
                ),
                (None, Some((slug, _))) => format!(
                    "`{}` names no scenario of `{}`; `link repin {} --spec \
                     <fact-slug>#<scenario name>` moves the link onto the new name",
                    link.spec.path,
                    fact_file(slug),
                    link.id
                ),
                (None, None) => "the spec element is gone from the live model; the version chain \
                                 locates the rename or removal"
                    .to_string(),
            }),
            link,
            state,
        });
    }
    let working_note = (link.spec.version.is_some() && !at_working)
        .then(|| "spec ref no longer resolves at Working".to_string());

    // The scenario side is read before the code side, and folded in after
    // it: a pair that parted on both sides names both, and that needs both
    // answers in hand.
    let moved = scenario_moved(root, &link);
    let checked = check_projection(root, roots, link, working_note)?;
    Ok(witness(checked, moved))
}

/// Grade the code side of a link: reachability, then the anchor, then the
/// watched hash. The spec side is the caller's — this is the projection
/// alone.
fn check_projection(
    root: &Path,
    roots: &Roots,
    link: Link,
    working_note: Option<String>,
) -> Result<Checked, String> {
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

    // Reachability precedes resolution: an absent checkout is not a lost
    // anchor, and grades — and observes — nothing.
    let member_root = match roots.of(&link.anchor.repo) {
        MemberRoot::At(r) => r,
        MemberRoot::Unmapped(member) => {
            let note = format!(
                "member `{member}` has no checkout here — `archi repo map {member} <dir>`; \
                 the code is not gone, this machine cannot see it"
            );
            return Ok(Checked {
                failing: false,
                note: Some(note),
                link,
                state: State::Unreachable { member },
            });
        }
        MemberRoot::Undeclared(member) => {
            let note = format!(
                "the journal names member `{member}`, which archi.toml does not declare — \
                 restore its [[repo]] row (a rename orphans every ref carrying the old name)"
            );
            return Ok(Checked {
                failing: false,
                note: Some(note),
                link,
                state: State::Unreachable { member },
            });
        }
    };

    // The projection.
    let path = member_root.join(&link.anchor.file);
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => {
            let (state, note) = scan_for_candidate(root, &member_root, &link);
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
                let (state, note) = scan_for_candidate(root, &member_root, &link);
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

/// Fold the scenario side into a graded projection and name the side that
/// moved. Only a link carrying a recorded digest reaches past the first
/// guard, so a link over an element path — and a scenario link journaled
/// before the pair was witnessed — grades exactly as it graded before
/// (`archi/requirements/world-facts/a-scenario-link-binds-two-hashes.md`).
///
/// A code side already lost, ambiguous or out of reach keeps its own state:
/// that failure is upstream of a moved witness, and its repair — `repin` —
/// binds the pair anyway. The note still names the scenario, so nothing the
/// verify saw goes unsaid.
fn witness(mut checked: Checked, moved: bool) -> Checked {
    if checked.link.pins.scenario.is_none() {
        return checked;
    }
    let Some((slug, _)) = checked.link.spec.scenario() else {
        return checked;
    };
    let fact = fact_file(slug);
    let anchor = checked.link.anchor.to_string();
    let repair = format!("`link repin {}` binds the pair again", checked.link.id);
    let (state, message) = match (moved, &checked.state) {
        (false, State::Drifted) => (
            None,
            format!(
                "the code side moved: `{anchor}` no longer matches the hash this link recorded; \
                 the scenario side holds — {repair}"
            ),
        ),
        (true, State::Clean) => (
            Some(State::ScenarioDrifted { code: false }),
            format!(
                "the scenario side moved: `{fact}` no longer matches the digest this link \
                 recorded; the code side holds — {repair}"
            ),
        ),
        (true, State::Drifted) => (
            Some(State::ScenarioDrifted { code: true }),
            format!("both sides moved: `{fact}` and `{anchor}` — {repair}"),
        ),
        (true, _) => (
            None,
            format!(
                "the scenario side moved: `{fact}` no longer matches the digest this link \
                 recorded — {repair}"
            ),
        ),
        (false, _) => return checked,
    };
    checked.note = Some(match checked.note.take() {
        Some(n) => format!("{message}; {n}"),
        None => message,
    });
    if let Some(state) = state {
        checked.failing = checked.link.standing == Standing::Asserted;
        checked.state = state;
    }
    checked
}

/// The anchor is gone: sweep the anchor's own member tree for a candidate —
/// an item with the same symbol (or a file with the same canonical hash),
/// body-equality ranking exact moves first. Moves never cross members: a
/// candidate in another repository is a new residence, asserted by hand.
fn scan_for_candidate(
    project_root: &Path,
    member_root: &Path,
    link: &Link,
) -> (State, Option<String>) {
    let extension = Path::new(&link.anchor.file)
        .extension()
        .map(|e| e.to_string_lossy().into_owned());
    let files = match link.anchor.repo.as_deref() {
        None => code_files(project_root),
        Some(member) => member_code_files(project_root, member_root, member),
    };
    let mut candidate: Option<(String, Option<String>, bool)> = None;
    for file in files {
        if file == link.anchor.file
            || Path::new(&file).extension().map(|e| e.to_string_lossy().into_owned()) != extension
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(member_root.join(&file)) else {
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
                "candidate: `{}{}`{} — confirm with `link repin <id> --to`",
                qualify(link.anchor.repo.as_deref(), &file),
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

/// One member-relative path against the exclusion patterns: a trailing `/`
/// is a directory prefix, a leading `*` a suffix, anything else an exact
/// path. A bare pattern applies in every member; a `member//`-qualified
/// pattern in exactly its member — one boundary, optionally scoped
/// (`archi/requirements/multi-repo/scans-see-every-mapped-member`).
fn excluded_in(member: Option<&str>, file: &str, patterns: &[String]) -> bool {
    let one = |pattern: &str| {
        if pattern.ends_with('/') {
            file.starts_with(pattern)
        } else if let Some(suffix) = pattern.strip_prefix('*') {
            file.ends_with(suffix)
        } else {
            file == pattern
        }
    };
    patterns.iter().any(|p| match split_qualified(p) {
        (None, bare) => one(bare),
        (Some(m), scoped) => member == Some(m) && one(scoped),
    })
}


/// Every code file of the project, root-relative: the tree minus VCS and
/// build dirs, the `archi/` tree (model, docs, archive, journal),
/// `.arch` sources — code is what the model is *about*, not the model —
/// and whatever the project's `[audit] exclude` patterns mute.
fn code_files(root: &Path) -> Vec<String> {
    let patterns = scan_exclusions(root);
    walk_code_files(root, None, &patterns, false)
}

/// A member's code files, member-root-relative: the same walk with the
/// boundary scoped to the member — and any subtree holding an
/// `archi.toml` skipped whole: that is someone else's project, not this
/// one's code.
fn member_code_files(project_root: &Path, member_root: &Path, member: &str) -> Vec<String> {
    let patterns = scan_exclusions(project_root);
    walk_code_files(member_root, Some(member), &patterns, true)
}

fn walk_code_files(
    root: &Path,
    member: Option<&str>,
    patterns: &[String],
    guard_nested_projects: bool,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if guard_nested_projects && dir != *root && dir.join("archi.toml").is_file() {
            continue;
        }
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
                if !excluded_in(member, &rel, patterns) {
                    out.push(rel);
                }
            }
        }
    }
    out.sort();
    out
}

/// The files git says changed since a rev, rebased into the member's
/// frame — the `--since` fast path. Git speaks top-level-relative paths;
/// every comparison crosses through the rebase
/// (`archi/requirements/multi-repo/git-speaks-from-its-own-root`).
fn changed_files(
    ctx: &crate::members::GitContext,
    rev: &str,
) -> Result<BTreeSet<String>, String> {
    let scope = if ctx.prefix.is_empty() { "." } else { ctx.prefix.as_str() };
    let out = Command::new("git")
        .arg("-C")
        .arg(&ctx.top)
        .args(["diff", "--name-only", rev, "--", scope])
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
        .filter_map(|l| ctx.rebase(l))
        .collect())
}

/// Whether `rev` names a commit present in this member's object database.
/// A baseline is a bare SHA archi holds no ref for, so a member can collect it
/// (gc after a deleted branch, a rewrite, a shallow clone that never fetched
/// it); the audit probes before it diffs so a gone floor degrades one member
/// instead of aborting the scan
/// (`archi/requirements/multi-repo/an-unresolvable-baseline-says-so`).
fn commit_present(ctx: &crate::members::GitContext, rev: &str) -> bool {
    crate::gitcmd::out(&ctx.top, &["cat-file", "-e", &format!("{rev}^{{commit}}")]).is_some()
}

// ---- audit -----------------------------------------------------------------

/// An advisory audit finding (`archi/requirements/code-link/the-audit-inverts-coverage.md`).
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
        /// Which rule produced the row — the reader decides what a decayed
        /// guess is worth against a decayed claim.
        rule: Rule,
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
                rule,
            } => {
                write!(
                    f,
                    "decayed evidence: {id} {rule} ({spec} ← {anchor}) — confidence \
                     {confidence:.2} is below the floor; confirm or retire"
                )
            }
        }
    }
}

/// Options of `archi link audit`.
#[derive(Default)]
pub struct AuditOptions {
    /// Delta source override, `[<member>=]<rev>` with bare rev meaning
    /// home; defaults to each member's baseline in the latest version.
    pub since: Option<String>,
    /// Sweep this scope's nodes for asserted-link coverage.
    pub scope: Option<String>,
    /// Audit only this member's delta (`home` for the project's own
    /// repository).
    pub repo: Option<String>,
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
    /// Of them, produced by inference from shared terms.
    pub inferred: usize,
    /// Of them, produced from a writer's declaration.
    pub declared: usize,
    /// Of them, produced by hand.
    pub authored: usize,
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
        inferred: folded.live.iter().filter(|l| l.rule == Rule::Inferred).count(),
        declared: folded.live.iter().filter(|l| l.rule == Rule::Declared).count(),
        authored: folded.live.iter().filter(|l| l.rule == Rule::Authored).count(),
        findings: Vec::new(),
        notes: folded
            .absorbed
            .iter()
            .map(|n| format!("journal: {n}"))
            .collect(),
        pruned: Vec::new(),
    };

    // Dark deltas, per member: every hunk since that member's delta source
    // either lands on a linked span or is unaccounted for. Each member
    // degrades alone — a missing baseline or absent checkout narrows the
    // scan and says so, never silently
    // (`archi/requirements/multi-repo/scans-see-every-mapped-member`).
    let roots = Roots::resolve(root)?;
    let scope_repo = opts.repo.as_deref().map(scope_member);
    let over = opts.since.as_deref().map(parse_since);
    let baselines = latest_version_baselines(root)?;
    let patterns = scan_exclusions(root);
    for m in &roots.set().members {
        if scope_repo.is_some_and(|s| s != m.name) {
            continue;
        }
        let is_home = m.name == crate::members::HOME;
        let label = if is_home { "home" } else { m.name.as_str() };
        let member = (!is_home).then_some(m.name.as_str());
        let Some(mroot) = &m.root else {
            report.notes.push(format!(
                "`{label}` is unreachable here — its delta is unaudited on this machine \
                 (`archi repo map {label} <dir>`)"
            ));
            continue;
        };
        let (rev, from_baseline) = match &over {
            Some((om, rev)) if *om == m.name => (Some(rev.clone()), false),
            _ => match baselines.get(&m.name) {
                Some((sha, born)) => {
                    if *born == versions::Born::Anchor {
                        report.notes.push(format!(
                            "`{label}`'s baseline is anchor-born — the span between the save \
                             and the anchor is unaudited"
                        ));
                    }
                    (Some(sha.clone()), true)
                }
                None => (None, false),
            },
        };
        let Some(rev) = rev else {
            report.notes.push(if is_home && roots.set().is_single() {
                // The memberless project: today's note, byte for byte.
                "no delta source: commit the tree and run `archi version anchor` so the latest \
                 version gains commit provenance, or pass --since <rev>"
                    .to_string()
            } else if is_home {
                "no delta source for home: commit the tree and run `archi version anchor` so \
                 the latest version gains commit provenance, or pass --since <rev>"
                    .to_string()
            } else {
                format!(
                    "no delta source for `{label}`: commit it and run `archi version anchor \
                     --repo {label}`, or pass --since {label}=<rev>"
                )
            });
            continue;
        };
        let Some(ctx) = crate::members::GitContext::of(mroot) else {
            report.notes.push(format!("`{label}` is not a git work tree — its delta is unaudited"));
            continue;
        };
        // The delta floor must still be an object here. A baseline is a bare
        // SHA the record holds no ref for, so a member is free to collect it —
        // gc after a deleted branch, a rewrite, a shallow clone that never
        // fetched it. Probe before the diff: a gone floor narrows this member's
        // scan and says so, never a `git diff` failure that aborts the others
        // (`archi/requirements/multi-repo/an-unresolvable-baseline-says-so`).
        if !commit_present(&ctx, &rev) {
            let short = &rev[..rev.len().min(7)];
            report.notes.push(if from_baseline {
                format!(
                    "`{label}`'s baseline `{short}` does not resolve here — the commit is absent \
                     (collected, rewritten, or a shallow clone); its delta is unaudited"
                )
            } else {
                format!(
                    "`{label}`'s `--since` rev `{short}` does not resolve here — its delta is \
                     unaudited"
                )
            });
            continue;
        }
        for (file, start, end) in delta_hunks(&ctx, member, &patterns, &rev)? {
            if let Some(finding) =
                unaccounted(mroot, member, &folded.live, &file, start, end)
            {
                report.findings.push(finding);
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

    // Decayed evidence: derived confidence below the floor. An unreachable
    // member is no observation — its links are neither graded nor pruned.
    let mut slots = Slots::new(root);
    let mut decayed = Vec::new();
    for link in folded.live.iter().filter(|l| l.standing == Standing::Evidence) {
        let checked = check_link(root, model, &roots, &mut slots, link.clone())?;
        if matches!(checked.state, State::Unreachable { .. }) {
            continue;
        }
        let confidence = confidence(link, &checked.state);
        if confidence < CONFIDENCE_FLOOR {
            decayed.push(link.id.clone());
            report.findings.push(AuditFinding::DecayedEvidence {
                id: link.id.clone(),
                spec: link.spec.to_string(),
                anchor: link.anchor.to_string(),
                confidence,
                rule: link.rule,
            });
        }
    }
    if opts.prune && !decayed.is_empty() {
        retire(root, &decayed)?;
        report.pruned = decayed;
    }
    Ok(report)
}

/// The latest archived version's delta sources, per member: home's
/// `commit` field beside the `commits` baselines, each with how it was
/// born — the audit words an anchor-born window honestly.
fn latest_version_baselines(
    root: &Path,
) -> Result<BTreeMap<String, (String, versions::Born)>, String> {
    let mut out = BTreeMap::new();
    if let Some(archive) = Archive::open(root)?
        && let Some(entry) = archive.entries().last()
    {
        if let Some(c) = &entry.commit {
            out.insert(
                crate::members::HOME.to_string(),
                (c.clone(), versions::Born::Save),
            );
        }
        for (member, b) in &entry.commits {
            out.insert(member.clone(), (b.sha.clone(), b.born));
        }
    }
    Ok(out)
}

/// New-side hunks of one member's code delta since a rev: `git diff` plus
/// untracked files, every path rebased from git's top level into the
/// member's frame before any boundary test — `archi/` (home), `.arch`
/// sources and the `[audit] exclude` patterns muted, the same boundary
/// `code_files` walks. Member-relative paths out.
fn delta_hunks(
    ctx: &crate::members::GitContext,
    member: Option<&str>,
    patterns: &[String],
    rev: &str,
) -> Result<Vec<(String, usize, usize)>, String> {
    let git = |args: &[&str]| -> Result<String, String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(&ctx.top)
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
    let scope = if ctx.prefix.is_empty() { "." } else { ctx.prefix.as_str() };
    let is_code = |file: &str| {
        let manifest = match member {
            // Home: today's boundary, byte for byte.
            None => file == "archi.toml",
            // A member: any nested manifest marks someone else's project.
            Some(_) => file == "archi.toml" || file.ends_with("/archi.toml"),
        };
        (member.is_some() || !file.starts_with("archi/"))
            && !file.ends_with(".arch")
            && !manifest
            && !excluded_in(member, file, patterns)
    };
    let mut hunks = Vec::new();
    let diff = git(&["diff", "--unified=0", "--no-color", rev, "--", scope])?;
    let mut current: Option<String> = None;
    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            current = ctx.rebase(path);
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
    let status = git(&["status", "--porcelain=v1", "--untracked-files=all", "--", scope])?;
    for line in status.lines() {
        if let Some(top_relative) = line.strip_prefix("?? ")
            && let Some(file) = ctx.rebase(top_relative)
            && is_code(&file)
            && let Ok(text) = fs::read_to_string(ctx.top.join(top_relative))
        {
            hunks.push((file, 1, text.lines().count().max(1)));
        }
    }
    Ok(hunks)
}

/// Whether a hunk is claimed by some live link: a file anchor claims the
/// whole file; a symbol anchor claims its item's current span. `file` is
/// member-relative; links match within the member and findings render the
/// qualified path.
fn unaccounted(
    member_root: &Path,
    member: Option<&str>,
    live: &[Link],
    file: &str,
    start: usize,
    end: usize,
) -> Option<AuditFinding> {
    let on_file: Vec<&Link> = live
        .iter()
        .filter(|l| l.anchor.repo.as_deref() == member && l.anchor.file == file)
        .collect();
    if on_file.iter().any(|l| l.anchor.symbol.is_none()) {
        return None;
    }
    let canonical = fs::read_to_string(member_root.join(file))
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
        file: qualify(member, file),
        start,
        end,
        symbol,
    })
}

// ---- rendering -------------------------------------------------------------

/// One link as a human line: id, kind/standing, the rule that made it,
/// spec ← anchor.
pub fn render_link(l: &Link) -> String {
    format!(
        "{}  {:8} {:8} {:10} {:8} {} ← {}",
        l.id,
        l.kind.describe(),
        l.standing.describe(),
        l.origin.to_string(),
        l.rule.describe(),
        l.spec,
        l.anchor
    )
}

/// The verify report as human lines: one per link, then the tally.
pub fn render_verify(report: &VerifyReport) -> String {
    let mut out = String::new();
    for note in &report.absorbed {
        out.push_str(&format!("journal: {note}\n"));
    }
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
        "links: {} live ({} asserted, {} evidence; {} declared, {} inferred, {} authored)\n",
        report.live,
        report.asserted,
        report.evidence,
        report.declared,
        report.inferred,
        report.authored
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
        fs::create_dir_all(dir.join("archi/src")).unwrap();
        fs::create_dir_all(dir.join("code")).unwrap();
        fs::write(dir.join("archi.toml"), "[project]\nname = \"t\"\n").unwrap();
        fs::write(dir.join("archi/src").join("model.arch"), MODEL).unwrap();
        fs::write(dir.join("code").join("auth.rs"), AUTH_RS).unwrap();
        dir
    }

    const MODEL: &str = "def conn wire := * -> *\n\
                         def node Auth:\n  port store\n\
                         def node Vault:\n  port inn\n\
                         Auth.store wire Vault.inn\n";

    const AUTH_RS: &str = "pub struct Vault {\n    salted: Vec<u8>,\n}\n\n\
                           impl Vault {\n    pub fn persist(&mut self, hash: &[u8]) {\n        self.salted.extend(hash);\n    }\n}\n";

    /// One world fact, whole: the record the docs pass reads and the
    /// `Scenarios` block a ref addresses into.
    const FACT: &str = "\
---
covers: []
sources: []
uses: []
---

# Users open the app on a train

The carriage drops the network for minutes at a time, so a call that must reach
the server fails for a reason the user cannot fix.

## What people do instead

Riders load the page at the platform and redo the trip's work when they
forget.

## Scenarios

### the app opens with no network

Given the device has no network
When the user opens the app
Then the last synced view appears
";

    /// A second scenario of the same fact — the sibling whose motion a link
    /// on the first scenario must not feel
    /// (`archi/requirements/world-facts/the-digest-witnesses-one-scenario.md`).
    const SIBLING: &str = "
### the app opens on a slow line

Given the device has one bar of signal
When the user opens the app
Then the view arrives late
";

    const FACT_SLUG: &str = "users-open-the-app-on-a-train";
    const SCENARIO: &str = "the app opens with no network";

    /// Write the fact into the tree — the live file the resolution reads.
    fn write_fact(root: &Path, text: &str) {
        let path = root.join(fact_file(FACT_SLUG));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn model_of(root: &Path) -> Workspace {
        modeling_lang::source::compile_project(root)
            .unwrap_or_else(|f| panic!("test model failed to compile:\n{}", f.render()))
            .workspace
    }

    /// One link as the verify graded it — state, gate and note together.
    fn checked_of(root: &Path, ws: &Workspace, id: &str) -> Checked {
        verify(root, ws.model(), &VerifyOptions::default())
            .unwrap()
            .checked
            .into_iter()
            .find(|c| c.link.id == id)
            .unwrap_or_else(|| panic!("no `{id}` in the report"))
    }

    fn state_of(root: &Path, ws: &Workspace, id: &str) -> (State, bool) {
        let c = checked_of(root, ws, id);
        (c.state, c.failing)
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
        // Ids: dense readable sequence, content-suffixed so parallel
        // branches cannot mint the same id (archi/requirements/self-hosting/parallel-editing-discipline.md).
        assert!(lit.id.starts_with("l0001-"), "{}", lit.id);
        assert!(ind.id.starts_with("l0002-"), "{}", ind.id);
        assert_ne!(lit.id, ind.id);
        assert_eq!(lit.standing, Standing::Asserted);
        assert_eq!(lit.birth.spans[0].file, "code/auth.rs");

        assert_eq!(state_of(&root, &ws, &lit.id), (State::Clean, false));

        // Reformatting is not drift: canonical tokens are the identity.
        fs::write(
            root.join("code/auth.rs"),
            AUTH_RS.replace(
                "pub fn persist(&mut self, hash: &[u8]) {\n        self.salted.extend(hash);\n    }",
                "// persists a salted hash\n    pub fn persist(&mut self,\n                   hash: &[u8]) {\n        self.salted.extend(hash);\n    }",
            ),
        )
        .unwrap();
        assert_eq!(state_of(&root, &ws, &lit.id), (State::Clean, false));

        // A body edit drifts the literal link and fails it; the indirect
        // link's watched interface holds.
        fs::write(
            root.join("code/auth.rs"),
            AUTH_RS.replace("self.salted.extend(hash);", "self.salted = hash.to_vec();"),
        )
        .unwrap();
        assert_eq!(state_of(&root, &ws, &lit.id), (State::Drifted, true));
        assert_eq!(state_of(&root, &ws, &ind.id), (State::Clean, false));

        // A signature edit drifts the indirect link too.
        fs::write(
            root.join("code/auth.rs"),
            AUTH_RS.replace("persist(&mut self, hash: &[u8])", "persist(&mut self, hash: Vec<u8>)"),
        )
        .unwrap();
        assert_eq!(state_of(&root, &ws, &ind.id), (State::Drifted, false));

        // Repin accepts the drift: the projection rewrites, birth stands.
        let before = ls(&root, None, false).unwrap()[0].birth.clone();
        repin(&root, &lit.id, None).unwrap();
        assert_eq!(state_of(&root, &ws, &lit.id), (State::Clean, false));
        assert_eq!(ls(&root, None, false).unwrap()[0].birth, before);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn moves_are_candidates_and_deletions_are_missing() {
        let root = temp_project();
        let ws = model_of(&root);
        let l = add(
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
        let (state, failing) = state_of(&root, &ws, &l.id);
        assert!(
            matches!(&state, State::Moved { file, exact: true, .. } if file == "code/store.rs"),
            "{state:?}"
        );
        assert!(!failing, "moved has a candidate; missing is what fails");

        repin(&root, &l.id, Some("code/store.rs#Vault::persist")).unwrap();
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));

        fs::remove_dir_all(root.join("code").join("store.rs")).ok();
        fs::remove_file(root.join("code").join("store.rs")).ok();
        let (state, failing) = state_of(&root, &ws, &l.id);
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
        let edge = add(
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
        let pinned =
            add(&root, ws.model(), "Vault@v0001", "code/auth.rs", LinkKind::Indirect).unwrap();
        let working = add(&root, ws.model(), "Vault", "code/auth.rs", LinkKind::Indirect).unwrap();
        fs::write(
            root.join("archi/src").join("model.arch"),
            MODEL.replace("Vault", "Safe"),
        )
        .unwrap();
        let ws2 = model_of(&root);
        let report = verify(&root, ws2.model(), &VerifyOptions::default()).unwrap();
        let by_id = |id: &str| report.checked.iter().find(|c| c.link.id == id).unwrap();
        assert_eq!(by_id(&pinned.id).state, State::Clean, "pinned slot holds");
        assert_eq!(by_id(&working.id).state, State::SpecDrifted);
        assert!(by_id(&working.id).failing);
        // The edge ref names Vault too: drifted with it.
        assert_eq!(by_id(&edge.id).state, State::SpecDrifted);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_spec_ref_resolves_a_port_path() {
        let root = temp_project();
        let ws = model_of(&root);

        // A port path resolves where E_MODEL_REF was raised before, exactly
        // where a canonical edge already does — a link and a satisfaction name
        // one vocabulary (satisfaction-names-the-interface).
        let port = add(&root, ws.model(), "Auth.store", "code/auth.rs", LinkKind::Indirect)
            .expect("a declared port resolves");
        assert_eq!(port.spec.path, "Auth.store");

        // It verifies and grades like any other link.
        let report = verify(&root, ws.model(), &VerifyOptions::default()).unwrap();
        let checked = report.checked.iter().find(|c| c.link.id == port.id).unwrap();
        assert_eq!(checked.state, State::Clean);

        // An undeclared port still refuses to mint.
        let err = add(&root, ws.model(), "Auth.nope", "code/auth.rs", LinkKind::Indirect)
            .unwrap_err();
        assert!(err.contains("E_MODEL_REF"), "{err}");

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
                    rule: Rule::Inferred,
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
                    rule: Rule::Inferred,
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
                    rule: Rule::Inferred,
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
                    rule: Rule::Inferred,
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
        let first = add(&root, ws.model(), "Vault", "code/auth.rs", LinkKind::Literal).unwrap();
        add(&root, ws.model(), "Auth", "code/auth.rs", LinkKind::Literal).unwrap();
        retire(&root, &[first.id.clone()]).unwrap();
        assert!(retire(&root, &[first.id.clone()]).is_err(), "already retired");
        let live = ls(&root, None, false).unwrap();
        assert_eq!(live.len(), 1);
        // The sequence counts past retirements: never reused, still readable.
        let third = add(&root, ws.model(), "Vault", "code/auth.rs", LinkKind::Indirect).unwrap();
        assert!(third.id.starts_with("l0003-"), "{}", third.id);
        // Bulk by spec.
        let retired = retire_spec(&root, "Vault").unwrap();
        assert_eq!(retired, vec![third.id.clone()]);
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

    #[test]
    fn refs_parse_their_member_and_render_it_back() {
        let a = Anchor::parse("backend//src/api.rs#Handler::serve").unwrap();
        assert_eq!(
            (a.repo.as_deref(), a.file.as_str(), a.symbol.as_deref()),
            (Some("backend"), "src/api.rs", Some("Handler::serve"))
        );
        assert_eq!(a.to_string(), "backend//src/api.rs#Handler::serve");
        assert_eq!(a.qualified_file(), "backend//src/api.rs");

        // Unqualified stays home — and folds identically to yesterday's parse.
        let bare = Anchor::parse("src/api.rs#serve").unwrap();
        assert_eq!(bare.repo, None);
        assert_eq!(bare.to_string(), "src/api.rs#serve");

        // A pre-member journal event replays with its anchor at home.
        let old: Anchor =
            serde_json::from_str(r#"{"file":"src/api.rs","symbol":"serve"}"#).unwrap();
        assert_eq!(old.repo, None);
        assert_eq!(old, bare);

        for bad in ["//src/api.rs", "backend//", "backend//src/api.rs#"] {
            assert!(Anchor::parse(bad).is_err(), "`{bad}` must refuse");
        }
    }

    #[test]
    fn a_scenario_ref_resolves_and_the_digest_decides_it() {
        let root = temp_project();
        let ws = model_of(&root);
        write_fact(&root, FACT);

        // The world form of a spec ref: a fact's slug, then a scenario name
        // inside it — read out of the tree the docs pass reads, never out of
        // a pinned render.
        let l = add(
            &root,
            ws.model(),
            &format!("{FACT_SLUG}#{SCENARIO}"),
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .expect("a scenario the fact holds resolves");
        assert_eq!(l.spec.path, format!("{FACT_SLUG}#{SCENARIO}"));
        assert_eq!(l.spec.version, None, "a fact stands in one slot: now");
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));

        // The step text is not the address: rewording one leaves the ref
        // resolving. The digest decides the link from here — the pair was
        // witnessed, and a witness that no longer matches is a failure
        // (archi/requirements/world-facts/a-scenario-link-binds-two-hashes.md).
        write_fact(
            &root,
            &FACT.replace(
                "Then the last synced view appears",
                "Then the view synced last is on screen",
            ),
        );
        let c = checked_of(&root, &ws, &l.id);
        assert_ne!(c.state, State::SpecDrifted, "the address still resolves");
        assert_eq!(c.state.describe(), "scenario-drifted");
        assert!(c.failing);

        fs::remove_dir_all(&root).unwrap();
    }

    /// The pair, side by side: each digest moves alone and both move
    /// together, and every failure names the side that parted
    /// (archi/requirements/world-facts/a-scenario-link-binds-two-hashes.md).
    #[test]
    fn a_witnessed_pair_names_the_side_that_moved() {
        let root = temp_project();
        let ws = model_of(&root);
        write_fact(&root, FACT);
        let l = add(
            &root,
            ws.model(),
            &format!("{FACT_SLUG}#{SCENARIO}"),
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();
        let repair = format!("`link repin {}` binds the pair again", l.id);

        // An unchanged pair: both witnesses hold.
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));

        // The scenario side alone: a reworded step.
        write_fact(
            &root,
            &FACT.replace(
                "Then the last synced view appears",
                "Then the view synced last is on screen",
            ),
        );
        let c = checked_of(&root, &ws, &l.id);
        assert_eq!(c.state.describe(), "scenario-drifted");
        assert!(c.failing);
        assert_eq!(
            c.note.as_deref(),
            Some(
                format!(
                    "the scenario side moved: `archi/world/facts/{FACT_SLUG}.md` no longer matches the \
                     digest this link recorded; the code side holds — {repair}"
                )
                .as_str()
            )
        );

        // `repin` binds the pair again, and the next verify is clean.
        repin(&root, &l.id, None).unwrap();
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));

        // The code side alone: the anchored item is rewritten.
        fs::write(
            root.join("code/auth.rs"),
            AUTH_RS.replace("self.salted.extend(hash);", "self.salted = hash.to_vec();"),
        )
        .unwrap();
        let c = checked_of(&root, &ws, &l.id);
        assert_eq!(c.state, State::Drifted);
        assert!(c.failing);
        assert_eq!(
            c.note.as_deref(),
            Some(
                format!(
                    "the code side moved: `code/auth.rs#Vault::persist` no longer matches the \
                     hash this link recorded; the scenario side holds — {repair}"
                )
                .as_str()
            )
        );

        // Both sides: the failure names both.
        write_fact(
            &root,
            &FACT.replace(
                "Then the last synced view appears",
                "Then the view is on screen",
            ),
        );
        let c = checked_of(&root, &ws, &l.id);
        assert_eq!(c.state.describe(), "pair-drifted");
        assert!(c.failing);
        assert_eq!(
            c.note.as_deref(),
            Some(
                format!(
                    "both sides moved: `archi/world/facts/{FACT_SLUG}.md` and \
                     `code/auth.rs#Vault::persist` — {repair}"
                )
                .as_str()
            )
        );

        // One repin binds both sides; the birth record stands.
        let before = ls(&root, None, false).unwrap()[0].birth.clone();
        repin(&root, &l.id, None).unwrap();
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));
        assert_eq!(ls(&root, None, false).unwrap()[0].birth, before);

        fs::remove_dir_all(&root).unwrap();
    }

    /// The digest witnesses the scenario the ref addresses: a sibling
    /// reworded in the same fact, and a scenario added beside it, move
    /// nothing this link stands on
    /// (`archi/requirements/world-facts/the-digest-witnesses-one-scenario.md`).
    #[test]
    fn a_sibling_scenario_moves_and_the_link_stays_clean() {
        let root = temp_project();
        let ws = model_of(&root);
        let both = format!("{FACT}{SIBLING}");
        write_fact(&root, &both);
        let l = add(
            &root,
            ws.model(),
            &format!("{FACT_SLUG}#{SCENARIO}"),
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));

        // The sibling's step is reworded.
        let reworded = both.replace("Then the view arrives late", "Then the view takes its time");
        write_fact(&root, &reworded);
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));

        // A third scenario joins the fact: the condition was elaborated, and
        // an elaborated condition must not cost its links anything.
        write_fact(
            &root,
            &format!(
                "{reworded}\n### the app opens in a tunnel\n\n\
                 Given the device has no network for minutes\n\
                 When the user opens the app\nThen the last synced view appears\n"
            ),
        );
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));

        fs::remove_dir_all(&root).unwrap();
    }

    /// The addressed scenario moves and the pair parts: the grade is the
    /// scenario side, the message names it, and one `repin` binds the narrow
    /// pair again (`the-digest-witnesses-one-scenario`).
    #[test]
    fn a_step_of_the_addressed_scenario_moves_and_the_note_names_the_scenario_side() {
        let root = temp_project();
        let ws = model_of(&root);
        let both = format!("{FACT}{SIBLING}");
        write_fact(&root, &both);
        let l = add(
            &root,
            ws.model(),
            &format!("{FACT_SLUG}#{SCENARIO}"),
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();

        write_fact(
            &root,
            &both.replace(
                "Then the last synced view appears",
                "Then the view synced last is on screen",
            ),
        );
        let c = checked_of(&root, &ws, &l.id);
        assert_eq!(c.state.describe(), "scenario-drifted");
        assert!(c.failing);
        assert_eq!(
            c.note.as_deref(),
            Some(
                format!(
                    "the scenario side moved: `archi/world/facts/{FACT_SLUG}.md` no longer matches the \
                     digest this link recorded; the code side holds — `link repin {}` binds the \
                     pair again",
                    l.id
                )
                .as_str()
            )
        );

        repin(&root, &l.id, None).unwrap();
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));

        fs::remove_dir_all(&root).unwrap();
    }

    /// A renamed address and a moved witness are two failures, and they keep
    /// reading differently: the name is the address, so a rename unresolves
    /// the ref and never reaches the digest at all
    /// (`the-digest-witnesses-one-scenario`).
    #[test]
    fn renaming_the_addressed_scenario_is_spec_drift_not_a_moved_digest() {
        let root = temp_project();
        let ws = model_of(&root);
        let both = format!("{FACT}{SIBLING}");
        write_fact(&root, &both);
        let l = add(
            &root,
            ws.model(),
            &format!("{FACT_SLUG}#{SCENARIO}"),
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();

        let renamed = "the app opens off the network";
        write_fact(
            &root,
            &both.replace(&format!("### {SCENARIO}"), &format!("### {renamed}")),
        );
        let c = checked_of(&root, &ws, &l.id);
        assert_eq!(c.state, State::SpecDrifted);
        assert_ne!(c.state.describe(), "scenario-drifted");
        assert!(c.failing);
        let note = c.note.unwrap_or_default();
        assert!(note.contains("names no scenario") && note.contains("--spec"), "{note}");

        // The repair is the one the note names, and the pair binds to the
        // scenario under its new name.
        repin_spec(&root, &l.id, &format!("{FACT_SLUG}#{renamed}")).unwrap();
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));

        fs::remove_dir_all(&root).unwrap();
    }

    /// The links journaled while the digest covered the whole block — six of
    /// them in this repository — keep the grain they recorded. A recorded
    /// digest that is the fact's whole story as it stands now is not motion,
    /// so nothing false-fails on the day the grain narrows; the first `repin`
    /// binds the link to the scenario it addresses, and the narrow grain
    /// rules it from there (`the-digest-witnesses-one-scenario`).
    #[test]
    fn a_link_recorded_over_the_whole_block_holds_until_a_repin_narrows_it() {
        let root = temp_project();
        let ws = model_of(&root);
        let both = format!("{FACT}{SIBLING}");
        write_fact(&root, &both);
        let whole = fact_digest(&root, FACT_SLUG, None).expect("the fact stands in the tree");
        let anchor = Anchor::parse("code/auth.rs#Vault::persist").unwrap();
        let resolved = resolve_anchor(&root, &anchor).unwrap();
        let old = serde_json::json!({
            "event": "add",
            "link": {
                "id": "l0001",
                "spec": {"ref": format!("{FACT_SLUG}#{SCENARIO}")},
                "anchor": {"file": "code/auth.rs", "symbol": "Vault::persist"},
                "kind": "literal",
                "standing": "asserted",
                "origin": {"kind": "authored"},
                "birth": {
                    "created": now(),
                    "spans": [{
                        "file": "code/auth.rs",
                        "start": resolved.span.start,
                        "end": resolved.span.end,
                        "hash": &resolved.span.hash,
                    }],
                },
                "pins": {
                    "canonicalizer": &resolved.pins.canonicalizer,
                    "interface": &resolved.pins.interface,
                    "body": &resolved.pins.body,
                    "scenario": &whole,
                },
            },
        })
        .to_string();
        fs::create_dir_all(root.join("archi").join("links")).unwrap();
        fs::write(journal_path(&root), format!("{old}\n")).unwrap();
        assert_eq!(state_of(&root, &ws, "l0001"), (State::Clean, false));

        // Its own grain still rules it: a sibling moving fails it, exactly as
        // it failed the day it was written. Narrowing costs it nothing and
        // gains it nothing until somebody looks.
        let reworded = both.replace("Then the view arrives late", "Then the view takes its time");
        write_fact(&root, &reworded);
        assert_eq!(checked_of(&root, &ws, "l0001").state.describe(), "scenario-drifted");

        // The repin binds the scenario the ref addresses, and the sibling
        // stops reaching the link.
        repin(&root, "l0001", None).unwrap();
        write_fact(
            &root,
            &reworded.replace("Then the view takes its time", "Then the view is late"),
        );
        assert_eq!(state_of(&root, &ws, "l0001"), (State::Clean, false));

        fs::remove_dir_all(&root).unwrap();
    }

    /// The regression that matters: every link this repository carries names
    /// an element path, and an element path has no story to witness — the
    /// journal line and the grade are yesterday's.
    #[test]
    fn an_element_link_binds_one_side_and_grades_as_it_did() {
        let root = temp_project();
        let ws = model_of(&root);
        let l = add(
            &root,
            ws.model(),
            "Vault",
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();
        let journal = fs::read_to_string(journal_path(&root)).unwrap();
        assert!(!journal.contains("scenario"), "{journal}");
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));

        fs::write(
            root.join("code/auth.rs"),
            AUTH_RS.replace("self.salted.extend(hash);", "self.salted = hash.to_vec();"),
        )
        .unwrap();
        let c = checked_of(&root, &ws, &l.id);
        assert_eq!(c.state, State::Drifted);
        assert!(c.failing);
        assert_eq!(c.note, None, "an element link has no side to name");

        repin(&root, &l.id, None).unwrap();
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));
        let journal = fs::read_to_string(journal_path(&root)).unwrap();
        assert!(!journal.contains("scenario"), "{journal}");

        fs::remove_dir_all(&root).unwrap();
    }

    /// A scenario link journaled before the pair was witnessed carries no
    /// digest. Absence is not motion: it replays, grades and reads exactly as
    /// it did, and the first `repin` binds the pair.
    #[test]
    fn a_link_with_no_recorded_digest_never_reads_as_moved() {
        let root = temp_project();
        let ws = model_of(&root);
        write_fact(&root, FACT);
        let anchor = Anchor::parse("code/auth.rs#Vault::persist").unwrap();
        let resolved = resolve_anchor(&root, &anchor).unwrap();
        // The line as the journal held it before the pair was witnessed:
        // written out by hand, with no key for the scenario side at all.
        let old = serde_json::json!({
            "event": "add",
            "link": {
                "id": "l0001",
                "spec": {"ref": format!("{FACT_SLUG}#{SCENARIO}")},
                "anchor": {"file": "code/auth.rs", "symbol": "Vault::persist"},
                "kind": "literal",
                "standing": "asserted",
                "origin": {"kind": "authored"},
                "birth": {
                    "created": now(),
                    "spans": [{
                        "file": "code/auth.rs",
                        "start": resolved.span.start,
                        "end": resolved.span.end,
                        "hash": &resolved.span.hash,
                    }],
                },
                "pins": {
                    "canonicalizer": &resolved.pins.canonicalizer,
                    "interface": &resolved.pins.interface,
                    "body": &resolved.pins.body,
                },
            },
        })
        .to_string();
        fs::create_dir_all(root.join("archi").join("links")).unwrap();
        fs::write(journal_path(&root), format!("{old}\n")).unwrap();
        assert_eq!(ls(&root, None, false).unwrap().len(), 1, "the line replays");
        assert_eq!(state_of(&root, &ws, "l0001"), (State::Clean, false));

        // The story moves and the link says nothing: it witnessed no story.
        write_fact(
            &root,
            &FACT.replace(
                "Then the last synced view appears",
                "Then the view synced last is on screen",
            ),
        );
        assert_eq!(state_of(&root, &ws, "l0001"), (State::Clean, false));

        // The first repin binds the pair; from there the digest decides.
        repin(&root, "l0001", None).unwrap();
        write_fact(
            &root,
            &FACT.replace(
                "Then the last synced view appears",
                "Then the view is on screen",
            ),
        );
        let c = checked_of(&root, &ws, "l0001");
        assert_eq!(c.state.describe(), "scenario-drifted");
        assert!(c.failing);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_renamed_scenario_unresolves_and_repin_moves_the_link() {
        let root = temp_project();
        let ws = model_of(&root);
        write_fact(&root, FACT);
        let l = add(
            &root,
            ws.model(),
            &format!("{FACT_SLUG}#{SCENARIO}"),
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();

        // The name is the address: renaming the scenario unresolves the link,
        // and the note names the repair.
        let renamed = "the app opens off the network";
        write_fact(
            &root,
            &FACT.replace(&format!("### {SCENARIO}"), &format!("### {renamed}")),
        );
        let report = verify(&root, ws.model(), &VerifyOptions::default()).unwrap();
        let checked = report.checked.iter().find(|c| c.link.id == l.id).unwrap();
        assert_eq!(checked.state, State::SpecDrifted);
        assert!(checked.failing);
        assert!(
            checked.note.as_deref().is_some_and(|n| n.contains("repin")),
            "{:?}",
            checked.note
        );

        // `repin --spec` moves the link onto the new name; birth and the code
        // side stand — the name moved, the code did not.
        let before = ls(&root, None, false).unwrap()[0].clone();
        let moved = repin_spec(&root, &l.id, &format!("{FACT_SLUG}#{renamed}")).unwrap();
        assert_eq!(moved.spec.path, format!("{FACT_SLUG}#{renamed}"));
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));
        let after = ls(&root, None, false).unwrap()[0].clone();
        assert_eq!(after.birth, before.birth);
        assert_eq!(after.anchor, before.anchor);
        // The code half of the witness stands, hash for hash. The scenario
        // half binds again: the name is part of the story the digest reads,
        // so a rename moves it and the repair rebinds it.
        assert_eq!(
            (
                &after.pins.canonicalizer,
                &after.pins.interface,
                &after.pins.body
            ),
            (
                &before.pins.canonicalizer,
                &before.pins.interface,
                &before.pins.body
            )
        );
        assert_ne!(after.pins.scenario, before.pins.scenario);

        // A name the fact does not hold refuses here as it refuses at add,
        // and an element path is not what this move repairs.
        let err = repin_spec(&root, &l.id, &format!("{FACT_SLUG}#no such walk")).unwrap_err();
        assert!(err.contains("E_MODEL_REF"), "{err}");
        let err = repin_spec(&root, &l.id, "Vault").unwrap_err();
        assert!(err.contains("<fact-slug>#<scenario name>"), "{err}");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_ambiguous_scenario_is_located_and_an_unheld_one_refuses() {
        let root = temp_project();
        let ws = model_of(&root);
        write_fact(&root, FACT);

        // A fact no file holds, and a name the fact does not hold: both
        // refuse where every other unresolvable ref refuses.
        let err = add(
            &root,
            ws.model(),
            &format!("no-such-fact#{SCENARIO}"),
            "code/auth.rs",
            LinkKind::Indirect,
        )
        .unwrap_err();
        assert!(err.contains("no world fact") && err.contains("E_MODEL_REF"), "{err}");
        let err = add(
            &root,
            ws.model(),
            &format!("{FACT_SLUG}#the train stops"),
            "code/auth.rs",
            LinkKind::Indirect,
        )
        .unwrap_err();
        assert!(err.contains("E_MODEL_REF"), "{err}");

        // Two scenarios of one name inside one fact: the address is
        // ambiguous, and the error locates the second one.
        let twin = format!(
            "{FACT}\n### {SCENARIO}\n\nGiven the device has no network\n\
             When the user opens the app\nThen the last synced view appears\n"
        );
        write_fact(&root, &twin);
        let err = add(
            &root,
            ws.model(),
            &format!("{FACT_SLUG}#{SCENARIO}"),
            "code/auth.rs",
            LinkKind::Indirect,
        )
        .unwrap_err();
        let second = twin
            .lines()
            .enumerate()
            .filter(|(_, l)| l.trim() == format!("### {SCENARIO}"))
            .map(|(i, _)| i + 1)
            .nth(1)
            .expect("two lines name the scenario");
        assert!(
            err.contains(&format!("archi/world/facts/{FACT_SLUG}.md:{second}")),
            "{err}"
        );

        fs::remove_dir_all(&root).unwrap();
    }

    /// The address is a `### ` heading, and the shape this one replaced holds
    /// no address at all: a `Scenario:` line names a scenario nowhere, and
    /// the form of such a block is `archi check`'s to report
    /// (`archi/requirements/world-facts/the-scenario-is-the-address-not-the-step.md`).
    #[test]
    fn a_heading_is_the_address_and_a_retired_scenario_line_is_not() {
        let root = temp_project();
        let ws = model_of(&root);

        // The retired shape, whole: the name stands on a `Scenario:` line.
        write_fact(
            &root,
            &FACT.replace(
                &format!("### {SCENARIO}"),
                &format!("Feature: Offline open\n\n  Scenario: {SCENARIO}"),
            ),
        );
        let err = add(
            &root,
            ws.model(),
            &format!("{FACT_SLUG}#{SCENARIO}"),
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap_err();
        assert!(
            err.contains("names no scenario") && err.contains("E_MODEL_REF"),
            "{err}"
        );

        // A deeper heading opens no scenario either: `### ` is the opener.
        write_fact(
            &root,
            &FACT.replace(&format!("### {SCENARIO}"), &format!("#### {SCENARIO}")),
        );
        assert!(
            add(
                &root,
                ws.model(),
                &format!("{FACT_SLUG}#{SCENARIO}"),
                "code/auth.rs#Vault::persist",
                LinkKind::Literal,
            )
            .is_err()
        );

        // The heading is the address, and the pair it binds verifies clean.
        write_fact(&root, FACT);
        let l = add(
            &root,
            ws.model(),
            &format!("{FACT_SLUG}#{SCENARIO}"),
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .expect("the heading names the scenario");
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn exclusion_patterns_scope_bare_everywhere_and_qualified_to_one_member() {
        let patterns = vec!["*.md".to_string(), "backend//vendor/".to_string()];
        // Bare patterns hold in every member.
        assert!(excluded_in(None, "README.md", &patterns));
        assert!(excluded_in(Some("backend"), "docs/x.md", &patterns));
        assert!(excluded_in(Some("web"), "notes.md", &patterns));
        // A qualified pattern holds in exactly its member.
        assert!(excluded_in(Some("backend"), "vendor/dep.rs", &patterns));
        assert!(!excluded_in(Some("web"), "vendor/dep.rs", &patterns));
        assert!(!excluded_in(None, "vendor/dep.rs", &patterns));
        // The member prefix never leaks into the path test.
        assert!(!excluded_in(Some("backend"), "src/vendor.rs", &patterns));
    }

    // ---- a requirement as an address ---------------------------------------

    const REQ_SLUG: &str = "the-vault-salts-what-it-stores";
    const REQ_INTENT: &str = "storage";

    /// One requirement where the discovery walks for it: an intent folder
    /// anchored by its own file, and the requirement beside it. The slug is
    /// the file stem, exactly as `plan verify` reads it for `owns:`.
    fn write_requirement(root: &Path, slug: &str) {
        let dir = root.join("archi/requirements").join(REQ_INTENT);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{REQ_INTENT}.md")),
            "# Storage\n\nWhat the vault keeps and how.\n",
        )
        .unwrap();
        fs::write(
            dir.join(format!("{slug}.md")),
            "---\nkind: functional\nsatisfied-by: [Vault]\ndeferred:\n---\n\n\
             # The vault salts what it stores\n\nEvery stored byte is salted first.\n",
        )
        .unwrap();
    }

    fn req_ref(slug: &str) -> String {
        format!("req:{slug}")
    }

    /// The column `ls` prints the producing rule in.
    fn rule_word(l: &Link) -> String {
        render_link(l)
            .split_whitespace()
            .nth(4)
            .expect("the rendered row has a rule column")
            .to_string()
    }

    /// The third shape resolves against the requirement set, and the two
    /// standing shapes are untouched — a bare slug still falls to the element
    /// branch and is refused in today's words
    /// (`archi/requirements/code-link/a-requirement-is-addressable-in-the-journal.md`).
    #[test]
    fn a_requirement_is_an_address_and_the_element_branch_is_untouched() {
        let root = temp_project();
        write_requirement(&root, REQ_SLUG);
        let ws = model_of(&root);

        let l = add(
            &root,
            ws.model(),
            &req_ref(REQ_SLUG),
            "code/auth.rs#Vault::persist",
            LinkKind::Indirect,
        )
        .expect("the requirement set holds the slug");
        assert_eq!(l.spec.to_string(), req_ref(REQ_SLUG));
        assert_eq!(l.spec.requirement(), Some(REQ_SLUG));
        assert_eq!(l.spec.scenario(), None, "a requirement is not a scenario");
        assert_eq!(state_of(&root, &ws, &l.id), (State::Clean, false));

        // A `req:` naming no requirement is refused, and the refusal names
        // the slug it could not find.
        let err = add(
            &root,
            ws.model(),
            "req:no-such-claim",
            "code/auth.rs#Vault::persist",
            LinkKind::Indirect,
        )
        .unwrap_err();
        assert!(err.contains("no-such-claim"), "{err}");
        assert!(err.contains("names no requirement"), "{err}");

        // A bare slug holds no prefix: it is an element ref, and its refusal
        // is today's, byte for byte.
        let err = add(
            &root,
            ws.model(),
            REQ_SLUG,
            "code/auth.rs#Vault::persist",
            LinkKind::Indirect,
        )
        .unwrap_err();
        assert_eq!(
            err,
            format!("`{REQ_SLUG}` names no element of the live model (E_MODEL_REF)")
        );

        // A requirement slug is stable across versions, so it takes no slot.
        let err = SpecRef::parse(&format!("req:{REQ_SLUG}@v0025")).unwrap_err();
        assert!(err.contains("carries no version pin"), "{err}");
        assert!(err.contains(REQ_SLUG), "{err}");
        // The element shape keeps its `@` handling.
        assert_eq!(
            SpecRef::parse("Vault@v0025").unwrap().version.as_deref(),
            Some("v0025")
        );

        fs::remove_dir_all(&root).unwrap();
    }

    /// The code side moving fails a `req:` link exactly as it fails an
    /// element link, and `repin` binds it again
    /// (`archi/requirements/code-link/a-requirement-is-addressable-in-the-journal.md`).
    #[test]
    fn a_requirement_link_fails_and_repins_as_an_element_link_does() {
        let root = temp_project();
        write_requirement(&root, REQ_SLUG);
        let ws = model_of(&root);
        let req = add(
            &root,
            ws.model(),
            &req_ref(REQ_SLUG),
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();
        let elem = add(
            &root,
            ws.model(),
            "Vault",
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();

        // The code moves out from under both rows at once, under a name the
        // candidate scan cannot follow.
        fs::write(
            root.join("code/vault.rs"),
            AUTH_RS.replace("persist", "store"),
        )
        .unwrap();
        fs::write(root.join("code/auth.rs"), "pub fn unrelated() {}\n").unwrap();
        let graded = state_of(&root, &ws, &req.id);
        assert_eq!(graded, state_of(&root, &ws, &elem.id), "one grader, one answer");
        assert_eq!(
            graded,
            (State::Missing, true),
            "the requirement link fails like the element one"
        );

        repin(&root, &req.id, Some("code/vault.rs#Vault::store")).unwrap();
        assert_eq!(state_of(&root, &ws, &req.id), (State::Clean, false));

        fs::remove_dir_all(&root).unwrap();
    }

    /// Retiring the requirement makes the link unresolvable — a state of its
    /// own, upstream of the code side being gone
    /// (`archi/requirements/code-link/a-requirement-is-addressable-in-the-journal.md`).
    #[test]
    fn a_retired_requirement_unresolves_and_that_is_not_missing() {
        let root = temp_project();
        write_requirement(&root, REQ_SLUG);
        let ws = model_of(&root);
        let req = add(
            &root,
            ws.model(),
            &req_ref(REQ_SLUG),
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();
        let elem = add(
            &root,
            ws.model(),
            "Vault",
            "code/auth.rs#Vault::persist",
            LinkKind::Literal,
        )
        .unwrap();

        fs::remove_file(
            root.join("archi/requirements")
                .join(REQ_INTENT)
                .join(format!("{REQ_SLUG}.md")),
        )
        .unwrap();
        let (state, failing) = state_of(&root, &ws, &req.id);
        assert_eq!(state, State::SpecDrifted);
        assert!(failing);
        assert_ne!(state, State::Missing, "unresolvable is not missing");

        // The code side leaving is the other state, and it still reads as it
        // always did.
        fs::remove_file(root.join("code/auth.rs")).unwrap();
        assert_eq!(state_of(&root, &ws, &elem.id), (State::Missing, true));

        fs::remove_dir_all(&root).unwrap();
    }

    /// Every row says which rule produced it, a row written before the field
    /// existed takes it from its origin with no migration, and the reverse
    /// view answers from declared and authored rows alone
    /// (`archi/requirements/code-link/the-journal-says-which-rule-made-a-row.md`).
    #[test]
    fn a_row_says_which_rule_made_it_and_an_unstamped_row_takes_it_from_its_origin() {
        let root = temp_project();
        write_requirement(&root, REQ_SLUG);
        let ws = model_of(&root);

        // By hand.
        let hand = add(
            &root,
            ws.model(),
            &req_ref(REQ_SLUG),
            "code/auth.rs#Vault::persist",
            LinkKind::Indirect,
        )
        .unwrap();
        assert_eq!(hand.rule, Rule::Authored);
        assert_eq!(rule_word(&hand), "authored");

        // From a declaration: the same mint, stamped with the rule that
        // produced it — the path capture reads a declaration through.
        let declared = mint(
            &root,
            ws.model(),
            &req_ref(REQ_SLUG),
            "code/auth.rs#Vault",
            LinkKind::Indirect,
            Rule::Declared,
            Origin::Captured { task: "t9".into() },
            Standing::Asserted,
        )
        .unwrap();
        assert_eq!(declared.rule, Rule::Declared);
        assert_eq!(rule_word(&declared), "declared");

        // A row journaled before the field existed carries no rule, and it
        // takes one from its origin — nothing was migrated. What capture
        // minted under the shared-term rule is the tool's guess; what `link
        // add` minted is a claim a person made by hand, and it stays one.
        let old = r#"{"event":"add","link":{"id":"__ID__","spec":{"ref":"__REQ__"},"anchor":{"file":"code/auth.rs","symbol":"Vault::persist"},"kind":"indirect","standing":"__STANDING__","origin":__ORIGIN__,"birth":{"created":"2020-01-01T00:00:00Z","spans":[]},"pins":{"canonicalizer":"rust-tok-v1","interface":"sha256:a","body":"sha256:b"}}}"#;
        let path = journal_path(&root);
        let mut text = fs::read_to_string(&path).unwrap();
        for (id, standing, origin) in [
            ("l0900-aaaaaa", "evidence", r#"{"kind":"captured","task":"t1"}"#),
            ("l0901-bbbbbb", "asserted", r#"{"kind":"authored"}"#),
        ] {
            text.push_str(
                &old.replace("__ID__", id)
                    .replace("__REQ__", &req_ref(REQ_SLUG))
                    .replace("__STANDING__", standing)
                    .replace("__ORIGIN__", origin),
            );
            text.push('\n');
        }
        fs::write(&path, text).unwrap();

        let rows = ls(&root, None, false).unwrap();
        let row = |id: &str| {
            rows.iter()
                .find(|l| l.id == id)
                .unwrap_or_else(|| panic!("the replayed row `{id}` folds"))
        };
        assert_eq!(row("l0900-aaaaaa").rule, Rule::Inferred);
        assert_eq!(rule_word(row("l0900-aaaaaa")), "inferred");
        assert_eq!(row("l0901-bbbbbb").rule, Rule::Authored);
        assert_eq!(rule_word(row("l0901-bbbbbb")), "authored");

        // The reverse view: what a person stood behind, and not the guesses.
        // A row `link add` minted before the field existed is still what a
        // person stood behind, so it answers here too.
        let view: Vec<&str> = ls(&root, Some(&req_ref(REQ_SLUG)), false)
            .unwrap()
            .iter()
            .map(|l| l.id.as_str())
            .map(|id| {
                if id == hand.id {
                    "hand"
                } else if id == declared.id {
                    "declared"
                } else if id == "l0901-bbbbbb" {
                    "old hand"
                } else {
                    "guess"
                }
            })
            .collect();
        assert_eq!(
            view,
            vec!["hand", "declared", "old hand"],
            "the inferred row is the only one left out"
        );

        // An element ref is not the reverse view: it lists what it always did.
        let all = ls(&root, Some("Vault"), false).unwrap();
        assert!(all.is_empty(), "no row hangs on `Vault` here");

        fs::remove_dir_all(&root).unwrap();
    }

    /// The audit carries the word on every line that names a row: the tally
    /// it opens with, and each finding that names a link
    /// (`archi/requirements/code-link/the-journal-says-which-rule-made-a-row.md`).
    #[test]
    fn the_audit_carries_the_producing_rule() {
        let root = temp_project();
        write_requirement(&root, REQ_SLUG);
        let ws = model_of(&root);
        add(
            &root,
            ws.model(),
            &req_ref(REQ_SLUG),
            "code/auth.rs#Vault::persist",
            LinkKind::Indirect,
        )
        .unwrap();
        mint(
            &root,
            ws.model(),
            &req_ref(REQ_SLUG),
            "code/auth.rs#Vault",
            LinkKind::Indirect,
            Rule::Declared,
            Origin::Captured { task: "t9".into() },
            Standing::Asserted,
        )
        .unwrap();

        let report = audit(&root, ws.model(), &AuditOptions::default()).unwrap();
        assert_eq!(report.declared, 1);
        assert_eq!(report.authored, 1);
        assert_eq!(report.inferred, 0);
        let head = render_audit(&report)
            .lines()
            .next()
            .expect("the audit opens with its tally")
            .to_string();
        for word in ["declared", "inferred", "authored"] {
            assert!(head.contains(word), "{head}");
        }

        // A finding that names a row says which rule made it.
        let line = AuditFinding::DecayedEvidence {
            id: "l0007-abcdef".into(),
            spec: req_ref(REQ_SLUG),
            anchor: "code/auth.rs#Vault::persist".into(),
            confidence: 0.1,
            rule: Rule::Inferred,
        }
        .to_string();
        assert!(line.contains("inferred"), "{line}");

        fs::remove_dir_all(&root).unwrap();
    }
}
