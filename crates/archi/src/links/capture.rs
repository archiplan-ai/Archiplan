//! Task-close capture: the wave-open item-hash index and the delta scan
//! that mints candidate links
//! (`requirements/code-link.md#code-links--tasks`).
//!
//! A wave opening records the tree as a **canonical item-hash index** —
//! file → symbol → body hash, by the canonicalizer of [`super::code`] — so
//! a closing task's delta is read off by hash comparison: symbol-granular,
//! cheap to store (no file contents), and git-free by construction, so
//! squashes and shallow clones cannot break it. The task's `spec_refs` ×
//! its changed symbols become candidate links — evidence, `indirect`,
//! `captured(task)` — which the closing agent reviews and selectively
//! asserts. Changed items in files no task claims are **leftovers**,
//! reported rather than guessed at.
//!
//! Capture is idempotent: a candidate that already lives (or was retired
//! at identical pins — a subtraction that must stick) is not re-minted.
//! Re-encounters and unreconfirmed rewrites journal as `touch` and `decay`
//! events instead, once per task — the observations confidence is derived
//! from.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use modeling_lang::Model;
use serde::{Deserialize, Serialize};

use super::code;
use super::{Anchor, Birth, Event, Link, LinkKind, Origin, SpecRef, Standing};
use crate::plans::{self, Task};
use crate::versions;

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

/// A tree snapshot: every code file's canonical item hashes.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct TreeIndex {
    /// File → its index, project-relative paths.
    pub files: BTreeMap<String, FileIndex>,
}

impl TreeIndex {
    /// Scan the working tree: every code file canonicalized and indexed.
    pub fn scan(root: &Path) -> TreeIndex {
        let mut files = BTreeMap::new();
        for file in super::code_files(root) {
            let Ok(text) = fs::read_to_string(root.join(&file)) else {
                continue;
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
                file,
                FileIndex {
                    hash: canonical.file_hash(),
                    symbols,
                },
            );
        }
        TreeIndex { files }
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
/// `plan next` write the index its wave's deltas diff against.
pub(crate) fn write_index(root: &Path, plan: &str, wave: usize) -> Result<(), String> {
    let index = TreeIndex::scan(root);
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

// ---- capture -----------------------------------------------------------------

/// One capture's outcome.
#[derive(Default, Serialize)]
pub struct CaptureOutcome {
    /// Freshly minted candidate links — evidence, awaiting review.
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

/// Capture a wave: diff the wave-open index against the current tree and
/// mint the in-flight tasks' candidate links; journal touches and decays.
/// `only` restricts minting to one task (`link capture --task`) while the
/// claim map still spans the whole wave.
pub(crate) fn capture_wave(
    root: &Path,
    plan_name: &str,
    wave: usize,
    in_flight: &[&Task],
    only: Option<&str>,
) -> Result<CaptureOutcome, String> {
    let opened = read_index(root, plan_name, wave)?;
    let current = TreeIndex::scan(root);
    let changes = delta(&opened, &current);
    let folded = super::load(root)?;
    let commit = versions::provenance(root);

    let mut out = CaptureOutcome::default();
    let mut shared: BTreeSet<&str> = BTreeSet::new();
    let mut events: Vec<Event> = Vec::new();
    // The working link set: the fold plus this batch's mints, so the decay
    // pass sees same-run mints from overlapping tasks symmetrically.
    let mut live: Vec<Link> = folded.live.clone();
    let mut adds = folded.adds;

    // Mint and touch, per change, per claiming task.
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
        let anchor = Anchor {
            file: change.file.clone(),
            symbol: change.symbol.clone(),
        };
        let resolved = match super::resolve_anchor(root, &anchor) {
            Ok(r) => r,
            Err(e) => {
                out.notes.push(format!("skipped `{anchor}`: {e}"));
                continue;
            }
        };
        for task in &claimants {
            if only.is_some_and(|o| o != task.id) {
                continue;
            }
            for spec_ref in &task.spec_refs {
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
                // A candidate retired at identical pins stays subtracted;
                // new content is new evidence.
                let subtracted = folded.retired.iter().any(|l| {
                    l.spec.version.is_none()
                        && l.spec.path == *spec_ref
                        && l.anchor == anchor
                        && l.pins == resolved.pins
                        && matches!(&l.origin, Origin::Captured { task: t } if *t == task.id)
                });
                if subtracted {
                    continue;
                }
                adds += 1;
                let link = Link {
                    id: format!("l{adds:04}"),
                    spec: SpecRef {
                        path: spec_ref.clone(),
                        version: None,
                    },
                    anchor: anchor.clone(),
                    kind: LinkKind::Indirect,
                    standing: Standing::Evidence,
                    origin: Origin::Captured {
                        task: task.id.clone(),
                    },
                    birth: Birth {
                        created: super::now(),
                        commit: commit.clone(),
                        spans: vec![resolved.span.clone()],
                    },
                    pins: resolved.pins.clone(),
                    touches: Vec::new(),
                    decays: Vec::new(),
                };
                events.push(Event::Add { link: link.clone() });
                out.minted.push(link.clone());
                live.push(link);
            }
        }
    }

    // Decay, after all mints: every claiming task presses on the evidence
    // links anchored at its changed items whose spec_ref it does not carry
    // — a rewrite without reconfirmation, observed exactly when it happens.
    // Overlapping claims cross-press same-run mints: split confidence.
    for change in &changes {
        let anchor = Anchor {
            file: change.file.clone(),
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
    capture_wave(root, &plan.name, wave, &in_flight, Some(task_id))
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
    for n in &o.notes {
        out.push_str(&format!("note: {n}\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    const STORE_RS: &str = "pub struct Store {\n    rows: Vec<u8>,\n}\n\n\
                            impl Store {\n    pub fn put(&mut self, row: u8) {\n        self.rows.push(row);\n    }\n}\n";

    fn temp_project() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-capture-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(dir.join("code")).unwrap();
        fs::write(dir.join("archi.toml"), "[project]\nname = \"t\"\n").unwrap();
        fs::write(dir.join("code/store.rs"), STORE_RS).unwrap();
        fs::write(dir.join("code/schema.sql"), "CREATE TABLE t (id INT);\n").unwrap();
        dir
    }

    fn task(id: &str, spec_refs: &[&str], outputs: &[&str]) -> Task {
        Task {
            id: id.to_string(),
            node: format!("Node{id}"),
            description: String::new(),
            spec_refs: spec_refs.iter().map(|s| s.to_string()).collect(),
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
        assert_eq!(delta(&opened, &TreeIndex::scan(&root)), Vec::<Changed>::new());

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
        let changed = delta(&opened, &TreeIndex::scan(&root));
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

    #[test]
    fn capture_mints_claimed_deltas_and_reports_leftovers() {
        let root = temp_project();
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

        let out = capture_wave(&root, "p", 1, &[&t1], None).unwrap();
        // spec_refs × changed symbols: 2 × 1.
        assert_eq!(out.minted.len(), 2, "{}", render_capture(&out));
        let l = &out.minted[0];
        assert_eq!(l.standing, Standing::Evidence);
        assert_eq!(l.kind, LinkKind::Indirect);
        assert_eq!(l.origin, Origin::Captured { task: "t1".into() });
        assert_eq!(l.anchor.to_string(), "code/store.rs#Store::put");
        assert_eq!(l.birth.spans[0].file, "code/store.rs");
        assert_eq!(out.leftovers.len(), 1);
        assert_eq!(out.leftovers[0].to_string(), "code/schema.sql");

        // Idempotent: the re-run mints nothing and touches nothing — the
        // links are this task's own.
        let again = capture_wave(&root, "p", 1, &[&t1], None).unwrap();
        assert!(again.minted.is_empty() && again.touched.is_empty(), "{}", render_capture(&again));
        assert_eq!(ls(&root).len(), 2);

        // A retired candidate stays subtracted while the code stands
        // still, and re-mints when the symbol moves again.
        let id = ls(&root)[0].id.clone();
        super::super::retire(&root, &[id]).unwrap();
        let after = capture_wave(&root, "p", 1, &[&t1], None).unwrap();
        assert!(after.minted.is_empty(), "{}", render_capture(&after));
        fs::write(
            root.join("code/store.rs"),
            STORE_RS.replace("self.rows.push(row);", "self.rows.push(row ^ 1);"),
        )
        .unwrap();
        let moved = capture_wave(&root, "p", 1, &[&t1], None).unwrap();
        assert_eq!(moved.minted.len(), 1, "{}", render_capture(&moved));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn touches_and_decays_journal_once_per_task() {
        let root = temp_project();
        fs::create_dir_all(plans::plan_dir(&root, "p")).unwrap();
        write_index(&root, "p", 1).unwrap();

        // Two tasks claim the same file; one carries `Store`, the other
        // does not.
        let t1 = task("t1", &["Store"], &["code/"]);
        let t2 = task("t2", &["Gate"], &["code/"]);
        fs::write(
            root.join("code/store.rs"),
            STORE_RS.replace("self.rows.push(row);", "self.rows.insert(0, row);"),
        )
        .unwrap();

        let out = capture_wave(&root, "p", 1, &[&t1, &t2], None).unwrap();
        // Each task mints its own candidate on the shared symbol, and each
        // presses on the other's: split confidence.
        assert_eq!(out.minted.len(), 2, "{}", render_capture(&out));
        assert_eq!(out.decayed.len(), 2, "{}", render_capture(&out));
        assert!(out.notes.iter().any(|n| n.contains("claimed by several")), "{:?}", out.notes);
        let live = ls(&root);
        let store_link = live.iter().find(|l| l.spec.path == "Store").unwrap();
        let gate_link = live.iter().find(|l| l.spec.path == "Gate").unwrap();
        assert_eq!(store_link.decays, vec!["t2".to_string()]);
        assert_eq!(gate_link.decays, vec!["t1".to_string()]);

        // Re-runs never double-journal; a third task carrying `Store`
        // touches the standing evidence instead of re-minting, and decays
        // the `Gate` link it does not carry.
        let again = capture_wave(&root, "p", 1, &[&t1, &t2], None).unwrap();
        assert!(again.minted.is_empty() && again.decayed.is_empty() && again.touched.is_empty());
        let t3 = task("t3", &["Store"], &["code/"]);
        let third = capture_wave(&root, "p", 1, &[&t1, &t2, &t3], None).unwrap();
        assert!(third.minted.is_empty(), "{}", render_capture(&third));
        assert!(third.touched.iter().any(|(id, task)| id == &store_link.id && task == "t3"));
        assert!(third.decayed.iter().any(|(id, task)| id == &gate_link.id && task == "t3"));

        fs::remove_dir_all(&root).unwrap();
    }
}
