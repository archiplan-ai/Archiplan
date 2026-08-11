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
use std::path::Path;

use modeling_lang::{Definition, Layer, Model, Statement};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::gherkin::{self, ScenarioBlock};
use super::md::slugify;
use super::world::{self, WorldDoc};
use super::{DocDiagnostic, Tree, is_md, read_doc, sorted_entries, stem};
use crate::members::{HOME, Member, MemberSet};

/// A `uses` chain of this many facts still reads; one deeper holds a theory
/// of the world instead of a record of it.
const CHAIN_MAX: usize = 3;

/// The ontology term whose instances leave the coverage question
/// (`archi/requirements/world-facts/coverage-reaches-down-the-graph.md`).
const DATA: &str = "Data";

/// The declaration of what is internal, beside the wing it quiets
/// (`archi/requirements/world-facts/an-internal-element-says-so.md`).
const IGNORE: &str = ".worldignore";

/// What one `.worldignore` line writes between the element and the reason it
/// is internal — `Element — why nothing outside reaches it`. No element path
/// carries the dash, so the space around it is free.
const REASON: char = '—';

/// One world fact as the wing holds it: the record the reader parsed and the
/// scenarios the grammar accepted.
pub(crate) struct WorldFact {
    /// The record — the three lists, the paragraph, the killer.
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

/// Walk `archi/world/` — a flat folder, one file per fact — and read each
/// file the way every doc primitive is read: structure, then schema, then
/// the grammar over the `Scenarios` block. Nothing here creates the folder.
pub(crate) fn discover(root: &Path, diags: &mut Vec<DocDiagnostic>) -> Vec<WorldFact> {
    let base = root.join("archi").join("world");
    let files: Vec<_> = sorted_entries(&base)
        .into_iter()
        .filter(|p| is_md(p))
        .collect();
    if files.is_empty() {
        return Vec::new();
    }
    // The `@runs:` tags resolve against the declared members. An unreadable
    // manifest is `members::check`'s to report, and it is a compile error
    // long before this — the wing reads on with home alone.
    let members = MemberSet::resolve(root).unwrap_or_else(|_| MemberSet {
        project_root: root.to_path_buf(),
        members: vec![Member {
            name: HOME.to_string(),
            url: None,
            declared_path: None,
            mapped_path: None,
            root: Some(root.to_path_buf()),
        }],
    });
    files
        .into_iter()
        .filter_map(|path| read_fact(root, &path, &members, diags))
        .collect()
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
    members: &MemberSet,
    diags: &mut Vec<DocDiagnostic>,
) -> Option<WorldFact> {
    let (file, doc) = read_doc(root, path, diags)?;
    let record = world::parse(&doc, &file, &stem(path), root, diags);
    let scenarios = record
        .scenarios
        .as_ref()
        .and_then(|b| gherkin::parse(b, &file, members, diags));
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
    coverage(root, model, tree, &mut findings, diags);
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
/// killer are exempt, because Gherkin describes the system's surface and
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
fn coverage(
    root: &Path,
    model: &Model,
    tree: &Tree,
    findings: &mut Vec<WorldFinding>,
    diags: &mut Vec<DocDiagnostic>,
) {
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
    for node in &nodes {
        if !covered.contains(node) && !carried.contains(*node) && !declared.contains(*node) {
            findings.push(WorldFinding::Unreached {
                element: (*node).to_string(),
            });
        }
    }
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

/// The fingerprint of a fact's scenarios: the feature, the names and the
/// steps, hashed to six hex digits. It is not the story — nothing that reads
/// it holds a copy of one — it is only enough to say the story moved. It
/// sits beside the parsed block so the plan and the link read one function.
pub(crate) fn scenario_digest(fact: &WorldFact) -> String {
    let mut text = String::new();
    if let Some(block) = &fact.scenarios {
        text.push_str(&block.feature);
        for s in &block.scenarios {
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

    /// The block every fact carries unless the test says otherwise.
    const SCENARIOS: &str = "\
Feature: Offline open
  Scenario: the app opens with no network
    Given the device has no network
    When the user opens the app
    Then the last synced view appears
";

    /// A `sources` entry that never touches the filesystem.
    const SOURCE: &str = "https://example.org/thread/42";

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

    /// One world fact under `archi/world/`: its three lists, its name, its
    /// conditioning paragraph and its `Scenarios` block.
    fn fact(root: &Path, slug: &str, lists: &str, title: &str, condition: &str, scenarios: &str) {
        put(
            root,
            &format!("archi/world/{slug}.md"),
            &format!(
                "---\n{lists}---\n\n# {title}\n\n{condition}\n\n\
                 ## What kills this\n\nThe condition ends.\n\n## Scenarios\n\n{scenarios}"
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

    /// The fingerprint of the tree's one fact.
    fn digest_at(root: &Path) -> String {
        let ws = modeling_lang::source::compile_project(root)
            .unwrap_or_else(|f| panic!("test model failed to compile:\n{}", f.render()))
            .workspace;
        let (tree, _) = load(root, ws.model());
        scenario_digest(&tree.world[0])
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
    /// clears it (`an-ungrounded-fact-says-so`).
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
            "Feature: Offline open\n  Scenario: the engine waits\n    \
             Given Engine.Inner is idle\n    Then the last synced view appears\n",
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
        fs::remove_dir_all(root.join("archi/world")).unwrap();
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

    /// The fingerprint the plan and the links both read: six hex digits over
    /// the parsed block, moving with a step and standing still under the
    /// prose around it.
    #[test]
    fn the_scenario_digest_reads_the_parsed_block() {
        let root = temp_project();
        healthy(&root, "riders-lose-the-signal", "Riders lose the signal");
        let first = digest_at(&root);
        assert_eq!(first.len(), 6, "{first}");
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()), "{first}");
        // The value is the one the plans already carry: the move keeps it.
        assert_eq!(first, "5b5815");

        // The conditioning paragraph moves; the story does not.
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("Gate", SOURCE, ""),
            "Riders lose the signal",
            "A tunnel runs for minutes on the northern line.",
            SCENARIOS,
        );
        assert_eq!(digest_at(&root), first);

        // One step moves, and the fingerprint says so.
        fact(
            &root,
            "riders-lose-the-signal",
            &lists("Gate", SOURCE, ""),
            "Riders lose the signal",
            "The carriage drops the network for minutes at a time.",
            "Feature: Offline open\n  Scenario: the app opens with no network\n    \
             Given the device has no network\n    When the user opens the app\n    \
             Then nothing appears at all\n",
        );
        assert_ne!(digest_at(&root), first);
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
