//! `archi search`: ranked lexical retrieval across every KB object —
//! model elements with their identity prose, intents, requirements,
//! stressors and sessions (`archi/requirements/agent-retrieval/`).
//!
//! The scan keeps no persisted derivative of the corpus: every query walks
//! the live doc tree and the compiled model it was handed, so a text edit
//! is searchable in the immediately following call and search can never
//! disagree with the files themselves (search-reads-the-tree-it-stands-on).
//! Corpus statistics are computed in the same pass — a term in every card
//! contributes nothing, a rare term decides the ranking — and both sides
//! normalize, with a shared-prefix rule standing in for a stemmer
//! (matching-forgives-the-phrasing). Each corpus degrades alone: no model
//! darkens only the element cards, an unparseable doc falls back to its
//! raw text (a-dark-corpus-stays-partial). Every hit carries its kind's
//! next hop as slugs and paths for the next verb (cards-carry-the-next-hop).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use modeling_lang::{Definition, Model, Statement};
use serde::Serialize;

use crate::docs;
use crate::docs::md;
use crate::docs::schema::{Origin, Outcome};

/// The object kinds a card can be. The order is the ranking tie-break.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// A model element: node, rel type, conn type or view.
    Element,
    /// An intent — the anchor of a requirements area.
    Intent,
    /// A requirement, at file scale.
    Requirement,
    /// A stressor of some stress session.
    Stressor,
    /// A stress session's round record.
    Session,
}

impl Kind {
    /// The surface name, as `--kind` reads and the envelope writes it.
    pub fn label(self) -> &'static str {
        match self {
            Kind::Element => "element",
            Kind::Intent => "intent",
            Kind::Requirement => "requirement",
            Kind::Stressor => "stressor",
            Kind::Session => "session",
        }
    }

    /// Parse a `--kind` value.
    pub fn parse(s: &str) -> Option<Kind> {
        Some(match s {
            "element" => Kind::Element,
            "intent" => Kind::Intent,
            "requirement" => Kind::Requirement,
            "stressor" => Kind::Stressor,
            "session" => Kind::Session,
            _ => return None,
        })
    }
}

/// A card's next-hop relations — the fields its kind already holds, kept
/// through the output boundary. Everything is slugs and paths, ready for
/// the next verb.
#[derive(Default, Serialize)]
pub struct Refs {
    /// Element: its identity prose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
    /// Element: requirements whose `satisfied-by` names it.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<String>,
    /// Element: stressors whose `affects` names it.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stressors: Vec<String>,
    /// Element: nodes sharing an edge with it.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub neighbors: Vec<String>,
    /// Requirement: its origin, in the frontmatter surface syntax.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// Requirement: the elements it claims satisfaction by.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub satisfied_by: Vec<String>,
    /// Requirement: `open`, `satisfied` or `deferred`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<&'static str>,
    /// Stressor: the session whose round holds it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,
    /// Stressor: the elements it presses.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub affects: Vec<String>,
    /// Stressor: `pending`, `surviving` or `breaking`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<&'static str>,
    /// Session: the version it presses on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Session: the closing seal, or `open`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<String>,
}

/// Field classes and their weights: a hit in a name outweighs the same hit
/// in a summary, which outweighs one in the body.
const WEIGHTS: [f64; 3] = [3.0, 2.0, 1.0];

/// One searchable object. Text lives in three line-addressed fields —
/// name, summary (or definition), body — tokenized once at build.
struct Card {
    kind: Kind,
    slug: String,
    file: Option<String>,
    /// 1-based line of the name; 0 for elements (their address is the path).
    line: usize,
    /// `[name, summary, body]`, each `(line, text)`.
    fields: [Vec<(usize, String)>; 3],
    /// Token sets per field, same order.
    toks: [BTreeSet<String>; 3],
    refs: Refs,
}

impl Card {
    fn new(kind: Kind, slug: String, file: Option<String>, line: usize) -> Card {
        Card {
            kind,
            slug,
            file,
            line,
            fields: Default::default(),
            toks: Default::default(),
            refs: Refs::default(),
        }
    }

    fn push(&mut self, field: usize, line: usize, text: impl Into<String>) {
        let text = text.into();
        if text.is_empty() {
            return;
        }
        self.toks[field].extend(tokenize(&text));
        self.fields[field].push((line, text));
    }
}

// ---- tokenizing and matching -------------------------------------------------

/// Lowercased tokens: alphanumeric runs, split further at camel-case
/// boundaries so `SourceTree` yields `source` and `tree`. Dots, hyphens and
/// every other separator split by not being alphanumeric. One-character
/// tokens carry no signal and are dropped.
fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut prev_lower = false;
    for c in text.chars() {
        if !c.is_alphanumeric() {
            flush(&mut out, &mut cur);
            prev_lower = false;
            continue;
        }
        if c.is_uppercase() && prev_lower {
            flush(&mut out, &mut cur);
        }
        prev_lower = c.is_lowercase() || c.is_numeric();
        cur.extend(c.to_lowercase());
    }
    flush(&mut out, &mut cur);
    out
}

fn flush(out: &mut Vec<String>, cur: &mut String) {
    if cur.chars().count() >= 2 {
        out.push(std::mem::take(cur));
    } else {
        cur.clear();
    }
}

/// The match rule, shared by scoring and frequencies: exact is full
/// strength; a shared prefix of three or more characters covering at least
/// half of the longer token is half strength, so `folding` reaches `fold`
/// and `versioning` reaches `versions` without a stemmer.
fn strength(query_tok: &str, tok: &str) -> f64 {
    if query_tok == tok {
        return 1.0;
    }
    let shared = query_tok
        .chars()
        .zip(tok.chars())
        .take_while(|(a, b)| a == b)
        .count();
    let longer = query_tok.chars().count().max(tok.chars().count());
    if shared >= 3 && shared * 2 >= longer {
        0.5
    } else {
        0.0
    }
}

/// The best match strength of one query token in one token set.
fn field_strength(qt: &str, toks: &BTreeSet<String>) -> f64 {
    if toks.contains(qt) {
        return 1.0;
    }
    toks.iter()
        .map(|t| strength(qt, t))
        .fold(0.0, f64::max)
}

// ---- the corpus ----------------------------------------------------------------

/// Project-relative path with `/` separators, mirroring the doc tree's own
/// normalization.
fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Render an origin back to its frontmatter surface.
fn origin_surface(o: &Origin) -> String {
    match o {
        Origin::Intent => "intent".into(),
        Origin::Parent => "parent".into(),
        Origin::Stressors(v) => format!("stressor({})", v.join(", ")),
        Origin::Fusion(v) => format!("fusion({})", v.join(", ")),
    }
}

/// Read one doc file's text fields into a card: name, summary, then every
/// heading and its content as body. Returns false when the file cannot be
/// read or parsed — the raw fallback takes it from there.
fn fill_doc_text(root: &Path, card: &mut Card) -> bool {
    let Some(file) = card.file.clone() else {
        return false;
    };
    let Ok(text) = fs::read_to_string(root.join(&file)) else {
        return false;
    };
    let Ok(doc) = md::parse(&text) else {
        return false;
    };
    card.push(0, doc.name_line, doc.name.clone());
    for (line, t) in &doc.summary {
        card.push(1, *line, t.clone());
    }
    for h in &doc.headings {
        card.push(2, h.line, h.text.clone());
        for (line, t) in &h.content {
            card.push(2, *line, t.clone());
        }
    }
    true
}

/// A file the schema walk dropped (unreadable, unparseable or misplaced)
/// still matches by its raw lines: a card with no schema fields
/// (a-dark-corpus-stays-partial).
fn raw_card(root: &Path, file: String, kind: Kind, slug: String) -> Card {
    let mut card = Card::new(kind, slug.clone(), Some(file.clone()), 1);
    card.push(0, 1, slug);
    if let Ok(text) = fs::read_to_string(root.join(&file)) {
        for (i, line) in text.lines().enumerate() {
            card.push(2, i + 1, line.to_string());
        }
    }
    card
}

/// Every `.md` under a doc root, with the kind its placement gives it:
/// a folder's anchor file is the container (intent, session), any other
/// member file is the leaf (requirement, stressor).
fn walk_md(root: &Path, base: &Path, out: &mut Vec<(String, Kind)>, docs_root: &str) {
    let Ok(rd) = fs::read_dir(base) else { return };
    let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            walk_md(root, &path, out, docs_root);
        } else if path.extension().is_some_and(|e| e == "md") {
            let file = rel(root, &path);
            let stem = path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let parent = path
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let anchored = parent == stem;
            let kind = match (docs_root, anchored) {
                ("requirements", true) => {
                    // Only a top-level folder's anchor is an intent; deeper
                    // anchors are promoted requirements.
                    if file == format!("archi/requirements/{stem}/{stem}.md") {
                        Kind::Intent
                    } else {
                        Kind::Requirement
                    }
                }
                ("requirements", false) => Kind::Requirement,
                (_, true) => Kind::Session,
                (_, false) => Kind::Stressor,
            };
            out.push((file, kind));
        }
    }
}

/// Build the corpus: one card per KB object, cross-references inverted in
/// the same pass. `model` is absent exactly when the compile failed — the
/// element corpus is dark, every doc card still stands.
fn corpus(root: &Path, model: Option<&Model>) -> Vec<Card> {
    let tree = docs::discover_tree(root);
    let mut cards: Vec<Card> = Vec::new();

    for i in &tree.intents {
        let mut c = Card::new(Kind::Intent, i.slug.clone(), Some(i.file.clone()), i.line);
        if !fill_doc_text(root, &mut c) {
            c = raw_card(root, i.file.clone(), Kind::Intent, i.slug.clone());
        }
        cards.push(c);
    }
    for r in &tree.requirements {
        // Section-scale requirements ride inside their file's card.
        let Some(fields) = &r.fields else { continue };
        let mut c = Card::new(
            Kind::Requirement,
            r.slug.clone(),
            Some(r.file.clone()),
            r.line,
        );
        if !fill_doc_text(root, &mut c) {
            c = raw_card(root, r.file.clone(), Kind::Requirement, r.slug.clone());
        }
        if let Some((o, line)) = &fields.origin {
            let surface = origin_surface(o);
            c.push(2, *line, surface.clone());
            c.refs.origin = Some(surface);
        }
        if let Some((entries, line)) = &fields.satisfied_by {
            c.push(2, *line, entries.join(" "));
            c.refs.satisfied_by = entries.clone();
        }
        c.refs.state = Some(if fields.satisfied() {
            "satisfied"
        } else if fields.deferred() {
            "deferred"
        } else {
            "open"
        });
        cards.push(c);
    }
    for s in &tree.sessions {
        let mut c = Card::new(Kind::Session, s.slug.clone(), Some(s.file.clone()), s.line);
        if !fill_doc_text(root, &mut c) {
            c = raw_card(root, s.file.clone(), Kind::Session, s.slug.clone());
        }
        c.refs.version = s.version.as_ref().map(|(v, _)| v.clone());
        c.refs.closed = s.closed.as_ref().map(|(v, _)| {
            if v.is_empty() {
                "open".to_string()
            } else {
                v.clone()
            }
        });
        cards.push(c);
    }
    for st in &tree.stressors {
        let mut c = Card::new(Kind::Stressor, st.slug.clone(), Some(st.file.clone()), st.line);
        if !fill_doc_text(root, &mut c) {
            c = raw_card(root, st.file.clone(), Kind::Stressor, st.slug.clone());
        }
        if let Some((entries, line)) = &st.affects {
            c.push(2, *line, entries.join(" "));
            c.refs.affects = entries.clone();
        }
        c.refs.session = Some(st.session.clone());
        c.refs.outcome = st.outcome.map(|o| match o {
            Outcome::Pending => "pending",
            Outcome::Surviving => "surviving",
            Outcome::Breaking => "breaking",
        });
        cards.push(c);
    }

    // Whatever the schema walk dropped — unreadable, unparseable, misplaced
    // — still matches by raw text.
    let covered: BTreeSet<String> = cards.iter().filter_map(|c| c.file.clone()).collect();
    let mut found: Vec<(String, Kind)> = Vec::new();
    walk_md(
        root,
        &root.join("archi").join("requirements"),
        &mut found,
        "requirements",
    );
    walk_md(root, &root.join("archi").join("stress"), &mut found, "stress");
    for (file, kind) in found {
        if !covered.contains(file.as_str()) {
            let slug = file
                .rsplit('/')
                .next()
                .and_then(|n| n.strip_suffix(".md"))
                .unwrap_or_default()
                .to_string();
            cards.push(raw_card(root, file, kind, slug));
        }
    }

    // The element corpus: definitions out of the model dump, neighbors off
    // the edges, requirement stamps and stressor affects inverted onto them.
    if let Some(model) = model {
        let dump = model.dump();
        let mut elements: Vec<Card> = Vec::new();
        let mut neighbors: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for s in &dump {
            match s {
                Statement::Define(d) => {
                    let (path, doc) = match d {
                        Definition::Node { path, doc, .. } => (path, doc),
                        Definition::View { name, doc } => (name, doc),
                        Definition::Rel { name, doc, .. } => (name, doc),
                        Definition::Conn { name, doc, .. } => (name, doc),
                    };
                    let mut c = Card::new(Kind::Element, path.clone(), None, 0);
                    c.push(0, 0, path.clone());
                    if let Some(doc) = doc {
                        c.push(1, 0, doc.clone());
                        c.refs.definition = Some(doc.clone());
                    }
                    if let Definition::Node {
                        ports, port_docs, ..
                    } = d
                    {
                        // A port name is the element's addressable interface
                        // (`Sessions.fold` pipes into `query`) — it weighs as
                        // summary; its doc is supporting prose, body weight.
                        for p in ports.iter().flatten() {
                            c.push(1, 0, p.clone());
                        }
                        for (p, doc) in port_docs.iter().flatten() {
                            c.push(2, 0, format!("{p} {doc}"));
                        }
                    }
                    elements.push(c);
                }
                Statement::RelEdge { source, target, .. } => {
                    neighbors
                        .entry(source.clone())
                        .or_default()
                        .insert(target.clone());
                    neighbors
                        .entry(target.clone())
                        .or_default()
                        .insert(source.clone());
                }
                Statement::ConnEdge { source, target, .. } => {
                    neighbors
                        .entry(source.node.clone())
                        .or_default()
                        .insert(target.node.clone());
                    neighbors
                        .entry(target.node.clone())
                        .or_default()
                        .insert(source.node.clone());
                }
                _ => {}
            }
        }
        let mut stamped: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        let mut pressed: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for r in &tree.requirements {
            if let Some((entries, _)) = r.fields.as_ref().and_then(|f| f.satisfied_by.as_ref()) {
                for e in entries {
                    stamped.entry(e.as_str()).or_default().insert(&r.slug);
                }
            }
        }
        for st in &tree.stressors {
            if let Some((entries, _)) = &st.affects {
                for e in entries {
                    pressed.entry(e.as_str()).or_default().insert(&st.slug);
                }
            }
        }
        for c in &mut elements {
            if let Some(n) = neighbors.get(&c.slug) {
                c.refs.neighbors = n.iter().filter(|p| **p != c.slug).cloned().collect();
            }
            if let Some(reqs) = stamped.get(c.slug.as_str()) {
                c.refs.requirements = reqs.iter().map(|s| s.to_string()).collect();
            }
            if let Some(sts) = pressed.get(c.slug.as_str()) {
                c.refs.stressors = sts.iter().map(|s| s.to_string()).collect();
            }
        }
        cards.extend(elements);
    }

    cards
}

// ---- scoring and the report ----------------------------------------------------

/// One hit of the ranked list.
#[derive(Serialize)]
pub struct Hit {
    /// The card's kind label.
    pub kind: &'static str,
    /// The slug (docs) or model path (elements) — the address for the next verb.
    pub slug: String,
    /// The score, rounded for a stable surface.
    pub score: f64,
    /// Project-relative file, docs only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// 1-based line of the name, docs only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// The best-matching line, trimmed.
    pub snippet: String,
    /// The next hop.
    pub refs: Refs,
}

/// The whole answer: the envelope `--json` prints verbatim.
#[derive(Serialize)]
pub struct SearchReport {
    /// Always `ok` — a search that ran is a search that answered.
    pub status: &'static str,
    /// The phrase as scored.
    pub query: String,
    /// Corpora that could not be searched, each with its first reason.
    pub dark: Vec<String>,
    /// The ranked hits.
    pub hits: Vec<Hit>,
}

/// Run one search. `model` is `None` exactly when the compile failed;
/// `dark` carries that story into the report. An empty `kinds` filter does
/// not restrict.
pub fn search(
    root: &Path,
    model: Option<&Model>,
    dark: Vec<String>,
    phrase: &str,
    kinds: &[Kind],
    limit: usize,
) -> SearchReport {
    let query: Vec<String> = {
        let mut seen = BTreeSet::new();
        tokenize(phrase)
            .into_iter()
            .filter(|t| seen.insert(t.clone()))
            .collect()
    };
    let mut cards = corpus(root, model);
    let n = cards.len() as f64;

    // Document frequencies over the whole scanned corpus, kind filter or
    // not — the statistics describe the KB, not the slice.
    let idf: Vec<f64> = query
        .iter()
        .map(|qt| {
            let df = cards
                .iter()
                .filter(|c| c.toks.iter().any(|toks| field_strength(qt, toks) > 0.0))
                .count() as f64;
            if df == 0.0 { 0.0 } else { (n / df).ln() }
        })
        .collect();

    let mut scored: Vec<(f64, usize)> = cards
        .iter()
        .enumerate()
        .filter(|(_, c)| kinds.is_empty() || kinds.contains(&c.kind))
        .filter_map(|(i, c)| {
            let mut score = 0.0;
            for (qt, idf) in query.iter().zip(&idf) {
                if *idf == 0.0 {
                    continue;
                }
                let best = (0..3)
                    .map(|f| WEIGHTS[f] * field_strength(qt, &c.toks[f]))
                    .fold(0.0, f64::max);
                score += idf * best;
            }
            (score > 0.0).then_some((score, i))
        })
        .collect();
    scored.sort_by(|(sa, ia), (sb, ib)| {
        let (ca, cb) = (&cards[*ia], &cards[*ib]);
        sb.partial_cmp(sa)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(ca.kind.cmp(&cb.kind))
            .then(ca.slug.cmp(&cb.slug))
    });

    let hits = scored
        .into_iter()
        .take(limit)
        .map(|(score, i)| {
            let c = &mut cards[i];
            let snippet = snippet(c, &query);
            Hit {
                kind: c.kind.label(),
                slug: c.slug.clone(),
                score: (score * 1000.0).round() / 1000.0,
                file: c.file.clone(),
                line: (c.line > 0).then_some(c.line),
                snippet,
                refs: std::mem::take(&mut c.refs),
            }
        })
        .collect();

    SearchReport {
        status: "ok",
        query: phrase.to_string(),
        dark,
        hits,
    }
}

/// The best-matching summary or body line, trimmed; a name-only hit shows
/// the first summary line (the definition, for elements).
fn snippet(card: &Card, query: &[String]) -> String {
    let mut best: Option<(f64, &str)> = None;
    for field in [1, 2] {
        for (_, text) in &card.fields[field] {
            let toks: BTreeSet<String> = tokenize(text).into_iter().collect();
            let s: f64 = query.iter().map(|qt| field_strength(qt, &toks)).sum();
            if s > 0.0 && best.is_none_or(|(b, _)| s > b) {
                best = Some((s, text));
            }
        }
    }
    let text = best.map(|(_, t)| t).or_else(|| {
        card.fields[1]
            .first()
            .map(|(_, t)| t.as_str())
    });
    let text = text.unwrap_or_default();
    let mut out: String = text.chars().take(160).collect();
    if text.chars().count() > 160 {
        out.push('…');
    }
    out
}

/// The human rendering: one block per hit — address line, snippet, refs.
pub fn render_human(report: &SearchReport) -> String {
    let mut out = String::new();
    for d in &report.dark {
        out.push_str(&format!("dark: {d}\n"));
    }
    if report.hits.is_empty() {
        out.push_str("no hits\n");
        return out;
    }
    for h in &report.hits {
        let addr = match (&h.file, h.line) {
            (Some(f), Some(l)) => format!("  {f}:{l}"),
            _ => String::new(),
        };
        out.push_str(&format!(
            "{:<11} {}  {:.3}{}\n",
            h.kind, h.slug, h.score, addr
        ));
        if !h.snippet.is_empty() {
            out.push_str(&format!("    {}\n", h.snippet));
        }
        let refs = render_refs(&h.refs);
        if !refs.is_empty() {
            out.push_str(&format!("    {refs}\n"));
        }
    }
    out
}

pub(crate) fn render_refs(r: &Refs) -> String {
    let cap = |v: &[String]| -> String {
        let shown: Vec<&str> = v.iter().take(6).map(String::as_str).collect();
        let more = v.len().saturating_sub(6);
        if more > 0 {
            format!("{} +{more}", shown.join(", "))
        } else {
            shown.join(", ")
        }
    };
    let mut parts: Vec<String> = Vec::new();
    if !r.requirements.is_empty() {
        parts.push(format!("reqs: {}", cap(&r.requirements)));
    }
    if !r.stressors.is_empty() {
        parts.push(format!("pressed-by: {}", cap(&r.stressors)));
    }
    if !r.neighbors.is_empty() {
        parts.push(format!("neighbors: {}", cap(&r.neighbors)));
    }
    if let Some(o) = &r.origin {
        parts.push(format!("origin: {o}"));
    }
    if !r.satisfied_by.is_empty() {
        parts.push(format!("satisfied-by: {}", cap(&r.satisfied_by)));
    }
    if let Some(s) = r.state {
        parts.push(format!("state: {s}"));
    }
    if let Some(s) = &r.session {
        parts.push(format!("session: {s}"));
    }
    if !r.affects.is_empty() {
        parts.push(format!("affects: {}", cap(&r.affects)));
    }
    if let Some(o) = r.outcome {
        parts.push(format!("outcome: {o}"));
    }
    if let Some(v) = &r.version {
        parts.push(format!("pins: {v}"));
    }
    if let Some(c) = &r.closed {
        parts.push(format!("closed: {c}"));
    }
    parts.join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    const MODEL: &str = "def node AuthService: // password login for the api\n  port handle_login\ndef node RateLimiter // sheds the replay burst before hashing\ndef node CredStore\nService type_of AuthService\nService type_of RateLimiter\n";

    fn temp_project() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-search-test-{}-{}",
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

    fn compiled(root: &Path) -> modeling_lang::Workspace {
        modeling_lang::source::compile_project(root)
            .unwrap_or_else(|f| panic!("test model failed to compile:\n{}", f.render()))
            .workspace
    }

    /// The worked KB: an intent, a satisfied requirement answering the
    /// breaking stressor, a deferred requirement, a closed session with a
    /// pending stressor beside the breaking one.
    fn full_kb(root: &Path) {
        put(
            root,
            "archi/requirements/secure-auth/secure-auth.md",
            "# Secure auth\n\nPassword authentication that leaks nothing.\n",
        );
        put(
            root,
            "archi/requirements/secure-auth/rate-limit-logins.md",
            "---\nkind: functional\norigin: stressor(credential-stuffing)\nsatisfied-by: [RateLimiter]\ndeferred:\n---\n\n# Rate limit logins\n\nRate limiting sheds the replay burst before hashing.\n\n## System Context\n\nThe archive seals versions forever; folding rounds keeps one record.\n\n## Satisfy\n\n`RateLimiter` sheds the burst.\n\n- test — replay a burst, organic logins stay fast\n",
        );
        put(
            root,
            "archi/requirements/secure-auth/token-rotation.md",
            "---\nkind: functional\norigin: intent\nsatisfied-by: []\ndeferred: postponed to the v2 key hierarchy\n---\n\n# Token rotation\n\nKeys rotate on a fixed cadence.\n\n## System Context\n\n## Satisfy\n",
        );
        put(
            root,
            "archi/stress/auth-hardening/auth-hardening.md",
            "---\nversion: v0001\nclosed: v0002\n---\n\n# Auth hardening\n\nFirst adversarial round, pressing the fold of the login path.\n",
        );
        put(
            root,
            "archi/stress/auth-hardening/credential-stuffing.md",
            "---\naffects: [AuthService, RateLimiter]\noutcome: breaking\n---\n\n# Credential stuffing\n\nBots replay leaked pairs at 100x the organic rate limiting budget.\n\n## Attractor\n\nThe login path saturates on hash verification.\n\n## Resolution\n\nRate limiting takes the burst off the hot path.\n",
        );
        put(
            root,
            "archi/stress/auth-hardening/limiter-bypass.md",
            "---\naffects: [RateLimiter]\noutcome: pending\n---\n\n# Limiter bypass\n\nDistributed bots stay under the per-ip threshold.\n\n## Attractor\n\nThe limiter sees no single hot key.\n\n## Resolution\n",
        );
    }

    fn run(root: &Path, phrase: &str, kinds: &[Kind], limit: usize) -> SearchReport {
        let ws = compiled(root);
        search(root, Some(ws.model()), Vec::new(), phrase, kinds, limit)
    }

    fn kinds_of(r: &SearchReport) -> BTreeSet<&'static str> {
        r.hits.iter().map(|h| h.kind).collect()
    }

    fn slugs_of(r: &SearchReport) -> Vec<&str> {
        r.hits.iter().map(|h| h.slug.as_str()).collect()
    }

    #[test]
    fn one_phrase_spans_kinds_and_filters_narrow_and_limits_bound() {
        let root = temp_project();
        full_kb(&root);
        let r = run(&root, "rate limiting", &[], 20);
        let kinds = kinds_of(&r);
        assert!(kinds.contains("element"), "{:?}", slugs_of(&r));
        assert!(kinds.contains("requirement"), "{:?}", slugs_of(&r));
        assert!(kinds.contains("stressor"), "{:?}", slugs_of(&r));

        let narrowed = run(&root, "rate limiting", &[Kind::Requirement], 20);
        assert_eq!(kinds_of(&narrowed), ["requirement"].into());

        let bounded = run(&root, "rate limiting", &[], 2);
        assert_eq!(bounded.hits.len(), 2);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_scan_reads_the_live_tree_and_writes_nothing() {
        let root = temp_project();
        full_kb(&root);
        assert!(run(&root, "zebra", &[], 10).hits.is_empty());

        // The edit is searchable in the immediately following call.
        put(
            &root,
            "archi/requirements/secure-auth/token-rotation.md",
            "---\nkind: functional\norigin: intent\nsatisfied-by: []\ndeferred: postponed to the v2 key hierarchy\n---\n\n# Token rotation\n\nKeys rotate on a zebra cadence.\n\n## System Context\n\n## Satisfy\n",
        );
        let r = run(&root, "zebra", &[], 10);
        assert_eq!(slugs_of(&r), ["token-rotation"]);

        // The scan leaves the tree byte-identical.
        let snapshot = |root: &Path| -> Vec<(String, Vec<u8>)> {
            fn walk(dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
                let mut entries: Vec<_> =
                    fs::read_dir(dir).unwrap().flatten().map(|e| e.path()).collect();
                entries.sort();
                for p in entries {
                    if p.is_dir() {
                        walk(&p, out);
                    } else {
                        out.push((p.to_string_lossy().into_owned(), fs::read(&p).unwrap()));
                    }
                }
            }
            let mut out = Vec::new();
            walk(root, &mut out);
            out
        };
        let before = snapshot(&root);
        run(&root, "rate limiting versions fold", &[], 50);
        assert_eq!(before, snapshot(&root));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn matching_forgives_the_phrasing() {
        let root = temp_project();
        full_kb(&root);
        // Prefix both ways: `folding` reaches `fold`, `versioning` reaches
        // `versions`.
        let r = run(&root, "folding", &[], 10);
        assert!(
            slugs_of(&r).contains(&"auth-hardening"),
            "{:?}",
            slugs_of(&r)
        );
        let r = run(&root, "versioning", &[], 10);
        assert!(
            slugs_of(&r).contains(&"rate-limit-logins"),
            "{:?}",
            slugs_of(&r)
        );

        // A token in every card contributes nothing: cards matching only
        // the flooded term score zero and drop out.
        let root2 = temp_project();
        put(
            &root2,
            "archi/requirements/a/a.md",
            "# A\n\nShared word everywhere.\n",
        );
        put(
            &root2,
            "archi/requirements/a/one.md",
            "---\nkind: functional\norigin: intent\nsatisfied-by: []\ndeferred:\n---\n\n# One\n\nShared word here too.\n\n## System Context\n\n## Satisfy\n",
        );
        put(
            &root2,
            "archi/requirements/a/two.md",
            "---\nkind: functional\norigin: intent\nsatisfied-by: []\ndeferred:\n---\n\n# Two\n\nShared word and a zebra.\n\n## System Context\n\n## Satisfy\n",
        );
        let r = search(&root2, None, Vec::new(), "shared zebra", &[], 10);
        assert_eq!(slugs_of(&r), ["two"], "the flooded term alone must not rank");
        fs::remove_dir_all(&root2).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_name_hit_outranks_a_body_hit_and_ranking_is_deterministic() {
        let root = temp_project();
        full_kb(&root);
        // `token-rotation` carries `rotate` in name+body; rate-limit's body
        // does not — use `rotation`: name hit on token-rotation, none else.
        let r = run(&root, "token rotation", &[], 10);
        assert_eq!(r.hits[0].slug, "token-rotation");

        let a = serde_json::to_string(&run(&root, "rate limiting fold", &[], 10)).unwrap();
        let b = serde_json::to_string(&run(&root, "rate limiting fold", &[], 10)).unwrap();
        assert_eq!(a, b);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_dark_corpus_stays_partial() {
        let root = temp_project();
        full_kb(&root);
        // No model: element cards dark, doc cards still answer, the dark
        // note rides the report.
        let r = search(
            &root,
            None,
            vec!["model: it does not compile".into()],
            "rate limiting",
            &[],
            10,
        );
        assert!(!r.hits.is_empty());
        assert!(!kinds_of(&r).contains("element"));
        assert_eq!(r.dark, ["model: it does not compile"]);

        // An unparseable doc degrades to raw text and still matches.
        put(
            &root,
            "archi/requirements/secure-auth/broken.md",
            "---\nnever closed\n\n# Broken\n\nA quagga hides in the raw text.\n",
        );
        let r = run(&root, "quagga", &[], 10);
        assert_eq!(slugs_of(&r), ["broken"]);
        assert_eq!(r.hits[0].kind, "requirement");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn cards_carry_the_next_hop() {
        let root = temp_project();
        full_kb(&root);
        let r = run(&root, "replay burst", &[], 10);
        let limiter = r
            .hits
            .iter()
            .find(|h| h.kind == "element" && h.slug == "RateLimiter")
            .expect("the element card");
        assert_eq!(
            limiter.refs.definition.as_deref(),
            Some("sheds the replay burst before hashing")
        );
        assert_eq!(limiter.refs.requirements, ["rate-limit-logins"]);
        assert_eq!(
            limiter.refs.stressors,
            ["credential-stuffing", "limiter-bypass"]
        );
        assert!(limiter.refs.neighbors.contains(&"Service".to_string()));
        assert!(limiter.file.is_none() && limiter.line.is_none());

        let req = run(&root, "rate limit logins", &[Kind::Requirement], 10);
        let req = &req.hits[0];
        assert_eq!(req.slug, "rate-limit-logins");
        assert_eq!(
            req.refs.origin.as_deref(),
            Some("stressor(credential-stuffing)")
        );
        assert_eq!(req.refs.satisfied_by, ["RateLimiter"]);
        assert_eq!(req.refs.state, Some("satisfied"));
        assert!(req.file.as_deref().unwrap().ends_with("rate-limit-logins.md"));
        assert!(req.line.is_some());

        let st = run(&root, "credential stuffing", &[Kind::Stressor], 10);
        let st = &st.hits[0];
        assert_eq!(st.refs.session.as_deref(), Some("auth-hardening"));
        assert_eq!(st.refs.affects, ["AuthService", "RateLimiter"]);
        assert_eq!(st.refs.outcome, Some("breaking"));

        let ses = run(&root, "auth hardening", &[Kind::Session], 10);
        let ses = &ses.hits[0];
        assert_eq!(ses.refs.version.as_deref(), Some("v0001"));
        assert_eq!(ses.refs.closed.as_deref(), Some("v0002"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn advisory_states_are_searchable_content() {
        let root = temp_project();
        full_kb(&root);
        // A deferred requirement and a pending stressor in a closed session
        // are hits like any other — never a refusal.
        let r = run(&root, "token rotation", &[], 10);
        assert_eq!(r.hits[0].refs.state, Some("deferred"));
        let r = run(&root, "limiter bypass", &[], 10);
        assert_eq!(r.hits[0].refs.outcome, Some("pending"));
        assert_eq!(r.status, "ok");
        fs::remove_dir_all(&root).unwrap();
    }
}
