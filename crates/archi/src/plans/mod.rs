//! Plans: a hardened spec projected into an executable task graph
//! (`archi/requirements/planning/`).
//!
//! A plan pins one archived spec version and cuts **one task per node**;
//! each task's `spec_refs` are seeded from the pinned model — the node plus
//! its incoming edges — and its requirements are **derived**: the reverse
//! lookup over the doc tree recomputes them on every read, because
//! requirements are living documents outside the version archive and a
//! stored match set could only go stale. Requirement identity is the slug
//! (unique project-wide, `E_SLUG`); the `slot` is a per-task ordinal for
//! short addresses, derived with the rest.
//!
//! The editing surface splits like the rest of the system: authored
//! content — envelope prose, descriptions, `inputs`, `outputs`, extra
//! `spec_refs`, `owns`, `verifications`, scenarios — lives in the record
//! folder's markdown files ([`records`]) and is edited there directly,
//! exactly as requirements are edited in their markdown; lifecycle state
//! moves only through verbs, into `state.json`, and every verb
//! re-validates the files on load, so a hand edit cannot drift silently.
//! A legacy `plan.json` stays readable forever — read-only: its lifecycle
//! verbs work, its authoring surface refuses
//! (`archi/requirements/planning/a-plan-is-a-folder-of-records.md`).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use modeling_lang::{Definition, ElementKind, Model, Statement, Workspace};
use serde::{Deserialize, Serialize};

use crate::docs;
use crate::links;
use crate::versions;

mod records;

// ---- the plan model ---------------------------------------------------------

/// Lifecycle state. `Draft` is authoring, `Started` runs waves,
/// `Completed` is closed — by the scenario dance or `plan close`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanState {
    /// Tasks are being cut and verifications authored.
    Draft,
    /// Waves are in flight; `plan next` advances.
    Started,
    /// All waves closed and scenarios latched, or manually closed.
    Completed,
}

impl PlanState {
    pub(crate) fn describe(self) -> &'static str {
        match self {
            PlanState::Draft => "draft",
            PlanState::Started => "started",
            PlanState::Completed => "completed",
        }
    }
}

/// One technology choice of the envelope, with its provenance — why this
/// tech, mandated by what.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TechChoice {
    /// The concrete technology.
    pub tech: String,
    /// Where the choice came from.
    #[serde(default)]
    pub provenance: String,
}

/// One line of the architecture summary: a top-level node and its role.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummaryLine {
    /// The node path.
    pub node: String,
    /// Its one-line role.
    pub role: String,
}

/// Which concrete tech realizes which summary node.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StackMapping {
    /// The technology, as named in the stack.
    pub tech: String,
    /// The node it realizes.
    pub node: String,
}

/// One task: pinned to one node of one scope at the plan's version.
/// `spec_refs` are seeded on create; everything else is authored by
/// editing `plan.json`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    /// Dense id, `t1` onward — never reused, even past hand deletions.
    pub id: String,
    /// The node this task implements, at the pinned version.
    pub node: String,
    /// What the task does.
    #[serde(default)]
    pub description: String,
    /// The spec elements it realizes: node paths and canonical edge text,
    /// version-free — the plan's pin applies to all of them.
    pub spec_refs: Vec<String>,
    /// The requirements this task owns, curated from the derived matched
    /// set — a strict subset, authored at the plan stage. Verification
    /// duty counts these, not every match: several tasks may touch one
    /// element without all of them answering for its requirements.
    #[serde(default)]
    pub owns: Vec<String>,
    /// Concrete tech detail for this task.
    #[serde(default)]
    pub stack_details: String,
    /// What this task consumes, keyed by the producing task's id — the
    /// single source of truth for inter-task dependencies.
    #[serde(default)]
    pub inputs: BTreeMap<String, String>,
    /// Files this task writes. An entry ending in `/` claims a directory;
    /// capture attributes deltas through these.
    #[serde(default)]
    pub outputs: Vec<String>,
    /// Authored proofs, keyed by matched requirement slug: how the
    /// implementer will show the requirement is met.
    #[serde(default)]
    pub verifications: BTreeMap<String, Vec<String>>,
}

/// One plan, as stored at `archi/plans/<name>/plan.json`.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    /// The plan's name — equals its directory.
    pub name: String,
    /// The pinned spec version (`vNNNN`); `plan repin` moves it.
    pub version: String,
    /// The pinned version's content hash (`sha256:…`), stamped at use and
    /// repin — a pin is verified by content, not id alone, so a remint
    /// cannot silently reinterpret it
    /// (`archi/requirements/worktree-parallelism/pins-survive-a-remint`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_hash: Option<String>,
    /// ISO-8601 UTC timestamp of creation.
    pub created: String,
    /// Lifecycle state.
    pub state: PlanState,
    /// Waves that passed the gate; the wave in flight is `closed_waves + 1`.
    #[serde(default)]
    pub closed_waves: usize,
    /// What the plan solves.
    #[serde(default)]
    pub problem: String,
    /// The technology stack, with provenance.
    #[serde(default)]
    pub technology_stack: Vec<TechChoice>,
    /// One-line role per top-level node.
    #[serde(default)]
    pub architecture_summary: Vec<SummaryLine>,
    /// Which concrete tech realizes which summary node.
    #[serde(default)]
    pub stack_mapping: Vec<StackMapping>,
    /// Free-text user stories, displayed as the final `plan next` step.
    #[serde(default)]
    pub scenarios: Vec<String>,
    /// The scenarios block was printed after the last wave closed.
    #[serde(default)]
    pub scenarios_displayed: bool,
    /// The scenario step was acknowledged; the plan completed through it.
    #[serde(default)]
    pub scenarios_closed: bool,
    /// The task graph.
    #[serde(default)]
    pub tasks: Vec<Task>,
}

// ---- storage ----------------------------------------------------------------

fn plans_dir(root: &Path) -> PathBuf {
    root.join("archi").join("plans")
}

pub(crate) fn plan_dir(root: &Path, name: &str) -> PathBuf {
    plans_dir(root).join(name)
}

fn plan_path(root: &Path, name: &str) -> PathBuf {
    plan_dir(root, name).join("plan.json")
}

fn marker_path(root: &Path) -> PathBuf {
    plans_dir(root).join(".current")
}

/// Plan names are slugs (`archi/requirements/spec-docs/slugs-are-the-reference-currency.md`): they become directories
/// and the `.current` marker's content.
fn validate_name(name: &str) -> Result<(), String> {
    let slug_bytes = name
        .bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if name.is_empty() || !slug_bytes || name.starts_with('-') || name.ends_with('-') {
        return Err(format!(
            "`{name}` is not a slug: lowercase, digits and `-` (E_SLUG)"
        ));
    }
    Ok(())
}

/// Both storage forms load into the one [`Plan`]: the record folder is
/// the format, `plan.json` is legacy read-only — carrying both at once
/// is a conflict the author resolves, never a merge.
pub(crate) fn load_plan(root: &Path, name: &str) -> Result<Plan, String> {
    let path = plan_path(root, name);
    if records::is_record(root, name) {
        if path.exists() {
            return Err(format!(
                "plan `{name}` carries both plan.json and the record folder — keep one: \
                 the folder is the format, plan.json is legacy read-only"
            ));
        }
        return records::load(root, name);
    }
    let text =
        fs::read_to_string(&path).map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
    let plan: Plan = serde_json::from_str(&text)
        .map_err(|e| format!("`{}` does not parse: {e}", path.display()))?;
    if plan.name != name {
        return Err(format!(
            "`{}` names plan `{}` but sits in `{name}/` — the directory is the identity",
            path.display(),
            plan.name
        ));
    }
    Ok(plan)
}

/// The legacy full write. A record plan refuses it — its content is its
/// files; [`save_state`] is the lifecycle-only path both forms share.
fn store_plan(root: &Path, plan: &Plan) -> Result<(), String> {
    if records::is_record(root, &plan.name) {
        return Err(
            "a record plan's content is its files — edit them; verbs move lifecycle alone".into(),
        );
    }
    let dir = plan_dir(root, &plan.name);
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create `{}`: {e}", dir.display()))?;
    let path = plan_path(root, &plan.name);
    let mut text = serde_json::to_string_pretty(plan).map_err(|e| format!("plan serializes: {e}"))?;
    text.push('\n');
    fs::write(&path, text).map_err(|e| format!("cannot write `{}`: {e}", path.display()))
}

/// Persist lifecycle alone — state, waves, latches, the pin. A record
/// plan writes `state.json`; a legacy plan rewrites plan.json whole, as
/// it always did (its content fields ride along unchanged).
fn save_state(root: &Path, plan: &Plan) -> Result<(), String> {
    if records::is_record(root, &plan.name) {
        records::write_state(root, plan)
    } else {
        store_plan(root, plan)
    }
}

fn write_marker(root: &Path, name: &str) -> Result<(), String> {
    let path = marker_path(root);
    let dir = path.parent().expect("marker has a directory");
    fs::create_dir_all(dir).map_err(|e| format!("cannot create `{}`: {e}", dir.display()))?;
    fs::write(&path, format!("{name}\n"))
        .map_err(|e| format!("cannot write `{}`: {e}", path.display()))
}

/// The name `.current` points at, if any.
pub(crate) fn active_name(root: &Path) -> Result<Option<String>, String> {
    let path = marker_path(root);
    if !path.exists() {
        return Ok(None);
    }
    let text =
        fs::read_to_string(&path).map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
    let name = text.trim().to_string();
    Ok((!name.is_empty()).then_some(name))
}

/// The active plan; an explicit error when none is.
pub(crate) fn load_active(root: &Path) -> Result<Plan, String> {
    match active_name(root)? {
        None => Err("no active plan: `archi plan use <name>` creates or switches".into()),
        Some(name) => load_plan(root, &name),
    }
}

/// Every stored plan, sorted by name — the handoff listing `status` prints.
pub(crate) fn all_plans(root: &Path) -> Result<Vec<Plan>, String> {
    let dir = plans_dir(root);
    let Ok(entries) = fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let mut names: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names.iter().map(|n| load_plan(root, n)).collect()
}

fn now() -> String {
    versions::iso8601_utc(SystemTime::now())
}

/// The version a plan may pin: the one the live model is *at*. A dirty or
/// unversioned model refuses — a plan projects a hardened spec, and
/// hardening is `archi version save`.
fn pinnable_version(root: &Path, model: &Model) -> Result<String, String> {
    match versions::current(root, model)? {
        versions::Current::At(id) => Ok(id),
        versions::Current::NoVersions => Err(
            "the project has no versions: a plan pins a hardened spec — \
             `archi version save -m <note>` first"
                .into(),
        ),
        versions::Current::DirtySince(id) => Err(format!(
            "the live model has unsaved changes since {id}: a plan pins a hardened spec — \
             save a version first"
        )),
    }
}

/// Compile the plan's pinned version out of the sealed archive.
fn compile_pinned(root: &Path, version: &str) -> Result<Workspace, String> {
    let archive = versions::Archive::open(root)?
        .ok_or_else(|| "the project has no version archive".to_string())?;
    docs::compile_version(root, &archive, version)
}

// ---- verbs: use, repin, task add --------------------------------------------

/// The outcome of `archi plan use`.
#[derive(Debug)]
pub enum Used {
    /// A fresh skeleton was created and made current.
    Created(Plan),
    /// An existing plan was made current.
    Switched(Plan),
}

/// `archi plan use <name>`: switch to a named plan, minting an empty
/// record folder pinned to the current spec version on first use —
/// charter and scenarios skeletons for the author, `state.json` for the
/// verbs. Switching reaches both forms; re-`use` of an existing plan is
/// a switch, as always.
pub fn use_plan(root: &Path, model: &Model, name: &str) -> Result<Used, String> {
    validate_name(name)?;
    if plan_path(root, name).exists() || records::is_record(root, name) {
        let plan = load_plan(root, name)?;
        write_marker(root, name)?;
        return Ok(Used::Switched(plan));
    }
    let version = pinnable_version(root, model)?;
    let version_hash = pin_hash(root, &version);
    let plan = records::mint(root, name, version, version_hash, now())?;
    write_marker(root, name)?;
    Ok(Used::Created(plan))
}

/// `archi plan repin`: move the active plan's pin to the version the live
/// model is at — the sanctioned fix when the spec advances mid-plan. The
/// next `plan verify` flags every task whose obligations no longer hold.
pub fn repin(root: &Path, model: &Model) -> Result<(Plan, String), String> {
    let mut plan = load_active(root)?;
    let to = pinnable_version(root, model)?;
    if to == plan.version {
        return Err(format!("already pinned to {to}"));
    }
    let from = std::mem::replace(&mut plan.version, to);
    plan.version_hash = pin_hash(root, &plan.version);
    save_state(root, &plan)?;
    Ok((plan, from))
}

/// The archived content hash a pin records; `None` leaves an unhashed pin,
/// which `check` treats as silent (pre-hash plans keep working).
fn pin_hash(root: &Path, version: &str) -> Option<String> {
    versions::Archive::open(root)
        .ok()
        .flatten()
        .and_then(|a| a.entry(version).map(|e| e.model.clone()))
}

/// A plan whose pin no longer means what it meant
/// (`archi/requirements/worktree-parallelism/pins-survive-a-remint`).
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlanFinding {
    StalePin { plan: String, version: String },
}

impl std::fmt::Display for PlanFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanFinding::StalePin { plan, version } => write!(
                f,
                "stale plan pin: `{plan}` pins {version}, whose archived content changed \
                 since the pin — `archi plan repin`"
            ),
        }
    }
}

/// Compare every stored plan's pin hash against the archive. Hash-less
/// plans and ids the archive no longer holds stay silent — the lazy
/// `compile_pinned` error owns those.
pub(crate) fn check(root: &Path) -> Result<Vec<PlanFinding>, String> {
    let Some(archive) = versions::Archive::open(root)? else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for p in all_plans(root)? {
        if let (Some(stored), Some(entry)) = (&p.version_hash, archive.entry(&p.version)) {
            if stored != &entry.model {
                out.push(PlanFinding::StalePin { plan: p.name.clone(), version: p.version.clone() });
            }
        }
    }
    Ok(out)
}

/// The refusal every content-writing verb meets on a legacy plan: the
/// old json form is history — readable forever, never grown.
const LEGACY_READ_ONLY: &str =
    "legacy plan.json is read-only — plans author as record folders now";

/// `archi plan task add <node>`: mint a task file pinned to a node of
/// the pinned version, spec_refs seeded as the node plus its incoming
/// edges — the skeleton whose slots the author fills. Re-minting a node
/// whose file still is its skeleton converges; a file that moved past it
/// is the author's and refuses.
pub fn task_add(root: &Path, node: &str, description: Option<&str>) -> Result<Task, String> {
    let plan = load_active(root)?;
    if !records::is_record(root, &plan.name) {
        return Err(LEGACY_READ_ONLY.into());
    }
    if plan.state != PlanState::Draft {
        return Err(format!(
            "the plan is {}: tasks are cut in draft — `plan reset` restructures",
            plan.state.describe()
        ));
    }
    let ws = compile_pinned(root, &plan.version)?;
    let model = ws.model();
    if !model.has_node(node) {
        return Err(format!(
            "`{node}` names no element of {} (E_MODEL_REF)",
            plan.version
        ));
    }
    let skeleton = |id: String| Task {
        id,
        node: node.to_string(),
        description: description.unwrap_or_default().to_string(),
        spec_refs: seed_spec_refs(model, node),
        owns: Vec::new(),
        stack_details: String::new(),
        inputs: BTreeMap::new(),
        outputs: Vec::new(),
        verifications: BTreeMap::new(),
    };
    if let Some(t) = plan.tasks.iter().find(|t| t.node == node) {
        // The node is already cut. Byte-equal to what this mint would
        // write means nothing happened yet — converge; anything else is
        // authored content this verb must not touch.
        let path = records::task_path(root, &plan.name, &t.id)
            .ok_or_else(|| format!("no file carries `{}`", t.id))?;
        let on_disk = fs::read_to_string(&path)
            .map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
        let task = skeleton(t.id.clone());
        let shown = path.strip_prefix(root).unwrap_or(&path).display();
        if on_disk == records::render_task(&task) {
            println!("already minted — {shown} stands; fill its slots");
            return Ok(task);
        }
        return Err(format!("`{shown}` moved past its skeleton — not re-mintable"));
    }
    let task = skeleton(next_task_id(&plan.tasks));
    records::write_task(root, &plan.name, &task)?;
    Ok(task)
}

/// `archi plan task rm <id>`: unmint a task file — draft-only, and
/// refused while any other task inputs it. Ids stay stable: the gap is
/// fine, the next `task add` mints past the highest ever seen.
pub fn task_rm(root: &Path, id: &str) -> Result<PathBuf, String> {
    let plan = load_active(root)?;
    if !records::is_record(root, &plan.name) {
        return Err(LEGACY_READ_ONLY.into());
    }
    if plan.state != PlanState::Draft {
        return Err(format!(
            "the plan is {}: past draft — `plan reset` first",
            plan.state.describe()
        ));
    }
    if !plan.tasks.iter().any(|t| t.id == id) {
        return Err(format!("no task `{id}` — `archi plan show` lists them"));
    }
    let dependents: Vec<&str> = plan
        .tasks
        .iter()
        .filter(|t| t.inputs.contains_key(id))
        .map(|t| t.id.as_str())
        .collect();
    if !dependents.is_empty() {
        return Err(format!(
            "`{id}` feeds {} — cut those inputs first",
            dependents.join(", ")
        ));
    }
    let path = records::task_path(root, &plan.name, id)
        .ok_or_else(|| format!("no file carries `{id}`"))?;
    fs::remove_file(&path).map_err(|e| format!("cannot remove `{}`: {e}", path.display()))?;
    Ok(path)
}

/// One past the highest id present: a removed middle task leaves its
/// gap, so ids never shift under the plan's feet.
fn next_task_id(tasks: &[Task]) -> String {
    let max = tasks
        .iter()
        .filter_map(|t| t.id.strip_prefix('t')?.parse::<usize>().ok())
        .max()
        .unwrap_or(0);
    format!("t{}", max + 1)
}

/// The task's obligations at birth: the node itself plus every edge of the
/// pinned model that comes into it — a directed edge whose target end lands
/// on the node or inside its subtree from outside, or an undirected edge
/// crossing that boundary either way.
fn seed_spec_refs(model: &Model, node: &str) -> Vec<String> {
    let dump = model.dump();
    let mut directed: BTreeMap<&str, bool> = BTreeMap::new();
    for s in &dump {
        match s {
            Statement::Define(Definition::Rel {
                name, directed: d, ..
            })
            | Statement::Define(Definition::Conn {
                name, directed: d, ..
            }) => {
                directed.insert(name.as_str(), *d);
            }
            _ => {}
        }
    }
    let prefix = format!("{node}.");
    let inside = |path: &str| path == node || path.starts_with(&prefix);
    let mut refs = vec![node.to_string()];
    for s in &dump {
        let (edge_type, src, dst) = match s {
            Statement::RelEdge {
                rel,
                source,
                target,
                ..
            } => (rel.as_str(), source.as_str(), target.as_str()),
            Statement::ConnEdge {
                conn,
                source,
                target,
                ..
            } => (conn.as_str(), source.node.as_str(), target.node.as_str()),
            _ => continue,
        };
        let (src_in, dst_in) = (inside(src), inside(dst));
        let incoming = dst_in && !src_in;
        let undirected_crossing =
            !directed.get(edge_type).copied().unwrap_or(true) && src_in != dst_in;
        if (incoming || undirected_crossing)
            && let Some(pseudo) = links::edge_pseudo(s)
        {
            refs.push(pseudo);
        }
    }
    refs
}

// ---- the derived view: reverse lookup and waves ------------------------------

/// One requirement the reverse lookup matched to a task. Identity is the
/// slug; `slot` is a per-task ordinal for short addresses, derived with
/// the rest and stable while the matched set is.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct MatchedReq {
    /// `r1`, `r2`, … in slug order.
    pub slot: String,
    /// The requirement's slug.
    pub req: String,
    /// Which of the task's own spec_refs pulled it in.
    pub matched_refs: Vec<String>,
    /// Whether the task owns this match (`owns` in plan.json) — the match
    /// is the candidate, ownership is the curation.
    #[serde(default)]
    pub owned: bool,
}

/// Everything derived from a plan against the spec — recomputed, never
/// stored.
#[derive(Serialize)]
pub struct Derived {
    /// Task id → matched requirements.
    pub matched: BTreeMap<String, Vec<MatchedReq>>,
    /// Topological layers of the inputs DAG: wave k holds the tasks whose
    /// inputs all close earlier. Empty when the structure is broken.
    pub waves: Vec<Vec<String>>,
}

/// Endpoint nodes of every edge of a model, keyed by canonical surface
/// text — how edge spec_refs reach requirements on their ends.
fn edge_endpoints(model: &Model) -> BTreeMap<String, (String, String)> {
    let mut out = BTreeMap::new();
    for s in model.dump() {
        let ends = match &s {
            Statement::RelEdge { source, target, .. } => (source.clone(), target.clone()),
            Statement::ConnEdge { source, target, .. } => {
                (source.node.clone(), target.node.clone())
            }
            _ => continue,
        };
        if let Some(pseudo) = links::edge_pseudo(&s) {
            out.insert(pseudo, ends);
        }
    }
    out
}

/// The reverse lookup (`archi/requirements/planning/`): a requirement matches a
/// task when its satisfied-by expansion against the pinned model
/// ([`Model::term_surface`], the expansion stressor affects share)
/// intersects the task's spec_refs; an edge ref matches through its
/// endpoint nodes. Entries that do not resolve at the pinned version
/// contribute nothing — their claim is against the live model, checked by
/// `archi check`.
fn matched_requirements(
    pinned: &Model,
    tree: &docs::Tree,
    endpoints: &BTreeMap<String, (String, String)>,
    task: &Task,
) -> Vec<MatchedReq> {
    let ref_nodes: Vec<(&str, BTreeSet<&str>)> = task
        .spec_refs
        .iter()
        .map(|r| {
            let nodes: BTreeSet<&str> = match endpoints.get(r) {
                Some((s, d)) => [s.as_str(), d.as_str()].into(),
                None => [r.as_str()].into(),
            };
            (r.as_str(), nodes)
        })
        .collect();
    let mut reqs: Vec<&docs::schema::Requirement> = tree.requirements.iter().collect();
    reqs.sort_by(|a, b| a.slug.cmp(&b.slug));
    reqs.dedup_by(|a, b| a.slug == b.slug); // colliding slugs are E_SLUG, reported by `check`
    let mut out = Vec::new();
    for r in reqs {
        let Some(fields) = &r.fields else { continue };
        let Some((entries, _)) = &fields.satisfied_by else {
            continue;
        };
        let mut surface: BTreeSet<String> = BTreeSet::new();
        for entry in entries {
            if let Some(terms) = pinned.term_surface(entry) {
                surface.extend(terms);
            } else if let Some((s, d)) = endpoints
                .get(entry.as_str())
                .or_else(|| endpoints.get(links::normalize_ref(entry).as_str()))
            {
                // canonical edge text: the claim rides its endpoints
                surface.insert(s.clone());
                surface.insert(d.clone());
            } else if pinned.resolve_element(entry) == Some(ElementKind::Port) {
                // a port names its owning node's interface — fold to the node
                if let Some((node, _)) = entry.rsplit_once('.') {
                    if let Some(terms) = pinned.term_surface(node) {
                        surface.extend(terms);
                    }
                }
            }
        }
        if surface.is_empty() {
            continue;
        }
        let matched_refs: Vec<String> = ref_nodes
            .iter()
            .filter(|(_, nodes)| nodes.iter().any(|n| surface.contains(*n)))
            .map(|(text, _)| (*text).to_string())
            .collect();
        if !matched_refs.is_empty() {
            out.push(MatchedReq {
                slot: format!("r{}", out.len() + 1),
                req: r.slug.clone(),
                matched_refs,
                owned: false,
            });
        }
    }
    out
}

/// Topological layers of the inputs DAG. Unknown input keys are treated as
/// satisfied — they are already reported — so the layering stays total;
/// a genuine cycle errors with its members.
fn layer(tasks: &[Task]) -> Result<Vec<Vec<String>>, String> {
    let ids: BTreeSet<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    let mut placed: BTreeSet<&str> = BTreeSet::new();
    let mut remaining: Vec<&Task> = tasks.iter().collect();
    let mut waves = Vec::new();
    while !remaining.is_empty() {
        let (ready, rest): (Vec<&Task>, Vec<&Task>) = remaining.iter().partition(|t| {
            t.inputs
                .keys()
                .all(|k| placed.contains(k.as_str()) || !ids.contains(k.as_str()))
        });
        if ready.is_empty() {
            let members: Vec<&str> = remaining.iter().map(|t| t.id.as_str()).collect();
            return Err(format!(
                "the inputs of {} form a cycle — inputs are a DAG",
                members.join(", ")
            ));
        }
        placed.extend(ready.iter().map(|t| t.id.as_str()));
        waves.push(ready.into_iter().map(|t| t.id.clone()).collect());
        remaining = rest;
    }
    Ok(waves)
}

// ---- verify ------------------------------------------------------------------

/// The outcome of `archi plan verify`: structural errors gate the
/// lifecycle; notes are advisory, findings-style.
#[derive(Serialize)]
pub struct PlanReport {
    /// The plan's name.
    pub plan: String,
    /// The pinned version.
    pub version: String,
    /// Lifecycle state.
    pub state: PlanState,
    /// Structural errors — `plan start` and `plan next` refuse while any
    /// stand.
    pub errors: Vec<String>,
    /// Advisory: spec drift at Working, a stale pin, doc-layer problems.
    pub notes: Vec<String>,
    /// The derived view.
    #[serde(flatten)]
    pub derived: Derived,
}

/// `archi plan verify`: structural invariants against the pinned version,
/// drift notes against the live model.
pub fn verify(root: &Path, model: &Model) -> Result<PlanReport, String> {
    let plan = load_active(root)?;
    verify_plan(root, model, &plan)
}

/// `archi plan show`: the active plan and its derived view — the authoring
/// read surface.
pub fn show(root: &Path, model: &Model) -> Result<(Plan, PlanReport), String> {
    let plan = load_active(root)?;
    let report = verify_plan(root, model, &plan)?;
    Ok((plan, report))
}

/// `archi plan show <name>`: the same view for a named plan — a pure read
/// that never consults or rewrites `.current`, so any checkout, seated or
/// not, reads any stored plan. An unknown name lists what exists.
pub fn show_named(root: &Path, model: &Model, name: &str) -> Result<(Plan, PlanReport), String> {
    if !plan_path(root, name).exists() && !records::is_record(root, name) {
        let names: Vec<String> = all_plans(root)?.into_iter().map(|p| p.name).collect();
        return Err(if names.is_empty() {
            format!("no plan `{name}` — none exist; `archi plan use <name>` creates one")
        } else {
            format!("no plan `{name}` — plans: {}", names.join(", "))
        });
    }
    let plan = load_plan(root, name)?;
    let report = verify_plan(root, model, &plan)?;
    Ok((plan, report))
}

pub(crate) fn verify_plan(root: &Path, live: &Model, plan: &Plan) -> Result<PlanReport, String> {
    let mut errors = Vec::new();
    let mut notes = Vec::new();
    let ws = compile_pinned(root, &plan.version)?;
    let pinned = ws.model();

    // Latch order — hand edits cannot outrun the verbs.
    if plan.scenarios_closed && !plan.scenarios_displayed {
        errors.push("scenarios_closed without scenarios_displayed — the latches are ordered".into());
    }
    if plan.state == PlanState::Draft && (plan.closed_waves > 0 || plan.scenarios_displayed) {
        errors.push("a draft plan carries lifecycle state — `plan reset` clears it whole".into());
    }

    // Tasks: ids, nodes, spec_refs — all against the pinned version.
    let mut ids: BTreeSet<&str> = BTreeSet::new();
    let mut nodes: BTreeSet<&str> = BTreeSet::new();
    for t in &plan.tasks {
        let well_formed = t
            .id
            .strip_prefix('t')
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
        if !well_formed {
            errors.push(format!("task ids are `t<N>`; got `{}`", t.id));
        }
        if !ids.insert(&t.id) {
            errors.push(format!("task id `{}` is used twice", t.id));
        }
        if !nodes.insert(&t.node) {
            errors.push(format!("two tasks pin `{}` — one task per node", t.node));
        }
        if !pinned.has_node(&t.node) {
            errors.push(format!(
                "{}: `{}` names no element of {} (E_MODEL_REF)",
                t.id, t.node, plan.version
            ));
        }
        if t.description.trim().is_empty() {
            errors.push(format!("{}: empty description — say what the task does", t.id));
        }
        if t.outputs.is_empty() {
            notes.push(format!(
                "{}: no outputs declared — capture cannot attribute its delta; \
                 name the files it will write",
                t.id
            ));
        }
        for r in &t.spec_refs {
            if r.contains('@') {
                errors.push(format!(
                    "{}: `{r}` carries a version — spec_refs are version-free; the plan's pin applies",
                    t.id
                ));
                continue;
            }
            let canonical = links::normalize_ref(r);
            if *r != canonical {
                errors.push(format!(
                    "{}: `{r}` is not canonical — write `{canonical}`",
                    t.id
                ));
                continue;
            }
            let spec = links::SpecRef {
                path: canonical,
                version: None,
            };
            if !links::resolves_in(pinned, &spec) {
                errors.push(format!(
                    "{}: `{r}` names no element of {} (E_MODEL_REF)",
                    t.id, plan.version
                ));
            } else if !links::resolves_in(live, &spec) {
                notes.push(format!(
                    "{}: `{r}` no longer resolves at Working — the spec moved on; \
                     `plan repin` when the plan should follow",
                    t.id
                ));
            }
        }
    }
    for t in &plan.tasks {
        for input in t.inputs.keys() {
            if input == &t.id {
                errors.push(format!("{}: inputs itself", t.id));
            } else if !ids.contains(input.as_str()) {
                errors.push(format!("{}: input `{input}` names no task", t.id));
            }
        }
    }

    // The envelope: every summary node mapped, every mapping summarized.
    let summary: BTreeSet<&str> =
        plan.architecture_summary.iter().map(|s| s.node.as_str()).collect();
    let mapped: BTreeSet<&str> = plan.stack_mapping.iter().map(|m| m.node.as_str()).collect();
    for s in summary.difference(&mapped) {
        errors.push(format!("summary node `{s}` has no stack mapping"));
    }
    for m in mapped.difference(&summary) {
        errors.push(format!(
            "stack mapping realizes `{m}`, which the summary does not name"
        ));
    }

    // Waves.
    let waves = match layer(&plan.tasks) {
        Ok(w) => w,
        Err(e) => {
            errors.push(e);
            Vec::new()
        }
    };
    if !waves.is_empty() && plan.closed_waves > waves.len() {
        errors.push(format!(
            "{} waves closed but the layering has {} — the graph was restructured mid-flight",
            plan.closed_waves,
            waves.len()
        ));
    }

    // Matched requirements — derived; verification keys must match them.
    let (tree, doc_report) = docs::load(root, live);
    if !doc_report.diagnostics.is_empty() {
        notes.push(format!(
            "{} doc diagnostics — the reverse lookup reads a best-effort tree (`archi check`)",
            doc_report.diagnostics.len()
        ));
    }
    let endpoints = edge_endpoints(pinned);
    let mut matched = BTreeMap::new();
    for t in &plan.tasks {
        let mut m = matched_requirements(pinned, &tree, &endpoints, t);
        // Curation: the matched set is the candidate list, `owns` the
        // authored selection — a strict subset, at least one when
        // candidates exist. Verification duty follows ownership.
        for slug in &t.owns {
            if !m.iter().any(|mr| &mr.req == slug) {
                errors.push(format!(
                    "{}: owns `{slug}`, which the reverse lookup does not match — \
                     drop it, or restore the spec_ref that carried it",
                    t.id
                ));
            }
        }
        if t.owns.is_empty() && !m.is_empty() {
            errors.push(format!(
                "{}: {} matched requirement{} and none owned — `owns` in plan.json \
                 selects the ones this task answers for; own at least one",
                t.id,
                m.len(),
                if m.len() == 1 { "" } else { "s" }
            ));
        }
        for mr in &mut m {
            mr.owned = t.owns.contains(&mr.req);
        }
        for slug in t.verifications.keys() {
            if !t.owns.contains(slug) {
                errors.push(format!(
                    "{}: verification for `{slug}`, which the task does not own — \
                     own it, or drop the stale key",
                    t.id
                ));
            }
        }
        for slug in &t.owns {
            if m.iter().any(|mr| &mr.req == slug)
                && t.verifications.get(slug).is_none_or(Vec::is_empty)
            {
                errors.push(format!(
                    "{}: owned `{slug}` has no verification — author the observable check",
                    t.id
                ));
            }
        }
        matched.insert(t.id.clone(), m);
    }

    // A requirement no task reaches is visible, never silent — outside the
    // plan's scope, or a missing task or spec_ref; the reader decides which.
    if !plan.tasks.is_empty() {
        let reached: BTreeSet<&str> =
            matched.values().flatten().map(|m| m.req.as_str()).collect();
        for r in &tree.requirements {
            let Some(f) = &r.fields else { continue };
            let Some((entries, _)) = &f.satisfied_by else { continue };
            if entries.is_empty() || f.deferred() || reached.contains(r.slug.as_str()) {
                continue;
            }
            notes.push(format!(
                "requirement `{}` reaches no task — outside this plan's scope, or a \
                 missing task or spec_ref",
                r.slug
            ));
        }
    }

    // A stale pin is advisory: the plan may finish against the version it
    // planned for.
    if let Some(archive) = versions::Archive::open(root)?
        && let Some(latest) = archive.entries().last()
        && latest.id != plan.version
    {
        notes.push(format!(
            "the spec advanced: pinned {}, latest {} — `plan repin` when the plan should follow",
            plan.version, latest.id
        ));
    }

    Ok(PlanReport {
        plan: plan.name.clone(),
        version: plan.version.clone(),
        state: plan.state,
        errors,
        notes,
        derived: Derived { matched, waves },
    })
}

// ---- lifecycle ---------------------------------------------------------------

fn gate_structure(report: &PlanReport) -> Result<(), String> {
    if report.errors.is_empty() {
        return Ok(());
    }
    Err(format!(
        "the plan is structurally broken:\n  {}",
        report.errors.join("\n  ")
    ))
}


/// Asserted code-link coverage, scoped to the delta: a ref gates only when
/// the closing capture pressed it — some claimed changed item of its task
/// carries the ref's terms. Unpressed refs never block; the uncovered ones
/// come back as suggested `link add` lines, since hand-authoring is the
/// expected move for surface the delta did not touch. Evidence never
/// gates, and an asserted link satisfies its ref however it was born.
fn gate_coverage(
    root: &Path,
    in_flight: &[&Task],
    pressed: &BTreeMap<String, BTreeSet<String>>,
) -> Result<Vec<String>, String> {
    let live = links::ls(root, None, false)?;
    let covered = |r: &str| {
        live.iter().any(|l| {
            l.standing == links::Standing::Asserted
                && l.spec.version.is_none()
                && l.spec.path == r
        })
    };
    let mut gaps = Vec::new();
    let mut suggested = Vec::new();
    for t in in_flight {
        for r in &t.spec_refs {
            if covered(r) {
                continue;
            }
            if pressed.get(&t.id).is_some_and(|p| p.contains(r)) {
                gaps.push(format!("{}: `{r}`", t.id));
            } else {
                suggested.push(format!(
                    "archi link add \"{r}\" <file#symbol> --kind indirect  # {}",
                    t.id
                ));
            }
        }
    }
    if gaps.is_empty() {
        return Ok(suggested);
    }
    let mut msg = format!(
        "asserted code-link coverage of the refs this delta presses is incomplete:\n  {}\n\
         review the captured candidates (`archi link ls --evidence`), assert the load-bearing \
         ones (`archi link confirm <id>`), then re-run `archi plan next`",
        gaps.join("\n  ")
    );
    if !suggested.is_empty() {
        msg.push_str(&format!(
            "\nrefs the delta does not press are not demanded — hand-author them when the \
             traceability is wanted:\n  {}",
            suggested.join("\n  ")
        ));
    }
    Err(msg)
}

/// `archi plan start`: gate on structure and verifications, open wave 1.
/// Returns the plan and the wave-1 task ids now in flight.
pub fn start(root: &Path, model: &Model) -> Result<(Plan, Vec<String>), String> {
    let mut plan = load_active(root)?;
    if plan.state != PlanState::Draft {
        return Err(format!(
            "the plan is {} — `plan reset` rewinds it",
            plan.state.describe()
        ));
    }
    if plan.tasks.is_empty() {
        return Err("the plan has no tasks: `archi plan task add <node>`".into());
    }
    let report = verify_plan(root, model, &plan)?;
    gate_structure(&report)?;
    links::capture::write_index(root, &plan.name, 1)?;
    plan.state = PlanState::Started;
    plan.closed_waves = 0;
    save_state(root, &plan)?;
    Ok((plan, report.derived.waves[0].clone()))
}

/// What one `archi plan next` did.
pub enum Step {
    /// The coverage gate refused; the capture above it already ran.
    Blocked(String),
    /// A wave closed and the next opened.
    Wave {
        /// The wave just closed, 1-based.
        closed: usize,
        /// The task ids now in flight.
        next_tasks: Vec<String>,
    },
    /// All waves closed: the scenarios block, printed as the final step.
    Scenarios(Vec<String>),
    /// The plan completed.
    Done,
}

/// The outcome of `archi plan next`: what capture produced (when a wave
/// was closing), then the step taken.
pub struct NextOutcome {
    /// The capture that ran before the gates, if one did.
    pub capture: Option<links::capture::CaptureOutcome>,
    /// The step.
    pub step: Step,
    /// Suggested `link add` lines for uncovered refs the closing delta did
    /// not press — the voluntary checklist; empty when the gate blocked
    /// (the message carries it) or nothing was left to suggest.
    pub checklist: Vec<String>,
}

/// `archi plan next`: capture the closing wave's deltas into candidate
/// links, then advance under the structural and coverage gates — the step
/// that demands links is the step that produces them, and it is
/// re-runnable: review (`link confirm`), then run it again.
pub fn next(root: &Path, model: &Model) -> Result<NextOutcome, String> {
    let mut plan = load_active(root)?;
    match plan.state {
        PlanState::Draft => {
            return Err("the plan is draft — `plan start` opens the first wave".into());
        }
        PlanState::Completed => {
            return Err("the plan is completed — `plan reset` runs it again".into());
        }
        PlanState::Started => {}
    }
    let report = verify_plan(root, model, &plan)?;
    let waves = &report.derived.waves;

    // Past the last wave: the scenario latch dance.
    if plan.closed_waves >= waves.len() {
        if plan.scenarios_displayed && !plan.scenarios_closed {
            plan.scenarios_closed = true;
            plan.state = PlanState::Completed;
            save_state(root, &plan)?;
            return Ok(NextOutcome {
                capture: None,
                step: Step::Done,
                checklist: Vec::new(),
            });
        }
        return Err("nothing in flight and no scenario step pending — `archi plan verify`".into());
    }

    gate_structure(&report)?;
    let wave = plan.closed_waves + 1;
    let in_flight: Vec<&Task> = plan
        .tasks
        .iter()
        .filter(|t| waves[wave - 1].contains(&t.id))
        .collect();
    let capture = links::capture::capture_wave(root, &plan.name, wave, &in_flight, None)?;
    let checklist = match gate_coverage(root, &in_flight, &capture.pressed) {
        Ok(suggested) => suggested,
        Err(gaps) => {
            return Ok(NextOutcome {
                capture: Some(capture),
                step: Step::Blocked(gaps),
                checklist: Vec::new(),
            });
        }
    };

    plan.closed_waves = wave;
    let step = if plan.closed_waves < waves.len() {
        links::capture::write_index(root, &plan.name, wave + 1)?;
        Step::Wave {
            closed: wave,
            next_tasks: waves[wave].clone(),
        }
    } else if plan.scenarios.is_empty() {
        // No scenarios recorded: skip the step and close the plan.
        plan.state = PlanState::Completed;
        Step::Done
    } else {
        plan.scenarios_displayed = true;
        Step::Scenarios(plan.scenarios.clone())
    };
    save_state(root, &plan)?;
    Ok(NextOutcome {
        capture: Some(capture),
        step,
        checklist,
    })
}

/// What `archi plan current-wave` reports.
pub enum InFlight {
    /// The 1-based wave and its task ids.
    Wave(usize, Vec<String>),
    /// All waves closed; the scenario step is pending.
    ScenarioStep,
}

/// `archi plan current-wave`: the tasks in flight.
pub fn current_wave(root: &Path, model: &Model) -> Result<(Plan, InFlight), String> {
    let plan = load_active(root)?;
    if plan.state != PlanState::Started {
        return Err(format!("the plan is {}", plan.state.describe()));
    }
    let report = verify_plan(root, model, &plan)?;
    let waves = report.derived.waves;
    if plan.closed_waves >= waves.len() {
        return Ok((plan, InFlight::ScenarioStep));
    }
    let ids = waves[plan.closed_waves].clone();
    let wave = plan.closed_waves + 1;
    Ok((plan, InFlight::Wave(wave, ids)))
}

/// `archi plan close`: manual override to Completed — no gates.
pub fn close(root: &Path) -> Result<Plan, String> {
    let mut plan = load_active(root)?;
    if plan.state == PlanState::Completed {
        return Err(format!("plan `{}` is already completed", plan.name));
    }
    plan.state = PlanState::Completed;
    save_state(root, &plan)?;
    Ok(plan)
}

/// `archi plan reset`: back to draft — waves rewound, latches unlatched,
/// wave indexes removed. Journaled links stay: the journal is append-only.
pub fn reset(root: &Path) -> Result<Plan, String> {
    let mut plan = load_active(root)?;
    plan.state = PlanState::Draft;
    plan.closed_waves = 0;
    plan.scenarios_displayed = false;
    plan.scenarios_closed = false;
    let waves_dir = plan_dir(root, &plan.name).join("waves");
    if waves_dir.exists() {
        fs::remove_dir_all(&waves_dir)
            .map_err(|e| format!("cannot remove `{}`: {e}", waves_dir.display()))?;
    }
    save_state(root, &plan)?;
    Ok(plan)
}

// ---- rendering ---------------------------------------------------------------

/// The verify report as human lines: errors, notes, the tally.
pub fn render_report(report: &PlanReport) -> String {
    let mut out = format!(
        "plan `{}` @ {} ({})\n",
        report.plan,
        report.version,
        report.state.describe()
    );
    for e in &report.errors {
        out.push_str(&format!("error: {e}\n"));
    }
    for n in &report.notes {
        out.push_str(&format!("note: {n}\n"));
    }
    let reqs: usize = report.derived.matched.values().map(Vec::len).sum();
    out.push_str(&format!(
        "{} tasks in {} waves, {} matched requirements: {}\n",
        report.derived.matched.len(),
        report.derived.waves.len(),
        reqs,
        if report.errors.is_empty() {
            "structurally clean"
        } else {
            "structurally broken"
        }
    ));
    out
}

/// The plan with its derived view as human lines — the authoring read
/// surface.
pub fn render_show(plan: &Plan, report: &PlanReport) -> String {
    let mut out = format!(
        "plan `{}` @ {} ({}), created {}\n",
        plan.name,
        plan.version,
        plan.state.describe(),
        plan.created
    );
    if !plan.problem.is_empty() {
        out.push_str(&format!("problem: {}\n", plan.problem));
    }
    for t in &plan.technology_stack {
        let provenance = if t.provenance.is_empty() {
            String::new()
        } else {
            format!(" — {}", t.provenance)
        };
        out.push_str(&format!("stack: {}{provenance}\n", t.tech));
    }
    for s in &plan.architecture_summary {
        out.push_str(&format!("summary: {} — {}\n", s.node, s.role));
    }
    for m in &plan.stack_mapping {
        out.push_str(&format!("mapping: {} realizes {}\n", m.tech, m.node));
    }
    for (i, wave) in report.derived.waves.iter().enumerate() {
        let in_flight = plan.state == PlanState::Started && i == plan.closed_waves;
        out.push_str(&format!(
            "wave {}{}:\n",
            i + 1,
            if in_flight { " (in flight)" } else { "" }
        ));
        for id in wave {
            let Some(task) = plan.tasks.iter().find(|t| &t.id == id) else {
                continue;
            };
            out.push_str(&format!("  {} {} — {}\n", task.id, task.node, task.description));
            out.push_str(&format!("    spec_refs: {}\n", task.spec_refs.join(", ")));
            for m in report.derived.matched.get(id).into_iter().flatten() {
                let proofs = task.verifications.get(&m.req).map_or(0, Vec::len);
                out.push_str(&format!(
                    "    {} {}{} (via {}) — {} verification{}\n",
                    m.slot,
                    m.req,
                    if m.owned { "" } else { " (unowned)" },
                    m.matched_refs.join(", "),
                    proofs,
                    if proofs == 1 { "" } else { "s" }
                ));
            }
        }
    }
    for s in &plan.scenarios {
        out.push_str(&format!("scenario: {s}\n"));
    }
    for e in &report.errors {
        out.push_str(&format!("error: {e}\n"));
    }
    for n in &report.notes {
        out.push_str(&format!("note: {n}\n"));
    }
    out
}

/// The standalone brief `archi plan task show` renders: everything a
/// sub-agent needs, no implicit context.
pub fn render_task_show(plan: &Plan, report: &PlanReport, id: &str) -> Result<String, String> {
    let Some(t) = plan.tasks.iter().find(|t| t.id == id) else {
        return Err(format!("no task `{id}` — `archi plan show` lists them"));
    };
    let mut out = format!("{} {} — {}\n", t.id, t.node, t.description);
    out.push_str(&format!("pinned: {}\n", plan.version));
    out.push_str(&format!("spec_refs: {}\n", t.spec_refs.join(", ")));
    if !t.stack_details.is_empty() {
        out.push_str(&format!("stack: {}\n", t.stack_details.replace('\n', "; ")));
    }
    for (from, note) in &t.inputs {
        out.push_str(&format!("input ← {from}: {note}\n"));
    }
    for o in &t.outputs {
        out.push_str(&format!("output: {o}\n"));
    }
    for m in report.derived.matched.get(id).into_iter().flatten() {
        let owned = if m.owned { "owned" } else { "unowned" };
        out.push_str(&format!(
            "{} {} ({owned}, via {})\n",
            m.slot,
            m.req,
            m.matched_refs.join(", ")
        ));
        for v in t.verifications.get(&m.req).into_iter().flatten() {
            out.push_str(&format!("  verify: {v}\n"));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    const MODEL: &str = "def conn wire := * -> *\n\
                         def rel peer := * <-> *\n\
                         def node Gate:\n  port out\n\
                         def node Auth:\n  port inn\n  port creds\n\
                         def node Store:\n  port inn\n\
                         def node Audit:\n  port inn\n\
                         Gate.out wire Auth.inn\n\
                         Auth.creds wire Store.inn\n\
                         Auth peer Audit\n\
                         Service type_of Auth\n";

    fn temp_project() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-plans-test-{}-{}",
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

    fn compiled(root: &Path) -> Workspace {
        modeling_lang::source::compile_project(root)
            .unwrap_or_else(|f| panic!("test model failed to compile:\n{}", f.render()))
            .workspace
    }

    fn save_version(root: &Path) -> String {
        let ws = compiled(root);
        match versions::save(root, ws.model(), "planned").unwrap() {
            versions::Saved::Written { id, .. } => id,
            versions::Saved::Unchanged { latest } => latest,
        }
    }

    fn requirement(satisfied_by: &str, name: &str) -> String {
        format!(
            "---\nkind: functional\norigin: intent\nsatisfied-by: [{satisfied_by}]\ndeferred:\n---\n\n\
             # {name}\n\nSummary paragraph.\n\n## System Context\n\n## Satisfy\n{}",
            if satisfied_by.is_empty() {
                "\n".to_string()
            } else {
                "\nProse claim.\n\n- test — proof sketch\n".to_string()
            }
        )
    }

    /// An intent with requirements shaped to exercise every matching rule:
    /// a term hit, a type expansion, an edge-endpoint hit, an open req.
    fn put_requirements(root: &Path) {
        put(
            root,
            "archi/requirements/hardening/hardening.md",
            "# Hardening\n\nThe area.\n",
        );
        put(
            root,
            "archi/requirements/hardening/store-encrypted.md",
            &requirement("Store", "Store encrypted"),
        );
        put(
            root,
            "archi/requirements/hardening/service-hardening.md",
            &requirement("Service", "Service hardening"),
        );
        put(
            root,
            "archi/requirements/hardening/gate-throughput.md",
            &requirement("Gate", "Gate throughput"),
        );
        put(
            root,
            "archi/requirements/hardening/open-req.md",
            &requirement("", "Open req"),
        );
    }

    fn active(root: &Path) -> Plan {
        load_active(root).unwrap()
    }

    /// The record form's "full save": author by rewriting the files —
    /// exactly what a human does between mints. Lifecycle is not
    /// touched; state.json stays the verbs' alone.
    fn store_authored(root: &Path, plan: &Plan) {
        let dir = plan_dir(root, &plan.name);
        fs::write(
            dir.join(format!("{}.md", plan.name)),
            records::render_charter(plan),
        )
        .unwrap();
        fs::write(dir.join("scenarios.md"), records::render_scenarios(&plan.scenarios)).unwrap();
        for t in &plan.tasks {
            fs::write(dir.join(records::task_file_name(t)), records::render_task(t)).unwrap();
        }
    }

    /// Author the curation whole: descriptions, own every matched
    /// requirement, one verification per owned — the old own-everything
    /// behavior, spelled out into the task files.
    fn curate_all(root: &Path, model: &Model) {
        let mut plan = active(root);
        let report = verify_plan(root, model, &plan).unwrap();
        for t in &mut plan.tasks {
            if t.description.trim().is_empty() {
                t.description = format!("realize {}", t.node);
            }
            let matched = report.derived.matched.get(&t.id).into_iter().flatten();
            t.owns = matched.clone().map(|m| m.req.clone()).collect();
            for m in matched {
                t.verifications
                    .entry(m.req.clone())
                    .or_insert_with(|| vec![format!("test — proves {}", m.req)]);
            }
        }
        store_authored(root, &plan);
    }

    #[test]
    fn use_pins_a_hardened_version_and_switches() {
        let root = temp_project();
        let ws = compiled(&root);

        // No versions: nothing to pin.
        let err = use_plan(&root, ws.model(), "mvp").unwrap_err();
        assert!(err.contains("version save"), "{err}");

        let v1 = save_version(&root);
        assert!(matches!(
            use_plan(&root, ws.model(), "mvp").unwrap(),
            Used::Created(p) if p.version == v1 && p.state == PlanState::Draft
        ));
        assert_eq!(active_name(&root).unwrap().as_deref(), Some("mvp"));

        // The mint is the record folder: charter and scenarios skeletons
        // plus state.json — no plan.json is ever born again.
        let dir = plan_dir(&root, "mvp");
        assert!(dir.join("mvp.md").exists());
        assert!(dir.join("scenarios.md").exists());
        assert!(dir.join("state.json").exists());
        assert!(!dir.join("plan.json").exists());

        // Both forms at once refuse loudly, naming the choice.
        fs::write(dir.join("plan.json"), "{}").unwrap();
        let err = load_plan(&root, "mvp").unwrap_err();
        assert!(err.contains("both plan.json and the record folder"), "{err}");
        fs::remove_file(dir.join("plan.json")).unwrap();

        // A dirty model refuses to mint a new plan, but switching to an
        // existing one is free.
        fs::write(
            root.join("archi/src/model.arch"),
            format!("{MODEL}def node Extra\n"),
        )
        .unwrap();
        let ws2 = compiled(&root);
        let err = use_plan(&root, ws2.model(), "next").unwrap_err();
        assert!(err.contains("unsaved changes"), "{err}");
        assert!(matches!(
            use_plan(&root, ws2.model(), "mvp").unwrap(),
            Used::Switched(_)
        ));

        // Saving the change permits the second plan; repin moves the first.
        let v2 = save_version(&root);
        assert!(matches!(
            use_plan(&root, ws2.model(), "next").unwrap(),
            Used::Created(p) if p.version == v2
        ));
        use_plan(&root, ws2.model(), "mvp").unwrap();
        let (repinned, from) = repin(&root, ws2.model()).unwrap();
        assert_eq!((from.as_str(), repinned.version.as_str()), (v1.as_str(), v2.as_str()));
        assert!(repin(&root, ws2.model()).is_err(), "already pinned");

        // Names are slugs.
        assert!(use_plan(&root, ws2.model(), "Not A Slug").is_err());

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn task_add_seeds_the_node_and_its_incoming_edges() {
        let root = temp_project();
        let ws = compiled(&root);
        save_version(&root);
        use_plan(&root, ws.model(), "mvp").unwrap();

        let store = task_add(&root, "Store", Some("persist creds")).unwrap();
        assert_eq!(store.id, "t1");
        assert_eq!(
            store.spec_refs,
            vec!["Store".to_string(), "Auth.creds wire Store.inn".to_string()]
        );

        // Auth pulls its incoming wire, the undirected peer crossing, and
        // its classification — outgoing edges stay with their targets.
        let auth = task_add(&root, "Auth", None).unwrap();
        let mut refs = auth.spec_refs.clone();
        refs.sort();
        assert_eq!(
            refs,
            vec![
                "Auth".to_string(),
                "Auth peer Audit".to_string(),
                "Gate.out wire Auth.inn".to_string(),
                "Service type_of Auth".to_string(),
            ]
        );

        // The mint is a file; a byte-equal re-mint converges, a request
        // that would write different bytes refuses — the file is the
        // author's the moment it differs from the skeleton.
        assert!(plan_dir(&root, "mvp").join("t2-auth.md").exists());
        let again = task_add(&root, "Auth", None).unwrap();
        assert_eq!(again.id, "t2", "byte-equal re-mint converges");
        let err = task_add(&root, "Store", None).unwrap_err();
        assert!(err.contains("moved past its skeleton"), "{err}");

        // An edited file refuses the re-mint even with the same request.
        let file = plan_dir(&root, "mvp").join("t2-auth.md");
        let text = fs::read_to_string(&file)
            .unwrap()
            .replace("# t2 — Auth", "# t2 — Auth\n\nnow described");
        fs::write(&file, text).unwrap();
        let err = task_add(&root, "Auth", None).unwrap_err();
        assert!(err.contains("moved past its skeleton"), "{err}");

        // Nodes resolve at the pinned version.
        let err = task_add(&root, "Nope", None).unwrap_err();
        assert!(err.contains("E_MODEL_REF"), "{err}");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn task_rm_unmints_draft_files_and_guards_dependents() {
        let root = temp_project();
        let ws = compiled(&root);
        save_version(&root);
        use_plan(&root, ws.model(), "mvp").unwrap();
        task_add(&root, "Store", None).unwrap();
        task_add(&root, "Auth", None).unwrap();

        // t2 inputs t1: the producer is held in place, the error lists
        // who depends on it.
        let mut plan = active(&root);
        plan.tasks[1].inputs.insert("t1".into(), "the store api".into());
        store_authored(&root, &plan);
        let err = task_rm(&root, "t1").unwrap_err();
        assert!(err.contains("feeds t2"), "{err}");
        let err = task_rm(&root, "t9").unwrap_err();
        assert!(err.contains("no task `t9`"), "{err}");

        // The consumer removes — file gone, plan smaller, and the next
        // mint counts past the highest id present.
        let gone = task_rm(&root, "t2").unwrap();
        assert!(!gone.exists());
        assert_eq!(active(&root).tasks.len(), 1);
        assert_eq!(task_add(&root, "Auth", None).unwrap().id, "t2");

        // Past draft the structure is frozen.
        close(&root).unwrap();
        let err = task_rm(&root, "t1").unwrap_err();
        assert!(err.contains("past draft"), "{err}");
        assert!(err.contains("plan reset"), "{err}");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_reverse_lookup_matches_terms_types_and_edge_endpoints() {
        let root = temp_project();
        let ws = compiled(&root);
        save_version(&root);
        put_requirements(&root);
        use_plan(&root, ws.model(), "mvp").unwrap();
        task_add(&root, "Store", None).unwrap();
        task_add(&root, "Auth", None).unwrap();
        curate_all(&root, ws.model());

        let report = verify(&root, ws.model()).unwrap();
        assert_eq!(report.errors, Vec::<String>::new());

        // Store: its own node ref pulls store-encrypted — through the node
        // ref and again through the edge's Store endpoint — and the
        // incoming edge's far endpoint (Auth, classified `Service`) pulls
        // service-hardening across the boundary: the wire is Store's
        // obligation, so the requirements pressing on it ride along.
        let store = &report.derived.matched["t1"];
        let pairs: Vec<(&str, &str)> = store
            .iter()
            .map(|m| (m.slot.as_str(), m.req.as_str()))
            .collect();
        assert_eq!(
            pairs,
            vec![("r1", "service-hardening"), ("r2", "store-encrypted")]
        );
        assert_eq!(
            store[0].matched_refs,
            vec!["Auth.creds wire Store.inn".to_string()]
        );
        assert_eq!(
            store[1].matched_refs,
            vec!["Store".to_string(), "Auth.creds wire Store.inn".to_string()]
        );

        // Auth: `Service` expands through type_of to Auth (term surface),
        // and gate-throughput arrives through the incoming edge's Gate
        // endpoint. Slots follow slug order; the open requirement (empty
        // satisfied-by) matches nothing.
        let auth = &report.derived.matched["t2"];
        let pairs: Vec<(&str, &str)> = auth
            .iter()
            .map(|m| (m.slot.as_str(), m.req.as_str()))
            .collect();
        assert_eq!(
            pairs,
            vec![("r1", "gate-throughput"), ("r2", "service-hardening")]
        );
        assert_eq!(
            auth[0].matched_refs,
            vec!["Gate.out wire Auth.inn".to_string()]
        );
        assert!(auth[1].matched_refs.contains(&"Auth".to_string()));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_lifecycle_captures_gates_and_latches_scenarios() {
        let root = temp_project();
        let ws = compiled(&root);
        save_version(&root);
        put_requirements(&root);
        put(
            &root,
            "code/store.rs",
            "pub struct Store;\nimpl Store {\n    pub fn put(&mut self) {}\n}\n",
        );
        put(&root, "code/auth.rs", "pub fn login() -> bool { true }\n");
        use_plan(&root, ws.model(), "mvp").unwrap();
        task_add(&root, "Store", None).unwrap();
        task_add(&root, "Auth", None).unwrap();

        // Author the plan by rewriting its files: outputs, an input edge
        // t1 → t2, a scenario.
        let mut plan = active(&root);
        plan.scenarios.push("a user logs in end to end".into());
        plan.tasks[0].outputs.push("code/store.rs".into());
        plan.tasks[1].outputs.push("code/auth.rs".into());
        plan.tasks[1].inputs.insert("t1".into(), "the store api".into());
        store_authored(&root, &plan);

        // The start gate: unowned matches and empty descriptions refuse —
        // curation is authored before the plan runs.
        let err = start(&root, ws.model()).unwrap_err();
        assert!(err.contains("none owned"), "{err}");
        assert!(err.contains("empty description"), "{err}");
        curate_all(&root, ws.model());
        let (started, wave1) = start(&root, ws.model()).unwrap();
        assert_eq!(started.state, PlanState::Started);
        assert_eq!(wave1, vec!["t1".to_string()]);
        assert!(plan_dir(&root, "mvp").join("waves/w01.index.json").exists());
        assert!(start(&root, ws.model()).is_err(), "already started");

        // Closing wave 1: the claimed delta becomes candidates — 2 refs ×
        // 1 changed symbol — and the coverage gate blocks until asserted.
        put(
            &root,
            "code/store.rs",
            "pub struct Store;\nimpl Store {\n    pub fn put(&mut self, n: u8) { let _ = n; }\n}\n",
        );
        let outcome = next(&root, ws.model()).unwrap();
        let Step::Blocked(why) = &outcome.step else {
            panic!("the coverage gate should block");
        };
        assert!(why.contains("t1"), "{why}");
        let capture = outcome.capture.expect("capture ran");
        assert_eq!(
            capture.minted.len(),
            2,
            "{}",
            links::capture::render_capture(&capture)
        );

        // The loop the spec promises: confirm, re-run — idempotent capture,
        // gate passes, wave 2 opens.
        for l in &capture.minted {
            links::confirm(&root, &l.id).unwrap();
        }
        let outcome = next(&root, ws.model()).unwrap();
        assert!(outcome.capture.as_ref().is_some_and(|c| c.minted.is_empty()));
        let Step::Wave { closed, next_tasks } = &outcome.step else {
            panic!("the gate should pass now");
        };
        assert_eq!((*closed, next_tasks.clone()), (1, vec!["t2".to_string()]));
        let (_, in_flight) = current_wave(&root, ws.model()).unwrap();
        assert!(matches!(in_flight, InFlight::Wave(2, ids) if ids == vec!["t2".to_string()]));

        // Wave 2's delta shares no term with any of t2's refs: nothing is
        // pressed, so nothing gates — the wave closes straight through,
        // the no-signal product suppressed and the uncovered surface
        // suggested for hand-authoring. A pre-asserted ref is silent in
        // the checklist: covered is covered, however the link was born.
        links::add(&root, ws.model(), "Auth", "code/auth.rs", links::LinkKind::Indirect).unwrap();
        put(
            &root,
            "code/auth.rs",
            "pub fn login(u: &str) -> bool { !u.is_empty() }\n",
        );
        let outcome = next(&root, ws.model()).unwrap();
        let capture = outcome.capture.expect("capture ran");
        assert!(
            capture.minted.is_empty(),
            "{}",
            links::capture::render_capture(&capture)
        );
        assert_eq!(capture.suppressed.len(), 4, "every pair lacks signal");
        assert!(capture.pressed.is_empty(), "{:?}", capture.pressed);
        let Step::Scenarios(scenarios) = &outcome.step else {
            panic!("the unpressed wave closes to the scenario step");
        };
        assert_eq!(scenarios, &vec!["a user logs in end to end".to_string()]);
        assert!(active(&root).scenarios_displayed);
        assert_eq!(outcome.checklist.len(), 3, "{:?}", outcome.checklist);
        assert!(
            outcome
                .checklist
                .iter()
                .all(|s| s.contains("archi link add") && s.ends_with("# t2")),
            "{:?}",
            outcome.checklist
        );
        assert!(
            !outcome.checklist.iter().any(|s| s.contains("\"Auth\"")),
            "the asserted ref is silent: {:?}",
            outcome.checklist
        );
        assert!(outcome.checklist.iter().any(|s| s.contains("Auth peer Audit")));

        // One more next completes; a completed plan refuses.
        let outcome = next(&root, ws.model()).unwrap();
        assert!(matches!(outcome.step, Step::Done));
        assert_eq!(active(&root).state, PlanState::Completed);
        assert!(next(&root, ws.model()).is_err());

        // Reset rewinds whole; without scenarios the waves sail through on
        // the standing asserted links and the plan closes directly.
        let plan = reset(&root).unwrap();
        assert_eq!((plan.state, plan.closed_waves), (PlanState::Draft, 0));
        assert!(!plan.scenarios_displayed && !plan.scenarios_closed);
        assert!(!plan_dir(&root, "mvp").join("waves").exists());
        let mut plan = active(&root);
        plan.scenarios.clear();
        store_authored(&root, &plan);
        start(&root, ws.model()).unwrap();
        let outcome = next(&root, ws.model()).unwrap();
        assert!(matches!(outcome.step, Step::Wave { closed: 1, .. }));
        let outcome = next(&root, ws.model()).unwrap();
        assert!(matches!(outcome.step, Step::Done));
        assert_eq!(active(&root).state, PlanState::Completed);

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_gate_presses_the_delta_and_suggests_the_rest() {
        let root = temp_project();
        let ws = compiled(&root);
        save_version(&root);
        put_requirements(&root);
        put(&root, "code/auth.rs", "pub fn login() -> bool { true }\n");
        use_plan(&root, ws.model(), "mvp").unwrap();
        task_add(&root, "Auth", None).unwrap();

        let mut plan = active(&root);
        plan.tasks[0].outputs.push("code/auth.rs".into());
        store_authored(&root, &plan);
        curate_all(&root, ws.model());
        start(&root, ws.model()).unwrap();

        // The delta names the incoming wire's ports but never the node:
        // exactly one ref is pressed and gates; the rest arrive inside the
        // blocked message as suggestions, not gaps.
        put(
            &root,
            "code/auth.rs",
            "pub fn login() -> bool { true }\npub fn inn_wire_probe() -> bool { true }\n",
        );
        let outcome = next(&root, ws.model()).unwrap();
        let Step::Blocked(why) = &outcome.step else {
            panic!("the pressed ref gates");
        };
        assert!(why.contains("coverage of the refs this delta presses"), "{why}");
        assert!(why.contains("t1: `Gate.out wire Auth.inn`"), "{why}");
        assert!(!why.contains("t1: `Auth`"), "unpressed refs never gap: {why}");
        assert!(why.contains("hand-author"), "{why}");
        assert!(why.contains("archi link add \"Auth\""), "{why}");
        let capture = outcome.capture.expect("capture ran");
        let minted: Vec<&str> = capture.minted.iter().map(|l| l.spec.path.as_str()).collect();
        assert_eq!(minted, vec!["Gate.out wire Auth.inn"], "{minted:?}");
        assert_eq!(capture.suppressed.len(), 3, "{:?}", capture.suppressed);

        // Confirm the pressed candidate: the wave closes and the same
        // suggestions ride the passing step as the voluntary checklist.
        links::confirm(&root, &capture.minted[0].id).unwrap();
        let outcome = next(&root, ws.model()).unwrap();
        assert!(matches!(outcome.step, Step::Done));
        assert_eq!(outcome.checklist.len(), 3, "{:?}", outcome.checklist);
        assert!(
            outcome
                .checklist
                .iter()
                .any(|s| s.contains("archi link add \"Auth\" <file#symbol> --kind indirect")),
            "{:?}",
            outcome.checklist
        );

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn curation_selects_from_matched_and_scopes_the_verification_duty() {
        let root = temp_project();
        let ws = compiled(&root);
        save_version(&root);
        put_requirements(&root);
        use_plan(&root, ws.model(), "mvp").unwrap();
        task_add(&root, "Store", Some("keep the creds")).unwrap();

        // Own one of the two matches: the unowned one demands nothing.
        let mut plan = active(&root);
        plan.tasks[0].owns = vec!["store-encrypted".into()];
        plan.tasks[0]
            .verifications
            .insert("store-encrypted".into(), vec!["test — encrypted at rest".into()]);
        store_authored(&root, &plan);
        let report = verify(&root, ws.model()).unwrap();
        assert_eq!(report.errors, Vec::<String>::new(), "{:?}", report.errors);
        let flags: Vec<(String, bool)> = report.derived.matched["t1"]
            .iter()
            .map(|m| (m.req.clone(), m.owned))
            .collect();
        assert_eq!(
            flags,
            vec![
                ("service-hardening".to_string(), false),
                ("store-encrypted".to_string(), true)
            ]
        );
        // The show surface marks the unowned candidate; the missing
        // outputs ride as a note, never an error.
        let rendered = render_show(&active(&root), &report);
        assert!(rendered.contains("service-hardening (unowned)"), "{rendered}");
        assert!(!rendered.contains("store-encrypted (unowned)"), "{rendered}");
        assert!(report.notes.join("\n").contains("no outputs declared"), "{:?}", report.notes);

        // The envelope cross-check: a summary node without a mapping and a
        // mapping outside the summary both refuse.
        let mut plan = active(&root);
        plan.architecture_summary.push(SummaryLine {
            node: "Store".into(),
            role: "keeps the rows".into(),
        });
        plan.stack_mapping.push(StackMapping { tech: "sqlite".into(), node: "Auth".into() });
        store_authored(&root, &plan);
        let report = verify(&root, ws.model()).unwrap();
        let all = report.errors.join("\n");
        assert!(all.contains("summary node `Store` has no stack mapping"), "{all}");
        assert!(all.contains("stack mapping realizes `Auth`"), "{all}");
        let mut plan = active(&root);
        plan.architecture_summary.clear();
        plan.stack_mapping.clear();
        store_authored(&root, &plan);

        // Owning a requirement the lookup never matched is a lie —
        // verify refuses it as an error, since owns is hand-edited.
        let mut plan = active(&root);
        plan.tasks[0].owns.push("ghost-req".into());
        store_authored(&root, &plan);
        let report = verify(&root, ws.model()).unwrap();
        let all = report.errors.join("\n");
        assert!(all.contains("owns `ghost-req`"), "{all}");

        // A verification under an unowned slug never loads at all — the
        // record form catches the stale key before verify runs.
        let file = plan_dir(&root, "mvp").join("t1-store.md");
        let text = fs::read_to_string(&file).unwrap()
            + "\n### service-hardening\n\n- test — hardened\n";
        fs::write(&file, text).unwrap();
        let err = verify(&root, ws.model()).err().unwrap();
        assert!(err.contains("`### service-hardening`"), "{err}");
        assert!(err.contains("own it first"), "{err}");
        assert!(err.contains("t1-store.md"), "{err}");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn port_and_edge_pins_fold_to_their_nodes_and_orphans_surface() {
        let root = temp_project();
        let ws = compiled(&root);
        save_version(&root);
        put(
            &root,
            "archi/requirements/hardening/hardening.md",
            "# Hardening\n\nThe area.\n",
        );
        put(
            &root,
            "archi/requirements/hardening/port-pinned.md",
            &requirement("Store.inn", "Port pinned"),
        );
        put(
            &root,
            "archi/requirements/hardening/edge-pinned.md",
            &requirement("Auth.creds wire Store.inn", "Edge pinned"),
        );
        put(
            &root,
            "archi/requirements/hardening/gate-throughput.md",
            &requirement("Gate", "Gate throughput"),
        );
        use_plan(&root, ws.model(), "mvp").unwrap();
        task_add(&root, "Store", Some("persist rows")).unwrap();
        curate_all(&root, ws.model());

        let report = verify(&root, ws.model()).unwrap();
        assert_eq!(report.errors, Vec::<String>::new(), "{:?}", report.errors);
        let reqs: Vec<&str> =
            report.derived.matched["t1"].iter().map(|m| m.req.as_str()).collect();
        assert!(reqs.contains(&"port-pinned"), "a port folds to its node: {reqs:?}");
        assert!(reqs.contains(&"edge-pinned"), "an edge folds to its endpoints: {reqs:?}");

        // The requirement nothing reaches is a note, never a silence.
        let notes = report.notes.join("\n");
        assert!(notes.contains("`gate-throughput` reaches no task"), "{notes}");
        assert!(!notes.contains("`port-pinned`"), "{notes}");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn verify_flags_structure_and_notes_drift() {
        let root = temp_project();
        let ws = compiled(&root);
        save_version(&root);
        put_requirements(&root);
        use_plan(&root, ws.model(), "mvp").unwrap();
        task_add(&root, "Store", None).unwrap();
        task_add(&root, "Auth", None).unwrap();

        // Hand-edit the task files: an input cycle, an unknown input, a
        // dangling spec_ref, a versioned ref — all load, verify refuses.
        let mut plan = active(&root);
        plan.tasks[0].inputs.insert("t2".into(), "the auth api".into());
        plan.tasks[1].inputs.insert("t1".into(), "the store".into());
        plan.tasks[1].inputs.insert("t9".into(), "a ghost".into());
        plan.tasks[0].spec_refs.push("Phantom".into());
        plan.tasks[1].spec_refs.push("Auth@v0001".into());
        store_authored(&root, &plan);

        let report = verify(&root, ws.model()).unwrap();
        let all = report.errors.join("\n");
        assert!(all.contains("form a cycle"), "{all}");
        assert!(all.contains("`t9` names no task"), "{all}");
        assert!(all.contains("`Phantom` names no element"), "{all}");
        assert!(all.contains("carries a version"), "{all}");
        assert!(report.derived.waves.is_empty(), "a cycle has no layering");

        // Untangle, then drift the live model: verify notes, not errors.
        let mut plan = active(&root);
        plan.tasks[0].inputs.clear();
        plan.tasks[0].spec_refs.retain(|r| r != "Phantom");
        plan.tasks[1].inputs.remove("t9");
        plan.tasks[1].spec_refs.retain(|r| !r.contains('@'));
        store_authored(&root, &plan);
        curate_all(&root, ws.model());
        fs::write(
            root.join("archi/src/model.arch"),
            MODEL.replace("def node Store:\n  port inn\n", "def node Safe:\n  port inn\n")
                .replace("Store.inn", "Safe.inn"),
        )
        .unwrap();
        let live = compiled(&root);
        let report = verify(&root, live.model()).unwrap();
        assert_eq!(report.errors, Vec::<String>::new(), "{:?}", report.errors);
        let notes = report.notes.join("\n");
        assert!(notes.contains("no longer resolves at Working"), "{notes}");
        assert_eq!(
            report.derived.waves,
            vec![vec!["t1".to_string()], vec!["t2".to_string()]]
        );

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_legacy_plan_json_reads_read_only_and_its_lifecycle_moves() {
        let root = temp_project();
        let ws = compiled(&root);
        let v1 = save_version(&root);
        put_requirements(&root);
        put(
            &root,
            "code/store.rs",
            "pub struct Store;\nimpl Store {\n    pub fn put(&mut self) {}\n}\n",
        );

        // Hand-construct the old form — no verb mints it anymore.
        let legacy = Plan {
            name: "old".into(),
            version: v1,
            version_hash: None,
            created: "2026-01-01T00:00:00Z".into(),
            state: PlanState::Draft,
            closed_waves: 0,
            problem: "kept as it was".into(),
            technology_stack: Vec::new(),
            architecture_summary: Vec::new(),
            stack_mapping: Vec::new(),
            scenarios: Vec::new(),
            scenarios_displayed: false,
            scenarios_closed: false,
            tasks: vec![Task {
                id: "t1".into(),
                node: "Store".into(),
                description: "persist rows".into(),
                spec_refs: vec!["Store".into(), "Auth.creds wire Store.inn".into()],
                owns: vec!["service-hardening".into(), "store-encrypted".into()],
                stack_details: String::new(),
                inputs: BTreeMap::new(),
                outputs: vec!["code/store.rs".into()],
                verifications: [
                    ("service-hardening".to_string(), vec!["test — hardened".to_string()]),
                    ("store-encrypted".to_string(), vec!["test — sealed".to_string()]),
                ]
                .into(),
            }],
        };
        store_plan(&root, &legacy).unwrap();
        assert!(matches!(
            use_plan(&root, ws.model(), "old").unwrap(),
            Used::Switched(p) if p.problem == "kept as it was"
        ));

        // The form only shrinks: the mint verbs refuse.
        let err = task_add(&root, "Auth", None).unwrap_err();
        assert!(err.contains("read-only"), "{err}");
        let err = task_rm(&root, "t1").unwrap_err();
        assert!(err.contains("read-only"), "{err}");

        // A stale verification key is a verify error here — the record
        // form refuses it at load, the legacy form at verify.
        let mut bad = legacy.clone();
        bad.tasks[0]
            .verifications
            .insert("no-such-req".into(), vec!["test — never matches".into()]);
        store_plan(&root, &bad).unwrap();
        let report = verify(&root, ws.model()).unwrap();
        assert!(
            report.errors.join("\n").contains("`no-such-req`, which the task does not own"),
            "{:?}",
            report.errors
        );
        store_plan(&root, &legacy).unwrap();

        // Lifecycle still moves the old form: start, next to done, reset.
        let (started, wave1) = start(&root, ws.model()).unwrap();
        assert_eq!(started.state, PlanState::Started);
        assert_eq!(wave1, vec!["t1".to_string()]);
        let outcome = next(&root, ws.model()).unwrap();
        assert!(matches!(outcome.step, Step::Done));
        assert_eq!(active(&root).state, PlanState::Completed);
        reset(&root).unwrap();
        assert_eq!(active(&root).state, PlanState::Draft);

        // Everything stayed json: no record files appeared, the content
        // rode along untouched.
        let dir = plan_dir(&root, "old");
        assert!(dir.join("plan.json").exists());
        assert!(!dir.join("old.md").exists());
        assert!(!dir.join("state.json").exists());
        assert_eq!(active(&root).problem, "kept as it was");

        fs::remove_dir_all(&root).unwrap();
    }
}
