//! The record form of a plan: a folder of markdown files with lifecycle
//! apart in `state.json`
//! (`archi/requirements/planning/a-plan-is-a-folder-of-records.md`).
//!
//! Content is the files — there is no write command for prose. The charter
//! `<name>.md` carries the envelope: problem prose, `## Stack` bullets
//! with provenance, `## Architecture` bullets for summary lines and stack
//! mappings. Each task is `t<N>-<node-slug>.md`: `node` and hand-curated
//! `owns` in the frontmatter, description prose, then `## Spec`,
//! `## Inputs`, `## Outputs`, `## Stack` bullets and `## Verifications`
//! keyed by owned slug. `scenarios.md` is a bullet list. `state.json`
//! alone moves through commands — the mint writes it, `save_state` rewrites
//! it, and nothing else in the folder is machine-written past its mint.
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

use super::{Plan, PlanState, StackMapping, SummaryLine, Task, TechChoice, plan_dir};
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

fn scenarios_path(root: &Path, name: &str) -> PathBuf {
    plan_dir(root, name).join("scenarios.md")
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

/// One task file: frontmatter, description prose, the bullet sections.
/// Empty sections keep their heading — the slots the author fills.
pub(crate) fn render_task(task: &Task) -> String {
    let mut out = format!("---\nnode: {}\nowns: [{}]\n---\n", task.node, task.owns.join(", "));
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

/// `scenarios.md`: a heading and one bullet per scenario.
pub(crate) fn render_scenarios(scenarios: &[String]) -> String {
    let mut out = String::from("# Scenarios\n");
    if !scenarios.is_empty() {
        out.push('\n');
        for s in scenarios {
            out.push_str(&format!("- {s}\n"));
        }
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
            Some(_) if raw.trim().is_empty() => {}
            Some("Stack") => {
                let b = bullet(label, line, raw, "stack bullets are `- <tech> — <provenance>`")?;
                let (tech, provenance) = b.split_once(" — ").unwrap_or((b, ""));
                stack.push(TechChoice {
                    tech: tech.trim().to_string(),
                    provenance: provenance.trim().to_string(),
                });
            }
            Some(_) => {
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
            }
        }
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

/// One task file. The id arrives from the file name — the `t<N>-` prefix
/// is the identity; frontmatter carries the node and the curated owns.
fn parse_task(label: &str, id: &str, text: &str) -> Result<Task, String> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.first().map(|l| l.trim_end()) != Some("---") {
        return Err(format!("`{label}`: a task file opens with `---` frontmatter"));
    }
    let mut node: Option<String> = None;
    let mut owns: Vec<String> = Vec::new();
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
            other => {
                return Err(shape_err(
                    label,
                    line,
                    &format!("unknown frontmatter key `{other}` — task files carry `node` and `owns`"),
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
            Some(_) if raw.trim().is_empty() => {}
            Some("Spec") => {
                let b = bullet(label, line, raw, "spec bullets are `- `<ref>``")?;
                let (r, rest) = backticked(b)
                    .ok_or_else(|| shape_err(label, line, "spec refs are backtick-wrapped"))?;
                if !rest.trim().is_empty() {
                    return Err(shape_err(label, line, "spec bullets carry one ref and nothing else"));
                }
                task.spec_refs.push(r);
            }
            Some("Inputs") => {
                let b = bullet(label, line, raw, "input bullets are `- from <task> — <note>`")?;
                let b = b.strip_prefix("from ").ok_or_else(|| {
                    shape_err(label, line, "input bullets are `- from <task> — <note>`")
                })?;
                let (from, note) = b.split_once(" — ").unwrap_or((b, ""));
                task.inputs.insert(from.trim().to_string(), note.trim().to_string());
            }
            Some("Outputs") => {
                let b = bullet(label, line, raw, "output bullets are `- <path>`")?;
                task.outputs.push(b.to_string());
            }
            Some("Stack") => {
                let b = bullet(label, line, raw, "stack bullets are `- <detail>`")?;
                stack_lines.push(b.to_string());
            }
            Some(_) => {
                let b = bullet(label, line, raw, "verifications ride under a `### <slug>`")?;
                let Some(slug) = &slug else {
                    return Err(shape_err(label, line, "verifications ride under a `### <slug>`"));
                };
                task.verifications.get_mut(slug).expect("opened above").push(b.to_string());
            }
        }
    }
    if !seen_h1 {
        return Err(format!("`{label}`: a task file opens with `# t<N> — <node>`"));
    }
    task.description = join_prose(&desc_lines);
    task.stack_details = stack_lines.join("\n");
    Ok(task)
}

/// `scenarios.md`: a heading, then bullets — nothing else.
fn parse_scenarios(label: &str, text: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut seen_h1 = false;
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        if raw.trim().is_empty() {
            continue;
        }
        if !seen_h1 {
            if raw.starts_with("# ") {
                seen_h1 = true;
                continue;
            }
            return Err(shape_err(label, line, "scenarios open with `# Scenarios`"));
        }
        let b = bullet(label, line, raw, "scenarios are `- <text>` bullets")?;
        out.push(b.to_string());
    }
    Ok(out)
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

    let scenarios_file = scenarios_path(root, name);
    let scenarios = if scenarios_file.exists() {
        let text = fs::read_to_string(&scenarios_file)
            .map_err(|e| format!("cannot read `{}`: {e}", scenarios_file.display()))?;
        parse_scenarios(&scenarios_file.display().to_string(), &text)?
    } else {
        Vec::new()
    };

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
        scenarios,
        cleanup_displayed: state.cleanup_displayed,
        scenarios_displayed: state.scenarios_displayed,
        scenarios_closed: state.scenarios_closed,
        tasks: by_ordinal.into_values().map(|(_, t)| t).collect(),
    })
}

/// Mint a fresh record plan: the charter and scenarios skeletons plus the
/// lifecycle file — every prose slot empty for the author to fill.
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
        tasks: Vec::new(),
    };
    let dir = plan_dir(root, name);
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create `{}`: {e}", dir.display()))?;
    let write = |path: PathBuf, text: String| {
        fs::write(&path, text).map_err(|e| format!("cannot write `{}`: {e}", path.display()))
    };
    write(charter_path(root, name), render_charter(&plan))?;
    write(scenarios_path(root, name), render_scenarios(&plan.scenarios))?;
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
        let parsed = parse_task("t", "t2", &text).unwrap();
        assert_eq!(parsed, task);

        // The skeleton: empty slots keep their headings, owns is `[]`.
        let bare = Task {
            id: "t1".into(),
            node: "Store".into(),
            description: String::new(),
            spec_refs: vec!["Store".into()],
            owns: Vec::new(),
            stack_details: String::new(),
            inputs: BTreeMap::new(),
            outputs: Vec::new(),
            verifications: BTreeMap::new(),
        };
        let text = render_task(&bare);
        assert!(text.contains("owns: []"), "{text}");
        assert!(text.contains("\n## Verifications\n"), "{text}");
        assert_eq!(parse_task("t", "t1", &text).unwrap(), bare);
        assert_eq!(task_file_name(&task), "t2-auth-gate.md");
    }

    #[test]
    fn scenarios_round_trip() {
        let scenarios = vec!["a user logs in".to_string(), "a row survives a restart".to_string()];
        let text = render_scenarios(&scenarios);
        assert_eq!(parse_scenarios("s", &text).unwrap(), scenarios);
        assert_eq!(parse_scenarios("s", &render_scenarios(&[])).unwrap(), Vec::<String>::new());
        let err = parse_scenarios("s", "# Scenarios\n\nprose\n").unwrap_err();
        assert!(err.contains("`- <text>` bullets"), "{err}");
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
        let err = parse_task("t", "t1", "---\nowns: []\n---\n\n# t1 — X\n").unwrap_err();
        assert!(err.contains("names no `node`"), "{err}");
        let err = parse_task("t", "t1", "# t1 — X\n").unwrap_err();
        assert!(err.contains("opens with `---`"), "{err}");
    }

    #[test]
    fn state_json_refuses_drift() {
        // The latch-less shape an old binary wrote parses — the latches
        // default unflipped; a flipped cleanup latch parses too.
        let ok = r#"{"state":"draft","closed_waves":0,"version":"v0001","created":"now"}"#;
        assert!(serde_json::from_str::<StateFile>(ok).is_ok());
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
