//! The record form of a plan: a folder of markdown files with lifecycle
//! apart in `state.json`
//! (`archi/requirements/planning/a-plan-is-a-folder-of-records.md`).
//!
//! Content is the files — there is no write command for prose. The charter
//! `<name>.md` carries the envelope: problem prose, `## Stack` bullets
//! with provenance, `## Architecture` bullets for summary lines and stack
//! mappings. Each task is `t<N>-<node-slug>.md`: `node`, hand-curated
//! `owns` and machine-resolved `facts` in the frontmatter, description
//! prose, then `## Spec`, `## Inputs`, `## Outputs`, `## Stack` bullets
//! and `## Verifications` keyed by owned slug. `state.json` alone moves
//! through commands — the mint writes it, `save_state` rewrites it — and
//! the one other machine write in the folder is [`write_facts`], which
//! moves the `facts` line and nothing else: a covering fact is carried,
//! never curated
//! (`archi/requirements/world-facts/a-task-carries-the-facts-that-cover-its-node.md`).
//!
//! A `scenarios.md` a pre-world plan was written with is read by nobody,
//! written by nobody and deleted by nobody: the stories live in the world
//! now and history is left exactly as it is
//! (`archi/requirements/world-facts/a-plan-s-own-scenarios-block-retires.md`,
//! `archi/decisions/the-old-plans-are-left-alone.md`).
//!
//! Parsing is tolerant on whitespace and strict on shape: an unknown
//! section, a shapeless bullet, a verification under an unowned slug are
//! load errors carrying the file path — the file is the truth, and a
//! malformed truth refuses loudly instead of loading as less than what
//! was written. Loading never rewrites a file: reads are free.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    CoveringFact, Plan, PlanState, StackMapping, SummaryLine, Task, TechChoice, plan_dir,
};
use crate::docs::md::slugify;

// ---- the folder --------------------------------------------------------------

/// The charter is the marker: a plan whose `<name>.md` exists is
/// record-form, whatever else the directory holds.
pub(crate) fn is_record(root: &Path, name: &str) -> bool {
    charter_path(root, name).exists()
}

pub(crate) fn charter_path(root: &Path, name: &str) -> PathBuf {
    plan_dir(root, name).join(format!("{name}.md"))
}

fn state_path(root: &Path, name: &str) -> PathBuf {
    plan_dir(root, name).join("state.json")
}

/// `t<N>-<node-slug>.md` — the name a task file is minted under.
pub(crate) fn task_file_name(task: &Task) -> String {
    format!("{}-{}.md", task.id, slugify(&task.node))
}

/// The ordinal a file name carries, when it is shaped like a task file.
/// The slug part is free — identity is the `t<N>-` prefix alone.
fn task_ordinal(file_name: &str) -> Option<usize> {
    let stem = file_name.strip_suffix(".md")?;
    let (digits, _slug) = stem.strip_prefix('t')?.split_once('-')?;
    (!digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
        .then(|| digits.parse().ok())?
}

/// The file carrying task `id`, found by ordinal — the slug part of the
/// name is free, so the scan matches the loader, not a recomputation.
pub(crate) fn task_path(root: &Path, name: &str, id: &str) -> Option<PathBuf> {
    let want: usize = id.strip_prefix('t')?.parse().ok()?;
    let dir = plan_dir(root, name);
    let entries = fs::read_dir(&dir).ok()?;
    for e in entries.filter_map(Result::ok) {
        let file = e.file_name().to_string_lossy().into_owned();
        if task_ordinal(&file) == Some(want) {
            return Some(dir.join(file));
        }
    }
    None
}

// ---- state.json --------------------------------------------------------------

/// The lifecycle file: exactly the fields commands move — state, waves, the
/// latches, the pin. Unknown fields refuse: this is the one machine-owned
/// file of the folder, and drift in it cannot be tolerated silently.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StateFile {
    state: PlanState,
    #[serde(default)]
    closed_waves: usize,
    version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    version_hash: Option<String>,
    created: String,
    // The cleanup and scenario latches are lifecycle too — `plan next`
    // moves them — but they serialize only once flipped, so a fresh
    // state.json stays the five-field record the mint wrote, and a
    // legacy file without them parses as unflipped.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    cleanup_displayed: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    scenarios_displayed: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    scenarios_closed: bool,
    // The mark of the world: the mint stamps it, nothing else moves it, and
    // a plan written before the world carries no such field and parses as
    // what it is — which is what decides whether an empty closing block may
    // close the plan
    // (`archi/requirements/world-facts/a-plan-s-own-scenarios-block-retires.md`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    minted_after_the_world: bool,
}

/// Persist the lifecycle fields of a record plan — the only write any
/// command performs on the folder past its mint.
pub(crate) fn write_state(root: &Path, plan: &Plan) -> Result<(), String> {
    let state = StateFile {
        state: plan.state,
        closed_waves: plan.closed_waves,
        version: plan.version.clone(),
        version_hash: plan.version_hash.clone(),
        created: plan.created.clone(),
        cleanup_displayed: plan.cleanup_displayed,
        scenarios_displayed: plan.scenarios_displayed,
        scenarios_closed: plan.scenarios_closed,
        minted_after_the_world: plan.minted_after_the_world,
    };
    let path = state_path(root, &plan.name);
    let mut text =
        serde_json::to_string_pretty(&state).map_err(|e| format!("state serializes: {e}"))?;
    text.push('\n');
    fs::write(&path, text).map_err(|e| format!("cannot write `{}`: {e}", path.display()))
}

fn load_state(root: &Path, name: &str) -> Result<StateFile, String> {
    let path = state_path(root, name);
    let text = fs::read_to_string(&path).map_err(|e| {
        format!(
            "cannot read `{}`: {e} — a record plan keeps its lifecycle there",
            path.display()
        )
    })?;
    serde_json::from_str(&text).map_err(|e| format!("`{}` does not parse: {e}", path.display()))
}

// ---- rendering ---------------------------------------------------------------
//
// The renderers are the mint's skeletons and the parsers' inverse: a plan
// rendered and parsed comes back field-for-field, which the tests hold.

/// The charter: name as title, problem prose, the envelope as bullets.
pub(crate) fn render_charter(plan: &Plan) -> String {
    let mut out = format!("# {}\n", plan.name);
    if !plan.problem.is_empty() {
        out.push('\n');
        out.push_str(&plan.problem);
        out.push('\n');
    }
    out.push_str("\n## Stack\n");
    if !plan.technology_stack.is_empty() {
        out.push('\n');
        for t in &plan.technology_stack {
            if t.provenance.is_empty() {
                out.push_str(&format!("- {}\n", t.tech));
            } else {
                out.push_str(&format!("- {} — {}\n", t.tech, t.provenance));
            }
        }
    }
    out.push_str("\n## Architecture\n");
    if !(plan.architecture_summary.is_empty() && plan.stack_mapping.is_empty()) {
        out.push('\n');
        for s in &plan.architecture_summary {
            out.push_str(&format!("- `{}` — {}\n", s.node, s.role));
        }
        for m in &plan.stack_mapping {
            out.push_str(&format!("- `{}` realizes {}\n", m.node, m.tech));
        }
    }
    out
}

/// The `facts` frontmatter line: the covering facts as `<slug>@<digest>`.
/// It rides only when the world reaches the node, so a project without one
/// sees no new line in its task files.
fn facts_line(facts: &[CoveringFact]) -> String {
    let entries: Vec<String> = facts.iter().map(CoveringFact::render).collect();
    format!("facts: [{}]", entries.join(", "))
}

/// One task file: frontmatter, description prose, the bullet sections.
/// Empty sections keep their heading — the slots the author fills.
pub(crate) fn render_task(task: &Task) -> String {
    let mut out = format!("---\nnode: {}\nowns: [{}]\n", task.node, task.owns.join(", "));
    if !task.facts.is_empty() {
        out.push_str(&facts_line(&task.facts));
        out.push('\n');
    }
    out.push_str("---\n");
    out.push_str(&format!("\n# {} — {}\n", task.id, task.node));
    if !task.description.is_empty() {
        out.push('\n');
        out.push_str(&task.description);
        out.push('\n');
    }
    let bullets = |out: &mut String, items: &[String]| {
        if !items.is_empty() {
            out.push('\n');
            for i in items {
                out.push_str(&format!("- {i}\n"));
            }
        }
    };
    out.push_str("\n## Spec\n");
    let refs: Vec<String> = task.spec_refs.iter().map(|r| format!("`{r}`")).collect();
    bullets(&mut out, &refs);
    out.push_str("\n## Inputs\n");
    let inputs: Vec<String> = task
        .inputs
        .iter()
        .map(|(from, note)| {
            if note.is_empty() {
                format!("from {from}")
            } else {
                format!("from {from} — {note}")
            }
        })
        .collect();
    bullets(&mut out, &inputs);
    out.push_str("\n## Outputs\n");
    bullets(&mut out, &task.outputs);
    out.push_str("\n## Stack\n");
    let stack: Vec<String> = task.stack_details.lines().map(str::to_string).collect();
    bullets(&mut out, &stack);
    out.push_str("\n## Verifications\n");
    for (slug, proofs) in &task.verifications {
        out.push_str(&format!("\n### {slug}\n"));
        bullets(&mut out, proofs);
    }
    out
}

// ---- parsing -----------------------------------------------------------------

fn shape_err(label: &str, line: usize, message: &str) -> String {
    format!("`{label}` line {line}: {message}")
}

/// Strip a `- ` bullet, or refuse with the section's shape.
fn bullet<'a>(label: &str, line: usize, raw: &'a str, shape: &str) -> Result<&'a str, String> {
    raw.trim_end()
        .strip_prefix("- ")
        .ok_or_else(|| shape_err(label, line, shape))
}

/// Fold one line of a bullet section into the bullet standing open, and hand
/// back the bullet that line closed. A bullet section is read whole: it
/// splits at the lines that open with `- `, and every other line continues
/// the piece the last such line opened, its own break and any blank line
/// around it collapsing to a single space. There is no rule about which bare
/// line continues and which does not — that rule is what refused a wrapped
/// bullet in the first place
/// (`archi/requirements/planning/a-record-bullet-may-wrap.md`).
///
/// The piece keeps the line it opened on, so a refusal names the line the
/// author wrote the bullet on and never a continuation. A section opening on
/// a bare line hands that line back as a bullet with no prefix, which refuses
/// exactly where it always did; a paragraph left *under* the bullets joins
/// the last one instead, which the requirement states and accepts.
fn fold_bullet(
    open: &mut Option<(usize, String)>,
    line: usize,
    raw: &str,
) -> Option<(usize, String)> {
    if raw.starts_with("- ") {
        return open.replace((line, raw.trim_end().to_string()));
    }
    let text = raw.trim();
    if text.is_empty() {
        return None;
    }
    match open {
        Some((_, bullet)) => {
            bullet.push(' ');
            bullet.push_str(text);
        }
        None => *open = Some((line, raw.trim_end().to_string())),
    }
    None
}

/// Prose slot: raw lines joined verbatim, leading and trailing blank
/// lines dropped — paragraph breaks inside survive the round trip.
fn join_prose(lines: &[&str]) -> String {
    let mut lines = lines.to_vec();
    while lines.first().is_some_and(|l| l.trim().is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

/// One folded bullet of the charter, under the section it stands in.
fn charter_bullet(
    label: &str,
    section: &str,
    line: usize,
    raw: &str,
    stack: &mut Vec<TechChoice>,
    summary: &mut Vec<SummaryLine>,
    mapping: &mut Vec<StackMapping>,
) -> Result<(), String> {
    if section == "Stack" {
        let b = bullet(label, line, raw, "stack bullets are `- <tech> — <provenance>`")?;
        let (tech, provenance) = b.split_once(" — ").unwrap_or((b, ""));
        stack.push(TechChoice {
            tech: tech.trim().to_string(),
            provenance: provenance.trim().to_string(),
        });
        return Ok(());
    }
    let b = bullet(
        label,
        line,
        raw,
        "architecture bullets are `- `<node>` — <role>` or `- `<node>` realizes <tech>`",
    )?;
    let (node, rest) = backticked(b).ok_or_else(|| {
        shape_err(label, line, "architecture bullets open with a backticked node")
    })?;
    if let Some(role) = rest.strip_prefix(" — ") {
        summary.push(SummaryLine { node, role: role.trim().to_string() });
    } else if let Some(tech) = rest.strip_prefix(" realizes ") {
        mapping.push(StackMapping { tech: tech.trim().to_string(), node });
    } else {
        return Err(shape_err(
            label,
            line,
            "after the node comes `— <role>` or `realizes <tech>`",
        ));
    }
    Ok(())
}

/// The charter's fields: problem prose, then `## Stack` and
/// `## Architecture` bullets. Anything else is a shape error.
fn parse_charter(
    label: &str,
    text: &str,
) -> Result<(String, Vec<TechChoice>, Vec<SummaryLine>, Vec<StackMapping>), String> {
    let mut problem_lines: Vec<&str> = Vec::new();
    let mut stack = Vec::new();
    let mut summary = Vec::new();
    let mut mapping = Vec::new();
    let mut seen_h1 = false;
    let mut section: Option<&str> = None;
    // The bullet standing open, if any: a heading ends the section, and the
    // section is what the fold reads whole ([`fold_bullet`]).
    let mut open: Option<(usize, String)> = None;
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        if !seen_h1 {
            if raw.trim().is_empty() {
                continue;
            }
            if raw.starts_with("# ") {
                seen_h1 = true;
                continue;
            }
            return Err(shape_err(label, line, "a charter opens with `# <name>`"));
        }
        if let Some(heading) = raw.trim_end().strip_prefix("## ") {
            if let Some((at, folded)) = open.take() {
                let sec = section.expect("a bullet stands inside a section");
                charter_bullet(label, sec, at, &folded, &mut stack, &mut summary, &mut mapping)?;
            }
            section = match heading.trim() {
                "Stack" => Some("Stack"),
                "Architecture" => Some("Architecture"),
                other => {
                    return Err(format!(
                        "`{label}`: unknown section `## {other}` — a charter carries \
                         `## Stack` and `## Architecture`"
                    ));
                }
            };
            continue;
        }
        match section {
            None => problem_lines.push(raw),
            Some(sec) => {
                if let Some((at, folded)) = fold_bullet(&mut open, line, raw) {
                    charter_bullet(label, sec, at, &folded, &mut stack, &mut summary, &mut mapping)?;
                }
            }
        }
    }
    if let Some((at, folded)) = open.take() {
        let sec = section.expect("a bullet stands inside a section");
        charter_bullet(label, sec, at, &folded, &mut stack, &mut summary, &mut mapping)?;
    }
    if !seen_h1 {
        return Err(format!("`{label}`: a charter opens with `# <name>`"));
    }
    Ok((join_prose(&problem_lines), stack, summary, mapping))
}

/// `` `node` rest`` → (node, rest).
fn backticked(text: &str) -> Option<(String, &str)> {
    let rest = text.strip_prefix('`')?;
    let close = rest.find('`')?;
    Some((rest[..close].to_string(), &rest[close + 1..]))
}

/// One folded bullet of a task file, under the section it stands in — and,
/// in `Verifications`, under the `### <slug>` it rides.
fn task_bullet(
    label: &str,
    section: &str,
    slug: Option<&String>,
    (line, raw): (usize, &str),
    task: &mut Task,
    stack_lines: &mut Vec<String>,
) -> Result<(), String> {
    match section {
        "Spec" => {
            let b = bullet(label, line, raw, "spec bullets are `- `<ref>``")?;
            let (r, rest) = backticked(b)
                .ok_or_else(|| shape_err(label, line, "spec refs are backtick-wrapped"))?;
            if !rest.trim().is_empty() {
                return Err(shape_err(label, line, "spec bullets carry one ref and nothing else"));
            }
            task.spec_refs.push(r);
        }
        "Inputs" => {
            let b = bullet(label, line, raw, "input bullets are `- from <task> — <note>`")?;
            let b = b.strip_prefix("from ").ok_or_else(|| {
                shape_err(label, line, "input bullets are `- from <task> — <note>`")
            })?;
            let (from, note) = b.split_once(" — ").unwrap_or((b, ""));
            task.inputs.insert(from.trim().to_string(), note.trim().to_string());
        }
        "Outputs" => {
            let b = bullet(label, line, raw, "output bullets are `- <path>`")?;
            task.outputs.push(b.to_string());
        }
        "Stack" => {
            let b = bullet(label, line, raw, "stack bullets are `- <detail>`")?;
            stack_lines.push(b.to_string());
        }
        _ => {
            let b = bullet(label, line, raw, "verifications ride under a `### <slug>`")?;
            let Some(slug) = slug else {
                return Err(shape_err(label, line, "verifications ride under a `### <slug>`"));
            };
            task.verifications.get_mut(slug).expect("opened above").push(b.to_string());
        }
    }
    Ok(())
}

/// One task file. The id arrives from the file name — the `t<N>-` prefix
/// is the identity; frontmatter carries the node and the curated owns.
fn parse_task(label: &str, id: &str, text: &str) -> Result<Task, String> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.first().map(|l| l.trim_end()) != Some("---") {
        return Err(format!("`{label}`: a task file opens with `---` frontmatter"));
    }
    let mut node: Option<String> = None;
    let mut owns: Vec<String> = Vec::new();
    let mut facts: Vec<CoveringFact> = Vec::new();
    let mut body_at = None;
    for (i, raw) in lines.iter().enumerate().skip(1) {
        let line = i + 1;
        if raw.trim_end() == "---" {
            body_at = Some(i + 1);
            break;
        }
        let Some((key, value)) = raw.split_once(':') else {
            return Err(shape_err(label, line, "frontmatter lines are `key: value`"));
        };
        match key.trim() {
            "node" => node = Some(value.trim().to_string()),
            "owns" => {
                let inner = value
                    .trim()
                    .strip_prefix('[')
                    .and_then(|v| v.strip_suffix(']'))
                    .ok_or_else(|| shape_err(label, line, "owns is an inline list: `[a, b]`"))?;
                owns = inner
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect();
            }
            "facts" => {
                let inner = value
                    .trim()
                    .strip_prefix('[')
                    .and_then(|v| v.strip_suffix(']'))
                    .ok_or_else(|| {
                        shape_err(label, line, "facts is an inline list: `[<slug>@<digest>]`")
                    })?;
                for entry in inner.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                    facts.push(
                        CoveringFact::parse(entry).map_err(|m| shape_err(label, line, &m))?,
                    );
                }
            }
            other => {
                return Err(shape_err(
                    label,
                    line,
                    &format!(
                        "unknown frontmatter key `{other}` — task files carry `node`, `owns` \
                         and `facts`"
                    ),
                ));
            }
        }
    }
    let Some(body_at) = body_at else {
        return Err(format!("`{label}`: unterminated frontmatter — no closing `---`"));
    };
    let Some(node) = node.filter(|n| !n.is_empty()) else {
        return Err(format!("`{label}`: the frontmatter names no `node`"));
    };

    let mut task = Task {
        id: id.to_string(),
        node,
        description: String::new(),
        spec_refs: Vec::new(),
        owns,
        facts,
        stack_details: String::new(),
        inputs: BTreeMap::new(),
        outputs: Vec::new(),
        verifications: BTreeMap::new(),
    };
    let mut desc_lines: Vec<&str> = Vec::new();
    let mut stack_lines: Vec<String> = Vec::new();
    let mut seen_h1 = false;
    let mut section: Option<&str> = None;
    let mut slug: Option<String> = None;
    // The bullet standing open: a heading — `## ` or the `### <slug>` of a
    // verification — ends the section the fold reads whole ([`fold_bullet`]).
    let mut open: Option<(usize, String)> = None;
    for (i, raw) in lines.iter().enumerate().skip(body_at) {
        let line = i + 1;
        if !seen_h1 {
            if raw.trim().is_empty() {
                continue;
            }
            if raw.starts_with("# ") {
                seen_h1 = true;
                continue;
            }
            return Err(shape_err(label, line, "a task file opens with `# t<N> — <node>`"));
        }
        if let Some(heading) = raw.trim_end().strip_prefix("## ") {
            if let Some((at, folded)) = open.take() {
                let sec = section.expect("a bullet stands inside a section");
                task_bullet(label, sec, slug.as_ref(), (at, &folded), &mut task, &mut stack_lines)?;
            }
            let heading = heading.trim();
            section = match heading {
                "Spec" | "Inputs" | "Outputs" | "Stack" | "Verifications" => Some(heading),
                other => {
                    return Err(format!(
                        "`{label}`: unknown section `## {other}` — task files carry Spec, \
                         Inputs, Outputs, Stack and Verifications"
                    ));
                }
            };
            slug = None;
            continue;
        }
        if section == Some("Verifications")
            && let Some(head) = raw.trim_end().strip_prefix("### ")
        {
            if let Some((at, folded)) = open.take() {
                task_bullet(
                    label,
                    "Verifications",
                    slug.as_ref(),
                    (at, &folded),
                    &mut task,
                    &mut stack_lines,
                )?;
            }
            let head = head.trim().to_string();
            // Owns is the curation; a proof for a requirement the task
            // never owned is structural, not advisory — own it first.
            if !task.owns.contains(&head) {
                return Err(format!(
                    "`{label}`: verification under `### {head}`, which the task does not \
                     own — own it first (`owns:` in the frontmatter)"
                ));
            }
            task.verifications.entry(head.clone()).or_default();
            slug = Some(head);
            continue;
        }
        match section {
            None => desc_lines.push(raw),
            Some(sec) => {
                if let Some((at, folded)) = fold_bullet(&mut open, line, raw) {
                    task_bullet(
                        label,
                        sec,
                        slug.as_ref(),
                        (at, &folded),
                        &mut task,
                        &mut stack_lines,
                    )?;
                }
            }
        }
    }
    if let Some((at, folded)) = open.take() {
        let sec = section.expect("a bullet stands inside a section");
        task_bullet(label, sec, slug.as_ref(), (at, &folded), &mut task, &mut stack_lines)?;
    }
    if !seen_h1 {
        return Err(format!("`{label}`: a task file opens with `# t<N> — <node>`"));
    }
    task.description = join_prose(&desc_lines);
    task.stack_details = stack_lines.join("\n");
    Ok(task)
}

// ---- loading and minting -----------------------------------------------------

/// Load a record folder into the one [`Plan`] every read already serves.
/// Errors carry the file they rose from; nothing is rewritten.
pub(crate) fn load(root: &Path, name: &str) -> Result<Plan, String> {
    let dir = plan_dir(root, name);
    let charter = charter_path(root, name);
    let text = fs::read_to_string(&charter)
        .map_err(|e| format!("cannot read `{}`: {e}", charter.display()))?;
    let (problem, technology_stack, architecture_summary, stack_mapping) =
        parse_charter(&charter.display().to_string(), &text)?;

    // Task files, sorted by ordinal; two files claiming one ordinal is an
    // identity collision — refuse naming both, the author picks.
    let mut names: Vec<String> = fs::read_dir(&dir)
        .map_err(|e| format!("cannot read `{}`: {e}", dir.display()))?
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut by_ordinal: BTreeMap<usize, (String, Task)> = BTreeMap::new();
    for file in names {
        let Some(ord) = task_ordinal(&file) else {
            continue;
        };
        if let Some((first, _)) = by_ordinal.get(&ord) {
            return Err(format!(
                "duplicate task id `t{ord}` — `{first}` and `{file}` both carry it"
            ));
        }
        let path = dir.join(&file);
        let text = fs::read_to_string(&path)
            .map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
        let task = parse_task(&path.display().to_string(), &format!("t{ord}"), &text)?;
        by_ordinal.insert(ord, (file, task));
    }

    let state = load_state(root, name)?;
    Ok(Plan {
        name: name.to_string(),
        version: state.version,
        version_hash: state.version_hash,
        created: state.created,
        state: state.state,
        closed_waves: state.closed_waves,
        problem,
        technology_stack,
        architecture_summary,
        stack_mapping,
        scenarios: Vec::new(),
        cleanup_displayed: state.cleanup_displayed,
        scenarios_displayed: state.scenarios_displayed,
        scenarios_closed: state.scenarios_closed,
        minted_after_the_world: state.minted_after_the_world,
        tasks: by_ordinal.into_values().map(|(_, t)| t).collect(),
    })
}

/// Mint a fresh record plan: the charter skeleton plus the lifecycle file
/// — every prose slot empty for the author to fill. No `scenarios.md`: the
/// plan authors no stories, it collects them from the world at its close.
pub(crate) fn mint(
    root: &Path,
    name: &str,
    version: String,
    version_hash: Option<String>,
    created: String,
) -> Result<Plan, String> {
    let plan = Plan {
        name: name.to_string(),
        version,
        version_hash,
        created,
        state: PlanState::Draft,
        closed_waves: 0,
        problem: String::new(),
        technology_stack: Vec::new(),
        architecture_summary: Vec::new(),
        stack_mapping: Vec::new(),
        scenarios: Vec::new(),
        cleanup_displayed: false,
        scenarios_displayed: false,
        scenarios_closed: false,
        minted_after_the_world: true,
        tasks: Vec::new(),
    };
    let dir = plan_dir(root, name);
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create `{}`: {e}", dir.display()))?;
    let path = charter_path(root, name);
    fs::write(&path, render_charter(&plan))
        .map_err(|e| format!("cannot write `{}`: {e}", path.display()))?;
    write_state(root, &plan)?;
    Ok(plan)
}

/// Write one task file under its minted name.
pub(crate) fn write_task(root: &Path, name: &str, task: &Task) -> Result<PathBuf, String> {
    let path = plan_dir(root, name).join(task_file_name(task));
    fs::write(&path, render_task(task))
        .map_err(|e| format!("cannot write `{}`: {e}", path.display()))?;
    Ok(path)
}

/// Move one task file's `facts` line and nothing else — the re-resolution
/// `plan repin` performs. The rest of the file is the author's and is left
/// line for line; an unchanged list writes nothing at all.
pub(crate) fn write_facts(
    root: &Path,
    name: &str,
    id: &str,
    facts: &[CoveringFact],
) -> Result<(), String> {
    let path =
        task_path(root, name, id).ok_or_else(|| format!("no file carries `{id}`"))?;
    let text =
        fs::read_to_string(&path).map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
    let mut out: Vec<String> = Vec::new();
    let mut in_frontmatter = false;
    for (i, raw) in text.lines().enumerate() {
        if i == 0 {
            in_frontmatter = raw.trim_end() == "---";
        } else if in_frontmatter {
            // The machine's line is rewritten, not edited around.
            if raw.trim_start().starts_with("facts:") {
                continue;
            }
            if raw.trim_end() == "---" {
                if !facts.is_empty() {
                    out.push(facts_line(facts));
                }
                in_frontmatter = false;
            }
        }
        out.push(raw.to_string());
    }
    let mut fresh = out.join("\n");
    fresh.push('\n');
    if fresh == text {
        return Ok(());
    }
    fs::write(&path, fresh).map_err(|e| format!("cannot write `{}`: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-records-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn task() -> Task {
        Task {
            id: "t2".into(),
            node: "Auth.Gate".into(),
            description: "guard the door\n\nand keep the log".into(),
            spec_refs: vec!["Auth.Gate".into(), "Gate.out wire Auth.inn".into()],
            owns: vec!["gate-throughput".into(), "service-hardening".into()],
            facts: vec![CoveringFact {
                fact: "riders-lose-the-signal".into(),
                digest: "9f3ab1".into(),
            }],
            stack_details: "axum 0.7\ntower layers".into(),
            inputs: [("t1".to_string(), "the store api".to_string())].into(),
            outputs: vec!["code/auth.rs".into()],
            verifications: [(
                "gate-throughput".to_string(),
                vec!["test — burst returns 429".to_string()],
            )]
            .into(),
        }
    }

    fn empty_plan(name: &str) -> Plan {
        Plan {
            name: name.to_string(),
            version: "v0001".into(),
            version_hash: None,
            created: "now".into(),
            state: PlanState::Draft,
            closed_waves: 0,
            problem: String::new(),
            technology_stack: Vec::new(),
            architecture_summary: Vec::new(),
            stack_mapping: Vec::new(),
            scenarios: Vec::new(),
            cleanup_displayed: false,
            scenarios_displayed: false,
            scenarios_closed: false,
            minted_after_the_world: true,
            tasks: Vec::new(),
        }
    }

    #[test]
    fn the_charter_round_trips() {
        let mut plan = empty_plan("mvp");
        plan.problem = "a tiny hardened store\n\nwith an audit trail".into();
        plan.technology_stack = vec![
            TechChoice { tech: "Rust".into(), provenance: "user choice".into() },
            TechChoice { tech: "sqlite".into(), provenance: String::new() },
        ];
        plan.architecture_summary =
            vec![SummaryLine { node: "Store".into(), role: "keeps the rows".into() }];
        plan.stack_mapping =
            vec![StackMapping { tech: "sqlite".into(), node: "Store".into() }];
        let text = render_charter(&plan);
        let (problem, stack, summary, mapping) = parse_charter("c", &text).unwrap();
        assert_eq!(problem, plan.problem);
        assert_eq!(stack, plan.technology_stack);
        assert_eq!(summary, plan.architecture_summary);
        assert_eq!(mapping, plan.stack_mapping);

        // The empty skeleton parses to empty fields.
        plan.problem = String::new();
        plan.technology_stack = Vec::new();
        plan.architecture_summary = Vec::new();
        plan.stack_mapping = Vec::new();
        let skeleton = render_charter(&plan);
        assert_eq!(skeleton, "# mvp\n\n## Stack\n\n## Architecture\n");
        let (problem, stack, summary, mapping) = parse_charter("c", &skeleton).unwrap();
        assert!(problem.is_empty() && stack.is_empty() && summary.is_empty() && mapping.is_empty());

        // Shape errors: an unknown section, a shapeless bullet.
        let err = parse_charter("c", "# mvp\n\n## Extras\n").unwrap_err();
        assert!(err.contains("unknown section `## Extras`"), "{err}");
        let err = parse_charter("c", "# mvp\n\n## Stack\n\nprose\n").unwrap_err();
        assert!(err.contains("line 5"), "{err}");
        let err = parse_charter("c", "## Stack\n").unwrap_err();
        assert!(err.contains("opens with `# <name>`"), "{err}");
    }

    #[test]
    fn a_task_file_round_trips() {
        let task = task();
        let text = render_task(&task);
        assert!(text.contains("facts: [riders-lose-the-signal@9f3ab1]"), "{text}");
        let parsed = parse_task("t", "t2", &text).unwrap();
        assert_eq!(parsed, task);

        // The skeleton: empty slots keep their headings, owns is `[]`, and
        // a node no fact covers carries no `facts` line at all.
        let bare = Task {
            id: "t1".into(),
            node: "Store".into(),
            description: String::new(),
            spec_refs: vec!["Store".into()],
            owns: Vec::new(),
            facts: Vec::new(),
            stack_details: String::new(),
            inputs: BTreeMap::new(),
            outputs: Vec::new(),
            verifications: BTreeMap::new(),
        };
        let text = render_task(&bare);
        assert!(text.contains("owns: []"), "{text}");
        assert!(!text.contains("facts:"), "{text}");
        assert!(text.contains("\n## Verifications\n"), "{text}");
        assert_eq!(parse_task("t", "t1", &text).unwrap(), bare);
        assert_eq!(task_file_name(&task), "t2-auth-gate.md");

        // A shapeless fact entry refuses with the shape.
        let text = render_task(&task).replace("@9f3ab1", "");
        let err = parse_task("t", "t2", &text).unwrap_err();
        assert!(err.contains("`<fact-slug>@<digest>`"), "{err}");
    }

    /// The `facts` line moves alone: the rest of an authored file is left
    /// exactly as its author wrote it, and an unchanged list writes nothing.
    #[test]
    fn write_facts_moves_the_line_and_nothing_else() {
        let root = temp_dir();
        mint(&root, "mvp", "v0001".into(), None, "now".into()).unwrap();
        let mut task = task();
        task.id = "t1".into();
        let path = write_task(&root, "mvp", &task).unwrap();
        let authored = fs::read_to_string(&path).unwrap() + "\nhand-written tail\n";
        fs::write(&path, &authored).unwrap();

        // A different list rewrites the one line.
        let moved = vec![CoveringFact { fact: "tunnels-run-long".into(), digest: "abc123".into() }];
        write_facts(&root, "mvp", "t1", &moved).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("facts: [tunnels-run-long@abc123]"), "{after}");
        assert!(!after.contains("riders-lose-the-signal"), "{after}");
        assert_eq!(
            after.replace("tunnels-run-long@abc123", "riders-lose-the-signal@9f3ab1"),
            authored,
            "only the one line moved"
        );

        // The same list writes nothing; an empty one drops the line.
        write_facts(&root, "mvp", "t1", &moved).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), after);
        write_facts(&root, "mvp", "t1", &[]).unwrap();
        let bare = fs::read_to_string(&path).unwrap();
        assert!(!bare.contains("facts:"), "{bare}");
        assert!(bare.contains("hand-written tail"), "{bare}");

        // A file that carries no line yet gets one — the world reaching a
        // node it did not reach before.
        write_facts(&root, "mvp", "t1", &moved).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), after);

        fs::remove_dir_all(&root).unwrap();
    }

    /// A charter bullet may run onto further lines, wrapped where the
    /// author's editor wrapped it: the section splits at the lines that open
    /// with `- ` and each piece collapses to one line
    /// (`archi/requirements/planning/a-record-bullet-may-wrap.md`).
    #[test]
    fn a_charter_bullet_may_wrap() {
        let text = "# mvp\n\nthe problem\nover two lines\n\n\
                    ## Stack\n\n\
                    - Rust —\n  user choice\n\n\
                    - sqlite\n\n\
                    ## Architecture\n\n\
                    - `Store` — keeps\n  the rows\n\
                    - `Store` realizes\n  sqlite\n";
        let (problem, stack, summary, mapping) = parse_charter("c", text).unwrap();
        // Prose keeps its line breaks; the bullets lose theirs.
        assert_eq!(problem, "the problem\nover two lines");
        assert_eq!(
            stack,
            vec![
                TechChoice { tech: "Rust".into(), provenance: "user choice".into() },
                TechChoice { tech: "sqlite".into(), provenance: String::new() },
            ]
        );
        assert_eq!(
            summary,
            vec![SummaryLine { node: "Store".into(), role: "keeps the rows".into() }]
        );
        assert_eq!(mapping, vec![StackMapping { tech: "sqlite".into(), node: "Store".into() }]);

        // A section that opens on prose still refuses, at the line the piece
        // opened on and not at the line that continues it.
        let err = parse_charter("c", "# mvp\n\n## Stack\n\nprose\n  and more prose\n").unwrap_err();
        assert!(err.contains("line 5"), "{err}");
    }

    /// The same fold in every bullet section a task file carries, over two
    /// lines, over three, and over a blank line inside one bullet
    /// (`archi/requirements/planning/a-record-bullet-may-wrap.md`).
    #[test]
    fn every_bullet_section_of_a_task_may_wrap() {
        let text = "---\nnode: Store\nowns: [store-encrypted]\n---\n\n\
                    # t1 — Store\n\n\
                    persist rows\nover two lines\n\n\
                    ## Spec\n\n\
                    - `Auth.creds wire\n  Store.inn`\n\n\
                    - `Store`\n\n\
                    ## Inputs\n\n\
                    - from t2 — the store api\n  the gate calls\n\n\
                    ## Outputs\n\n\
                    - crates/archi/src/plans/records.rs,\n  crates/archi/src/plans/mod.rs\n\n\
                    ## Stack\n\n\
                    - axum 0.7, the\n\n  tower layer\n\n  it rides on\n\n\
                    ## Verifications\n\n\
                    ### store-encrypted\n\n\
                    - test — a row written\n  through the gate\n  comes back sealed\n";
        let task = parse_task("t", "t1", text).unwrap();
        assert_eq!(task.description, "persist rows\nover two lines");
        assert_eq!(task.spec_refs, ["Auth.creds wire Store.inn", "Store"]);
        assert_eq!(task.inputs["t2"], "the store api the gate calls");
        // The outputs section is opaque strings to the parser: the fold is
        // what this bullet proves, not the shape of a path.
        assert_eq!(
            task.outputs,
            ["crates/archi/src/plans/records.rs, crates/archi/src/plans/mod.rs"]
        );
        assert_eq!(task.stack_details, "axum 0.7, the tower layer it rides on");
        assert_eq!(
            task.verifications["store-encrypted"],
            ["test — a row written through the gate comes back sealed"]
        );

        // A blank line between bullets changes nothing, and a bullet
        // refused is refused at the line it opened on.
        let spaced = text.replace("- `Store`\n", "\n- `Store`\n\n");
        assert_eq!(parse_task("t", "t1", &spaced).unwrap(), task);
        let bad = text.replace("- `Auth.creds wire\n", "- Auth.creds wire\n");
        let err = parse_task("t", "t1", &bad).unwrap_err();
        assert!(err.contains("line 13"), "{err}");
        assert!(err.contains("backtick-wrapped"), "{err}");
    }

    /// The fold changes nothing about the records this repository stands on:
    /// every bullet the parser produces from them is one line of its own file
    /// (`archi/requirements/planning/a-record-bullet-may-wrap.md`).
    #[test]
    fn every_record_standing_in_this_repository_parses_as_it_did() {
        let plans = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../archi/plans");
        let mut read = 0;
        for plan in fs::read_dir(&plans).unwrap().filter_map(Result::ok) {
            let name = plan.file_name().to_string_lossy().into_owned();
            if !plan.path().is_dir() {
                continue;
            }
            for entry in fs::read_dir(plan.path()).unwrap().filter_map(Result::ok) {
                let file = entry.file_name().to_string_lossy().into_owned();
                if !file.ends_with(".md") {
                    continue;
                }
                let label = format!("archi/plans/{name}/{file}");
                let text = fs::read_to_string(entry.path()).unwrap();
                let rendered = if let Some(ord) = task_ordinal(&file) {
                    let task = parse_task(&label, &format!("t{ord}"), &text)
                        .unwrap_or_else(|e| panic!("{e}"));
                    render_task(&task)
                } else if file == format!("{name}.md") {
                    let (problem, stack, summary, mapping) =
                        parse_charter(&label, &text).unwrap_or_else(|e| panic!("{e}"));
                    let mut plan = empty_plan(&name);
                    plan.problem = problem;
                    plan.technology_stack = stack;
                    plan.architecture_summary = summary;
                    plan.stack_mapping = mapping;
                    render_charter(&plan)
                } else {
                    // A `scenarios.md` an old plan was written with is read
                    // by nobody, here as anywhere else.
                    continue;
                };
                let lines: std::collections::BTreeSet<&str> =
                    text.lines().map(str::trim_end).collect();
                for bullet in rendered.lines().filter(|l| l.starts_with("- ")) {
                    assert!(lines.contains(bullet), "`{label}` folded `{bullet}`");
                }
                read += 1;
            }
        }
        assert!(read > 40, "the records were not read: {read} files");
    }

    #[test]
    fn task_shape_errors_carry_the_file() {
        // An unknown section names the file and the heading.
        let text = render_task(&task()).replace("## Stack", "## Extras");
        let err = parse_task("archi/plans/mvp/t2-auth-gate.md", "t2", &text).unwrap_err();
        assert!(err.contains("t2-auth-gate.md"), "{err}");
        assert!(err.contains("unknown section `## Extras`"), "{err}");

        // A verification under an unowned slug is structural.
        let text = render_task(&task()).replace("### gate-throughput", "### ghost-req");
        let err = parse_task("t", "t2", &text).unwrap_err();
        assert!(err.contains("`### ghost-req`"), "{err}");
        assert!(err.contains("own it first"), "{err}");

        // Frontmatter: unknown keys refuse, `node` is required.
        let text = render_task(&task()).replace("node:", "extra: x\nnode:");
        let err = parse_task("t", "t2", &text).unwrap_err();
        assert!(err.contains("unknown frontmatter key `extra`"), "{err}");
        assert!(err.contains("`facts`"), "{err}");
        let err = parse_task("t", "t1", "---\nowns: []\n---\n\n# t1 — X\n").unwrap_err();
        assert!(err.contains("names no `node`"), "{err}");
        let err = parse_task("t", "t1", "# t1 — X\n").unwrap_err();
        assert!(err.contains("opens with `---`"), "{err}");
    }

    #[test]
    fn state_json_refuses_drift() {
        // The latch-less shape an old binary wrote parses — the latches
        // default unflipped; a flipped cleanup latch parses too. The world
        // mark defaults with them: a plan from before the world is one.
        let ok = r#"{"state":"draft","closed_waves":0,"version":"v0001","created":"now"}"#;
        assert!(serde_json::from_str::<StateFile>(ok).is_ok());
        assert!(!serde_json::from_str::<StateFile>(ok).unwrap().minted_after_the_world);
        let latched = r#"{"state":"started","closed_waves":1,"version":"v0001","created":"now","cleanup_displayed":true}"#;
        assert!(serde_json::from_str::<StateFile>(latched).unwrap().cleanup_displayed);
        let unknown = r#"{"state":"draft","closed_waves":0,"version":"v0001","created":"now","extra":1}"#;
        let err = serde_json::from_str::<StateFile>(unknown).err().unwrap().to_string();
        assert!(err.contains("extra"), "{err}");
        let bad_state = r#"{"state":"paused","closed_waves":0,"version":"v0001","created":"now"}"#;
        assert!(serde_json::from_str::<StateFile>(bad_state).is_err());
    }

    #[test]
    fn a_folder_loads_whole_and_duplicate_ordinals_refuse() {
        let root = temp_dir();
        let plan = mint(&root, "mvp", "v0001".into(), None, "now".into()).unwrap();
        assert_eq!(load(&root, "mvp").unwrap(), plan);

        // The mint stamps the world and writes no story block of its own.
        assert!(plan.minted_after_the_world);
        assert!(!plan_dir(&root, "mvp").join("scenarios.md").exists());

        let mut task = task();
        task.id = "t1".into();
        write_task(&root, "mvp", &task).unwrap();
        let loaded = load(&root, "mvp").unwrap();
        assert_eq!(loaded.tasks, vec![task.clone()]);

        // A second file claiming t1 refuses naming both.
        let dupe = plan_dir(&root, "mvp").join("t1-zzz.md");
        fs::copy(plan_dir(&root, "mvp").join(task_file_name(&task)), &dupe).unwrap();
        let err = load(&root, "mvp").unwrap_err();
        assert!(err.contains("duplicate task id `t1`"), "{err}");
        assert!(err.contains("t1-auth-gate.md") && err.contains("t1-zzz.md"), "{err}");
        fs::remove_file(&dupe).unwrap();

        // Lifecycle lives in state.json alone; losing it is fatal.
        fs::remove_file(plan_dir(&root, "mvp").join("state.json")).unwrap();
        let err = load(&root, "mvp").unwrap_err();
        assert!(err.contains("state.json"), "{err}");

        fs::remove_dir_all(&root).unwrap();
    }
}
