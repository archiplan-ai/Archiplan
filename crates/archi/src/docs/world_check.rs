//! The world wing's pass over the doc tree
//! (`archi/requirements/world-facts/`): discovery under `archi/world/`, the
//! two references the record leaves to the compiler — `covers` against the
//! compiled model, `uses` against the other facts — the `uses` graph, and the
//! reports.
//!
//! Three reports, on three subjects. One line per fact carries every state
//! that fact holds at once
//! (`archi/requirements/world-facts/one-fact-reports-one-finding.md`), the
//! `uses` graph carries the ring and the deep chain
//! (`archi/requirements/world-facts/the-wing-reports-what-stands-in-the-air.md`),
//! and the model carries what no fact reaches
//! (`archi/requirements/world-facts/coverage-reaches-down-the-graph.md`) —
//! minus what its type puts outside the question and what `.worldignore`
//! declares internal
//! (`archi/requirements/world-facts/an-internal-element-says-so.md`).
//! Nothing here blocks except an unresolved reference and a ring, and a tree
//! with no `archi/world/` folder reports nothing at all — a project that has
//! not opted into the wing is not behind on it
//! (`archi/requirements/world-facts/the-wing-arrives-without-noise.md`).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use modeling_lang::{Definition, Layer, Model, Statement};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::gherkin::{self, ScenarioBlock};
use super::md::slugify;
use super::world::{self, WORLD, WorldDoc};
use super::{DocDiagnostic, Tree, is_md, read_doc, rel, sorted_entries, stem};

/// A `uses` chain of this many facts still reads; one deeper holds a theory
/// of the world instead of a record of it.
const CHAIN_MAX: usize = 3;

/// The ontology term whose instances leave the coverage question
/// (`archi/requirements/world-facts/coverage-reaches-down-the-graph.md`).
const DATA: &str = "Data";

/// The declaration of what is internal, beside the wing it quiets
/// (`archi/requirements/world-facts/an-internal-element-says-so.md`).
const IGNORE: &str = ".worldignore";

/// The layer of the strict record: the condition, its workaround, its scenarios
/// and the three lists (`archi/requirements/world-facts/the-world-holds-four-layers.md`).
const FACTS: &str = "facts";

/// The layer of a claim somebody means to settle and has not.
const HYPOTHESES: &str = "hypotheses";

/// The layer of what was seen or heard and not yet shaped into either.
const NOTES: &str = "notes";

/// The layer of raw material. Never parsed — listed so a `sources` entry can
/// resolve against it, and opened by nothing else.
const RESOURCES: &str = "resources";

/// The four layers, in the order a reader meets them.
const LAYERS: [&str; 4] = [FACTS, HYPOTHESES, NOTES, RESOURCES];

/// What one `.worldignore` line writes between the element and the reason it
/// is internal — `Element — why nothing outside reaches it`. No element path
/// carries the dash, so the space around it is free.
const REASON: char = '—';

/// One world fact as the wing holds it: the record the reader parsed and the
/// scenarios the grammar accepted.
pub(crate) struct WorldFact {
    /// The record — the three lists, the paragraph, the workaround.
    pub(crate) doc: WorldDoc,
    /// The scenarios that parsed; `None` when none was sound.
    pub(crate) scenarios: Option<ScenarioBlock>,
}

/// The wing keyed by what it covers — the one question the planner, the
/// links and the query surface ask of it: which facts condition this
/// element.
pub(crate) struct Wing<'a> {
    facts: &'a [WorldFact],
    by_element: BTreeMap<&'a str, Vec<usize>>,
}

impl<'a> Wing<'a> {
    /// The facts naming `element` in `covers`, in slug order.
    pub(crate) fn covering(&self, element: &str) -> Vec<&'a WorldFact> {
        self.by_element
            .get(element)
            .map(|v| v.iter().map(|i| &self.facts[*i]).collect())
            .unwrap_or_default()
    }

    /// Every element any fact covers, in path order.
    pub(crate) fn elements(&self) -> impl Iterator<Item = &'a str> {
        self.by_element.keys().copied()
    }
}

/// A state a fact holds, or a decay of the wing around it. Kinds are
/// append-only, like the doc findings beside them.
#[derive(Serialize)]
#[serde(tag = "kind")]
pub enum WorldFinding {
    /// Every state one fact holds, in one line
    /// (`archi/requirements/world-facts/one-fact-reports-one-finding.md`).
    #[serde(rename = "world_state")]
    State {
        /// The fact's slug.
        fact: String,
        /// The states it holds, named as the requirements name them.
        states: Vec<String>,
    },
    /// A `uses` chain running deeper than [`CHAIN_MAX`], top first.
    #[serde(rename = "world_chain_deep")]
    ChainDeep {
        /// The facts on the chain, in `uses` order.
        facts: Vec<String>,
    },
    /// A model element no fact's coverage reaches.
    #[serde(rename = "world_unreached")]
    Unreached {
        /// The element's path.
        element: String,
    },
}

impl fmt::Display for WorldFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorldFinding::State { fact, states } => {
                write!(f, "world fact `{fact}`: {}", states.join(", "))
            }
            WorldFinding::ChainDeep { facts } => write!(
                f,
                "world_chain_deep: {} — a fact that rests on a fact that rests on a fact holds \
                 a theory of the world, not a record of one",
                facts.join(" → ")
            ),
            WorldFinding::Unreached { element } => {
                write!(f, "world_unreached: {element} — no world fact reaches it")
            }
        }
    }
}

/// The wing in two numbers — how many facts stand and how many rest on
/// nothing recorded
/// (`archi/requirements/world-facts/the-check-counts-the-wing.md`).
#[derive(Serialize)]
pub struct WorldCount {
    /// The facts that loaded.
    pub facts: usize,
    /// Those of them with an empty `sources`.
    pub ungrounded: usize,
}

impl fmt::Display for WorldCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "world — {} facts · {} ungrounded",
            self.facts, self.ungrounded
        )
    }
}

/// What the wing's pass reports: its advisory findings and its count line.
/// A tree with no wing carries neither.
#[derive(Default)]
pub struct WorldReport {
    /// Advisory, never blocking.
    pub findings: Vec<WorldFinding>,
    /// The closing line; `None` when the tree holds no fact.
    pub count: Option<WorldCount>,
}

/// Walk `archi/world/` — four folders, and the folder decides what a file is
/// (`archi/requirements/world-facts/the-world-holds-four-layers.md`).
/// [`FACTS`] holds the strict record and is read the way every doc primitive
/// is read: structure, then schema, then the grammar over the `Scenarios`
/// block. [`HYPOTHESES`] and [`NOTES`] hold a name and their prose and are
/// read for exactly that. [`RESOURCES`] is opened by nothing. Four is the
/// whole count, so a document the four do not hold is refused by path before
/// any of them is read ([`outside_the_layers`]).
///
/// Nothing here creates a folder, and a tree holding none of the four says
/// nothing at all — a project that has not opted into a layer is not behind
/// on it (`archi/requirements/world-facts/the-wing-arrives-without-noise.md`).
pub(crate) fn discover(root: &Path, diags: &mut Vec<DocDiagnostic>) -> Vec<WorldFact> {
    let base = root.join(WORLD);
    if !base.is_dir() {
        return Vec::new();
    }
    // A document no layer holds sits in no layer, and the layer is what says
    // how a file is read.
    for path in outside_the_layers(&base) {
        diags.push(no_layer(root, &base, &path));
    }
    // The loose layers carry a name and their prose and nothing else: no
    // workaround, no scenarios, no lists to keep true.
    for layer in [HYPOTHESES, NOTES] {
        for path in layer_files(&base, layer) {
            loose(root, &path, diags);
        }
    }
    layer_files(&base, FACTS)
        .into_iter()
        .filter_map(|path| read_fact(root, &path, diags))
        .collect()
}

/// The documents of one layer, in path order.
fn layer_files(base: &Path, layer: &str) -> Vec<PathBuf> {
    sorted_entries(&base.join(layer))
        .into_iter()
        .filter(|p| is_md(p))
        .collect()
}

/// Every document under the wing that no layer holds: a file loose at the
/// top, and every `.md` under a folder that is not one of [`LAYERS`], at any
/// depth. The fifth folder is why the walk descends — a rule that reads the
/// four by name and steps past the rest holds four layers and a remainder no
/// rule describes, and a folder somebody made in a hurry takes files, keeps
/// them out of every reading and reports nothing
/// (`archi/requirements/world-facts/the-world-holds-four-layers.md`).
///
/// A document is a `.md`, so [`IGNORE`] is no document and stays out of the
/// walk, and so does raw material of any other extension. A folder holding
/// none says nothing: the rule locates a file, and there is no file to
/// locate.
fn outside_the_layers(base: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in sorted_entries(base) {
        if entry.is_dir() {
            if !LAYERS.iter().any(|l| entry.ends_with(l)) {
                documents_under(&entry, &mut out);
            }
        } else if is_md(&entry) {
            out.push(entry);
        }
    }
    out
}

/// Every `.md` under `dir`, at any depth, in path order.
fn documents_under(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in sorted_entries(dir) {
        if entry.is_dir() {
            documents_under(&entry, out);
        } else if is_md(&entry) {
            out.push(entry);
        }
    }
}

/// The refusal the placement rule raises, wherever the walk found the file:
/// the four layers by path, and the document by the path it sits at under
/// the wing. It is built in one place, so the loose file at the top and the
/// document in a fifth folder can never name different layers
/// (`archi/requirements/world-facts/the-world-holds-four-layers.md`).
fn no_layer(root: &Path, base: &Path, path: &Path) -> DocDiagnostic {
    DocDiagnostic::new(
        "E_PLACEMENT",
        format!(
            "`{}` sits in no layer of the world — a file lives under one of `{}`, and the \
             folder is what says how it is read",
            rel(base, path),
            LAYERS
                .iter()
                .map(|l| format!("{WORLD}{l}/"))
                .collect::<Vec<_>>()
                .join("`, `")
        ),
        &rel(root, path),
        1,
    )
}

/// One file of a loose layer. It needs a name and its prose, and the reader
/// already locates a file that carries no name; this locates one that carries
/// no prose under it. Nothing else is asked of it, and nothing is kept: the
/// wing reads these layers so a fact's `sources` has something to resolve
/// against (`archi/requirements/world-facts/the-world-holds-four-layers.md`).
fn loose(root: &Path, path: &Path, diags: &mut Vec<DocDiagnostic>) {
    let Some((file, doc)) = read_doc(root, path, diags) else {
        return;
    };
    if doc.summary.is_empty() {
        diags.push(DocDiagnostic::new(
            "E_DOC",
            "a note or a hypothesis is a name and the prose under it",
            &file,
            doc.name_line,
        ));
    }
}

/// One fact, read from one file: structure, then schema, then the grammar
/// over the `Scenarios` block. `None` when nothing readable stands there —
/// [`discover`] skips such a file, and a reader after one fact has none.
///
/// It is the one reader: the wing walks the folder through it and a link
/// reads a single slug through it, so the two can never read one file into
/// two different stories.
pub(crate) fn read_fact(
    root: &Path,
    path: &Path,
    diags: &mut Vec<DocDiagnostic>,
) -> Option<WorldFact> {
    let (file, doc) = read_doc(root, path, diags)?;
    let record = world::parse(&doc, &file, &stem(path), root, diags);
    let scenarios = record
        .scenarios
        .as_ref()
        .and_then(|b| gherkin::parse(b, &file, diags));
    Some(WorldFact {
        doc: record,
        scenarios,
    })
}

/// The wing of a loaded tree, keyed by what its facts cover.
pub(crate) fn serve_world(tree: &Tree) -> Wing<'_> {
    let mut by_element: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, f) in tree.world.iter().enumerate() {
        for e in f.doc.covers.iter().flat_map(|(v, _)| v) {
            by_element.entry(e.as_str()).or_default().push(i);
        }
    }
    Wing {
        facts: &tree.world,
        by_element,
    }
}

/// Cross-check the wing: the two references the record left open, the shape
/// of the `uses` graph, the state of every fact, and the reach of the whole
/// wing over the model. An empty wing is checked by saying nothing — the
/// declaration file beside it is read only where there is coverage to quiet.
pub(crate) fn check(
    root: &Path,
    model: &Model,
    tree: &Tree,
    diags: &mut Vec<DocDiagnostic>,
) -> WorldReport {
    let facts = &tree.world;
    if facts.is_empty() {
        return WorldReport::default();
    }
    resolve_refs(model, facts, diags);
    let mut findings = Vec::new();
    per_fact(model, facts, &mut findings);
    graph(facts, &mut findings, diags);
    findings.extend(
        unreached(root, model, tree, diags)
            .into_iter()
            .map(|element| WorldFinding::Unreached { element }),
    );
    WorldReport {
        findings,
        count: Some(WorldCount {
            facts: facts.len(),
            ungrounded: facts.iter().filter(|f| f.doc.ungrounded()).count(),
        }),
    }
}

/// The two lists the record carried unresolved: `covers` against the live
/// model, the vocabulary `satisfied-by` speaks, and `uses` against the other
/// facts by slug. Both are located on the frontmatter line they sit on.
fn resolve_refs(model: &Model, facts: &[WorldFact], diags: &mut Vec<DocDiagnostic>) {
    let slugs: BTreeSet<&str> = facts.iter().map(|f| f.doc.slug.as_str()).collect();
    for f in facts {
        if let Some((entries, line)) = &f.doc.covers {
            for p in entries {
                if model.resolve_element(p).is_none() {
                    diags.push(DocDiagnostic::new(
                        "E_MODEL_REF",
                        format!("covers names no element `{p}` of the current model"),
                        &f.doc.file,
                        *line,
                    ));
                }
            }
        }
        if let Some((entries, line)) = &f.doc.uses {
            for u in entries {
                if !slugs.contains(u.as_str()) {
                    diags.push(DocDiagnostic::new(
                        "E_DOC_REF",
                        format!("uses names no world fact `{u}`"),
                        &f.doc.file,
                        *line,
                    ));
                }
            }
        }
    }
}

/// One line per fact, carrying every state it holds at once
/// (`archi/requirements/world-facts/one-fact-reports-one-finding.md`). A
/// healthy fact carries none and says nothing.
fn per_fact(model: &Model, facts: &[WorldFact], findings: &mut Vec<WorldFinding>) {
    let names = element_names(model);
    let used: BTreeSet<&str> = facts
        .iter()
        .filter_map(|f| f.doc.uses.as_ref())
        .flat_map(|(v, _)| v.iter().map(String::as_str))
        .collect();
    for f in facts {
        let mut states = Vec::new();
        // The grounding, and its absence, whatever else the fact carries.
        if f.doc.ungrounded() {
            states.push("world_ungrounded".to_string());
        }
        // An empty `covers` is early work, not a defect: the condition is
        // recorded and the model has not reached it yet.
        if f.doc.covers.as_ref().is_some_and(|(v, _)| v.is_empty()) {
            states.push("world_uncovered".to_string());
        }
        // Dead weight: nothing rests on it and it dictates nothing.
        if !used.contains(f.doc.slug.as_str()) && f.scenarios.is_none() {
            states.push("world_orphan".to_string());
        }
        if let Some(element) = speaks_the_model(f, &names) {
            states.push(format!("world_speaks_the_model({element})"));
        }
        if !states.is_empty() {
            findings.push(WorldFinding::State {
                fact: f.doc.slug.clone(),
                states,
            });
        }
    }
}

/// The element the fact's own vocabulary borrowed from the model, the
/// fullest name first; `None` when it borrowed none. The name and the
/// conditioning paragraph are the whole search: the `Scenarios` block and the
/// workaround are exempt, because Gherkin describes the system's surface and
/// naming it there is the point
/// (`archi/requirements/world-facts/the-fact-speaks-the-world-and-check-says-when-it-does-not.md`).
fn speaks_the_model(fact: &WorldFact, names: &BTreeSet<String>) -> Option<String> {
    let condition = fact.doc.condition.as_ref().map_or("", |b| b.text.as_str());
    names
        .iter()
        .filter(|n| names_it(condition, n) || in_name(&fact.doc.slug, n))
        .min_by_key(|n| (std::cmp::Reverse(n.len()), n.as_str()))
        .cloned()
}

/// Whether the prose names `name` — a whole word, case as the model writes
/// it, so `Service` inside `AuthService` is no match.
fn names_it(text: &str, name: &str) -> bool {
    let edge = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric() && c != '_');
    let mut at = 0;
    while let Some(i) = text[at..].find(name) {
        let (start, end) = (at + i, at + i + name.len());
        if edge(text[..start].chars().next_back()) && edge(text[end..].chars().next()) {
            return true;
        }
        at = end;
    }
    false
}

/// Whether the fact's name holds the element's — read on the slug both
/// derive to, which is what the tree keeps of the title.
fn in_name(slug: &str, name: &str) -> bool {
    let derived = slugify(name);
    !derived.is_empty() && format!("-{slug}-").contains(&format!("-{derived}-"))
}

/// The reports the shape of the `uses` graph carries: the ring, which
/// blocks, and the chain that outgrew a reading, which does not
/// (`archi/requirements/world-facts/the-wing-reports-what-stands-in-the-air.md`).
fn graph(facts: &[WorldFact], findings: &mut Vec<WorldFinding>, diags: &mut Vec<DocDiagnostic>) {
    let slugs: BTreeSet<&str> = facts.iter().map(|f| f.doc.slug.as_str()).collect();
    let adj: BTreeMap<&str, Vec<&str>> = facts
        .iter()
        .map(|f| {
            let out = f
                .doc
                .uses
                .iter()
                .flat_map(|(v, _)| v.iter().map(String::as_str))
                .filter(|u| slugs.contains(u))
                .collect();
            (f.doc.slug.as_str(), out)
        })
        .collect();

    let rings = rings(&adj);
    for ring in &rings {
        let fact = facts
            .iter()
            .find(|f| f.doc.slug == ring[0])
            .expect("a ring is walked over the loaded facts");
        diags.push(DocDiagnostic::new(
            "E_DOC_REF",
            format!(
                "`uses` closes a ring: {} — a fact holds only while the facts it uses hold, and \
                 a ring holds nothing up",
                ring.join(" → ")
            ),
            &fact.doc.file,
            fact.doc.uses.as_ref().map_or(fact.doc.line, |(_, l)| *l),
        ));
    }
    // A ring has no longest chain, and the wing already knows about it.
    if !rings.is_empty() {
        return;
    }
    for chain in deep_chains(&adj) {
        findings.push(WorldFinding::ChainDeep { facts: chain });
    }
}

/// Every ring in the `uses` graph, each named once — the walk re-enters a
/// fact it is still standing on, and the ring is the stack from that fact
/// back to it.
fn rings<'a>(adj: &BTreeMap<&'a str, Vec<&'a str>>) -> Vec<Vec<&'a str>> {
    let mut open: BTreeSet<&str> = BTreeSet::new();
    let mut done: BTreeSet<&str> = BTreeSet::new();
    let mut stack: Vec<&str> = Vec::new();
    let mut seen: BTreeSet<BTreeSet<&str>> = BTreeSet::new();
    let mut out = Vec::new();
    for node in adj.keys() {
        if !done.contains(node) {
            ring_walk(
                node, adj, &mut open, &mut done, &mut stack, &mut seen, &mut out,
            );
        }
    }
    out
}

fn ring_walk<'a>(
    node: &'a str,
    adj: &BTreeMap<&'a str, Vec<&'a str>>,
    open: &mut BTreeSet<&'a str>,
    done: &mut BTreeSet<&'a str>,
    stack: &mut Vec<&'a str>,
    seen: &mut BTreeSet<BTreeSet<&'a str>>,
    out: &mut Vec<Vec<&'a str>>,
) {
    open.insert(node);
    stack.push(node);
    for next in adj.get(node).into_iter().flatten() {
        if open.contains(next) {
            let from = stack.iter().position(|s| s == next).unwrap_or(0);
            let mut ring: Vec<&str> = stack[from..].to_vec();
            ring.push(next);
            if seen.insert(ring.iter().copied().collect()) {
                out.push(ring);
            }
        } else if !done.contains(next) {
            ring_walk(next, adj, open, done, stack, seen, out);
        }
    }
    stack.pop();
    open.remove(node);
    done.insert(node);
}

/// The longest chain under every fact nothing rests on — the tops of the
/// graph — kept when it runs deeper than [`CHAIN_MAX`]. The graph is a DAG
/// here: the rings were reported and the walk stopped.
fn deep_chains<'a>(adj: &BTreeMap<&'a str, Vec<&'a str>>) -> Vec<Vec<String>> {
    let used: BTreeSet<&str> = adj.values().flatten().copied().collect();
    let mut longest: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut out = Vec::new();
    for top in adj.keys().filter(|s| !used.contains(*s)) {
        let chain = chain_under(top, adj, &mut longest);
        if chain.len() > CHAIN_MAX {
            out.push(chain.iter().map(|s| (*s).to_string()).collect());
        }
    }
    out
}

fn chain_under<'a>(
    node: &'a str,
    adj: &BTreeMap<&'a str, Vec<&'a str>>,
    longest: &mut BTreeMap<&'a str, Vec<&'a str>>,
) -> Vec<&'a str> {
    if let Some(known) = longest.get(node) {
        return known.clone();
    }
    let mut best: Vec<&str> = Vec::new();
    for next in adj.get(node).into_iter().flatten() {
        let under = chain_under(next, adj, longest);
        if under.len() > best.len() {
            best = under;
        }
    }
    let mut chain = vec![node];
    chain.extend(best);
    longest.insert(node, chain.clone());
    chain
}

/// What the wing reaches, and what it leaves standing alone. A node is
/// covered when a fact names it, when a declared connection edge carries the
/// coverage into it, or when it sits inside a covered node — scenarios run
/// from the surface inward, so a fact on an outer service carries most of a
/// model (`archi/requirements/world-facts/coverage-reaches-down-the-graph.md`).
///
/// Two kinds of element never stand on the list: what the model classifies
/// as [`DATA`], which the question does not apply to, and what a person
/// declared internal in [`IGNORE`]
/// (`archi/requirements/world-facts/an-internal-element-says-so.md`).
///
/// This is the one computation, and it has two readers: `check` renders each
/// element as an advisory finding, and `archi version save` refuses on the
/// whole set ([`unreached_at`]). One function, so the report and the refusal
/// can never disagree about an element
/// (`archi/requirements/world-facts/the-save-refuses-an-unconditioned-element.md`).
fn unreached(
    root: &Path,
    model: &Model,
    tree: &Tree,
    diags: &mut Vec<DocDiagnostic>,
) -> Vec<String> {
    let carried = data_elements(model);
    let declared = internal(root, model, &carried, diags);
    let dump = model.dump();
    let mut nodes: BTreeSet<&str> = BTreeSet::new();
    let mut directed: BTreeMap<&str, bool> = BTreeMap::new();
    for s in &dump {
        match s {
            Statement::Define(Definition::Node { path, .. }) => {
                nodes.insert(path);
            }
            Statement::Define(Definition::Conn {
                name, directed: d, ..
            }) => {
                directed.insert(name, *d);
            }
            _ => {}
        }
    }
    let mut edges: Vec<(&str, &str, bool)> = Vec::new();
    for s in &dump {
        if let Statement::ConnEdge {
            conn,
            source,
            target,
            ..
        } = s
        {
            let two_way = !directed.get(conn.as_str()).copied().unwrap_or(true);
            edges.push((&source.node, &target.node, two_way));
        }
    }

    // The seeds: what the facts name. A port entry seeds the node that
    // declares it — a condition on the interface is a condition on what
    // stands behind it. An entry naming neither was already reported.
    let wing = serve_world(tree);
    let mut frontier: Vec<&str> = Vec::new();
    for e in wing.elements() {
        if nodes.contains(e) {
            frontier.push(e);
        } else if let Some((owner, _)) = e.rsplit_once('.')
            && nodes.contains(owner)
        {
            frontier.push(owner);
        }
    }
    let mut covered: BTreeSet<&str> = BTreeSet::new();
    while let Some(node) = frontier.pop() {
        if !covered.insert(node) {
            continue;
        }
        for &(src, dst, two_way) in &edges {
            if src == node {
                frontier.push(dst);
            } else if two_way && dst == node {
                frontier.push(src);
            }
        }
        let inside = format!("{node}.");
        frontier.extend(nodes.iter().filter(|p| p.starts_with(&inside)).copied());
    }
    nodes
        .iter()
        .filter(|n| !covered.contains(*n) && !carried.contains(**n) && !declared.contains(**n))
        .map(|n| (*n).to_string())
        .collect()
}

/// The same set, off a tree the caller has not loaded — what
/// `archi version save` refuses on
/// (`archi/requirements/world-facts/the-save-refuses-an-unconditioned-element.md`).
///
/// It reads the `facts/` layer alone, because the wing's threshold is a fact:
/// a project that has written none saves exactly as it did before the gate
/// existed (`archi/requirements/world-facts/the-wing-arrives-without-noise.md`).
/// What the read locates on the way — a loose file, a broken declaration — is
/// `check`'s to report and is dropped here: the save says one thing, and the
/// operator who wants the rest runs `archi check`.
pub(crate) fn unreached_at(root: &Path, model: &Model) -> Vec<String> {
    let mut diags = Vec::new();
    let facts = discover(root, &mut diags);
    if facts.is_empty() {
        return Vec::new();
    }
    let tree = Tree {
        world: facts,
        ..Tree::default()
    };
    unreached(root, model, &tree, &mut diags)
}

/// Every element the model classifies as [`DATA`] — by its type, not by a
/// list of names. A payload rides inside a connection and is never its
/// destination, so asking whether a behavior arrives at one is a question of
/// the wrong kind. A model whose ontology holds no such classifier — or
/// classifies nothing under it — leaves nothing out
/// (`archi/requirements/world-facts/coverage-reaches-down-the-graph.md`).
fn data_elements(model: &Model) -> BTreeSet<String> {
    if model.layer_of(DATA) != Some(Layer::Epistemic) {
        return BTreeSet::new();
    }
    model.term_surface(DATA).into_iter().flatten().collect()
}

/// The elements a person declared internal, read from [`IGNORE`] beside the
/// wing. One line is one element path, then [`REASON`], then why nothing
/// outside reaches it; blank lines and `#` comments are skipped, and a tree
/// with no file declares nothing.
///
/// Every entry resolves against the compiled model — the contract `covers`
/// carries, so a rename breaks the file loudly instead of muting an element
/// that no longer exists. The reason is mandatory: the entry is a claim a
/// reader can argue with, not a mute. And an element the model already
/// classifies as [`DATA`] is refused, because its type excludes it already
/// (`archi/requirements/world-facts/an-internal-element-says-so.md`).
fn internal(
    root: &Path,
    model: &Model,
    carried: &BTreeSet<String>,
    diags: &mut Vec<DocDiagnostic>,
) -> BTreeSet<String> {
    let path = root.join("archi").join("world").join(IGNORE);
    if !path.is_file() {
        return BTreeSet::new();
    }
    let file = format!("archi/world/{IGNORE}");
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                format!("cannot read the file: {e}"),
                &file,
                1,
            ));
            return BTreeSet::new();
        }
    };
    let mut out = BTreeSet::new();
    for (i, raw) in text.lines().enumerate() {
        let (entry, line) = (raw.trim(), i + 1);
        if entry.is_empty() || entry.starts_with('#') {
            continue;
        }
        let (element, reason) = match entry.split_once(REASON) {
            Some((e, r)) => (e.trim(), r.trim()),
            None => (entry, ""),
        };
        if reason.is_empty() {
            diags.push(DocDiagnostic::new(
                "E_DOC",
                format!(
                    "`{IGNORE}` states no reason for `{element}` — one line is `<element> {REASON} \
                     why nothing outside reaches it`, and a claim a reader cannot argue with is a \
                     mute"
                ),
                &file,
                line,
            ));
        } else if model.resolve_element(element).is_none() {
            diags.push(DocDiagnostic::new(
                "E_MODEL_REF",
                format!("`{IGNORE}` names no element `{element}` of the current model"),
                &file,
                line,
            ));
        } else if carried.contains(element) {
            diags.push(DocDiagnostic::new(
                "E_MODEL_REF",
                format!(
                    "`{IGNORE}` names `{element}`, which the model classifies as `{DATA}` — its \
                     type already leaves the coverage question"
                ),
                &file,
                line,
            ));
        } else {
            out.insert(element.to_string());
        }
    }
    out
}

/// The fingerprint of a fact's scenarios: the names and the steps, hashed to
/// six hex digits. It is not the story — nothing that reads it holds a copy
/// of one — it is only enough to say the story moved. It sits beside the
/// parsed block so the plan and the link read one function.
///
/// The grain is the argument, and that is why there is one function
/// (`archi/requirements/world-facts/the-digest-witnesses-one-scenario.md`).
/// `Some(name)` fingerprints that scenario alone — its heading name, its step
/// keywords and its step text — so a sibling reworded in the same fact moves
/// nothing a link on this scenario stands on. `None` is every scenario of the
/// block in source order: the plan's drift line asks whether the fact moved,
/// not whether one scenario did.
///
/// The whole-block value moved once, when the shape retired the `Feature:`
/// line the digest used to hash first. Nothing carries the old value forward:
/// a drift line pinned under the old shape is re-pinned, and a link that
/// witnessed a whole block is repinned by the operator who reads the two
/// sides against each other.
///
/// The name is matched as an address is matched — inner whitespace collapsed
/// on both sides — because the grammar keeps a name as written and a ref
/// arrives normalized. A name the fact does not hold fingerprints the empty
/// story, the same nothing a block the grammar refused gives.
pub(crate) fn scenario_digest(fact: &WorldFact, scenario: Option<&str>) -> String {
    let wanted = scenario.map(crate::links::normalize_ref);
    let mut text = String::new();
    if let Some(block) = &fact.scenarios {
        for s in &block.scenarios {
            if wanted
                .as_deref()
                .is_some_and(|w| w != crate::links::normalize_ref(&s.name))
            {
                continue;
            }
            text.push('\u{1f}');
            text.push_str(&s.name);
            for step in &s.steps {
                text.push('\u{1f}');
                text.push_str(&step.keyword);
                text.push(' ');
                text.push_str(&step.text);
            }
        }
    }
    Sha256::digest(text.as_bytes())
        .iter()
        .take(3)
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Every name the compiled model answers to: a node's path and the bare name
/// it wears inside its container, and a declared port under its node's path.
/// A bare port name never stands alone — it is an ordinary English word too
/// often for a match in prose to mean anything.
fn element_names(model: &Model) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for s in model.dump() {
        if let Statement::Define(Definition::Node { path, ports, .. }) = s {
            if let Some(leaf) = path.rsplit('.').next() {
                out.insert(leaf.to_string());
            }
            for p in ports.iter().flatten() {
                out.insert(format!("{path}.{p}"));
            }
            out.insert(path);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::super::{DocReport, check as docs_check, load};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    /// A gate that reaches an engine holding a child, an island nothing
    /// reaches, and a payload the ontology classifies as `Data` — the
    /// smallest model the coverage walk has something to say about.
    const MODEL: &str = "\
def node Gate:
  port out
def node Engine:
  port take
  def node Inner
def node Island
def node Payload
def conn calls := * -> *
Gate.out calls Engine.take
Data type_of Payload
";

    /// The block every fact carries unless the test says otherwise: a `### `
    /// heading names the scenario, and the four keywords open its steps.
    const SCENARIOS: &str = "\
### the app opens with no network

Given the device has no network
When the user opens the app
Then the last synced view appears
";

    /// A second scenario of the same block — the sibling the grain is about.
    const SIBLING: &str = "
### the app opens on a slow line

Given the device has one bar of signal
When the user opens the app
Then the view arrives late
";

    /// The scenario the narrow grain is read on, as a ref addresses it.
    const NAMED: &str = "the app opens with no network";

    /// The material every grounded fact here rests on: a note of the world,
    /// named as a `sources` entry names it — a path from the project root
    /// into `archi/world/`.
    const SOURCE: &str = "archi/world/notes/the-guard-walked-the-platform.md";

    fn temp_project() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-world-check-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(dir.join("archi/src")).unwrap();
        fs::write(
            dir.join("archi.toml"),
            "[project]\nname = \"t\"\npreset = \"default\"\n",
        )
        .unwrap();
        fs::write(dir.join("archi/src").join("model.arch"), MODEL).unwrap();
        dir
    }

    fn put(root: &Path, rel_path: &str, text: &str) {
        let path = root.join(rel_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    /// The note [`SOURCE`] names: a name and its prose, which is the whole
    /// schema of a loose layer.
    fn note(root: &Path) {
        put(
            root,
            SOURCE,
            "# The guard walked the platform\n\nHe timed the tunnel once at four minutes.\n",
        );
    }

    /// One world fact under `archi/world/facts/`: its three lists, its name,
    /// its conditioning paragraph and its `Scenarios` block. The note it may
    /// name arrives with it, so a grounded fact is grounded wherever it is
    /// written.
    fn fact(root: &Path, slug: &str, lists: &str, title: &str, condition: &str, scenarios: &str) {
        note(root);
        put(
            root,
            &format!("archi/world/facts/{slug}.md"),
            &format!(
                "---\n{lists}---\n\n# {title}\n\n{condition}\n\n\
                 ## What people do instead\n\nThey work around it by hand.\n\n\
                 ## Scenarios\n\n{scenarios}"
            ),
        );
    }

    /// The three lists, written out.
    fn lists(covers: &str, sources: &str, uses: &str) -> String {
        format!("covers: [{covers}]\nsources: [{sources}]\nuses: [{uses}]\n")
    }

    /// A whole, healthy fact: grounded, covering `Gate`, with a scenario.
    fn healthy(root: &Path, slug: &str, title: &str) {
        fact(
            root,
            slug,
            &lists("Gate", SOURCE, ""),
            title,
            "The carriage drops the network for minutes at a time.",
            SCENARIOS,
        );
    }

    /// The declaration file beside the wing, written whole.
    fn ignore(root: &Path, text: &str) {
        put(root, &format!("archi/world/{IGNORE}"), text);
    }

    fn check_at(root: &Path) -> DocReport {
        let ws = modeling_lang::source::compile_project(root)
            .unwrap_or_else(|f| panic!("test model failed to compile:\n{}", f.render()))
            .workspace;
        docs_check(root, ws.model())
    }

    /// The fingerprint of the tree's one fact, at the grain the caller names.
    fn digest_at(root: &Path, scenario: Option<&str>) -> String {
        let ws = modeling_lang::source::compile_project(root)
            .unwrap_or_else(|f| panic!("test model failed to compile:\n{}", f.render()))
            .workspace;
        let (tree, _) = load(root, ws.model());
        scenario_digest(&tree.world[0], scenario)
    }

    fn rendered(diags: &[DocDiagnostic]) -> Vec<String> {
        diags.iter().map(ToString::to_string).collect()
    }

    /// The per-fact lines, rendered.
    fn states(report: &DocReport) -> Vec<String> {
        picked(report, |f| matches!(f, WorldFinding::State { .. }))
    }

    /// The deep-chain lines, rendered.
    fn chains(report: &DocReport) -> Vec<String> {
        picked(report, |f| matches!(f, WorldFinding::ChainDeep { .. }))
    }

    /// The elements no fact reaches, by path.
    fn unreached(report: &DocReport) -> Vec<String> {
        report
            .world
            .findings
            .iter()
            .filter_map(|f| match f {
                WorldFinding::Unreached { element } => Some(element.clone()),
                _ => None,
            })
            .collect()
    }

    fn picked(report: &DocReport, want: fn(&WorldFinding) -> bool) -> Vec<String> {
        report
            .world
            .findings
            .iter()
            .filter(|f| want(f))
            .map(ToString::to_string)
            .collect()
    }

    /// A healthy fact holds no state, and its states never block.
    #[test]
    fn a_whole_fact_stands_with_no_state_line() {
        let root = temp_project();
        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        let report = check_at(&root);
        assert_eq!(rendered(&report.diagnostics), Vec::<String>::new());
        assert_eq!(states(&report), Vec::<String>::new());
        fs::remove_dir_all(&root).unwrap();
    }

    /// An empty `covers` is a legal state — the fact stands before the model
    /// reaches it — and a fact carrying a scenario is no orphan
    /// (`a-fact-may-stand-before-the-model-does`).
    #[test]
    fn an_empty_covers_is_uncovered_and_never_an_orphan() {
        let root = temp_project();
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("", SOURCE, ""),
            "Riders lose the signal",
            "The carriage drops the network for minutes at a time.",
            SCENARIOS,
        );
        let report = check_at(&root);
        assert_eq!(rendered(&report.diagnostics), Vec::<String>::new());
        assert_eq!(
            states(&report),
            ["world fact `riders-lose-the-signal`: world_uncovered"]
        );
        fs::remove_dir_all(&root).unwrap();
    }

    /// Nothing uses it and it dictates nothing: dead weight, not early work.
    #[test]
    fn a_fact_nothing_uses_and_no_scenario_is_an_orphan() {
        let root = temp_project();
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("Gate", SOURCE, ""),
            "Riders lose the signal",
            "The carriage drops the network for minutes at a time.",
            "",
        );
        let report = check_at(&root);
        // The empty block is the reader's error; the state is the wing's.
        assert_eq!(
            report
                .diagnostics
                .iter()
                .map(|d| d.code)
                .collect::<Vec<_>>(),
            ["E_DOC"]
        );
        assert_eq!(
            states(&report),
            ["world fact `riders-lose-the-signal`: world_orphan"]
        );
        fs::remove_dir_all(&root).unwrap();
    }

    /// An empty `sources` says so, whatever else the fact carries; any entry
    /// clears it (`an-ungrounded-fact-says-so`,
    /// `a-source-is-reachable-and-lives-in-the-world`). Emptiness is a legal
    /// state and never an error: nobody has grounded the fact yet, and the
    /// line says exactly that.
    #[test]
    fn an_empty_sources_says_so_and_one_entry_clears_it() {
        let root = temp_project();
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("Gate", "", ""),
            "Riders lose the signal",
            "The carriage drops the network for minutes at a time.",
            SCENARIOS,
        );
        let report = check_at(&root);
        assert_eq!(rendered(&report.diagnostics), Vec::<String>::new());
        assert_eq!(
            states(&report),
            ["world fact `riders-lose-the-signal`: world_ungrounded"]
        );

        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        let report = check_at(&root);
        assert_eq!(states(&report), Vec::<String>::new());
        fs::remove_dir_all(&root).unwrap();
    }

    /// Three states, one line (`one-fact-reports-one-finding`).
    #[test]
    fn three_states_ride_one_line() {
        let root = temp_project();
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("", "", ""),
            "Riders lose the signal",
            "The Engine takes the call while the carriage drops the network.",
            SCENARIOS,
        );
        let report = check_at(&root);
        assert_eq!(
            states(&report),
            [
                "world fact `riders-lose-the-signal`: world_ungrounded, world_uncovered, \
                 world_speaks_the_model(Engine)"
            ]
        );
        fs::remove_dir_all(&root).unwrap();
    }

    /// The paragraph speaks the world; the scenarios speak the system, and
    /// naming it there is the point
    /// (`the-fact-speaks-the-world-and-check-says-when-it-does-not`).
    #[test]
    fn the_paragraph_speaks_the_model_and_the_scenario_is_exempt() {
        let root = temp_project();
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("Gate", SOURCE, ""),
            "Riders lose the signal",
            "The carriage drops the network while Engine.Inner waits.",
            SCENARIOS,
        );
        let report = check_at(&root);
        assert_eq!(
            states(&report),
            ["world fact `riders-lose-the-signal`: world_speaks_the_model(Engine.Inner)"]
        );

        // The same name, inside a step: nothing.
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("Gate", SOURCE, ""),
            "Riders lose the signal",
            "The carriage drops the network for minutes at a time.",
            "### the engine waits\n\nGiven Engine.Inner is idle\n\
             Then the last synced view appears\n",
        );
        let report = check_at(&root);
        assert_eq!(states(&report), Vec::<String>::new());
        fs::remove_dir_all(&root).unwrap();
    }

    /// A ring holds nothing up: a located error naming it
    /// (`the-wing-reports-what-stands-in-the-air`).
    #[test]
    fn a_uses_ring_is_a_located_error() {
        let root = temp_project();
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("Gate", SOURCE, "tunnels-run-long"),
            "Riders lose the signal",
            "The carriage drops the network for minutes at a time.",
            SCENARIOS,
        );
        fact(
            &root,
            "tunnels-run-long",
            &lists("Gate", SOURCE, "riders-lose-the-signal"),
            "Tunnels run long",
            "A tunnel runs for minutes on the northern line.",
            SCENARIOS,
        );
        let report = check_at(&root);
        let all = rendered(&report.diagnostics).join("\n");
        assert!(
            report.diagnostics.iter().any(|d| d.code == "E_DOC_REF"),
            "{all}"
        );
        assert!(all.contains("riders-lose-the-signal"), "{all}");
        assert!(all.contains("tunnels-run-long"), "{all}");

        // A fact resting on itself is the same ring, one link long.
        fs::remove_dir_all(root.join("archi/world/facts")).unwrap();
        fact(
            &root,
            "tunnels-run-long",
            &lists("Gate", SOURCE, "tunnels-run-long"),
            "Tunnels run long",
            "A tunnel runs for minutes on the northern line.",
            SCENARIOS,
        );
        let report = check_at(&root);
        let all = rendered(&report.diagnostics).join("\n");
        assert!(
            report.diagnostics.iter().any(|d| d.code == "E_DOC_REF"),
            "{all}"
        );
        assert!(all.contains("tunnels-run-long → tunnels-run-long"), "{all}");
        fs::remove_dir_all(&root).unwrap();
    }

    /// Three facts deep reads; four does not.
    #[test]
    fn a_chain_of_four_runs_deep_and_three_does_not() {
        let root = temp_project();
        let chain = ["first-fact", "second-fact", "third-fact", "fourth-fact"];
        let title = |slug: &str| {
            let mut t = slug.replace('-', " ");
            t[..1].make_ascii_uppercase();
            t
        };
        for (i, slug) in chain.iter().take(3).enumerate() {
            let uses = chain
                .get(i + 1)
                .filter(|_| i + 1 < 3)
                .copied()
                .unwrap_or("");
            fact(
                &root,
                slug,
                &lists("Gate", SOURCE, uses),
                &title(slug),
                "The carriage drops the network for minutes at a time.",
                SCENARIOS,
            );
        }
        let report = check_at(&root);
        assert_eq!(rendered(&report.diagnostics), Vec::<String>::new());
        assert_eq!(chains(&report), Vec::<String>::new());

        // A fourth link, and the chain stops fitting in a head.
        fact(
            &root,
            "third-fact",
            &lists("Gate", SOURCE, "fourth-fact"),
            &title("third-fact"),
            "The carriage drops the network for minutes at a time.",
            SCENARIOS,
        );
        fact(
            &root,
            "fourth-fact",
            &lists("Gate", SOURCE, ""),
            &title("fourth-fact"),
            "The carriage drops the network for minutes at a time.",
            SCENARIOS,
        );
        let report = check_at(&root);
        assert_eq!(rendered(&report.diagnostics), Vec::<String>::new());
        assert_eq!(
            chains(&report).len(),
            1,
            "{:?}",
            report
                .world
                .findings
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        let line = &chains(&report)[0];
        for slug in chain {
            assert!(line.contains(slug), "{line}");
        }
        fs::remove_dir_all(&root).unwrap();
    }

    /// Coverage runs down the connection edges and into the children; what
    /// nothing reaches is named by path (`coverage-reaches-down-the-graph`).
    #[test]
    fn coverage_runs_down_the_edges_and_into_the_children() {
        let root = temp_project();
        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        let report = check_at(&root);
        // `Engine` rides the edge, `Engine.Inner` sits inside it, and both
        // stay off the list; the island is the whole report.
        assert_eq!(unreached(&report), ["Island"]);
        fs::remove_dir_all(&root).unwrap();
    }

    /// A `Data`-classified element is outside the coverage question, covered
    /// or not: a payload rides inside a connection and is never its
    /// destination (`coverage-reaches-down-the-graph`).
    #[test]
    fn a_data_element_is_never_reported_covered_or_not() {
        let root = temp_project();
        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        assert_eq!(unreached(&check_at(&root)), ["Island"]);

        // Named by a fact, it is silent for the other reason; the report
        // does not change.
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("Gate, Payload", SOURCE, ""),
            "Riders lose the signal",
            "The carriage drops the network for minutes at a time.",
            SCENARIOS,
        );
        assert_eq!(unreached(&check_at(&root)), ["Island"]);
        fs::remove_dir_all(&root).unwrap();
    }

    /// The declaration beside the wing: one line names an element and why
    /// nothing outside reaches it, and the element leaves the list. A tree
    /// with no file behaves exactly as it does today
    /// (`an-internal-element-says-so`).
    #[test]
    fn a_declared_element_leaves_the_coverage_list() {
        let root = temp_project();
        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        assert_eq!(unreached(&check_at(&root)), ["Island"]);

        ignore(
            &root,
            "# what no condition outside will ever reach\n\n\
             Island — a scratch fixture; no behavior from outside arrives at it\n",
        );
        let report = check_at(&root);
        assert_eq!(rendered(&report.diagnostics), Vec::<String>::new());
        assert_eq!(unreached(&report), Vec::<String>::new());
        fs::remove_dir_all(&root).unwrap();
    }

    /// The file resolves rather than matches by string: a renamed element
    /// breaks it loudly, on its own line (`an-internal-element-says-so`).
    #[test]
    fn a_declared_entry_naming_nothing_is_a_located_error() {
        let root = temp_project();
        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        ignore(&root, "# the list\nGhost — it left the model two versions ago\n");
        let report = check_at(&root);
        let all = rendered(&report.diagnostics).join("\n");
        assert_eq!(
            report
                .diagnostics
                .iter()
                .map(|d| (d.code, d.file.as_str(), d.line))
                .collect::<Vec<_>>(),
            [("E_MODEL_REF", "archi/world/.worldignore", 2)],
            "{all}"
        );
        assert!(all.contains("`Ghost`"), "{all}");
        // The unresolved entry silences nothing.
        assert_eq!(unreached(&report), ["Island"]);
        fs::remove_dir_all(&root).unwrap();
    }

    /// The reason is mandatory: the entry is a claim somebody can be wrong
    /// about, not a mute (`an-internal-element-says-so`).
    #[test]
    fn a_declared_entry_with_no_reason_is_a_located_error() {
        let root = temp_project();
        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        for text in ["\nIsland\n", "\nIsland — \n"] {
            ignore(&root, text);
            let report = check_at(&root);
            let all = rendered(&report.diagnostics).join("\n");
            assert_eq!(
                report
                    .diagnostics
                    .iter()
                    .map(|d| (d.code, d.file.as_str(), d.line))
                    .collect::<Vec<_>>(),
                [("E_DOC", "archi/world/.worldignore", 2)],
                "{all}"
            );
            assert!(all.contains("`Island`"), "{all}");
            assert_eq!(unreached(&report), ["Island"], "{all}");
        }
        fs::remove_dir_all(&root).unwrap();
    }

    /// A `Data` entry is a located error: its type already excludes it, so
    /// the line claims nothing the model does not already say
    /// (`an-internal-element-says-so`).
    #[test]
    fn a_data_entry_in_the_declaration_is_a_located_error() {
        let root = temp_project();
        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        ignore(&root, "Payload — it never leaves the process\n");
        let report = check_at(&root);
        let all = rendered(&report.diagnostics).join("\n");
        assert_eq!(
            report
                .diagnostics
                .iter()
                .map(|d| (d.code, d.file.as_str(), d.line))
                .collect::<Vec<_>>(),
            [("E_MODEL_REF", "archi/world/.worldignore", 1)],
            "{all}"
        );
        assert!(all.contains("`Data`"), "{all}");
        fs::remove_dir_all(&root).unwrap();
    }

    /// The set the save reads, off a tree the caller has not loaded.
    fn save_set(root: &Path) -> Vec<String> {
        let ws = modeling_lang::source::compile_project(root)
            .unwrap_or_else(|f| panic!("test model failed to compile:\n{}", f.render()))
            .workspace;
        super::unreached_at(root, ws.model())
    }

    /// The `Data` classification leaves the coverage question, and the save
    /// asks the same question `check` does — so a payload never stands
    /// between an operator and a version
    /// (`the-save-refuses-an-unconditioned-element`).
    #[test]
    fn a_data_element_never_enters_the_set_the_save_reads() {
        let root = temp_project();
        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        // `Payload` is `Data type_of`; the island is the whole set.
        assert_eq!(save_set(&root), ["Island"]);
        fs::remove_dir_all(&root).unwrap();
    }

    /// `check`'s finding and the save's refusal are one computation read
    /// twice: the finding renders what [`unreached`] returns and the save
    /// refuses on it, so the two can never disagree about an element — on a
    /// bare wing, under a cover, and under a declaration
    /// (`the-save-refuses-an-unconditioned-element`).
    #[test]
    fn the_finding_and_the_refusal_read_one_set() {
        let root = temp_project();
        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        assert_eq!(unreached(&check_at(&root)), save_set(&root));
        assert_eq!(save_set(&root), ["Island"]);

        // The island covered: both go empty.
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("Gate, Island", SOURCE, ""),
            "Riders lose the signal",
            "The carriage drops the network for minutes at a time.",
            SCENARIOS,
        );
        assert_eq!(unreached(&check_at(&root)), save_set(&root));
        assert_eq!(save_set(&root), Vec::<String>::new());

        // The island declared internal instead: both go empty again.
        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        ignore(&root, "Island — a fixture; nothing outside arrives at it\n");
        assert_eq!(unreached(&check_at(&root)), save_set(&root));
        assert_eq!(save_set(&root), Vec::<String>::new());
        fs::remove_dir_all(&root).unwrap();
    }

    /// A tree with no fact reaches nothing and refuses nothing: the wing's
    /// threshold, at the save (`the-wing-arrives-without-noise`).
    #[test]
    fn a_tree_with_no_fact_hands_the_save_nothing() {
        let root = temp_project();
        assert_eq!(save_set(&root), Vec::<String>::new());
        // A wing folder with only a note in it is still no fact.
        note(&root);
        assert_eq!(save_set(&root), Vec::<String>::new());
        fs::remove_dir_all(&root).unwrap();
    }

    /// The fingerprint the plan's drift line reads: six hex digits over the
    /// whole parsed block, moving with a step and standing still under the
    /// prose around it. Naming no scenario is that grain
    /// (`the-digest-witnesses-one-scenario`).
    ///
    /// The value moved with the shape, and it had to: the block held a
    /// `Feature:` line, the digest hashed it first, and the shape retired the
    /// line. What the digest reads now is the scenarios alone, so every drift
    /// line a plan carries is re-pinned once and reads the new value from
    /// there.
    #[test]
    fn the_scenario_digest_reads_the_parsed_block() {
        let root = temp_project();
        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        let first = digest_at(&root, None);
        assert_eq!(first.len(), 6, "{first}");
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()), "{first}");
        // The block under the retired shape hashed as `5b5815`, the feature
        // line `Offline open` first. The line is gone and the value with it.
        assert_eq!(first, "fc6e9d");

        // The conditioning paragraph moves; the story does not.
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("Gate", SOURCE, ""),
            "Riders lose the signal",
            "A tunnel runs for minutes on the northern line.",
            SCENARIOS,
        );
        assert_eq!(digest_at(&root, None), first);

        // One step moves, and the fingerprint says so.
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("Gate", SOURCE, ""),
            "Riders lose the signal",
            "The carriage drops the network for minutes at a time.",
            &SCENARIOS.replace(
                "Then the last synced view appears",
                "Then nothing appears at all",
            ),
        );
        assert_ne!(digest_at(&root, None), first);
        fs::remove_dir_all(&root).unwrap();
    }

    /// The grain is an argument: a named scenario is fingerprinted alone, so
    /// a sibling moving in the same fact moves nothing, while the whole block
    /// moves with either of them (`the-digest-witnesses-one-scenario`).
    #[test]
    fn a_named_scenario_is_fingerprinted_alone() {
        let root = temp_project();
        let both = format!("{SCENARIOS}{SIBLING}");
        let write = |scenarios: &str| {
            fact(
                &root,
                "riders-lose-the-signal",
                &lists("Gate", SOURCE, ""),
                "Riders lose the signal",
                "The carriage drops the network for minutes at a time.",
                scenarios,
            );
        };
        write(&both);
        let one = digest_at(&root, Some(NAMED));
        let whole = digest_at(&root, None);
        assert_eq!(one.len(), 6, "{one}");
        assert_ne!(one, whole, "one scenario is not the block it sits in");
        // The address is normalized where the grammar keeps the name as
        // written, so a ref with wider spacing reads the same scenario.
        assert_eq!(digest_at(&root, Some("the  app opens  with no network")), one);

        // A sibling's step is reworded: the block moved, the named scenario
        // did not.
        write(&both.replace("Then the view arrives late", "Then the view takes its time"));
        assert_eq!(digest_at(&root, Some(NAMED)), one);
        assert_ne!(digest_at(&root, None), whole);

        // The named scenario's own step moves, and its fingerprint says so.
        write(&both.replace("Then the last synced view appears", "Then nothing appears at all"));
        assert_ne!(digest_at(&root, Some(NAMED)), one);

        // A name the fact does not hold witnesses the empty story — the same
        // nothing a block the grammar refused witnesses.
        write(&both);
        assert_eq!(
            digest_at(&root, Some("the train stops")),
            digest_at(&root, Some("the doors open"))
        );
        assert_ne!(digest_at(&root, Some("the train stops")), one);

        fs::remove_dir_all(&root).unwrap();
    }

    /// One place in the crate hashes a parsed block, and the grain being an
    /// argument is what keeps it one: a second fingerprint over the same
    /// story would start disagreeing with this one about what moved
    /// (`the-digest-witnesses-one-scenario`).
    #[test]
    fn exactly_one_item_in_the_crate_hashes_a_parsed_block() {
        assert_eq!(hashing_items(), ["docs/world_check.rs — scenario_digest"]);
    }

    /// Every top-level item of the crate's own source that hashes and reads a
    /// parsed step, named by its file and its first `fn`. Test modules are cut
    /// off first: the claim is about the code, and a test naming the hash is
    /// no second digest.
    fn hashing_items() -> Vec<String> {
        let src = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        let mut files = Vec::new();
        let mut stack = vec![src.clone()];
        while let Some(dir) = stack.pop() {
            for entry in fs::read_dir(&dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    files.push(path);
                }
            }
        }
        files.sort();
        let mut out = Vec::new();
        for path in &files {
            let text = fs::read_to_string(path).unwrap();
            let code = text.split("#[cfg(test)]").next().unwrap_or_default();
            // A top-level item stands until a closing brace in column one.
            for item in code.split("\n}\n") {
                if !(item.contains("Sha256") && item.contains(".steps")) {
                    continue;
                }
                let name = item
                    .lines()
                    .find_map(|l| l.split_once("fn "))
                    .and_then(|(_, rest)| rest.split(['(', '<']).next())
                    .unwrap_or("")
                    .to_string();
                let file = path.strip_prefix(&src).unwrap_or(path).display();
                out.push(format!("{file} — {name}"));
            }
        }
        out
    }

    // ---- the four layers ---------------------------------------------------

    /// The strict record is the `facts/` layer's alone, and it holds every
    /// slot the schema names (`the-world-holds-four-layers`).
    #[test]
    fn a_fact_missing_its_workaround_or_its_scenarios_is_a_located_error() {
        let root = temp_project();
        let file = "archi/world/facts/riders-lose-the-signal.md";
        let head = "---\ncovers: []\nsources: []\nuses: []\n---\n\n\
                    # Riders lose the signal\n\n\
                    The carriage drops the network for minutes at a time.\n";
        // No workaround.
        put(
            &root,
            file,
            &format!("{head}\n## Scenarios\n\n{SCENARIOS}"),
        );
        let report = check_at(&root);
        assert_eq!(
            report
                .diagnostics
                .iter()
                .map(|d| (d.code, d.file.as_str()))
                .collect::<Vec<_>>(),
            [("E_DOC", file)]
        );
        assert!(
            report.diagnostics[0].message.contains("What people do instead"),
            "{}",
            report.diagnostics[0]
        );

        // No scenarios.
        put(
            &root,
            file,
            &format!("{head}\n## What people do instead\n\nThey work around it by hand.\n"),
        );
        let report = check_at(&root);
        assert_eq!(
            report
                .diagnostics
                .iter()
                .map(|d| (d.code, d.file.as_str()))
                .collect::<Vec<_>>(),
            [("E_DOC", file)]
        );
        assert!(
            report.diagnostics[0].message.contains("Scenarios"),
            "{}",
            report.diagnostics[0]
        );
        fs::remove_dir_all(&root).unwrap();
    }

    /// What was seen and not yet shaped needs a name and its prose and
    /// nothing else — no workaround, no scenarios, no lists
    /// (`the-world-holds-four-layers`).
    #[test]
    fn a_note_needs_only_a_name_and_its_prose() {
        let root = temp_project();
        put(
            &root,
            "archi/world/notes/the-guard-walked-the-platform.md",
            "# The guard walked the platform\n\nHe timed the tunnel once at four minutes.\n",
        );
        let report = check_at(&root);
        assert_eq!(rendered(&report.diagnostics), Vec::<String>::new());
        assert!(report.world.findings.is_empty());
        // A note is no fact, so the count has nothing to close on.
        assert!(report.world.count.is_none());
        fs::remove_dir_all(&root).unwrap();
    }

    /// A claim somebody means to settle wears the same looseness
    /// (`the-world-holds-four-layers`).
    #[test]
    fn a_hypothesis_needs_only_a_name_and_its_prose() {
        let root = temp_project();
        put(
            &root,
            "archi/world/hypotheses/the-tunnel-is-the-cause.md",
            "# The tunnel is the cause\n\nNobody has measured the dead zone against it.\n",
        );
        let report = check_at(&root);
        assert_eq!(rendered(&report.diagnostics), Vec::<String>::new());
        assert!(report.world.findings.is_empty());
        assert!(report.world.count.is_none());
        fs::remove_dir_all(&root).unwrap();
    }

    /// Raw material is listed so a source can resolve against it, and opened
    /// by nothing: whatever it holds, the wing says nothing about it
    /// (`the-world-holds-four-layers`).
    #[test]
    fn a_resource_is_never_parsed_and_never_reported() {
        let root = temp_project();
        // Nothing a reader could make sense of: no name, a half-written
        // header, and the markers a merge leaves behind.
        put(
            &root,
            "archi/world/resources/the-support-thread.md",
            "---\nkind: ???\n\n<<<<<<< ours\nnot a document at all\n>>>>>>> theirs\n",
        );
        put(
            &root,
            "archi/world/resources/the-recording.txt",
            "00:14 the guard says the tunnel takes four minutes\n",
        );
        let report = check_at(&root);
        assert_eq!(rendered(&report.diagnostics), Vec::<String>::new());
        assert!(report.world.findings.is_empty());
        assert!(report.world.count.is_none());
        fs::remove_dir_all(&root).unwrap();
    }

    /// The located codes and paths of a report, in order.
    fn located(report: &DocReport) -> Vec<(&str, &str, usize)> {
        report
            .diagnostics
            .iter()
            .map(|d| (d.code, d.file.as_str(), d.line))
            .collect()
    }

    /// A folder the four do not name takes files and keeps them out of every
    /// reading, so the walk descends into it and locates each document it
    /// holds. The refusal is the loose file's own, differing only in the file
    /// it names: one construction, two arms, and they can never disagree
    /// about what the layers are (`the-world-holds-four-layers`).
    #[test]
    fn a_document_under_a_fifth_folder_is_a_located_error_at_any_depth() {
        let root = temp_project();
        let parked = "# Thing\n\nSomething somebody left here on the way past.\n";

        put(&root, "archi/world/stray.md", parked);
        let report = check_at(&root);
        assert_eq!(located(&report), [("E_PLACEMENT", "archi/world/stray.md", 1)]);
        let loose = report.diagnostics[0].message.clone();
        for layer in LAYERS {
            assert!(loose.contains(&format!("{WORLD}{layer}/")), "{loose}");
        }
        fs::remove_file(root.join("archi/world/stray.md")).unwrap();

        for (path, named) in [
            ("archi/world/attic/thing.md", "attic/thing.md"),
            ("archi/world/attic/deep/thing.md", "attic/deep/thing.md"),
        ] {
            put(&root, path, parked);
            let report = check_at(&root);
            assert_eq!(located(&report), [("E_PLACEMENT", path, 1)]);
            assert_eq!(
                report.diagnostics[0].message,
                loose.replace("`stray.md`", &format!("`{named}`"))
            );
            fs::remove_file(root.join(path)).unwrap();
        }
        fs::remove_dir_all(&root).unwrap();
    }

    /// A folder outside the four that holds no document says nothing: the
    /// rule locates a file, and a file that is not a document is not one to
    /// locate (`the-world-holds-four-layers`).
    #[test]
    fn a_folder_outside_the_four_holding_no_document_says_nothing() {
        let root = temp_project();
        fs::create_dir_all(root.join("archi/world/attic/deep")).unwrap();
        put(
            &root,
            "archi/world/attic/the-recording.txt",
            "00:14 the guard says the tunnel takes four minutes\n",
        );
        let report = check_at(&root);
        assert_eq!(rendered(&report.diagnostics), Vec::<String>::new());
        assert!(report.world.findings.is_empty());
        assert!(report.world.count.is_none());
        fs::remove_dir_all(&root).unwrap();
    }

    /// A wing folder holding none of the four layers behaves exactly as a
    /// tree with no wing does: nothing reported, nothing created
    /// (`the-world-holds-four-layers`, `the-wing-arrives-without-noise`).
    #[test]
    fn a_tree_with_none_of_the_four_folders_reports_nothing() {
        let root = temp_project();
        fs::create_dir_all(root.join("archi/world")).unwrap();
        let report = check_at(&root);
        assert_eq!(rendered(&report.diagnostics), Vec::<String>::new());
        assert!(report.world.findings.is_empty());
        assert!(report.world.count.is_none());
        fs::remove_dir_all(&root).unwrap();
    }

    // ---- a source lives inside the world ------------------------------------

    /// A source names a file of the world — a note, a hypothesis or a
    /// resource — and it resolves
    /// (`a-source-is-reachable-and-lives-in-the-world`).
    #[test]
    fn a_source_names_a_file_of_the_world_and_resolves() {
        let root = temp_project();
        put(
            &root,
            "archi/world/hypotheses/the-tunnel-is-the-cause.md",
            "# The tunnel is the cause\n\nNobody has measured the dead zone against it.\n",
        );
        put(
            &root,
            "archi/world/resources/the-recording.txt",
            "00:14 the guard says the tunnel takes four minutes\n",
        );
        fact(
            &root,
            "riders-lose-the-signal",
            &lists(
                "Gate",
                &format!(
                    "{SOURCE}, archi/world/hypotheses/the-tunnel-is-the-cause.md, \
                     archi/world/resources/the-recording.txt"
                ),
                "",
            ),
            "Riders lose the signal",
            "The carriage drops the network for minutes at a time.",
            SCENARIOS,
        );
        let report = check_at(&root);
        assert_eq!(rendered(&report.diagnostics), Vec::<String>::new());
        assert_eq!(states(&report), Vec::<String>::new());
        fs::remove_dir_all(&root).unwrap();
    }

    /// A source that reaches nothing is no source
    /// (`a-source-is-reachable-and-lives-in-the-world`).
    #[test]
    fn a_source_that_resolves_to_nothing_is_a_located_error() {
        let root = temp_project();
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("Gate", "archi/world/notes/nobody-wrote-this.md", ""),
            "Riders lose the signal",
            "The carriage drops the network for minutes at a time.",
            SCENARIOS,
        );
        let report = check_at(&root);
        let all = rendered(&report.diagnostics).join("\n");
        assert_eq!(
            report
                .diagnostics
                .iter()
                .map(|d| (d.code, d.file.as_str(), d.line))
                .collect::<Vec<_>>(),
            [("E_DOC", "archi/world/facts/riders-lose-the-signal.md", 3)],
            "{all}"
        );
        assert!(all.contains("`archi/world/notes/nobody-wrote-this.md`"), "{all}");
        assert!(all.contains("a source that cannot be read is no source"), "{all}");
        fs::remove_dir_all(&root).unwrap();
    }

    /// A source pointing into the spec is a located error whose text says
    /// why: the spec is what the world conditions, so a fact grounded in a
    /// requirement grounds itself in what it explains
    /// (`a-source-is-reachable-and-lives-in-the-world`).
    #[test]
    fn a_source_outside_the_world_is_a_located_error_that_says_why() {
        let root = temp_project();
        // A file of the spec that really is there: existing is not the
        // question the rule asks.
        put(
            &root,
            "archi/requirements/an-intent/an-intent.md",
            "# An intent\n\nA problem worth modeling.\n",
        );
        for entry in [
            "archi/requirements/an-intent/an-intent.md",
            "archi/world/../requirements/an-intent/an-intent.md",
        ] {
            fact(
                &root,
                "riders-lose-the-signal",
                &lists("Gate", entry, ""),
                "Riders lose the signal",
                "The carriage drops the network for minutes at a time.",
                SCENARIOS,
            );
            let report = check_at(&root);
            let all = rendered(&report.diagnostics).join("\n");
            assert_eq!(
                report
                    .diagnostics
                    .iter()
                    .map(|d| (d.code, d.file.as_str(), d.line))
                    .collect::<Vec<_>>(),
                [("E_DOC", "archi/world/facts/riders-lose-the-signal.md", 3)],
                "on `{entry}`: {all}"
            );
            assert!(all.contains(&format!("`{entry}`")), "{all}");
            assert!(
                all.contains("a path into the spec grounds the fact in what the fact explains"),
                "{all}"
            );
        }
        fs::remove_dir_all(&root).unwrap();
    }

    /// The external-locator form is retired: a source nobody here can open is
    /// a claim about evidence rather than evidence, and it earns the error a
    /// path outside the world earns
    /// (`a-source-is-reachable-and-lives-in-the-world`,
    /// `the-source-lives-outside-the-tree`).
    #[test]
    fn a_source_carrying_a_uri_scheme_is_the_same_error() {
        let root = temp_project();
        for entry in [
            "https://example.org/thread/42",
            "mailto:guard@rail.example",
            "jira:RAIL-77",
        ] {
            fact(
                &root,
                "riders-lose-the-signal",
                &lists("Gate", entry, ""),
                "Riders lose the signal",
                "The carriage drops the network for minutes at a time.",
                SCENARIOS,
            );
            let report = check_at(&root);
            let all = rendered(&report.diagnostics).join("\n");
            assert_eq!(
                report
                    .diagnostics
                    .iter()
                    .map(|d| (d.code, d.file.as_str(), d.line))
                    .collect::<Vec<_>>(),
                [("E_DOC", "archi/world/facts/riders-lose-the-signal.md", 3)],
                "on `{entry}`: {all}"
            );
            assert!(all.contains(&format!("`{entry}`")), "{all}");
            assert!(
                all.contains("a path into the spec grounds the fact in what the fact explains"),
                "{all}"
            );
        }
        fs::remove_dir_all(&root).unwrap();
    }

    /// A project that has not opted into the wing is not behind on it: no
    /// coverage report, no count, nothing at all.
    #[test]
    fn an_empty_wing_says_nothing_at_all() {
        let root = temp_project();
        let report = check_at(&root);
        assert!(report.world.findings.is_empty());
        assert!(report.world.count.is_none());
        assert!(!root.join("archi/world").exists());
        fs::remove_dir_all(&root).unwrap();
    }

    /// The closing line counts the wing and its ungrounded half
    /// (`the-check-counts-the-wing`).
    #[test]
    fn the_count_names_the_wing_and_its_ungrounded_half() {
        let root = temp_project();
        for i in 1..=7 {
            let slug = format!("fact-number-{i}");
            let sources = if i <= 2 { "" } else { SOURCE };
            fact(
                &root,
                &slug,
                &lists("Gate", sources, ""),
                &format!("Fact number {i}"),
                "The carriage drops the network for minutes at a time.",
                SCENARIOS,
            );
        }
        let report = check_at(&root);
        assert_eq!(
            report.world.count.as_ref().map(ToString::to_string),
            Some("world — 7 facts · 2 ungrounded".to_string())
        );
        fs::remove_dir_all(&root).unwrap();
    }

    /// The two references the record leaves to the compiler, each located at
    /// the line its list sits on.
    #[test]
    fn an_unresolved_covers_or_uses_is_a_located_error() {
        let root = temp_project();
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("Ghost", SOURCE, "no-such-fact"),
            "Riders lose the signal",
            "The carriage drops the network for minutes at a time.",
            SCENARIOS,
        );
        let report = check_at(&root);
        let codes: Vec<&str> = report.diagnostics.iter().map(|d| d.code).collect();
        assert_eq!(codes, ["E_MODEL_REF", "E_DOC_REF"]);
        let all = rendered(&report.diagnostics).join("\n");
        assert!(all.contains("`Ghost`"), "{all}");
        assert!(all.contains("`no-such-fact`"), "{all}");
        // Located on their own frontmatter lines: `covers` is line 2, `uses`
        // line 4.
        assert_eq!(
            report
                .diagnostics
                .iter()
                .map(|d| d.line)
                .collect::<Vec<_>>(),
            [2, 4]
        );
        fs::remove_dir_all(&root).unwrap();
    }

    /// The surface later tasks read: which facts cover this element.
    #[test]
    fn the_wing_is_served_keyed_by_what_it_covers() {
        let root = temp_project();
        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        fact(
            &root,
            "tunnels-run-long",
            &lists("Gate, Engine", SOURCE, ""),
            "Tunnels run long",
            "A tunnel runs for minutes on the northern line.",
            SCENARIOS,
        );
        let ws = modeling_lang::source::compile_project(&root)
            .unwrap_or_else(|f| panic!("test model failed to compile:\n{}", f.render()))
            .workspace;
        let (tree, _) = load(&root, ws.model());
        let wing = serve_world(&tree);
        assert_eq!(
            wing.covering("Gate")
                .iter()
                .map(|f| f.doc.slug.as_str())
                .collect::<Vec<_>>(),
            ["riders-lose-the-signal", "tunnels-run-long"]
        );
        assert_eq!(
            wing.covering("Engine")
                .iter()
                .map(|f| f.doc.slug.as_str())
                .collect::<Vec<_>>(),
            ["tunnels-run-long"]
        );
        assert!(wing.covering("Island").is_empty());
        assert_eq!(wing.elements().collect::<Vec<_>>(), ["Engine", "Gate"]);
        fs::remove_dir_all(&root).unwrap();
    }
}
