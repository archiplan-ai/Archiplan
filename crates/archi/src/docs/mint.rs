//! Doc skeletons come from commands (`archi/requirements/spec-docs/
//! skeletons-come-from-a-verb.md`): `req add|rm`, `stress open|add|rm`,
//! `world add|rm`.
//!
//! The split is the settled boundary — records through commands, prose by
//! editing. A mint writes the full schema shape with every machine field
//! decided by an explicit parameter or derived from an invariant (never a
//! default), and leaves the text slots empty: the schema's own E_DOC
//! diagnostics hold them as the un-skippable worklist until the author
//! writes the prose. A removal pre-flights the blast radius — inbound
//! references, sealed history — and refuses with the list instead of
//! leaving danglers for `check` to find.

use std::fs;
use std::path::{Path, PathBuf};

use modeling_lang::Model;

use super::md::slugify;
use super::schema::Origin;
use super::world::WORLD;
use crate::versions;

/// Project-relative requirements dir.
fn requirements_dir(root: &Path) -> PathBuf {
    root.join("archi").join("requirements")
}

/// Project-relative stress dir.
fn stress_dir(root: &Path) -> PathBuf {
    root.join("archi").join("stress")
}

/// Mint a requirement skeleton in an intent folder. Every parameter is an
/// explicit choice; the only optional one is `deferred`, whose absence is
/// itself the state "not deferred".
pub fn req_add(
    root: &Path,
    title: &str,
    intent: &str,
    kind: &str,
    origin: &str,
    deferred: Option<&str>,
) -> Result<PathBuf, String> {
    let slug = slug_of(title)?;
    if !matches!(kind, "functional" | "non-functional") {
        return Err(format!(
            "`--kind {kind}` is not a requirement kind — functional | non-functional"
        ));
    }
    let tree = super::discover_tree(root);
    let intents: Vec<&str> = tree.intents.iter().map(|i| i.slug.as_str()).collect();
    if !intents.contains(&intent) {
        return Err(if intents.is_empty() {
            "no intent folders exist yet — capture the intent first: \
             archi/requirements/<intent>/<intent>.md"
                .to_string()
        } else {
            format!(
                "no intent `{intent}` — existing intents: {}; re-run with --intent <folder>",
                intents.join(", ")
            )
        });
    }
    // origin: the two mintable states. `parent` belongs to subrequirement
    // sections and `fusion` to merge ceremonies — neither is born here.
    match super::schema::parse_origin(origin) {
        Ok(Origin::Intent) => {}
        Ok(Origin::Stressors(slugs)) => {
            for s in &slugs {
                if !tree.stressors.iter().any(|st| &st.slug == s) {
                    return Err(format!(
                        "origin names no stressor `{s}` — `archi search {s} --kind stressor` \
                         finds the slug, `archi stress add` mints one"
                    ));
                }
            }
        }
        Ok(_) => {
            return Err(format!(
                "`--origin {origin}` is not mintable — a new requirement is `intent` or \
                 `stressor(<slug>)`"
            ));
        }
        Err(e) => return Err(e),
    }
    if tree.intents.iter().any(|i| i.slug == slug) {
        return Err(format!("`{slug}` names an intent charter — pick another title"));
    }
    let path = requirements_dir(root).join(intent).join(format!("{slug}.md"));
    let deferred = deferred.map(|d| format!(" {d}")).unwrap_or_default();
    let text = format!(
        "---\nkind: {kind}\norigin: {origin}\nsatisfied-by: []\ndeferred:{deferred}\n---\n\n\
         # {title}\n\n## System Context\n\n## Satisfy\n"
    );
    // A replayed batch converges: the identical skeleton is already minted —
    // say so and succeed; a file that moved past its skeleton stays loud
    // (refusals-name-the-continuation).
    if let Ok(standing) = fs::read_to_string(&path) {
        return if standing == text {
            println!(
                "already minted — {} stands; fill the summary, System Context and Satisfy",
                rel(root, &path)
            );
            Ok(path)
        } else {
            Err(format!(
                "{} stands and has moved past its skeleton — it is not re-mintable; \
                 continue editing it",
                rel(root, &path)
            ))
        };
    }
    if let Some(r) = tree.requirements.iter().find(|r| r.slug == slug) {
        return Err(format!("slug `{slug}` is taken — {}", r.file));
    }
    fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

/// Remove a requirement. Plans that own the slug hold it in place: the
/// refusal lists them with the release recipe.
pub fn req_rm(root: &Path, slug: &str) -> Result<PathBuf, String> {
    let tree = super::discover_tree(root);
    let Some(req) = tree.requirements.iter().find(|r| r.slug == slug) else {
        return Err(format!(
            "no requirement `{slug}` — `archi search {slug} --kind requirement` finds the slug"
        ));
    };
    let mut holders: Vec<String> = Vec::new();
    for plan in crate::plans::all_plans(root)? {
        let tasks: Vec<&str> = plan
            .tasks
            .iter()
            .filter(|t| t.owns.iter().any(|o| o == slug))
            .map(|t| t.id.as_str())
            .collect();
        if !tasks.is_empty() {
            holders.push(format!("plan `{}` ({})", plan.name, tasks.join(", ")));
        }
    }
    if !holders.is_empty() {
        return Err(format!(
            "`{slug}` is owned — {}; release it first: `archi plan use <name>`, then \
             `archi plan task req remove <task> {slug}`",
            holders.join("; ")
        ));
    }
    let path = root.join(&req.file);
    fs::remove_file(&path).map_err(|e| format!("cannot remove {}: {e}", path.display()))?;
    Ok(path)
}

/// Open a stress round. Nothing here is a choice: the version is the one
/// just saved (a moved model refuses toward `version save`), the folder is
/// the slug, at most one round is open.
pub fn stress_open(root: &Path, model: &Model, title: &str) -> Result<PathBuf, String> {
    let slug = slug_of(title)?;
    let tree = super::discover_tree(root);
    if let Some(open) = tree.sessions.iter().find(|s| s.open()) {
        // The re-run of the round's own open converges — this is the round
        // (refusals-name-the-continuation); a different round stays a wall.
        if open.slug == slug {
            println!(
                "round `{slug}` is already open — this is it: continue with \
                 `archi stress add <title> --affects <terms>`"
            );
            return Ok(root.join(&open.file));
        }
        return Err(format!(
            "round `{}` is already open — close it (`archi version save`) or fold it before \
             opening another",
            open.slug
        ));
    }
    let version = match versions::current(root, model)? {
        versions::Current::At(id) => id,
        versions::Current::DirtySince(id) => {
            return Err(format!(
                "the model moved since `{id}` — a round presses a saved version: \
                 `archi version save -m <note>` first"
            ));
        }
        versions::Current::NoVersions => {
            return Err(
                "no saved version to press — `archi version save -m <note>` first".to_string()
            );
        }
    };
    let dir = stress_dir(root).join(&slug);
    if dir.exists() {
        return Err(format!("{} already exists", rel(root, &dir)));
    }
    let path = dir.join(format!("{slug}.md"));
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let text = format!("---\nversion: {version}\nclosed:\n---\n\n# {title}\n");
    fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

/// Mint a stressor into the open round. The folder is derived — no open
/// round refuses toward `stress open`; `affects` resolve against the
/// round's pinned version at the write, every miss named in one message.
pub fn stress_add(root: &Path, title: &str, affects: &[String]) -> Result<PathBuf, String> {
    let slug = slug_of(title)?;
    if affects.is_empty() {
        return Err("`--affects` is required and non-empty — terms or types of the pinned \
                    version, comma-separated"
            .to_string());
    }
    let tree = super::discover_tree(root);
    let Some(open) = tree.sessions.iter().find(|s| s.open()) else {
        return Err(
            "no open round — `archi stress open <title>` starts one against the saved version"
                .to_string(),
        );
    };
    let Some((version, _)) = open.version.clone() else {
        return Err(format!(
            "round `{}` pins no version — repair its `version:` frontmatter",
            open.slug
        ));
    };
    let archive = versions::Archive::open(root)?
        .ok_or_else(|| format!("no archive to resolve `{version}` against"))?;
    let ws = super::compile_version(root, &archive, &version)?;
    let bad: Vec<&str> = affects
        .iter()
        .filter(|p| !ws.model().has_node(p))
        .map(String::as_str)
        .collect();
    if !bad.is_empty() {
        return Err(format!(
            "affects name no element of version `{version}`: {} — terms or types of the \
             pinned version, never edges",
            bad.join(", ")
        ));
    }
    let path = stress_dir(root).join(&open.slug).join(format!("{slug}.md"));
    let text = format!(
        "---\naffects: [{}]\noutcome: pending\n---\n\n# {title}\n\n## Attractor\n\n## Resolution\n",
        affects.join(", ")
    );
    // The replayed line converges on the identical skeleton; an edited or
    // foreign file stays loud (refusals-name-the-continuation).
    if let Ok(standing) = fs::read_to_string(&path) {
        return if standing == text {
            println!(
                "already minted — {} stands; write the pressure, Attractor, and the verdict",
                rel(root, &path)
            );
            Ok(path)
        } else {
            Err(format!(
                "{} stands and has moved past its skeleton — it is not re-mintable; \
                 continue editing it",
                rel(root, &path)
            ))
        };
    }
    if let Some(st) = tree.stressors.iter().find(|st| st.slug == slug) {
        return Err(format!("slug `{slug}` is taken — {}", st.file));
    }
    fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

/// Remove a stressor. Two holds: a closed round is sealed history, and a
/// requirement whose origin names the stressor would dangle.
pub fn stress_rm(root: &Path, slug: &str) -> Result<PathBuf, String> {
    let tree = super::discover_tree(root);
    let Some(st) = tree.stressors.iter().find(|s| s.slug == slug) else {
        return Err(format!(
            "no stressor `{slug}` — `archi search {slug} --kind stressor` finds the slug"
        ));
    };
    let sealed = tree
        .sessions
        .iter()
        .find(|s| s.slug == st.session)
        .is_none_or(|s| !s.open());
    if sealed {
        return Err(format!(
            "round `{}` is closed — its record is sealed history; a stressor retires by \
             verdict, not deletion",
            st.session
        ));
    }
    let derived: Vec<&str> = tree
        .requirements
        .iter()
        .filter(|r| {
            r.fields.as_ref().is_some_and(|f| {
                matches!(&f.origin, Some((Origin::Stressors(slugs), _))
                    if slugs.iter().any(|s| s == slug))
            })
        })
        .map(|r| r.slug.as_str())
        .collect();
    if !derived.is_empty() {
        return Err(format!(
            "requirements record `{slug}` as origin: {} — remove or re-origin them first",
            derived.join(", ")
        ));
    }
    let path = root.join(&st.file);
    fs::remove_file(&path).map_err(|e| format!("cannot remove {}: {e}", path.display()))?;
    Ok(path)
}

/// The layer of the strict record, under the world
/// (`archi/requirements/world-facts/the-world-holds-four-layers.md`). One
/// function answers where a fact lives, and everything that mints, walks or
/// retires one asks it — a second answer is a tool that disagrees with itself
/// about what a fact is.
///
/// It is not required to exist: a tree that never minted a fact carries no
/// wing (`archi/requirements/world-facts/the-wing-arrives-without-noise.md`).
pub fn facts_dir(root: &Path) -> PathBuf {
    root.join(WORLD).join("facts")
}

/// Every world fact on disk, by slug — the wing's own walk, which the
/// removal reads for the records alone. A tree with no wing yields none —
/// the folder arrives with the first mint, and no read makes it. A file
/// the structural reader cannot open is skipped, and every diagnostic the
/// walk raises is dropped: a broken record is a `check` finding, not a hold
/// on someone else's removal.
fn world_facts(root: &Path) -> Vec<super::world::WorldDoc> {
    super::world_check::discover(root, &mut Vec::new())
        .into_iter()
        .map(|f| f.doc)
        .collect()
}

/// Mint a world-fact skeleton. Nothing here is a choice: the slug comes
/// from the title as it does for a requirement and a stressor, the three
/// header keys are born empty — `sources: []` is the recorded hypothesis
/// state — and the optional `Open questions` is not born at all. The
/// prose slots stay empty and the schema's own E_DOC diagnostics hold
/// them until an author fills them. A repeated mint converges as `req add`
/// and `stress add` converge — the skeleton it would write is already on
/// disk, so it says so and succeeds
/// (`archi/requirements/world-facts/one-verb-mints-the-world-fact.md`).
pub fn world_add(root: &Path, title: &str) -> Result<PathBuf, String> {
    let slug = slug_of(title)?;
    let dir = facts_dir(root);
    let path = dir.join(format!("{slug}.md"));
    let text = format!(
        "---\ncovers: []\nsources: []\nuses: []\n---\n\n\
         # {title}\n\n## What people do instead\n\n## Scenarios\n"
    );
    // A replayed line converges on the untouched skeleton — the exact bytes
    // this mint writes, never a guess at emptiness. One byte apart is
    // authored content, and authored content is a wall: it is what no verb
    // overwrites (refusals-name-the-continuation).
    if let Ok(standing) = fs::read_to_string(&path) {
        return if standing == text {
            println!(
                "already minted — {} stands; write the condition, what people do instead \
                 and its scenarios",
                rel(root, &path)
            );
            Ok(path)
        } else {
            Err(format!(
                "{} stands and has moved past its skeleton — it is not re-mintable; \
                 continue editing it",
                rel(root, &path)
            ))
        };
    }
    // The wing arrives with the file: no verb requires `archi/world/facts/`,
    // and the layer and the folder over it are made together
    // (`archi/requirements/world-facts/the-wing-arrives-without-noise.md`).
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    fs::write(&path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(path)
}

/// Retire a world fact. Three things can stand on it — a plan in flight,
/// a link anchoring one of its scenarios, another fact naming it in
/// `uses` — and all three are pre-flighted before anything is deleted.
/// Any hold refuses with one message listing every blocker, and nothing
/// cascades: a link is a recorded human assertion and a plan is a promise
/// in flight, so the operator retires them, never this verb
/// (`archi/requirements/world-facts/removal-names-the-code-it-strands.md`,
/// `archi/requirements/world-facts/retirement-refuses-a-plan-in-flight.md`).
pub fn world_rm(root: &Path, slug: &str) -> Result<PathBuf, String> {
    let facts = world_facts(root);
    if !facts.iter().any(|f| f.slug == slug) {
        return Err(format!(
            "no world fact `{slug}` — `archi world ls` lists them"
        ));
    }
    let dependants: Vec<String> = facts
        .iter()
        .filter(|f| f.slug != slug)
        .filter(|f| {
            f.uses
                .as_ref()
                .is_some_and(|(names, _)| names.iter().any(|n| n == slug))
        })
        .map(|f| f.slug.clone())
        .collect();
    let stranded = stranded_links(root, slug)?;
    let holders = holding_plans(root, slug)?;
    if !holders.is_empty() || !stranded.is_empty() || !dependants.is_empty() {
        return Err(refusal(slug, &holders, &stranded, &dependants));
    }
    let path = facts_dir(root).join(format!("{slug}.md"));
    fs::remove_file(&path).map_err(|e| format!("cannot remove {}: {e}", path.display()))?;
    Ok(path)
}

/// A plan in flight and the tasks of it that name the fact.
struct Holder {
    /// The plan's name.
    plan: String,
    /// The task ids naming the fact, in plan order.
    tasks: Vec<String>,
}

/// The live links on one scenario ref — the unit `link rm --spec` retires.
struct Stranded {
    /// The scenario ref, `<fact-slug>#<scenario name>`.
    spec: String,
    /// One `<id> at <anchor>` per link: the id, and the code it names.
    links: Vec<String>,
}

/// Whether a spec ref addresses a scenario of the fact — `<slug>#<name>`,
/// the form a link anchors.
fn scenario_of(spec: &str, slug: &str) -> bool {
    spec.strip_prefix(slug)
        .is_some_and(|rest| rest.starts_with('#'))
}

/// The folded live links whose spec ref addresses a scenario of the fact,
/// grouped by ref: one `link rm --spec` clears one group.
fn stranded_links(root: &Path, slug: &str) -> Result<Vec<Stranded>, String> {
    let mut out: Vec<Stranded> = Vec::new();
    for link in crate::links::ls(root, None, false)? {
        if !scenario_of(&link.spec.path, slug) {
            continue;
        }
        let named = format!("{} at {}", link.id, link.anchor);
        match out.iter_mut().find(|s| s.spec == link.spec.path) {
            Some(group) => group.links.push(named),
            None => out.push(Stranded {
                spec: link.spec.path.clone(),
                links: vec![named],
            }),
        }
    }
    Ok(out)
}

/// The plans whose lifecycle is still open and whose tasks name the fact —
/// the fact itself, or one of its scenarios, in `spec_refs`, or the fact
/// carried in the task's resolved `facts`. A completed plan is a record,
/// and a record holds nothing in place.
fn holding_plans(root: &Path, slug: &str) -> Result<Vec<Holder>, String> {
    let mut out = Vec::new();
    for plan in crate::plans::all_plans(root)? {
        if plan.state == crate::plans::PlanState::Completed {
            continue;
        }
        let tasks: Vec<String> = plan
            .tasks
            .iter()
            .filter(|t| {
                t.spec_refs
                    .iter()
                    .any(|r| r == slug || scenario_of(r, slug))
                    || t.facts.iter().any(|f| f.fact == slug)
            })
            .map(|t| t.id.clone())
            .collect();
        if !tasks.is_empty() {
            out.push(Holder {
                plan: plan.name,
                tasks,
            });
        }
    }
    Ok(out)
}

/// The one refusal, as an ordered continuation: the plan verbs first, then
/// the link retirement, then the dependant facts — the order they must run
/// in, each line runnable as written with its arguments filled in and its
/// reason in a trailing shell comment
/// (`archi/requirements/world-facts/the-refusal-is-an-ordered-continuation.md`).
fn refusal(
    slug: &str,
    holders: &[Holder],
    stranded: &[Stranded],
    dependants: &[String],
) -> String {
    let mut lines: Vec<String> = Vec::new();
    for h in holders {
        lines.push(format!(
            "  archi plan use {plan} && archi plan close  \
             # plan `{plan}` carries `{slug}` at {tasks}",
            plan = h.plan,
            tasks = h.tasks.join(", ")
        ));
    }
    for s in stranded {
        // The ref carries the scenario name and its spaces: quote it, so
        // the line survives the shell it is pasted into.
        lines.push(format!(
            "  archi link rm --spec '{}' --yes  # {}",
            s.spec,
            s.links.join(", ")
        ));
    }
    for d in dependants {
        lines.push(format!(
            "  archi world rm {d}  # `{d}` names `{slug}` in uses"
        ));
    }
    let links = stranded.iter().map(|s| s.links.len()).sum::<usize>();
    format!(
        "`{slug}` is held — {}; nothing was retired. Run these in order, then \
         `archi world rm {slug}` again:\n{}",
        tally(holders.len(), links, dependants.len()),
        lines.join("\n")
    )
}

/// The head line's count, naming only the classes that hold, in the order
/// their commands run.
fn tally(holders: usize, links: usize, dependants: usize) -> String {
    [
        (holders, "plan in flight", "plans in flight"),
        (links, "stranded link", "stranded links"),
        (dependants, "dependant fact", "dependant facts"),
    ]
    .iter()
    .filter(|(n, _, _)| *n > 0)
    .map(|(n, one, many)| format!("{n} {}", if *n == 1 { one } else { many }))
    .collect::<Vec<_>>()
    .join(", ")
}

fn slug_of(title: &str) -> Result<String, String> {
    let slug = slugify(title);
    if slug.is_empty() {
        return Err(format!("`{title}` slugs to nothing — give it a word"));
    }
    Ok(slug)
}

fn rel<'a>(root: &Path, path: &'a Path) -> std::path::Display<'a> {
    path.strip_prefix(root).unwrap_or(path).display()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    const TITLE: &str = "The train has no signal";
    const SLUG: &str = "the-train-has-no-signal";
    /// The scenario name a link anchors — `<fact-slug>#<scenario name>`.
    const SCENARIO: &str = "the-train-has-no-signal#the app opens with no network";

    fn temp_root() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-mint-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A fact on disk, titled so its name derives back to its slug.
    fn put_fact(root: &Path, slug: &str, uses: &[&str]) {
        let dir = facts_dir(root);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{slug}.md")),
            format!(
                "---\ncovers: []\nsources: []\nuses: [{}]\n---\n\n# {slug}\n\nIt is so.\n\n\
                 ## What people do instead\n\nThey go around it.\n\n## Scenarios\n\n\
                 Feature: F\n",
                uses.join(", ")
            ),
        )
        .unwrap();
    }

    /// A legacy-form plan with one task naming `refs` — the shape
    /// `all_plans` reads.
    fn put_plan(root: &Path, name: &str, state: &str, refs: &[&str]) {
        let dir = root.join("archi").join("plans").join(name);
        fs::create_dir_all(&dir).unwrap();
        let refs: Vec<String> = refs.iter().map(|r| format!("\"{r}\"")).collect();
        fs::write(
            dir.join("plan.json"),
            format!(
                "{{\"name\":\"{name}\",\"version\":\"v0001\",\
                 \"created\":\"2026-01-01T00:00:00Z\",\"state\":\"{state}\",\
                 \"tasks\":[{{\"id\":\"t1\",\"node\":\"DocMint\",\"spec_refs\":[{}]}}]}}\n",
                refs.join(",")
            ),
        )
        .unwrap();
    }

    /// A plan whose one task carries the fact the way the planner resolves
    /// it now: in `facts`, named nowhere in `spec_refs`.
    fn put_plan_carrying(root: &Path, name: &str, state: &str, facts: &[&str]) {
        let dir = root.join("archi").join("plans").join(name);
        fs::create_dir_all(&dir).unwrap();
        let facts: Vec<String> = facts
            .iter()
            .map(|f| format!("{{\"fact\":\"{f}\",\"digest\":\"9f3ab1\"}}"))
            .collect();
        fs::write(
            dir.join("plan.json"),
            format!(
                "{{\"name\":\"{name}\",\"version\":\"v0001\",\
                 \"created\":\"2026-01-01T00:00:00Z\",\"state\":\"{state}\",\
                 \"tasks\":[{{\"id\":\"t1\",\"node\":\"DocMint\",\"spec_refs\":[],\
                 \"facts\":[{}]}}]}}\n",
                facts.join(",")
            ),
        )
        .unwrap();
    }

    /// One `add` event on the journal — the folded live set is its fold.
    fn put_link(root: &Path, id: &str, spec: &str, anchor: &str) {
        let dir = root.join("archi").join("links");
        fs::create_dir_all(&dir).unwrap();
        let (file, symbol) = anchor.split_once('#').expect("anchor names an item");
        let line = format!(
            "{{\"event\":\"add\",\"link\":{{\"id\":\"{id}\",\"spec\":{{\"ref\":\"{spec}\"}},\
             \"anchor\":{{\"file\":\"{file}\",\"symbol\":\"{symbol}\"}},\"kind\":\"indirect\",\
             \"standing\":\"asserted\",\"origin\":{{\"kind\":\"authored\"}},\
             \"birth\":{{\"created\":\"2026-01-01T00:00:00Z\",\"spans\":[]}},\
             \"pins\":{{\"canonicalizer\":\"rust-tok-v1\",\"interface\":\"sha256:a\",\
             \"body\":\"sha256:b\"}}}}}}\n"
        );
        let mut f = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(dir.join("journal.jsonl"))
            .unwrap();
        f.write_all(line.as_bytes()).unwrap();
    }

    /// The fact read back through the schema — what `check` would say.
    fn diagnostics(root: &Path, slug: &str) -> Vec<String> {
        let file = format!("{WORLD}facts/{slug}.md");
        let text = fs::read_to_string(root.join(&file)).unwrap();
        let mut diags = Vec::new();
        match super::super::md::parse(&text) {
            Ok(doc) => {
                super::super::world::parse(&doc, &file, slug, root, &mut diags);
            }
            Err(e) => diags.push(super::super::DocDiagnostic::new("E_DOC", e.message, &file, e.line)),
        }
        diags.iter().map(ToString::to_string).collect()
    }

    /// The command lines of a refusal — everything the head line does not
    /// carry.
    fn commands(refusal: &str) -> Vec<String> {
        refusal
            .lines()
            .skip(1)
            .map(|l| l.trim().to_string())
            .collect()
    }

    /// Run one printed line the way a shell would, through the library the
    /// named verb calls. Every argument comes out of the line itself: what
    /// is not written cannot run.
    fn run(root: &Path, line: &str) {
        let command = line.split("  #").next().unwrap().trim();
        let words: Vec<&str> = command.split_whitespace().collect();
        match words.as_slice() {
            // `archi plan use <name> && archi plan close`
            ["archi", "plan", "use", name, "&&", "archi", "plan", "close"] => {
                let marker = root.join("archi").join("plans").join(".current");
                fs::write(&marker, format!("{name}\n")).unwrap();
                crate::plans::close(root).unwrap();
            }
            // `archi link rm --spec '<ref>' --yes`
            ["archi", "link", "rm", "--spec", ..] => {
                let spec = command
                    .split('\'')
                    .nth(1)
                    .expect("the ref is quoted for the shell");
                crate::links::retire_spec(root, spec).unwrap();
            }
            // `archi world rm <slug>`
            ["archi", "world", "rm", slug] => {
                world_rm(root, slug).unwrap();
            }
            _ => panic!("`{command}` is not a runnable line"),
        }
    }

    #[test]
    fn the_mint_writes_the_schema_shape() {
        let root = temp_root();
        let path = world_add(&root, TITLE).unwrap();
        // The strict record lives in the layer of the strict record, and the
        // mint writes nowhere else
        // (`archi/requirements/world-facts/the-world-holds-four-layers.md`).
        assert_eq!(
            path,
            root.join("archi")
                .join("world")
                .join("facts")
                .join("the-train-has-no-signal.md")
        );
        let text = fs::read_to_string(&path).unwrap();
        // The three keys, present and empty; the headings in order; the
        // optional one omitted.
        assert_eq!(
            text,
            "---\ncovers: []\nsources: []\nuses: []\n---\n\n\
             # The train has no signal\n\n## What people do instead\n\n## Scenarios\n"
        );
        assert!(!text.contains("Open questions"));
        fs::remove_dir_all(&root).unwrap();
    }

    /// The replayed mint converges on the untouched skeleton, exactly as
    /// `req add` and `stress add` do: it says so, writes nothing, and
    /// succeeds.
    #[test]
    fn a_second_mint_on_the_untouched_skeleton_converges() {
        let root = temp_root();
        let path = world_add(&root, TITLE).unwrap();
        let minted = fs::read_to_string(&path).unwrap();
        // A stamp no write survives: a re-written file, byte-identical or
        // not, carries a new modification time.
        let stamp = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_000_000);
        fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(stamp))
            .unwrap();

        assert_eq!(world_add(&root, TITLE).unwrap(), path);
        assert_eq!(fs::read_to_string(&path).unwrap(), minted);
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), stamp);
        fs::remove_dir_all(&root).unwrap();
    }

    /// One authored byte makes the file a record, and a record is what no
    /// verb overwrites — a stripped trailing newline included.
    #[test]
    fn a_second_mint_on_written_prose_names_the_standing_file() {
        let root = temp_root();
        let path = world_add(&root, TITLE).unwrap();
        let minted = fs::read_to_string(&path).unwrap();
        let authored = minted.replace(
            "# The train has no signal\n",
            "# The train has no signal\n\nThe carriage drops the network.\n",
        );
        fs::write(&path, &authored).unwrap();

        let e = world_add(&root, TITLE).unwrap_err();
        assert!(e.contains("archi/world/facts/the-train-has-no-signal.md"), "{e}");
        // The refusal left the standing file as it was.
        assert_eq!(fs::read_to_string(&path).unwrap(), authored);

        // The comparison is the bytes: a skeleton an editor stripped the
        // last newline from is not the skeleton the mint writes.
        fs::write(&path, minted.trim_end()).unwrap();
        let e = world_add(&root, TITLE).unwrap_err();
        assert!(e.contains("archi/world/facts/the-train-has-no-signal.md"), "{e}");
        assert_eq!(fs::read_to_string(&path).unwrap(), minted.trim_end());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_skeleton_holds_the_check_until_the_prose_lands() {
        let root = temp_root();
        let path = world_add(&root, TITLE).unwrap();
        // Three empty slots, three located errors — the un-skippable worklist.
        let held = diagnostics(&root, SLUG);
        assert_eq!(held.len(), 3, "{held:#?}");
        assert!(held.iter().all(|d| d.contains("E_DOC")), "{held:#?}");

        let filled = fs::read_to_string(&path)
            .unwrap()
            .replace(
                "# The train has no signal\n",
                "# The train has no signal\n\nThe carriage drops the network.\n",
            )
            .replace(
                "## What people do instead\n",
                "## What people do instead\n\nRiders load the page at the platform.\n",
            )
            .replace(
                "## Scenarios\n",
                "## Scenarios\n\nFeature: Offline open\n",
            );
        fs::write(&path, filled).unwrap();
        assert_eq!(diagnostics(&root, SLUG), Vec::<String>::new());
        fs::remove_dir_all(&root).unwrap();
    }

    /// The first mint makes the layer and the folder over it in one call: a
    /// tree that holds no `archi/world/` at all takes its first fact
    /// (`archi/requirements/world-facts/the-wing-arrives-without-noise.md`).
    #[test]
    fn the_wing_arrives_with_the_first_mint() {
        let root = temp_root();
        // A tree with no wing reads as no facts, and no verb makes the folder.
        assert!(world_facts(&root).is_empty());
        assert!(!root.join(WORLD).exists());
        assert!(!facts_dir(&root).exists());

        world_add(&root, TITLE).unwrap();
        assert!(root.join(WORLD).is_dir());
        assert!(facts_dir(&root).is_dir());
        assert!(facts_dir(&root).join("the-train-has-no-signal.md").is_file());
        // The walk that reads the wing finds it where the mint put it.
        let slugs: Vec<String> = world_facts(&root).into_iter().map(|f| f.slug).collect();
        assert_eq!(slugs, [SLUG]);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_free_fact_retires_in_one_call() {
        let root = temp_root();
        world_add(&root, TITLE).unwrap();
        let path = world_rm(&root, SLUG).unwrap();
        assert!(!path.exists());

        let e = world_rm(&root, SLUG).unwrap_err();
        assert!(e.contains(SLUG), "{e}");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_dependant_fact_holds_the_removal() {
        let root = temp_root();
        put_fact(&root, SLUG, &[]);
        put_fact(&root, "the-carriage-is-a-tunnel", &[SLUG]);
        let e = world_rm(&root, SLUG).unwrap_err();
        assert!(e.contains("the-carriage-is-a-tunnel"), "{e}");
        assert_eq!(
            commands(&e),
            ["archi world rm the-carriage-is-a-tunnel  \
              # `the-carriage-is-a-tunnel` names `the-train-has-no-signal` in uses"]
        );
        // Nothing was retired.
        assert!(facts_dir(&root).join(format!("{SLUG}.md")).is_file());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_stranded_link_holds_the_removal_and_names_its_code() {
        let root = temp_root();
        put_fact(&root, SLUG, &[]);
        put_link(&root, "l0001-aaaaaa", SCENARIO, "crates/app/src/open.rs#open");
        put_link(&root, "l0002-bbbbbb", SCENARIO, "crates/app/src/sync.rs#Sync::last");

        let e = world_rm(&root, SLUG).unwrap_err();
        // By id, and by the code item each one names.
        assert!(e.contains("l0001-aaaaaa"), "{e}");
        assert!(e.contains("l0002-bbbbbb"), "{e}");
        assert!(e.contains("crates/app/src/open.rs#open"), "{e}");
        assert!(e.contains("crates/app/src/sync.rs#Sync::last"), "{e}");
        // Nothing cascaded: both links are still live.
        assert_eq!(crate::links::ls(&root, None, false).unwrap().len(), 2);

        // `link rm --spec` retires them, and the same removal proceeds.
        crate::links::retire_spec(&root, SCENARIO).unwrap();
        world_rm(&root, SLUG).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_link_on_another_fact_s_scenario_holds_nothing() {
        let root = temp_root();
        put_fact(&root, SLUG, &[]);
        put_link(
            &root,
            "l0001-aaaaaa",
            "the-carriage-is-a-tunnel#the app opens with no network",
            "crates/app/src/open.rs#open",
        );
        world_rm(&root, SLUG).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_plan_in_flight_holds_the_removal() {
        let root = temp_root();
        put_fact(&root, SLUG, &[]);
        put_plan(&root, "offline-open", "started", &[SCENARIO]);

        let e = world_rm(&root, SLUG).unwrap_err();
        assert!(e.contains("offline-open"), "{e}");
        assert!(e.contains("t1"), "{e}");
        // The refusing verb closed nothing.
        assert!(
            fs::read_to_string(root.join("archi/plans/offline-open/plan.json"))
                .unwrap()
                .contains("\"state\":\"started\"")
        );

        // The same fact retires once that plan is completed.
        put_plan(&root, "offline-open", "completed", &[SCENARIO]);
        world_rm(&root, SLUG).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    /// The covering fact a planner resolves lands in the task's `facts`, not
    /// in its `spec_refs`: the plan in flight holds the removal all the same
    /// (`archi/requirements/world-facts/retirement-refuses-a-plan-in-flight.md`).
    #[test]
    fn a_carried_fact_holds_the_removal() {
        let root = temp_root();
        put_fact(&root, SLUG, &[]);
        put_plan_carrying(&root, "offline-open", "started", &[SLUG]);

        let e = world_rm(&root, SLUG).unwrap_err();
        assert!(e.contains("offline-open"), "{e}");
        assert!(e.contains("t1"), "{e}");
        // Nothing was retired.
        assert!(facts_dir(&root).join(format!("{SLUG}.md")).is_file());

        // The same fact retires once that plan is completed.
        put_plan_carrying(&root, "offline-open", "completed", &[SLUG]);
        world_rm(&root, SLUG).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_draft_plan_holds_the_removal_too() {
        let root = temp_root();
        put_fact(&root, SLUG, &[]);
        // A task may name the fact itself, not only one of its scenarios.
        put_plan(&root, "offline-open", "draft", &[SLUG]);
        let e = world_rm(&root, SLUG).unwrap_err();
        assert!(e.contains("offline-open"), "{e}");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_refusal_is_one_message_of_three_ordered_commands() {
        let root = temp_root();
        put_fact(&root, SLUG, &[]);
        put_fact(&root, "the-carriage-is-a-tunnel", &[SLUG]);
        put_link(&root, "l0001-aaaaaa", SCENARIO, "crates/app/src/open.rs#open");
        put_plan(&root, "offline-open", "started", &[SCENARIO]);

        let e = world_rm(&root, SLUG).unwrap_err();
        // One message: the head line tallies all three classes.
        assert!(e.starts_with("`the-train-has-no-signal` is held — "), "{e}");
        assert!(
            e.lines().next().unwrap().contains("1 plan in flight")
                && e.lines().next().unwrap().contains("1 stranded link")
                && e.lines().next().unwrap().contains("1 dependant fact"),
            "{e}"
        );
        // Plan verbs, then the link retirement, then the dependant facts.
        let lines = commands(&e);
        assert_eq!(lines.len(), 3, "{e}");
        assert!(lines[0].starts_with("archi plan use offline-open"), "{e}");
        assert!(lines[1].starts_with("archi link rm --spec "), "{e}");
        assert!(lines[2].starts_with("archi world rm the-carriage-is-a-tunnel"), "{e}");
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn each_printed_command_clears_its_blocker() {
        let root = temp_root();
        put_fact(&root, SLUG, &[]);
        put_fact(&root, "the-carriage-is-a-tunnel", &[SLUG]);
        put_link(&root, "l0001-aaaaaa", SCENARIO, "crates/app/src/open.rs#open");
        put_plan(&root, "offline-open", "started", &[SCENARIO]);

        // Run them as written, in the printed order; each clears its own.
        let mut e = world_rm(&root, SLUG).unwrap_err();
        for _ in 0..3 {
            let line = commands(&e).first().expect("a blocker prints a line").clone();
            run(&root, &line);
            match world_rm(&root, SLUG) {
                Ok(path) => {
                    assert!(!path.exists());
                    fs::remove_dir_all(&root).unwrap();
                    return;
                }
                Err(next) => {
                    assert!(next.len() < e.len(), "the blocker did not clear: {next}");
                    e = next;
                }
            }
        }
        panic!("three commands left the removal held: {e}");
    }

    #[test]
    fn a_single_blocker_prints_a_single_command() {
        let root = temp_root();
        put_fact(&root, SLUG, &[]);
        put_link(&root, "l0001-aaaaaa", SCENARIO, "crates/app/src/open.rs#open");
        let e = world_rm(&root, SLUG).unwrap_err();
        let lines = commands(&e);
        assert_eq!(lines.len(), 1, "{e}");
        run(&root, &lines[0]);
        world_rm(&root, SLUG).unwrap();
        fs::remove_dir_all(&root).unwrap();
    }
}
