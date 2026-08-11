//! Task-close capture: the wave-open item-hash index and the delta scan
//! that mints candidate links
//! (`archi/requirements/self-hosting/capture-at-the-join.md`).
//!
//! A wave opening records the tree as a **canonical item-hash index** —
//! file → symbol → body hash, by the canonicalizer of [`super::code`] — so
//! a closing task's delta is read off by hash comparison: symbol-granular,
//! cheap to store (no file contents), and git-free by construction, so
//! squashes and shallow clones cannot break it. The delta says which symbols
//! moved and the task outputs say who claims them; changed items in files no
//! task claims are **leftovers**, reported rather than guessed at.
//!
//! Capture is idempotent: a pair the journal already holds is not minted
//! twice. Re-encounters and unreconfirmed rewrites of the evidence rows the
//! journal already carries journal as `touch` and `decay` events, once per
//! task — the observations confidence is derived from.
//!
//! The mint comes from the writer, not from the words: each in-flight task
//! writes one **declaration file** beside the index, naming for every symbol
//! it changed the port or requirement that symbol answers and the test that
//! proves it. What the file names becomes an asserted link; what it does not
//! name becomes nothing
//! (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`).
//!
//! The **signal test** stays, and it no longer mints: a (changed item,
//! spec_ref) pair carries signal when the ref's surface terms overlap the
//! item's symbol path or canonical body tokens. The refs each task's delta
//! carries signal for come back as `pressed` — the set the wave-close
//! coverage gate demands links for — and the no-signal pairs come back as
//! `suppressed`, counted in the render and listed whole under `--json`.
//! Neither is journaled and neither is subtracted, so a hand `link add`
//! mints any of them asserted at any time.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use modeling_lang::{Model, Statement};
use serde::{Deserialize, Serialize};
use toml::Spanned;

use super::code;
use super::{Anchor, Event, Link, LinkKind, Origin, Rule, SpecRef, Standing};
use crate::plans::{self, Task};

// ---- the wave-open index ----------------------------------------------------

/// One file's index: the canonical file hash and, for Rust files, the body
/// hash per symbol. Colliding symbol paths (trait methods of one type)
/// fold into one combined hash — the change still registers; minting skips
/// the ambiguous anchor with a note.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct FileIndex {
    /// Hash of the whole file's canonical tokens.
    pub hash: String,
    /// Body hash per symbol path.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub symbols: BTreeMap<String, String>,
}

/// A tree snapshot: every code file's canonical item hashes, across every
/// member mapped at scan time. Keys are scan keys — `member//file`, bare
/// for home — so a memberless project's index is byte-identical to
/// yesterday's, and a pre-member index replays as home-only.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct TreeIndex {
    /// Scan key → its index.
    pub files: BTreeMap<String, FileIndex>,
    /// The members this snapshot walked (home implied). Recorded at wave
    /// open so capture diffs exactly what the open saw: a member mapped
    /// afterwards is outside the set and skipped with a note, never diffed
    /// against an index that never saw it
    /// (`archi/stress/split-tree-pressure/the-wave-that-outlived-its-map`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scanned: Vec<String>,
}

impl TreeIndex {
    /// Scan the working trees: every code file of home and of the given
    /// mapped members, canonicalized and indexed under scan keys.
    pub fn scan(root: &Path, members: &[&crate::members::Member]) -> TreeIndex {
        let mut files = BTreeMap::new();
        let mut index_into = |member: Option<&str>, member_root: &Path, file: String| {
            let Ok(text) = fs::read_to_string(member_root.join(&file)) else {
                return;
            };
            let canonical = code::canonicalize(&file, &text);
            let mut symbols: BTreeMap<String, String> = BTreeMap::new();
            for item in &canonical.items {
                symbols
                    .entry(item.symbol.clone())
                    .and_modify(|h| {
                        *h = code::hash_bytes(format!("{h}+{}", item.body).as_bytes());
                    })
                    .or_insert_with(|| item.body.clone());
            }
            files.insert(
                super::qualify(member, &file),
                FileIndex {
                    hash: canonical.file_hash(),
                    symbols,
                },
            );
        };
        for file in super::code_files(root) {
            index_into(None, root, file);
        }
        let mut scanned = Vec::new();
        for m in members {
            let Some(mroot) = &m.root else { continue };
            scanned.push(m.name.clone());
            for file in super::member_code_files(root, mroot, &m.name) {
                index_into(Some(&m.name), mroot, file);
            }
        }
        TreeIndex { files, scanned }
    }
}

/// One changed item: a symbol whose body hash moved or appeared since the
/// index, or — for files with no symbol index — the file itself. Deletions
/// are invisible on purpose: capture links code that exists.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Changed {
    /// Project-relative file.
    pub file: String,
    /// The changed symbol; `None` when the file is the item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

impl std::fmt::Display for Changed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.symbol {
            None => write!(f, "{}", self.file),
            Some(s) => write!(f, "{}#{s}", self.file),
        }
    }
}

/// The items whose canonical hashes changed or appeared since the index.
fn delta(opened: &TreeIndex, current: &TreeIndex) -> Vec<Changed> {
    let mut out = Vec::new();
    for (file, now) in &current.files {
        let before = opened.files.get(file);
        if before.is_some_and(|b| b == now) {
            continue;
        }
        if now.symbols.is_empty() {
            if before.is_none_or(|b| b.hash != now.hash) {
                out.push(Changed {
                    file: file.clone(),
                    symbol: None,
                });
            }
            continue;
        }
        for (symbol, hash) in &now.symbols {
            let moved = before
                .and_then(|b| b.symbols.get(symbol))
                .is_none_or(|h| h != hash);
            if moved {
                out.push(Changed {
                    file: file.clone(),
                    symbol: Some(symbol.clone()),
                });
            }
        }
    }
    out
}

// ---- index storage -----------------------------------------------------------

fn index_path(root: &Path, plan: &str, wave: usize) -> PathBuf {
    plans::plan_dir(root, plan)
        .join("waves")
        .join(format!("w{wave:02}.index.json"))
}

/// Record the wave-open snapshot — `plan start` and each passing
/// `plan next` write the index its wave's deltas diff against. The scan
/// covers every member mapped right now; the index remembers which.
pub(crate) fn write_index(root: &Path, plan: &str, wave: usize) -> Result<(), String> {
    let set = crate::members::MemberSet::resolve(root)?;
    let mapped: Vec<&crate::members::Member> =
        set.declared().iter().filter(|m| m.root.is_some()).collect();
    let index = TreeIndex::scan(root, &mapped);
    let path = index_path(root, plan, wave);
    let dir = path.parent().expect("the index has a directory");
    fs::create_dir_all(dir).map_err(|e| format!("cannot create `{}`: {e}", dir.display()))?;
    let mut text =
        serde_json::to_string_pretty(&index).map_err(|e| format!("index serializes: {e}"))?;
    text.push('\n');
    fs::write(&path, text).map_err(|e| format!("cannot write `{}`: {e}", path.display()))
}

fn read_index(root: &Path, plan: &str, wave: usize) -> Result<TreeIndex, String> {
    let path = index_path(root, plan, wave);
    let text = fs::read_to_string(&path).map_err(|e| {
        format!(
            "cannot read `{}`: {e} — opening a wave writes its index",
            path.display()
        )
    })?;
    serde_json::from_str(&text).map_err(|e| format!("`{}` does not parse: {e}", path.display()))
}

// ---- the signal test ---------------------------------------------------------

/// Tokens of the built-in classification rel (`type_of`), excluded from
/// ref surfaces: they are structure, not signal — left in, every body
/// that says `type` would press a classification edge.
const STOPPED: [&str; 2] = ["type", "of"];

/// Fold surface text into comparable terms: split on non-alphanumeric
/// bytes and on camel-case boundaries (`ModelGraph` → `modelgraph`,
/// `model`, `graph`; `NKPReport` also yields `nkp`, `report`), lowercased,
/// single characters dropped.
fn terms_into(text: &str, out: &mut BTreeSet<String>) {
    for raw in text.split(|c: char| !c.is_ascii_alphanumeric()) {
        if raw.is_empty() {
            continue;
        }
        keep(raw, out);
        let bytes = raw.as_bytes();
        let mut start = 0;
        for i in 1..bytes.len() {
            let boundary = bytes[i].is_ascii_uppercase()
                && (bytes[i - 1].is_ascii_lowercase()
                    || bytes[i - 1].is_ascii_digit()
                    || bytes.get(i + 1).is_some_and(|b| b.is_ascii_lowercase()));
            if boundary && start < i {
                keep(&raw[start..i], out);
                start = i;
            }
        }
        if start > 0 {
            keep(&raw[start..], out);
        }
    }
}

fn keep(term: &str, out: &mut BTreeSet<String>) {
    if term.len() >= 2 {
        out.insert(term.to_ascii_lowercase());
    }
}

/// A spec_ref's comparable surface: node path segments, edge endpoints,
/// payload types — the ref text's terms minus the stoplist.
fn ref_terms(spec_ref: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    terms_into(spec_ref, &mut out);
    for s in STOPPED {
        out.remove(s);
    }
    out
}

/// A changed item's comparable content: its symbol path plus the canonical
/// tokens of its span — or, for file-level items, the file path plus every
/// token of the file.
fn item_terms(change: &Changed, canonical: &code::Canonical) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    match &change.symbol {
        Some(symbol) => {
            terms_into(symbol, &mut out);
            for item in canonical.find(symbol) {
                for t in &canonical.tokens {
                    if t.line >= item.start_line && t.line <= item.end_line {
                        terms_into(&t.text, &mut out);
                    }
                }
            }
        }
        None => {
            terms_into(&change.file, &mut out);
            for t in &canonical.tokens {
                terms_into(&t.text, &mut out);
            }
        }
    }
    out
}

// ---- the writer's declaration -------------------------------------------------

/// One task's declaration file, project-relative: beside the index the wave
/// already writes, and named for the wave and the task, so two tasks of one
/// wave never write over each other.
fn declares_rel(plan: &str, wave: usize, task: &str) -> String {
    format!("archi/plans/{plan}/waves/w{wave:02}.{task}.declares.toml")
}

/// The file's shape: one array of tables, each naming the changed symbol,
/// what it answers and the test that proves it. Every field is required and
/// no unknown key is tolerated — an optional field is the beginning of a file
/// that always parses
/// (`archi/requirements/planning/the-declaration-refusal-repairs-without-guessing.md`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Declarations {
    declares: Vec<Declared>,
}

/// One declaration. Each field is read spanned, so a name that parses and
/// then resolves to nothing is refused on the line it was written on and not
/// on the table above it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Declared {
    /// The changed code, as `link add` takes it:
    /// `[<member>//]<file>[#<symbol>]`.
    symbol: Spanned<String>,
    /// The port or requirement that code answers.
    answers: Spanned<String>,
    /// The test that proves it answers.
    proved_by: Spanned<String>,
}

/// A task's declaration file as it was read: its path and its bytes, kept so
/// every refusal can quote the line it is about, and the declarations in it.
struct DeclarationFile {
    path: String,
    text: String,
    declares: Vec<Declared>,
}

impl DeclarationFile {
    /// One located refusal: the file, the task that owes it, the line, what
    /// is wrong there, and the lines that stood there. The reader of this
    /// refusal did not write the file and cannot ask its writer, so the
    /// refusal costs one read to fix and never restates the grammar alone
    /// (`archi/requirements/planning/the-declaration-refusal-repairs-without-guessing.md`).
    fn refuse(&self, task: &str, span: Option<Range<usize>>, what: &str) -> String {
        let Some(span) = span else {
            return format!("`{}`, task `{task}`: {what}", self.path);
        };
        let first = line_at(&self.text, span.start);
        let last = line_at(&self.text, span.end);
        let mut out = format!("`{}`:{first}, task `{task}`: {what}", self.path);
        for (i, line) in self
            .text
            .lines()
            .enumerate()
            .take(last)
            .skip(first.saturating_sub(1))
        {
            out.push_str(&format!("\n  {} | {line}", i + 1));
        }
        out
    }
}

/// The 1-based line a byte offset sits on.
fn line_at(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].matches('\n').count() + 1
}

/// One task's declarations, parsed. `Ok(None)` when the file is absent: an
/// absent file is the wave gate's refusal to raise
/// (`archi/requirements/planning/an-undeclared-change-refuses-the-wave.md`),
/// and until that gate stands, a task that declared nothing mints nothing and
/// capture says so.
fn read_declarations(
    root: &Path,
    plan: &str,
    wave: usize,
    task: &str,
) -> Result<Option<DeclarationFile>, String> {
    let rel = declares_rel(plan, wave, task);
    let path = root.join(&rel);
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("cannot read `{rel}`: {e}")),
    };
    let mut file = DeclarationFile {
        path: rel,
        text,
        declares: Vec::new(),
    };
    match toml::from_str::<Declarations>(&file.text) {
        Ok(parsed) => {
            file.declares = parsed.declares;
            Ok(Some(file))
        }
        Err(e) => Err(file.refuse(task, e.span(), e.message())),
    }
}

/// The canonical surface text of every edge of the model, to the two ends it
/// joins. A declaration's `answers` is matched against this before it is
/// resolved, so a writer who named an edge is told which ends stand behind it
/// instead of being told the name resolves to something it must not name
/// (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`).
fn edge_ends(model: &Model) -> BTreeMap<String, (String, String)> {
    model
        .dump()
        .into_iter()
        .filter_map(|s| {
            let text = super::edge_pseudo(&s)?;
            let ends = match &s {
                Statement::RelEdge { source, target, .. } => (source.clone(), target.clone()),
                Statement::ConnEdge { source, target, .. } => (
                    format!("{}.{}", source.node, source.port),
                    format!("{}.{}", target.node, target.port),
                ),
                _ => return None,
            };
            Some((text, ends))
        })
        .collect()
}

/// Mint one task's declarations: what the file names becomes an asserted
/// link on the symbol that named it, stamped `declared` and carrying the
/// test. A pair the journal already holds is not minted twice, so a wave that
/// re-runs its capture is a no-op
/// (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`,
/// `archi/requirements/code-link/a-declaration-names-the-test-that-proves-it.md`).
fn mint_declarations(
    root: &Path,
    model: &Model,
    file: &DeclarationFile,
    task: &str,
    live: &[Link],
) -> Result<Vec<Link>, String> {
    if file.declares.is_empty() {
        return Ok(Vec::new());
    }
    let roots = super::Roots::resolve(root)?;
    let edges = edge_ends(model);
    let mut minted = Vec::new();
    for d in &file.declares {
        let answers = d.answers.get_ref();
        let symbol = d.symbol.get_ref();
        let proved_by = d.proved_by.get_ref();

        // What it answers: a port or a requirement, never an edge.
        let spec = SpecRef::parse(answers)
            .map_err(|e| file.refuse(task, Some(d.answers.span()), &e))?;
        if let Some((source, target)) = edges.get(&spec.path) {
            return Err(file.refuse(
                task,
                Some(d.answers.span()),
                &format!(
                    "`{}` is an edge — an edge is a caller, and the code behind a port does not \
                     know its callers; name the end this symbol answers: `{source}` or `{target}`",
                    spec.path
                ),
            ));
        }

        // Where the code is. It is resolved here, before the mint, so a
        // symbol the tree does not hold is refused on the line that named it.
        let anchor = Anchor::parse(symbol)
            .map_err(|e| file.refuse(task, Some(d.symbol.span()), &e))?;
        let symbol_root = roots
            .require(&anchor.repo)
            .map_err(|e| file.refuse(task, Some(d.symbol.span()), &e))?;
        super::resolve_anchor(&symbol_root, &anchor)
            .map_err(|e| file.refuse(task, Some(d.symbol.span()), &e))?;

        // The test that proves it. Archi runs nothing: the test's existence
        // is checked, its passing is the suite's business.
        let test = Anchor::parse(proved_by)
            .map_err(|e| file.refuse(task, Some(d.proved_by.span()), &e))?;
        if test.symbol.is_none() {
            return Err(file.refuse(
                task,
                Some(d.proved_by.span()),
                &format!(
                    "the test `{test}` names a file and no symbol — `proved_by` names the test \
                     itself, as `<file>#<test fn>`"
                ),
            ));
        }
        let test_root = roots
            .require(&test.repo)
            .map_err(|e| file.refuse(task, Some(d.proved_by.span()), &e))?;
        super::resolve_anchor(&test_root, &test).map_err(|e| {
            file.refuse(
                task,
                Some(d.proved_by.span()),
                &format!("the test `{test}` does not resolve: {e}"),
            )
        })?;

        // A pair the journal already holds is the same claim, not a second
        // one: the wave's capture re-runs, and the writer's file does not
        // change what it said.
        let held = live
            .iter()
            .chain(&minted)
            .any(|l| l.spec.version.is_none() && l.spec.path == spec.path && l.anchor == anchor);
        if held {
            continue;
        }
        // The spec side is the mint's own resolution; what is left of its
        // refusals is the name this line wrote.
        let link = super::mint(
            root,
            model,
            answers,
            symbol,
            LinkKind::Indirect,
            Rule::Declared,
            Origin::Captured {
                task: task.to_string(),
            },
            Standing::Asserted,
            Some(test),
        )
        .map_err(|e| file.refuse(task, Some(d.answers.span()), &e))?;
        minted.push(link);
    }
    Ok(minted)
}

// ---- capture -----------------------------------------------------------------

/// One suppressed pair: a (spec_ref, changed item) pair the signal test
/// found no shared term for. Reporting, not state — nothing is journaled,
/// and a hand `link add` mints the pair asserted.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Suppressed {
    /// The claiming task.
    pub task: String,
    /// The spec_ref that found no signal.
    pub spec_ref: String,
    /// The changed item, as `file` or `file#symbol`.
    pub item: String,
}

/// One capture's outcome.
#[derive(Default, Serialize)]
pub struct CaptureOutcome {
    /// Freshly minted links — what the in-flight tasks' declarations named,
    /// asserted and stamped `declared`.
    pub minted: Vec<Link>,
    /// `(link id, task)`: a live evidence link re-encountered by another
    /// task carrying the same spec_ref — confidence accrues.
    pub touched: Vec<(String, String)>,
    /// `(link id, task)`: the anchored item changed under a task that does
    /// not carry the link's spec_ref — confidence decays.
    pub decayed: Vec<(String, String)>,
    /// Changed items no in-flight task claims — code motion the plan does
    /// not account for.
    pub leftovers: Vec<Changed>,
    /// Pairs the signal test found no shared term for.
    pub suppressed: Vec<Suppressed>,
    /// Task id → the spec_refs at least one of its claimed changed items
    /// carries signal for: what this delta presses, read by the gate.
    pub pressed: BTreeMap<String, BTreeSet<String>>,
    /// Anchors skipped and files claimed by several tasks.
    pub notes: Vec<String>,
}

/// Whether a task's outputs claim a file: an exact path, or a directory
/// prefix for entries ending in `/`.
fn claims_file(task: &Task, file: &str) -> bool {
    task.outputs
        .iter()
        .any(|o| o.strip_suffix('/').map_or(o == file, |dir| {
            file.strip_prefix(dir)
                .is_some_and(|rest| rest.starts_with('/'))
        }))
}

/// Capture a wave: mint the in-flight tasks' declarations, then diff the
/// wave-open index against the current tree to say what the delta presses and
/// what it leaves over; journal touches and decays. `only` restricts the mint
/// and the press to one task (`link capture --task`) while the claim map
/// still spans the whole wave.
pub(crate) fn capture_wave(
    root: &Path,
    model: &Model,
    plan_name: &str,
    wave: usize,
    in_flight: &[&Task],
    only: Option<&str>,
) -> Result<CaptureOutcome, String> {
    let opened = read_index(root, plan_name, wave)?;
    let set = crate::members::MemberSet::resolve(root)?;
    let mut out = CaptureOutcome::default();
    // Diff exactly the scan set the open recorded: a member outside it has
    // no baseline to diff against; one unreachable now cannot be read. Both
    // narrow the capture and say so.
    let mut rescanned = Vec::new();
    for m in set.declared() {
        let in_open = opened.scanned.contains(&m.name);
        match (&m.root, in_open) {
            (Some(_), true) => rescanned.push(m),
            (Some(_), false) => out.notes.push(format!(
                "`{}` was mapped after this wave opened — its delta is unattributed; it joins \
                 at the next wave open",
                m.name
            )),
            (None, true) => out.notes.push(format!(
                "`{}` was scanned at wave open but is unreachable now — its delta is unseen \
                 this close",
                m.name
            )),
            (None, false) => {}
        }
    }
    let current = TreeIndex::scan(root, &rescanned);
    let changes = delta(&opened, &current);
    let folded = super::load(root)?;

    let mut shared: BTreeSet<&str> = BTreeSet::new();
    let mut events: Vec<Event> = Vec::new();
    // The working link set: the fold plus this batch's mints, so the decay
    // pass sees same-run mints from overlapping tasks symmetrically.
    let mut live: Vec<Link> = folded.live.clone();
    let mut ref_term_cache: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();

    // The mint: the declarations the in-flight tasks wrote. A task that wrote
    // none mints nothing and says so — refusing an absent file is the wave
    // gate's, not this reader's.
    for task in in_flight
        .iter()
        .filter(|t| only.is_none_or(|o| o == t.id))
    {
        match read_declarations(root, plan_name, wave, &task.id)? {
            None => out.notes.push(format!(
                "`{}` declares nothing: `{}` is absent — no link is minted for its delta",
                task.id,
                declares_rel(plan_name, wave, &task.id)
            )),
            Some(file) => {
                for link in mint_declarations(root, model, &file, &task.id, &live)? {
                    out.minted.push(link.clone());
                    live.push(link);
                }
            }
        }
    }

    // Touch and press, per change, per claiming task. The claim map and the
    // term test are what they were; what they no longer do is mint.
    for change in &changes {
        let claimants: Vec<&Task> = in_flight
            .iter()
            .copied()
            .filter(|t| claims_file(t, &change.file))
            .collect();
        if claimants.is_empty() {
            out.leftovers.push(change.clone());
            continue;
        }
        if claimants.len() > 1 {
            shared.insert(change.file.as_str());
        }
        let (member, bare) = super::split_qualified(&change.file);
        let anchor = Anchor {
            repo: member.map(str::to_string),
            file: bare.to_string(),
            symbol: change.symbol.clone(),
        };
        let Some(member_root) = set.get(member.unwrap_or(crate::members::HOME)).and_then(|m| m.root.clone())
        else {
            out.notes.push(format!("skipped `{anchor}`: its member is unreachable"));
            continue;
        };
        if let Err(e) = super::resolve_anchor(&member_root, &anchor) {
            out.notes.push(format!("skipped `{anchor}`: {e}"));
            continue;
        }
        // Terms come from the bare path and body: the member qualifier is
        // identity, never signal.
        let content = fs::read_to_string(member_root.join(bare)).unwrap_or_default();
        let bare_change = Changed {
            file: bare.to_string(),
            symbol: change.symbol.clone(),
        };
        let item = item_terms(&bare_change, &code::canonicalize(bare, &content));
        for task in &claimants {
            if only.is_some_and(|o| o != task.id) {
                continue;
            }
            for spec_ref in &task.spec_refs {
                let signal = ref_term_cache
                    .entry(spec_ref.as_str())
                    .or_insert_with(|| ref_terms(spec_ref))
                    .iter()
                    .any(|t| item.contains(t));
                if signal {
                    out.pressed
                        .entry(task.id.clone())
                        .or_default()
                        .insert(spec_ref.clone());
                }
                let existing = live.iter().find(|l| {
                    l.spec.version.is_none() && l.spec.path == *spec_ref && l.anchor == anchor
                });
                if let Some(link) = existing {
                    // A re-encounter from another task accrues confidence;
                    // the same task re-running is a no-op.
                    let re_encounter = link.standing == Standing::Evidence
                        && !matches!(&link.origin, Origin::Captured { task: t } if *t == task.id)
                        && !link.touches.contains(&task.id);
                    if re_encounter {
                        events.push(Event::Touch {
                            id: link.id.clone(),
                            task: task.id.clone(),
                            at: super::now(),
                        });
                        out.touched.push((link.id.clone(), task.id.clone()));
                        let id = link.id.clone();
                        if let Some(l) = live.iter_mut().find(|l| l.id == id) {
                            l.touches.push(task.id.clone());
                        }
                    }
                    continue;
                }
                // No shared term between ref surface and item content: the
                // pair is reported — and not subtracted, so a hand `link add`
                // stays free to claim it.
                if !signal {
                    out.suppressed.push(Suppressed {
                        task: task.id.clone(),
                        spec_ref: spec_ref.clone(),
                        item: change.to_string(),
                    });
                }
            }
        }
    }

    // Decay, after all mints: every claiming task presses on the evidence
    // links anchored at its changed items whose spec_ref it does not carry
    // — a rewrite without reconfirmation, observed exactly when it happens.
    // Overlapping claims cross-press same-run mints: split confidence.
    for change in &changes {
        let (member, bare) = super::split_qualified(&change.file);
        let anchor = Anchor {
            repo: member.map(str::to_string),
            file: bare.to_string(),
            symbol: change.symbol.clone(),
        };
        for task in in_flight
            .iter()
            .filter(|t| claims_file(t, &change.file))
            .filter(|t| only.is_none_or(|o| o == t.id))
        {
            for link in live.iter_mut().filter(|l| {
                l.standing == Standing::Evidence
                    && l.spec.version.is_none()
                    && l.anchor == anchor
            }) {
                if task.spec_refs.contains(&link.spec.path) || link.decays.contains(&task.id) {
                    continue;
                }
                events.push(Event::Decay {
                    id: link.id.clone(),
                    task: task.id.clone(),
                    at: super::now(),
                });
                out.decayed.push((link.id.clone(), task.id.clone()));
                link.decays.push(task.id.clone());
            }
        }
    }

    for file in shared {
        out.notes.push(format!(
            "`{file}` is claimed by several tasks — their captures split confidence"
        ));
    }
    if !events.is_empty() {
        super::append(root, &events)?;
    }
    Ok(out)
}

/// `archi link capture --task <TASK>`: re-run the in-flight wave's capture
/// for one task by hand — capture normally fires from `archi plan next`.
pub fn run_manual(root: &Path, model: &Model, task_id: &str) -> Result<CaptureOutcome, String> {
    let plan = plans::load_active(root)?;
    if plan.state != plans::PlanState::Started {
        return Err(format!(
            "plan `{}` is not started — capture runs against the wave in flight",
            plan.name
        ));
    }
    let report = plans::verify_plan(root, model, &plan)?;
    if !report.errors.is_empty() {
        return Err(format!(
            "the plan is structurally broken — `archi plan verify`:\n  {}",
            report.errors.join("\n  ")
        ));
    }
    let waves = &report.derived.waves;
    if plan.closed_waves >= waves.len() {
        return Err("no wave in flight — the scenario step is pending".into());
    }
    let wave = plan.closed_waves + 1;
    let ids = &waves[wave - 1];
    let in_flight: Vec<&Task> = plan
        .tasks
        .iter()
        .filter(|t| ids.contains(&t.id))
        .collect();
    if !in_flight.iter().any(|t| t.id == task_id) {
        return Err(format!(
            "`{task_id}` is not in wave {wave} — in flight: {}",
            ids.join(", ")
        ));
    }
    capture_wave(root, model, &plan.name, wave, &in_flight, Some(task_id))
}

/// The capture outcome as human lines.
pub fn render_capture(o: &CaptureOutcome) -> String {
    let mut out = String::new();
    for l in &o.minted {
        out.push_str(&format!("captured {}\n", super::render_link(l)));
    }
    for (id, task) in &o.touched {
        out.push_str(&format!("touched {id} (re-encountered under {task})\n"));
    }
    for (id, task) in &o.decayed {
        out.push_str(&format!("decayed {id} (rewritten under {task} without its spec_ref)\n"));
    }
    for c in &o.leftovers {
        out.push_str(&format!("leftover {c} — no in-flight task claims it\n"));
    }
    if !o.suppressed.is_empty() {
        out.push_str(&format!(
            "suppressed {} no-signal pair(s) — whole under --json; `archi link add <ref> <file#symbol>` mints any of them asserted\n",
            o.suppressed.len()
        ));
    }
    for n in &o.notes {
        out.push_str(&format!("note: {n}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use modeling_lang::Workspace;

    use super::super::{append, load, now, resolve_anchor, Birth};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    const STORE_RS: &str = "pub struct Store {\n    rows: Vec<u8>,\n}\n\n\
                            impl Store {\n    pub fn put(&mut self, row: u8) {\n        self.rows.push(row);\n    }\n}\n";

    /// The smallest model that compiles. Nothing here mints, so nothing here
    /// resolves a name against it; what a declaration resolves to is the
    /// binary's own question, and `tests/link_e2e.rs` asks it.
    const MODEL: &str = "def node Store\n";

    fn temp_project() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-capture-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(dir.join("code")).unwrap();
        fs::create_dir_all(dir.join("archi/src")).unwrap();
        fs::write(
            dir.join("archi.toml"),
            "[project]\nname = \"t\"\npreset = \"default\"\n",
        )
        .unwrap();
        fs::write(dir.join("archi/src/model.arch"), MODEL).unwrap();
        fs::write(dir.join("code/store.rs"), STORE_RS).unwrap();
        fs::write(dir.join("code/schema.sql"), "CREATE TABLE t (id INT);\n").unwrap();
        dir
    }

    fn compiled(root: &Path) -> Workspace {
        modeling_lang::source::compile_project(root)
            .unwrap_or_else(|f| panic!("the fixture model compiles:\n{}", f.render()))
            .workspace
    }

    /// A standing evidence row, journaled as capture minted them while the
    /// shared-term rule ruled. Thousands of them stand in this project's own
    /// journal, and a wave that re-encounters or rewrites one still touches
    /// or decays it — which is what these tests are about. Nothing mints
    /// evidence anymore, so a test that needs one writes it.
    fn seed_evidence(root: &Path, spec: &str, code: &str, task: &str) -> String {
        let anchor = Anchor::parse(code).unwrap();
        let resolved = resolve_anchor(root, &anchor).unwrap();
        let link = Link {
            id: load(root).unwrap().next_id(spec),
            spec: SpecRef::parse(spec).unwrap(),
            anchor,
            kind: LinkKind::Indirect,
            standing: Standing::Evidence,
            origin: Origin::Captured {
                task: task.to_string(),
            },
            rule: Rule::Inferred,
            proves: None,
            birth: Birth {
                created: now(),
                commit: None,
                spans: vec![resolved.span],
            },
            pins: resolved.pins,
            touches: Vec::new(),
            decays: Vec::new(),
        };
        let id = link.id.clone();
        append(root, &[Event::Add { link }]).unwrap();
        id
    }

    fn task(id: &str, spec_refs: &[&str], outputs: &[&str]) -> Task {
        Task {
            id: id.to_string(),
            node: format!("Node{id}"),
            description: String::new(),
            spec_refs: spec_refs.iter().map(|s| s.to_string()).collect(),
            owns: Vec::new(),
            facts: Vec::new(),
            stack_details: String::new(),
            inputs: BTreeMap::new(),
            outputs: outputs.iter().map(|s| s.to_string()).collect(),
            verifications: BTreeMap::new(),
        }
    }

    fn ls(root: &Path) -> Vec<Link> {
        super::super::ls(root, None, false).unwrap()
    }

    #[test]
    fn the_index_is_symbol_granular_and_formatting_blind() {
        let root = temp_project();
        fs::create_dir_all(plans::plan_dir(&root, "p")).unwrap();
        write_index(&root, "p", 1).unwrap();
        let opened = read_index(&root, "p", 1).unwrap();
        assert!(opened.files["code/store.rs"].symbols.contains_key("Store::put"));
        assert!(opened.files["code/schema.sql"].symbols.is_empty());

        // Formatting and comments never register.
        fs::write(
            root.join("code/store.rs"),
            STORE_RS.replace(
                "pub fn put(&mut self, row: u8) {",
                "// appends one row\n    pub fn put(&mut self,\n               row: u8) {",
            ),
        )
        .unwrap();
        assert_eq!(delta(&opened, &TreeIndex::scan(&root, &[])), Vec::<Changed>::new());

        // A body edit registers that symbol; a new item appears; a text
        // file changes as a whole.
        fs::write(
            root.join("code/store.rs"),
            format!(
                "{}\npub fn wipe(s: &mut Store) {{ s.rows.clear(); }}\n",
                STORE_RS.replace("self.rows.push(row);", "self.rows.insert(0, row);")
            ),
        )
        .unwrap();
        fs::write(root.join("code/schema.sql"), "CREATE TABLE t (id BIGINT);\n").unwrap();
        let changed = delta(&opened, &TreeIndex::scan(&root, &[]));
        let texts: Vec<String> = changed.iter().map(ToString::to_string).collect();
        assert_eq!(
            texts,
            vec![
                "code/schema.sql".to_string(),
                "code/store.rs#Store::put".to_string(),
                "code/store.rs#wipe".to_string(),
            ]
        );

        fs::remove_dir_all(&root).unwrap();
    }

    /// The diff and the claim map are what they were — a changed symbol under
    /// a task's outputs is that task's, and a changed file nobody claims is a
    /// leftover. What no longer happens is the mint: a task that declared
    /// nothing gets nothing, however many words its refs share with its code
    /// (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`).
    #[test]
    fn an_undeclared_delta_mints_nothing_and_still_reports_its_leftovers() {
        let root = temp_project();
        let ws = compiled(&root);
        fs::create_dir_all(plans::plan_dir(&root, "p")).unwrap();
        write_index(&root, "p", 1).unwrap();

        // The task claims the code dir; the sql file belongs to nobody.
        let t1 = task("t1", &["Store", "Gate.out wire Store.inn"], &["code/store.rs"]);
        fs::write(
            root.join("code/store.rs"),
            STORE_RS.replace("self.rows.push(row);", "self.rows.insert(0, row);"),
        )
        .unwrap();
        fs::write(root.join("code/schema.sql"), "CREATE TABLE t (id BIGINT);\n").unwrap();

        let out = capture_wave(&root, ws.model(), "p", 1, &[&t1], None).unwrap();
        // Both refs share terms with the changed symbol, and neither mints.
        assert_eq!(out.pressed["t1"].len(), 2, "{}", render_capture(&out));
        assert!(out.minted.is_empty(), "{}", render_capture(&out));
        assert!(ls(&root).is_empty(), "{}", render_capture(&out));
        // The task is told where the file it did not write belongs.
        assert!(
            out.notes
                .iter()
                .any(|n| n.contains("w01.t1.declares.toml") && n.contains("declares nothing")),
            "{:?}",
            out.notes
        );
        assert_eq!(out.leftovers.len(), 1);
        assert_eq!(out.leftovers[0].to_string(), "code/schema.sql");

        // The re-run says the same and still mints nothing.
        let again = capture_wave(&root, ws.model(), "p", 1, &[&t1], None).unwrap();
        assert!(
            again.minted.is_empty() && again.touched.is_empty(),
            "{}",
            render_capture(&again)
        );

        fs::remove_dir_all(&root).unwrap();
    }

    /// The observations confidence is derived from still run over the
    /// evidence rows the journal already holds: a task that rewrites the item
    /// an evidence link is anchored at, without carrying its ref, decays it;
    /// another task carrying it touches it; and each of those is journaled
    /// once per task (`archi/requirements/self-hosting/capture-at-the-join.md`).
    #[test]
    fn touches_and_decays_journal_once_per_task() {
        let root = temp_project();
        let ws = compiled(&root);
        fs::create_dir_all(plans::plan_dir(&root, "p")).unwrap();
        write_index(&root, "p", 1).unwrap();

        // Two tasks claim the same file, and one standing evidence row per
        // task is anchored at the symbol they both rewrite.
        let t1 = task("t1", &["Store"], &["code/"]);
        let t2 = task("t2", &["Rows"], &["code/"]);
        let store_id = seed_evidence(&root, "Store", "code/store.rs#Store::put", "t1");
        let rows_id = seed_evidence(&root, "Rows", "code/store.rs#Store::put", "t2");
        fs::write(
            root.join("code/store.rs"),
            STORE_RS.replace("self.rows.push(row);", "self.rows.insert(0, row);"),
        )
        .unwrap();

        let out = capture_wave(&root, ws.model(), "p", 1, &[&t1, &t2], None).unwrap();
        // Each task presses on the other's row: split confidence.
        assert!(out.minted.is_empty(), "{}", render_capture(&out));
        assert_eq!(out.decayed.len(), 2, "{}", render_capture(&out));
        assert!(out.notes.iter().any(|n| n.contains("claimed by several")), "{:?}", out.notes);
        let live = ls(&root);
        let store_link = live.iter().find(|l| l.id == store_id).unwrap();
        let rows_link = live.iter().find(|l| l.id == rows_id).unwrap();
        assert_eq!(store_link.decays, vec!["t2".to_string()]);
        assert_eq!(rows_link.decays, vec!["t1".to_string()]);

        // Re-runs never double-journal; a third task carrying `Store`
        // touches the standing evidence, and decays the `Rows` link it does
        // not carry.
        let again = capture_wave(&root, ws.model(), "p", 1, &[&t1, &t2], None).unwrap();
        assert!(again.minted.is_empty() && again.decayed.is_empty() && again.touched.is_empty());
        let t3 = task("t3", &["Store"], &["code/"]);
        let third = capture_wave(&root, ws.model(), "p", 1, &[&t1, &t2, &t3], None).unwrap();
        assert!(third.minted.is_empty(), "{}", render_capture(&third));
        assert!(third.touched.iter().any(|(id, task)| id == &store_id && task == "t3"));
        assert!(third.decayed.iter().any(|(id, task)| id == &rows_id && task == "t3"));

        fs::remove_dir_all(&root).unwrap();
    }

    /// The signal test still splits the product and still reports both
    /// halves — what this delta presses, and what shares no word with it —
    /// and it mints neither half. Nothing it says is state: no pair is
    /// journaled and no pair is subtracted, so a hand `link add` stays free
    /// to claim any of them (`archi/requirements/code-link/candidates-carry-signal.md`).
    #[test]
    fn no_signal_pairs_suppress_but_never_subtract_or_eat_leftovers() {
        let root = temp_project();
        let ws = compiled(&root);
        fs::create_dir_all(plans::plan_dir(&root, "p")).unwrap();
        write_index(&root, "p", 1).unwrap();

        // One term-bearing ref, one foreign ref; the sql file changes
        // unclaimed beside them.
        let t1 = task("t1", &["Store", "Gate.out wire Auth.inn"], &["code/store.rs"]);
        fs::write(
            root.join("code/store.rs"),
            STORE_RS.replace("self.rows.push(row);", "self.rows.insert(0, row);"),
        )
        .unwrap();
        fs::write(root.join("code/schema.sql"), "CREATE TABLE t (id BIGINT);\n").unwrap();

        let out = capture_wave(&root, ws.model(), "p", 1, &[&t1], None).unwrap();
        // `Store` shares terms with the item, the gate edge shares none:
        // pressed carries the first, suppressed reports the second, and the
        // journal takes neither.
        assert!(out.minted.is_empty(), "{}", render_capture(&out));
        assert_eq!(
            out.suppressed,
            vec![Suppressed {
                task: "t1".into(),
                spec_ref: "Gate.out wire Auth.inn".into(),
                item: "code/store.rs#Store::put".into(),
            }]
        );
        assert_eq!(out.pressed["t1"], BTreeSet::from(["Store".to_string()]));
        // The unclaimed change stays a leftover whatever its terms.
        assert_eq!(out.leftovers.len(), 1);
        assert_eq!(out.leftovers[0].to_string(), "code/schema.sql");
        assert!(ls(&root).is_empty());
        assert!(render_capture(&out).contains("suppressed 1 no-signal pair"), "{}", render_capture(&out));

        // Suppression is reporting, not state: the re-run reports the same
        // pair.
        let again = capture_wave(&root, ws.model(), "p", 1, &[&t1], None).unwrap();
        assert_eq!(again.suppressed.len(), 1);

        // A file-level item compares through its path terms too: `Schema`
        // meets `code/schema.sql`, `Store` does not.
        let t2 = task("t2", &["Schema", "Store"], &["code/schema.sql"]);
        let filed = capture_wave(&root, ws.model(), "p", 1, &[&t1, &t2], None).unwrap();
        assert_eq!(filed.pressed["t2"], BTreeSet::from(["Schema".to_string()]));
        assert!(filed
            .suppressed
            .iter()
            .any(|s| s.task == "t2" && s.spec_ref == "Store" && s.item == "code/schema.sql"));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn terms_split_case_and_stop_the_classification_rel() {
        let mut refs = ref_terms("Agent.drive consult(->ModelGraph, <-NKPReport) Cli.nkp");
        for t in ["agent", "drive", "consult", "modelgraph", "model", "graph", "nkpreport", "nkp", "report", "cli"] {
            assert!(refs.remove(t), "missing `{t}`");
        }
        assert!(refs.is_empty(), "extra terms: {refs:?}");
        let stopped = ref_terms("Service type_of Auth");
        assert_eq!(
            stopped,
            BTreeSet::from(["service".to_string(), "auth".to_string()]),
            "the classification rel's own tokens are structure, not signal"
        );
    }
}
