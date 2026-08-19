//! Task-close capture: the wave-open item-hash index and the delta scan
//! that mints candidate links
//! (`archi/requirements/self-hosting/capture-at-the-join.md`).
//!
//! A wave opening records the tree as a **canonical item-hash index** —
//! file → symbol → body hash, by the canonicalizer of [`super::code`] — so
//! a closing task's delta is read off by hash comparison: symbol-granular,
//! cheap to store (no file contents), and git-free by construction, so
//! squashes and shallow clones cannot break it.
//!
//! Capture is idempotent: a pair the journal already holds is not minted
//! twice. A row another task declares again journals a `touch`, once per task
//! — a record of who else met the pair, and nothing more
//! (`archi/requirements/code-link/a-link-stands-asserted-or-it-does-not-stand.md`).
//!
//! The mint comes from the writer, not from the words: each in-flight task
//! writes one **declaration file** beside the index, naming for every symbol
//! it changed the port or requirement that symbol answers and the test that
//! proves it. What the file names becomes an asserted link; what it does not
//! name becomes nothing
//! (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`).
//!
//! Capture reads no task output and compares no term. What it hands the wave
//! gate is two file sets: `moved`, the files the delta touched, and
//! `declared`, the files the in-flight declarations name — read as one set
//! across the wave. A file in the first and not in the second holds the wave
//! open
//! (`archi/requirements/code-link/the-file-in-the-delta-is-the-unit-the-gate-demands.md`).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use modeling_lang::{Model, Statement};
use serde::{Deserialize, Serialize};
use toml::Spanned;

use super::code;
use super::{Anchor, Event, Link, LinkKind, Origin, Rule, SpecRef};
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

// ---- the writer's declaration -------------------------------------------------

/// One task's declaration file, project-relative: beside the index the wave
/// already writes, and named for the wave and the task, so two tasks of one
/// wave never write over each other. Capture hands the path to the wave gate
/// with the task that owes it, so the refusal names the file this function
/// named and no reader spells it twice
/// (`archi/requirements/planning/the-declaration-refusal-repairs-without-guessing.md`).
fn declares_rel(plan: &str, wave: usize, task: &str) -> String {
    format!("archi/plans/{plan}/waves/w{wave:02}.{task}.declares.toml")
}

/// The file's shape: one array of tables, each naming the changed symbol,
/// what it answers and the test that proves it. Every field of an entry is
/// required and no unknown key is tolerated — an optional field is the
/// beginning of a file that always parses
/// (`archi/requirements/planning/the-declaration-refusal-repairs-without-guessing.md`).
///
/// The array itself defaults to empty, because the wave open writes the file
/// before any entry exists: a template of comments holds no `declares` key and
/// still parses, as a file that declares nothing
/// (`archi/requirements/planning/the-wave-opens-a-declaration-file-for-every-task.md`).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Declarations {
    #[serde(default)]
    declares: Vec<Declared>,
}

// ---- the shape, written once ------------------------------------------------

/// The array-of-tables key, and the three keys one entry carries.
/// [`Declared`] deserializes exactly these and tolerates no other, so the
/// template the wave open writes, the entry the verb appends and the shape a
/// refusal prints are one list read three times — the format cannot drift from
/// what the reader accepts, because one side writes what the other reads
/// (`archi/requirements/code-link/a-verb-writes-the-declaration.md`).
const TABLE: &str = "declares";
const SYMBOL: &str = "symbol";
const ANSWERS: &str = "answers";
const PROVED_BY: &str = "proved_by";

/// What each key names, as the template and the refusals spell it.
const SYMBOL_HINT: &str = "<file>#<symbol>";
const ANSWERS_HINT: &str = "<node, port or req:slug>";
const PROVED_BY_HINT: &str = "<test file>#<test fn>";

/// One entry as TOML: the table header and the three keys, each value written
/// by the `toml` crate so the writer never has to think about quoting. The
/// template and the verb both come through here.
fn entry_table(values: [&str; 3]) -> String {
    let mut out = format!("[[{TABLE}]]\n");
    for (key, value) in [SYMBOL, ANSWERS, PROVED_BY].into_iter().zip(values) {
        out.push_str(&format!(
            "{key} = {}\n",
            toml::Value::String(value.to_string())
        ));
    }
    out
}

/// What a refusal that asks for an entry prints under its repair line: the
/// sentence, and one entry's shape — keys and hints — indented beneath it.
/// Both gates that ask for an entry ask in these words, from here, so the two
/// refusals cannot drift apart or from the reader.
pub(crate) fn entry_shape_block() -> String {
    let shape = entry_table([SYMBOL_HINT, ANSWERS_HINT, PROVED_BY_HINT])
        .lines()
        .map(|l| format!("  {l}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("each entry lands as one table of this shape:\n{shape}\n")
}

/// The verb with no task id filled in. The wave gate refuses on a file, and
/// the tasks of a wave share one tree, so that refusal cannot say whose entry
/// the file belongs in — the writer that touched the file knows its own id
/// (`archi/requirements/code-link/the-file-in-the-delta-is-the-unit-the-gate-demands.md`).
pub(crate) const TASK_HINT: &str = "<id>";

/// The one command that appends an entry to a task's file, as the template
/// prints it and every refusal names it.
pub(crate) fn declare_command(task: &str) -> String {
    format!(
        "archi plan task {task} link add --symbol \"{SYMBOL_HINT}\" \
         --answers \"{ANSWERS_HINT}\" --proved-by \"{PROVED_BY_HINT}\""
    )
}

/// The empty declaration file a wave open writes for one task in flight: the
/// shape as comments and no entry. The shape lives in the file rather than in
/// a skill or a prompt, because a prompt is retyped every wave and drifts from
/// the parser while this cannot — the same code writes the template and reads
/// it back
/// (`archi/requirements/planning/the-wave-opens-a-declaration-file-for-every-task.md`).
fn template(task: &str) -> String {
    let mut out = format!(
        "# `{task}` accounts for its work here: for every symbol it changed, what\n\
         # that code answers and the test that proves it. One entry closes the\n\
         # wave; what the file holds is the writer's to decide.\n\
         #\n\
         # Nobody types this file. The verb writes it, and it resolves all three\n\
         # names before it writes:\n\
         #\n\
         #   {}\n\
         #\n\
         # Each entry lands as one table of this shape:\n\
         #\n",
        declare_command(task)
    );
    for line in entry_table([SYMBOL_HINT, ANSWERS_HINT, PROVED_BY_HINT]).lines() {
        out.push_str(&format!("#   {line}\n"));
    }
    out
}

/// Open one wave's declaration files: one empty file per task the wave puts in
/// flight, beside the index the open already writes. So the close never meets
/// an absent file — it meets one that declares nothing, which is one refusal
/// instead of two. Create-only: a file that already stands is the writer's,
/// and re-opening never writes over what it says
/// (`archi/requirements/planning/the-wave-opens-a-declaration-file-for-every-task.md`).
pub(crate) fn open_declarations(
    root: &Path,
    plan: &str,
    wave: usize,
    tasks: &[String],
) -> Result<(), String> {
    for task in tasks {
        let rel = declares_rel(plan, wave, task);
        let path = root.join(&rel);
        if path.exists() {
            continue;
        }
        let dir = path.parent().expect("the declaration has a directory");
        fs::create_dir_all(dir).map_err(|e| format!("cannot create `{}`: {e}", dir.display()))?;
        fs::write(&path, template(task))
            .map_err(|e| format!("cannot write `{rel}`: {e}"))?;
    }
    Ok(())
}

/// Delete one closed wave's working files: the index [`write_index`] wrote
/// and the declaration files [`open_declarations`] opened. The writers name
/// the paths, and this deleter reads the same two functions, so the two
/// cannot disagree. Only a successful `plan next` calls this, after every
/// gate has passed — a blocked close keeps the files, because the retry
/// reads them; `link capture --task` re-reads and never deletes. A file
/// already gone is no error: the files are dead once the wave is closed
/// (`archi/requirements/planning/the-plan-cleans-up-after-itself.md`).
pub(crate) fn remove_wave_files(
    root: &Path,
    plan: &str,
    wave: usize,
    tasks: &[String],
) -> Result<(), String> {
    let mut paths = vec![index_path(root, plan, wave)];
    paths.extend(tasks.iter().map(|task| root.join(declares_rel(plan, wave, task))));
    for path in paths {
        match fs::remove_file(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(format!("cannot remove `{}`: {e}", path.display())),
        }
    }
    Ok(())
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

/// What an `answers` that named an edge is told: the two ends behind it. The
/// reader of a file and the verb that writes one raise it in the same words,
/// because it is the same mistake read at two moments.
fn edge_refusal(path: &str, ends: &(String, String)) -> String {
    let (source, target) = ends;
    format!(
        "`{path}` is an edge — an edge is a caller, and the code behind a port does not know its \
         callers; name the end this symbol answers: `{source}` or `{target}`"
    )
}

/// What one task's declaration file yielded.
#[derive(Default)]
struct Minted {
    /// Freshly minted links.
    links: Vec<Link>,
    /// Ids of live rows this task is a second reader of.
    touched: Vec<String>,
    /// The file side of every entry, as scan keys — read off the same
    /// [`Anchor::parse`] the mint resolves the entry with, so the set the wave
    /// gate matches the delta against cannot disagree with the set the mint
    /// stands on.
    files: BTreeSet<String>,
}

/// Mint one task's declarations: what the file names becomes an asserted
/// link on the symbol that named it, stamped `declared` and carrying the
/// test. A pair the journal already holds is not minted twice, so a wave that
/// re-runs its capture is a no-op — and when the row standing on it was born
/// under another task, this task is a second reader of the same pair and the
/// id comes back to be touched
/// (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`,
/// `archi/requirements/code-link/a-declaration-names-the-test-that-proves-it.md`,
/// `archi/requirements/self-hosting/link-truth-is-append-only.md`).
fn mint_declarations(
    root: &Path,
    model: &Model,
    file: &DeclarationFile,
    task: &str,
    live: &[Link],
) -> Result<Minted, String> {
    let mut out = Minted::default();
    if file.declares.is_empty() {
        return Ok(out);
    }
    let roots = super::Roots::resolve(root)?;
    let edges = edge_ends(model);
    for d in &file.declares {
        let answers = d.answers.get_ref();
        let symbol = d.symbol.get_ref();
        let proved_by = d.proved_by.get_ref();

        // What it answers: a port or a requirement, never an edge.
        let spec = SpecRef::parse(answers)
            .map_err(|e| file.refuse(task, Some(d.answers.span()), &e))?;
        if let Some(ends) = edges.get(&spec.path) {
            return Err(file.refuse(
                task,
                Some(d.answers.span()),
                &edge_refusal(&spec.path, ends),
            ));
        }

        // Where the code is. It is resolved here, before the mint, so a
        // symbol the tree does not hold is refused on the line that named it.
        // The file it names is the wave gate's unit, whatever the entry
        // answers and whether or not the pair below is already held.
        let anchor = Anchor::parse(symbol)
            .map_err(|e| file.refuse(task, Some(d.symbol.span()), &e))?;
        out.files.insert(anchor.qualified_file());
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
        // change what it said. Another task declaring it is another reader,
        // and the journal records who — once.
        let held = live
            .iter()
            .chain(&out.links)
            .find(|l| l.spec.version.is_none() && l.spec.path == spec.path && l.anchor == anchor)
            .map(|l| {
                let re_encounter = !matches!(&l.origin, Origin::Captured { task: t } if t == task)
                    && !l.touches.iter().any(|t| t == task);
                (l.id.clone(), re_encounter)
            });
        if let Some((id, re_encounter)) = held {
            if re_encounter {
                out.touched.push(id);
            }
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
            Some(test),
        )
        .map_err(|e| file.refuse(task, Some(d.answers.span()), &e))?;
        out.links.push(link);
    }
    Ok(out)
}

// ---- the verb that writes one -------------------------------------------------

/// What one `archi plan task <id> link add` did: the file it is about, and
/// whether an entry was appended to it.
pub enum Declaration {
    /// The entry was appended to this file.
    Appended(String),
    /// The file already holds this entry, so nothing was written. The verb
    /// exits 0 on it, so `archi batch -` does not stop on a claim that stands
    /// (`archi/requirements/code-link/a-verb-writes-the-declaration.md`).
    Stands(String),
}

/// The wave a started plan has in flight, and the task ids it holds. The
/// capture by hand and the declaration verb both read the lifecycle this way:
/// a plan whose layering is broken says so, a plan past its last wave says
/// `no_wave` in the caller's own words, and a task the wave does not hold is
/// named beside the ones it does. Only the state check stays with the caller,
/// because these two name different next steps for it.
fn wave_of(
    root: &Path,
    model: &Model,
    plan: &plans::Plan,
    task: &str,
    no_wave: &str,
) -> Result<(usize, Vec<String>), String> {
    let report = plans::verify_plan(root, model, plan)?;
    if !report.errors.is_empty() {
        return Err(format!(
            "the plan is structurally broken — `archi plan verify`:\n  {}",
            report.errors.join("\n  ")
        ));
    }
    let waves = &report.derived.waves;
    if plan.closed_waves >= waves.len() {
        return Err(no_wave.to_string());
    }
    let wave = plan.closed_waves + 1;
    let ids = waves[wave - 1].clone();
    if !ids.iter().any(|id| id == task) {
        return Err(format!(
            "`{task}` is not in wave {wave} — in flight: {}",
            ids.join(", ")
        ));
    }
    Ok((wave, ids))
}

/// The plan and the wave a task is in flight in. A declaration lands in the
/// file the wave open wrote, so outside a started wave there is no file to
/// append to — and the refusal names the lifecycle step that opens one
/// (`archi/requirements/code-link/a-verb-writes-the-declaration.md`).
fn wave_in_flight(root: &Path, model: &Model, task: &str) -> Result<(String, usize), String> {
    let plan = plans::load_active(root)?;
    match plan.state {
        plans::PlanState::Draft => {
            return Err(format!(
                "plan `{}` is draft — `archi plan start` opens wave 1 and writes the file an \
                 entry lands in",
                plan.name
            ));
        }
        plans::PlanState::Completed => {
            return Err(format!(
                "plan `{}` is completed — `archi plan reset` runs it again",
                plan.name
            ));
        }
        plans::PlanState::Started => {}
    }
    let no_wave = format!("plan `{}` has no wave in flight — every wave closed", plan.name);
    let (wave, _) = wave_of(root, model, &plan, task, &no_wave)?;
    Ok((plan.name, wave))
}

/// One anchor argument, resolved against the tree. Every refusal names the
/// flag it came from and what the resolution looked for, because the actor who
/// can fix it is reading this line and not a parser's line number.
fn resolve_named(roots: &super::Roots, flag: &str, text: &str) -> Result<Anchor, String> {
    let refuse = |e: String| format!("{flag} `{text}`: {e}");
    let anchor = Anchor::parse(text).map_err(refuse)?;
    let member_root = roots.require(&anchor.repo).map_err(refuse)?;
    super::resolve_anchor(&member_root, &anchor).map_err(refuse)?;
    Ok(anchor)
}

/// The `--answers` argument, resolved against the model and the requirement
/// set — the same resolution the mint runs, asked before the write instead of
/// after it.
fn resolve_answers(root: &Path, model: &Model, answers: &str) -> Result<(), String> {
    let refuse = |e: String| format!("--answers `{answers}`: {e}");
    let spec = SpecRef::parse(answers).map_err(refuse)?;
    if let Some(ends) = edge_ends(model).get(&spec.path) {
        return Err(refuse(edge_refusal(&spec.path, ends)));
    }
    if super::resolves_at_slot(root, model, &spec)? {
        return Ok(());
    }
    Err(refuse(super::spec_refusal(root, &spec)))
}

/// `archi plan task <id> link add --symbol <anchor> --answers <ref>
/// --proved-by <anchor>`: append one entry to that task's declaration file.
///
/// All three names resolve before anything is written — the symbol and the
/// test against the tree, the ref against the model and the requirement set —
/// so a refusal reaches the actor who can fix it in the same breath as the
/// mistake, and a partial write never happens. The entry is written as TOML by
/// the tool ([`entry_table`]), so nobody types the format and the format
/// cannot drift from what [`read_declarations`] accepts
/// (`archi/requirements/code-link/a-verb-writes-the-declaration.md`).
pub fn declare(
    root: &Path,
    model: &Model,
    task: &str,
    symbol: &str,
    answers: &str,
    proved_by: &str,
) -> Result<Declaration, String> {
    let (plan, wave) = wave_in_flight(root, model, task)?;

    let roots = super::Roots::resolve(root)?;
    resolve_named(&roots, "--symbol", symbol)?;
    let test = resolve_named(&roots, "--proved-by", proved_by)?;
    if test.symbol.is_none() {
        return Err(format!(
            "--proved-by `{proved_by}`: it names a file and no symbol — `--proved-by` names the \
             test itself, as `{PROVED_BY_HINT}`"
        ));
    }
    resolve_answers(root, model, answers)?;

    let rel = declares_rel(&plan, wave, task);
    let path = root.join(&rel);
    // A wave opened before the open wrote the files has none: the verb writes
    // the template under the entry rather than demanding the file first.
    let mut text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => template(task),
        Err(e) => return Err(format!("cannot read `{rel}`: {e}")),
    };
    let parsed: Declarations = toml::from_str(&text)
        .map_err(|e| format!("`{rel}` does not parse: {}", e.message()))?;
    // A repeated identical entry is the same claim, not a second one.
    let stands = parsed.declares.iter().any(|d| {
        d.symbol.get_ref() == symbol
            && d.answers.get_ref() == answers
            && d.proved_by.get_ref() == proved_by
    });
    if stands {
        return Ok(Declaration::Stands(rel));
    }
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push('\n');
    text.push_str(&entry_table([symbol, answers, proved_by]));
    let dir = path.parent().expect("the declaration has a directory");
    fs::create_dir_all(dir).map_err(|e| format!("cannot create `{}`: {e}", dir.display()))?;
    fs::write(&path, text).map_err(|e| format!("cannot write `{rel}`: {e}"))?;
    Ok(Declaration::Appended(rel))
}

// ---- capture -----------------------------------------------------------------

/// One capture's outcome.
#[derive(Default, Serialize)]
pub struct CaptureOutcome {
    /// Freshly minted links — what the in-flight tasks' declarations named,
    /// asserted and stamped `declared`.
    pub minted: Vec<Link>,
    /// `(link id, task)`: a live link another task declared again.
    pub touched: Vec<(String, String)>,
    /// Reporting lines: the members the diff could not cover and why, and the
    /// tasks in flight whose declaration file named nothing. Refusing on the
    /// second is the wave gate's, not this reader's.
    pub notes: Vec<String>,
    /// In-flight tasks that wrote no declaration file: `(task, path)`. What
    /// capture does with an absent file is a note; what the wave does with it
    /// is a refusal, and the gate reads this
    /// (`archi/requirements/planning/an-undeclared-change-refuses-the-wave.md`).
    #[serde(skip)]
    pub absent: Vec<(String, String)>,
    /// In-flight tasks whose file parses and names nothing: `(task, path)`.
    /// A file that declares nothing accounts for nothing, and the wave refuses
    /// on it exactly as it refuses on an absent one
    /// (`archi/requirements/planning/an-undeclared-change-refuses-the-wave.md`).
    #[serde(skip)]
    pub empty: Vec<(String, String)>,
    /// Every file the delta touched, as scan keys. Two gates read it: the
    /// drift gate, which refuses the wave that moved a declared pair and no
    /// other, and the wave gate, which demands an entry for every one of
    /// these files
    /// (`archi/requirements/code-link/a-drifted-declaration-refuses-the-wave-that-moved-it.md`,
    /// `archi/requirements/code-link/the-file-in-the-delta-is-the-unit-the-gate-demands.md`).
    #[serde(skip)]
    pub moved: BTreeSet<String>,
    /// Every file the in-flight declarations name, as scan keys — one set
    /// across the wave, because the tasks of a wave share one tree. A file of
    /// `moved` that is absent here holds the wave open
    /// (`archi/requirements/code-link/the-file-in-the-delta-is-the-unit-the-gate-demands.md`).
    #[serde(skip)]
    pub declared: BTreeSet<String>,
}

/// Capture a wave: mint the in-flight tasks' declarations, then diff the
/// wave-open index against the current tree for the files the delta touched;
/// journal the touches. `only` restricts the read to one task
/// (`link capture --task`), which narrows the declared set with it — the wave
/// gate always reads the whole wave.
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

    let mut events: Vec<Event> = Vec::new();
    // The working link set: the fold plus this batch's mints, so a task sees
    // the rows its wave-mates minted in the same run.
    let mut live: Vec<Link> = folded.live.clone();

    // The mint: the declarations the in-flight tasks wrote. A task that wrote
    // none mints nothing and says so — refusing an absent file is the wave
    // gate's, not this reader's.
    for task in in_flight
        .iter()
        .filter(|t| only.is_none_or(|o| o == t.id))
    {
        match read_declarations(root, plan_name, wave, &task.id)? {
            None => {
                let path = declares_rel(plan_name, wave, &task.id);
                out.notes.push(format!(
                    "`{}` declares nothing: `{path}` is absent — no link is minted for its delta",
                    task.id
                ));
                out.absent.push((task.id.clone(), path));
            }
            Some(file) => {
                // The file it read already holds the path it read it from.
                if file.declares.is_empty() {
                    out.notes.push(format!(
                        "`{}` declares nothing: `{}` names no entry — no link is minted for its \
                         delta",
                        task.id, file.path
                    ));
                    out.empty.push((task.id.clone(), file.path.clone()));
                }
                // The mint reads the file once: the pairs it stands up, the
                // rows it meets again, and the file side of every entry —
                // what the wave gate matches the delta against.
                let read = mint_declarations(root, model, &file, &task.id, &live)?;
                out.declared.extend(read.files);
                for link in read.links {
                    out.minted.push(link.clone());
                    live.push(link);
                }
                for id in read.touched {
                    events.push(Event::Touch {
                        id: id.clone(),
                        task: task.id.clone(),
                        at: super::now(),
                    });
                    out.touched.push((id.clone(), task.id.clone()));
                    if let Some(l) = live.iter_mut().find(|l| l.id == id) {
                        l.touches.push(task.id.clone());
                    }
                }
            }
        }
    }

    // The files the delta touched. The index the open wrote decides this
    // list, so no actor in the wave can shrink it — which is why both closing
    // gates read it.
    out.moved = changes.into_iter().map(|c| c.file).collect();

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
    let (wave, ids) = wave_of(
        root,
        model,
        &plan,
        task_id,
        "no wave in flight — the scenario step is pending",
    )?;
    let in_flight: Vec<&Task> = plan
        .tasks
        .iter()
        .filter(|t| ids.contains(&t.id))
        .collect();
    capture_wave(root, model, &plan.name, wave, &in_flight, Some(task_id))
}

/// The capture outcome as human lines.
pub fn render_capture(o: &CaptureOutcome) -> String {
    let mut out = String::new();
    for l in &o.minted {
        out.push_str(&format!("captured {}\n", super::render_link(l)));
    }
    for (id, task) in &o.touched {
        out.push_str(&format!("touched {id} (declared again under {task})\n"));
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

    use super::super::{append, load, now, resolve_anchor, Birth, Standing};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    const STORE_RS: &str = "pub struct Store {\n    rows: Vec<u8>,\n}\n\n\
                            impl Store {\n    pub fn put(&mut self, row: u8) {\n        self.rows.push(row);\n    }\n}\n";

    /// The smallest model a declaration here resolves against. What every
    /// other name resolves to is the binary's own question, and
    /// `tests/link_e2e.rs` asks it.
    const MODEL: &str = "def node Store\n";

    /// The test a declaration names. It stands before any index is written,
    /// so it is never a change of its own.
    const TESTS_RS: &str = "pub fn a_row_is_persisted() {\n    assert!(true);\n}\n";

    /// The proof every declaration here names.
    const PROOF: &str = "code/tests.rs#a_row_is_persisted";

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
        fs::write(dir.join("code/tests.rs"), TESTS_RS).unwrap();
        dir
    }

    /// One task's declaration file, in the shape the verb writes.
    fn declare_file(root: &Path, wave: usize, task: &str, entries: &[[&str; 2]]) {
        let path = root.join(declares_rel("p", wave, task));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let text: String = entries
            .iter()
            .map(|[symbol, answers]| entry_table([symbol, answers, PROOF]))
            .collect();
        fs::write(&path, text).unwrap();
    }

    fn compiled(root: &Path) -> Workspace {
        modeling_lang::source::compile_project(root)
            .unwrap_or_else(|f| panic!("the fixture model compiles:\n{}", f.render()))
            .workspace
    }

    /// A row a past capture journaled, written straight into the journal:
    /// what a wave does when it meets one again is what these tests are
    /// about, and no capture here mints it.
    fn seed_row(root: &Path, spec: &str, code: &str, task: &str) -> String {
        let anchor = Anchor::parse(code).unwrap();
        let resolved = resolve_anchor(root, &anchor).unwrap();
        let link = Link {
            id: load(root).unwrap().next_id(spec),
            spec: SpecRef::parse(spec).unwrap(),
            anchor,
            kind: LinkKind::Indirect,
            standing: Standing::Asserted,
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
        };
        let id = link.id.clone();
        append(root, &[Event::Add { link }]).unwrap();
        id
    }

    /// A task in flight. Its `spec_refs` and its `## Outputs` stay empty:
    /// capture reads neither, so a task is its id here and nothing else.
    fn task(id: &str) -> Task {
        Task {
            id: id.to_string(),
            node: format!("Node{id}"),
            description: String::new(),
            spec_refs: Vec::new(),
            owns: Vec::new(),
            facts: Vec::new(),
            stack_details: String::new(),
            inputs: BTreeMap::new(),
            outputs: Vec::new(),
            verifications: BTreeMap::new(),
        }
    }

    fn ls(root: &Path) -> Vec<Link> {
        super::super::ls(root, None).unwrap()
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

    /// A task that declared nothing gets nothing, whatever its delta moved.
    /// The delta itself is unchanged and whole: it holds every file that
    /// moved, the one the task was cut for and the one no task claims alike,
    /// and the gate reads that list
    /// (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`,
    /// `archi/requirements/code-link/the-file-in-the-delta-is-the-unit-the-gate-demands.md`).
    #[test]
    fn an_undeclared_delta_mints_nothing_and_the_moved_files_come_back_whole() {
        let root = temp_project();
        let ws = compiled(&root);
        fs::create_dir_all(plans::plan_dir(&root, "p")).unwrap();
        write_index(&root, "p", 1).unwrap();

        let t1 = task("t1");
        fs::write(
            root.join("code/store.rs"),
            STORE_RS.replace("self.rows.push(row);", "self.rows.insert(0, row);"),
        )
        .unwrap();
        fs::write(root.join("code/schema.sql"), "CREATE TABLE t (id BIGINT);\n").unwrap();

        let out = capture_wave(&root, ws.model(), "p", 1, &[&t1], None).unwrap();
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
        // Both files moved and nothing accounts for either.
        assert_eq!(
            out.moved,
            BTreeSet::from(["code/schema.sql".to_string(), "code/store.rs".to_string()])
        );
        assert!(out.declared.is_empty());

        // The re-run says the same and still mints nothing.
        let again = capture_wave(&root, ws.model(), "p", 1, &[&t1], None).unwrap();
        assert!(
            again.minted.is_empty() && again.touched.is_empty(),
            "{}",
            render_capture(&again)
        );

        fs::remove_dir_all(&root).unwrap();
    }

    /// The declared set is the union across the wave, and it is the file side
    /// of each entry: a bare file and a `file#symbol` name the same file, and
    /// an entry may name a file that is no output of the task that wrote it
    /// (`archi/requirements/code-link/the-file-in-the-delta-is-the-unit-the-gate-demands.md`).
    #[test]
    fn the_declared_set_is_the_union_of_the_files_the_entries_name() {
        let root = temp_project();
        let ws = compiled(&root);
        fs::create_dir_all(plans::plan_dir(&root, "p")).unwrap();
        write_index(&root, "p", 1).unwrap();

        let (t1, t2) = (task("t1"), task("t2"));
        fs::write(
            root.join("code/store.rs"),
            STORE_RS.replace("self.rows.push(row);", "self.rows.insert(0, row);"),
        )
        .unwrap();
        fs::write(root.join("code/schema.sql"), "CREATE TABLE t (id BIGINT);\n").unwrap();
        // One entry names a symbol, the other the file whole; neither task
        // has an output at all.
        declare_file(&root, 1, "t1", &[["code/store.rs#Store::put", "Store"]]);
        declare_file(&root, 1, "t2", &[["code/schema.sql", "Store"]]);

        let out = capture_wave(&root, ws.model(), "p", 1, &[&t1, &t2], None).unwrap();
        assert_eq!(out.declared, out.moved, "{}", render_capture(&out));
        assert_eq!(out.minted.len(), 2, "{}", render_capture(&out));

        // `--task` narrows the read, and the declared set with it.
        let one = capture_wave(&root, ws.model(), "p", 1, &[&t1, &t2], Some("t1")).unwrap();
        assert_eq!(one.declared, BTreeSet::from(["code/store.rs".to_string()]));
        assert_eq!(one.moved, out.moved, "the delta is the delta");

        fs::remove_dir_all(&root).unwrap();
    }

    /// A wave that meets a standing row again records who met it: a task
    /// whose declaration names a pair the journal already holds journals one
    /// touch — once, whatever the wave re-runs. The task the row was minted
    /// under records nothing, because it is not another reader
    /// (`archi/requirements/self-hosting/link-truth-is-append-only.md`).
    #[test]
    fn a_touch_journals_once_per_task() {
        let root = temp_project();
        let ws = compiled(&root);
        fs::create_dir_all(plans::plan_dir(&root, "p")).unwrap();
        write_index(&root, "p", 1).unwrap();

        // Two tasks declare the same pair; the row standing on it was minted
        // under `t1`.
        let (t1, t2) = (task("t1"), task("t2"));
        let store_id = seed_row(&root, "Store", "code/store.rs#Store::put", "t1");
        fs::write(
            root.join("code/store.rs"),
            STORE_RS.replace("self.rows.push(row);", "self.rows.insert(0, row);"),
        )
        .unwrap();
        for id in ["t1", "t2"] {
            declare_file(&root, 1, id, &[["code/store.rs#Store::put", "Store"]]);
        }

        let out = capture_wave(&root, ws.model(), "p", 1, &[&t1, &t2], None).unwrap();
        assert!(out.minted.is_empty(), "{}", render_capture(&out));
        assert_eq!(
            out.touched,
            vec![(store_id.clone(), "t2".to_string())],
            "{}",
            render_capture(&out)
        );
        let live = ls(&root);
        let store_link = live.iter().find(|l| l.id == store_id).unwrap();
        assert_eq!(store_link.touches, vec!["t2".to_string()]);

        // Re-runs never double-journal; a third task declaring it is a
        // reader of its own.
        let again = capture_wave(&root, ws.model(), "p", 1, &[&t1, &t2], None).unwrap();
        assert!(again.minted.is_empty() && again.touched.is_empty());
        let t3 = task("t3");
        declare_file(&root, 1, "t3", &[["code/store.rs#Store::put", "Store"]]);
        let third = capture_wave(&root, ws.model(), "p", 1, &[&t1, &t2, &t3], None).unwrap();
        assert!(third.minted.is_empty(), "{}", render_capture(&third));
        assert_eq!(third.touched, vec![(store_id, "t3".to_string())]);

        fs::remove_dir_all(&root).unwrap();
    }

    /// The template and the parser are one code path: the file the wave open
    /// writes parses, and declares nothing; the shape it carries as comments,
    /// uncommented, is exactly one entry the reader accepts — every key
    /// present, no key it refuses. A key that moved on either side fails here
    /// (`archi/requirements/planning/the-wave-opens-a-declaration-file-for-every-task.md`).
    #[test]
    fn the_template_is_the_shape_the_reader_accepts() {
        let root = temp_project();
        fs::create_dir_all(plans::plan_dir(&root, "p")).unwrap();
        open_declarations(&root, "p", 1, &["t1".to_string(), "t2".to_string()]).unwrap();

        for task in ["t1", "t2"] {
            let file = read_declarations(&root, "p", 1, task)
                .unwrap()
                .expect("the open wrote it");
            assert!(file.declares.is_empty(), "it declares nothing: {}", file.text);
            assert_eq!(file.path, declares_rel("p", 1, task));
            assert!(
                file.text.contains(&declare_command(task)),
                "it names the verb that fills it: {}",
                file.text
            );

            // The commented shape, uncommented, is one entry.
            let uncommented: String = file
                .text
                .lines()
                .filter_map(|l| l.strip_prefix("#   "))
                .skip_while(|l| !l.starts_with("[["))
                .collect::<Vec<_>>()
                .join("\n");
            let parsed: Declarations = toml::from_str(&uncommented)
                .unwrap_or_else(|e| panic!("the shape parses:\n{uncommented}\n{e}"));
            assert_eq!(parsed.declares.len(), 1, "{uncommented}");
            let one = &parsed.declares[0];
            assert_eq!(one.symbol.get_ref(), SYMBOL_HINT);
            assert_eq!(one.answers.get_ref(), ANSWERS_HINT);
            assert_eq!(one.proved_by.get_ref(), PROVED_BY_HINT);
        }

        // Create-only: a re-open never writes over what a file says.
        let entry = entry_table(["code/store.rs#Store::put", "Store", "code/t.rs#t"]);
        let path = root.join(declares_rel("p", 1, "t1"));
        fs::write(&path, &entry).unwrap();
        open_declarations(&root, "p", 1, &["t1".to_string()]).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), entry);
        // And what the tool writes is what the reader reads back.
        let file = read_declarations(&root, "p", 1, "t1").unwrap().unwrap();
        assert_eq!(file.declares.len(), 1);
        assert_eq!(file.declares[0].symbol.get_ref(), "code/store.rs#Store::put");

        fs::remove_dir_all(&root).unwrap();
    }
}
