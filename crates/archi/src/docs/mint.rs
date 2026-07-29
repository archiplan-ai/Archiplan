//! Doc skeletons come from verbs (`archi/requirements/spec-docs/
//! skeletons-come-from-a-verb.md`): `req add|rm`, `stress open|add|rm`.
//!
//! The split is the settled boundary — records through verbs, prose by
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
