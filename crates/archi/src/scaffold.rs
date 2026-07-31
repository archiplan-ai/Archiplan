//! `archi init` — the cold start (`archi/requirements/cold-start/`,
//! `archi/requirements/cli/`): stand a directory up as an archiplan project.
//!
//! Create-only per artifact: what exists is read and reported — `ok` when
//! it matches what init would write, `kept` when it differs — and never
//! rewritten (`init-changes-nothing-twice`). The manifest lands last, so a
//! project root only ever appears over a whole tree, and an interrupted
//! run completes on re-run. An existing manifest routes the starter
//! through the compiler's own reader (`init-honors-the-manifest`); the
//! briefing — the workflow skills and the fenced CLAUDE.md block — is an
//! init artifact (`the-agent-arrives-briefed`).

use std::fs;
use std::path::{Path, PathBuf};

use modeling_lang::source::{find_project_root, manifest_src};

/// The briefing, embedded at build time: skill name → SKILL.md text.
const SKILLS: [(&str, &str); 7] = [
    ("archi", include_str!("../../../skills/archi.md")),
    ("archi-plan", include_str!("../../../skills/archi-plan.md")),
    ("archi-implement", include_str!("../../../skills/archi-implement.md")),
    ("archi-merge", include_str!("../../../skills/archi-merge.md")),
    (
        "archi-finish-worktree",
        include_str!("../../../skills/archi-finish-worktree.md"),
    ),
    (
        "archi-migrate-fractal",
        include_str!("../../../skills/archi-migrate-fractal.md"),
    ),
    (("ste-writing"), include_str!("../../../skills/ste-writing.md")),
];

const FENCE_OPEN: &str = "<!-- archi:begin -->";
const FENCE_CLOSE: &str = "<!-- archi:end -->";
const DEFAULT_SRC: &str = "archi/src";

/// One artifact's fate, in emission order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    /// Absent, now written.
    Created,
    /// A CLAUDE.md that had no fence gained the block; nothing else moved.
    Appended,
    /// Present and different — overwritten with the binary's copy. Only
    /// `sync-skills` produces this; `init` is create-only.
    Updated,
    /// Present and already what init would write.
    Ok,
    /// Present and different — read, reported, left alone.
    Kept,
}

/// One report line: an act, the artifact's project-relative path, the
/// one-clause why.
#[derive(Debug)]
pub struct Step {
    pub act: Act,
    pub path: String,
    pub detail: Option<String>,
}

/// What one init did — the report `render` prints.
#[derive(Debug)]
pub struct Outcome {
    /// The project name the manifest carries (or would).
    pub name: String,
    /// An enclosing project root, when the target nests under one.
    pub enclosing: Option<PathBuf>,
    /// Per-artifact steps, manifest last.
    pub steps: Vec<Step>,
}

impl Outcome {
    /// Whether this run wrote anything.
    pub fn fresh(&self) -> bool {
        self.steps
            .iter()
            .any(|s| matches!(s.act, Act::Created | Act::Appended))
    }
}

/// Stand `target` up as an archiplan project. Reads before it writes,
/// writes only what is missing, and orders the writes so every
/// intermediate tree is honest: the manifest — the marker every other
/// command keys on — lands last.
pub fn init(target: &Path) -> Result<Outcome, String> {
    fs::create_dir_all(target)
        .map_err(|e| format!("cannot create {}: {e}", target.display()))?;
    let target = target
        .canonicalize()
        .map_err(|e| format!("cannot resolve {}: {e}", target.display()))?;

    // Read everything load-bearing before the first write: a manifest that
    // fails to parse aborts an untouched tree.
    let manifest = target.join("archi.toml");
    let src = if manifest.is_file() {
        manifest_src(&target).map_err(|d| d.message)?
    } else {
        DEFAULT_SRC.to_string()
    };
    let name = project_name(&target);
    let enclosing = target.parent().and_then(find_project_root);

    let mut steps = Vec::new();

    // The starter module — only when the source dir holds no module yet.
    let src_dir = target.join(&src);
    if holds_arch(&src_dir) {
        steps.push(Step {
            act: Act::Ok,
            path: src.clone(),
            detail: Some("modules present".into()),
        });
    } else {
        let path = src_dir.join("model.arch");
        write_new(&path, &starter(&name))?;
        steps.push(step(Act::Created, &target, &path, None));
    }

    // The briefing: every skill, verbatim from the binary.
    for (skill, text) in SKILLS {
        let path = target.join(".claude/skills").join(skill).join("SKILL.md");
        let act = match fs::read_to_string(&path) {
            Err(_) => {
                write_new(&path, text)?;
                (Act::Created, None)
            }
            Ok(found) if found == text => (Act::Ok, Some("matches this binary's copy".into())),
            Ok(_) => (Act::Kept, Some("differs from this binary's copy".into())),
        };
        steps.push(step(act.0, &target, &path, act.1));
    }

    // The CLAUDE.md block: create, append once, or leave as found.
    let block = claude_block(&src);
    let claude = target.join("CLAUDE.md");
    let act = match fs::read_to_string(&claude) {
        Err(_) => {
            write_new(&claude, &format!("{block}\n"))?;
            (Act::Created, None)
        }
        Ok(found) => match fenced(&found) {
            Some(inner) if inner == block => (Act::Ok, Some("the archi block is present".into())),
            Some(_) => (Act::Kept, Some("its archi block differs from this binary's".into())),
            None => {
                let sep = if found.is_empty() || found.ends_with('\n') { "\n" } else { "\n\n" };
                fs::write(&claude, format!("{found}{sep}{block}\n"))
                    .map_err(|e| format!("cannot write {}: {e}", claude.display()))?;
                (Act::Appended, Some("the archi block".into()))
            }
        },
    };
    steps.push(step(act.0, &target, &claude, act.1));

    // The gitignore: worktree artifacts are machine-local and never merged —
    // create it, append the missing lines once, or leave it as found.
    let gitignore = target.join(".gitignore");
    let wanted = ["archi/*.local.toml", "archi/plans/.current"];
    let act = match fs::read_to_string(&gitignore) {
        Err(_) => {
            write_new(&gitignore, &format!("{}\n", wanted.join("\n")))?;
            (Act::Created, None)
        }
        Ok(found) => {
            let missing: Vec<&str> = wanted
                .into_iter()
                .filter(|w| !found.lines().any(|l| l.trim() == *w))
                .collect();
            if missing.is_empty() {
                (Act::Ok, Some("the archi lines are present".into()))
            } else {
                let sep = if found.is_empty() || found.ends_with('\n') { "" } else { "\n" };
                fs::write(&gitignore, format!("{found}{sep}{}\n", missing.join("\n")))
                    .map_err(|e| format!("cannot write {}: {e}", gitignore.display()))?;
                (Act::Appended, Some("the archi lines".into()))
            }
        }
    };
    steps.push(step(act.0, &target, &gitignore, act.1));

    // The manifest, last: a tree is a project only once it is whole.
    if manifest.is_file() {
        steps.push(step(Act::Ok, &target, &manifest, Some("present".into())));
    } else {
        // the one-worktree rule is on from birth: mutation runs in worktrees,
        // never in the primary checkout; deleting the line is the opt-out
        write_new(
            &manifest,
            &format!(
                "[project]\nname = \"{}\"\npreset = \"default\"\nprotected = [\"main\"]\n",
                toml_escape(&name)
            ),
        )?;
        steps.push(step(Act::Created, &target, &manifest, None));
    }

    Ok(Outcome {
        name,
        enclosing,
        steps,
    })
}

/// Sync an initialized tree's briefing to this binary's copies. Where `init`
/// is create-only, `sync-skills` is the deliberate command that reconciles a
/// project with a newer binary (`the-installed-skill-drifts`): a skill or
/// CLAUDE.md block that already matches is `ok`, an absent one is `created`,
/// and any divergent one is `updated` — overwritten with the binary's copy,
/// unconditionally. It never touches the model, only the briefing, so a
/// reflexive re-run cannot lose source; init stays create-only
/// (`a-re-run-clobbers-the-tree` stays answered). Requires the tree to be a
/// project already; the target is located, not created.
pub fn sync_skills(target: &Path) -> Result<Outcome, String> {
    let target = target
        .canonicalize()
        .map_err(|e| format!("cannot resolve {}: {e}", target.display()))?;

    let manifest = target.join("archi.toml");
    if !manifest.is_file() {
        return Err(format!(
            "{} is not an archiplan project (no archi.toml) — run `archi init` first",
            target.display()
        ));
    }
    let src = manifest_src(&target).map_err(|d| d.message)?;
    let name = project_name(&target);
    let enclosing = target.parent().and_then(find_project_root);

    let mut steps = Vec::new();

    // The briefing skills: overwrite any that has drifted from the embedded copy.
    for (skill, text) in SKILLS {
        let path = target.join(".claude/skills").join(skill).join("SKILL.md");
        let act = match fs::read_to_string(&path) {
            Err(_) => {
                write_new(&path, text)?;
                (Act::Created, None)
            }
            Ok(found) if found == text => (Act::Ok, Some("matches this binary's copy".into())),
            Ok(_) => {
                fs::write(&path, text)
                    .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
                (Act::Updated, Some("refreshed to this binary's copy".into()))
            }
        };
        steps.push(step(act.0, &target, &path, act.1));
    }

    // The CLAUDE.md block: the fence marks the one region sync may reclaim, so
    // its inner content is overwritten in place while the surrounding prose is
    // left untouched.
    let block = claude_block(&src);
    let claude = target.join("CLAUDE.md");
    let act = match fs::read_to_string(&claude) {
        Err(_) => {
            write_new(&claude, &format!("{block}\n"))?;
            (Act::Created, None)
        }
        Ok(found) => match fenced(&found) {
            Some(inner) if inner == block => (Act::Ok, Some("the archi block is present".into())),
            Some(_) => {
                fs::write(&claude, replace_fence(&found, &block))
                    .map_err(|e| format!("cannot write {}: {e}", claude.display()))?;
                (Act::Updated, Some("its archi block refreshed".into()))
            }
            None => {
                let sep = if found.is_empty() || found.ends_with('\n') { "\n" } else { "\n\n" };
                fs::write(&claude, format!("{found}{sep}{block}\n"))
                    .map_err(|e| format!("cannot write {}: {e}", claude.display()))?;
                (Act::Appended, Some("the archi block".into()))
            }
        },
    };
    steps.push(step(act.0, &target, &claude, act.1));

    Ok(Outcome {
        name,
        enclosing,
        steps,
    })
}

/// The report, one line per artifact plus the verdict.
pub fn render(o: &Outcome) -> String {
    let mut out = String::new();
    if let Some(root) = &o.enclosing {
        out.push_str(&format!(
            "note: an enclosing project sits at {} — below this init, the nearest manifest wins\n",
            root.display()
        ));
    }
    for s in &o.steps {
        let command = act_verb(s.act);
        match &s.detail {
            Some(d) => out.push_str(&format!("{command:<9}{} ({d})\n", s.path)),
            None => out.push_str(&format!("{command:<9}{}\n", s.path)),
        }
    }
    if o.fresh() {
        out.push_str(&format!("initialized `{}` — next: archi build\n", o.name));
    } else {
        out.push_str("already initialized — nothing to create\n");
    }
    out
}

/// The report for `sync-skills`: one line per artifact, then the verdict.
pub fn render_sync(o: &Outcome) -> String {
    let mut out = String::new();
    if let Some(root) = &o.enclosing {
        out.push_str(&format!(
            "note: an enclosing project sits at {} — below this sync, the nearest manifest wins\n",
            root.display()
        ));
    }
    for s in &o.steps {
        let command = act_verb(s.act);
        match &s.detail {
            Some(d) => out.push_str(&format!("{command:<9}{} ({d})\n", s.path)),
            None => out.push_str(&format!("{command:<9}{}\n", s.path)),
        }
    }
    let wrote = o
        .steps
        .iter()
        .filter(|s| matches!(s.act, Act::Created | Act::Appended | Act::Updated))
        .count();
    if wrote > 0 {
        out.push_str(&format!(
            "synced `{}` — the briefing now matches this binary\n",
            o.name
        ));
    } else {
        out.push_str("already in sync — the briefing matches this binary\n");
    }
    out
}

/// The report verb for one act.
fn act_verb(act: Act) -> &'static str {
    match act {
        Act::Created => "created",
        Act::Appended => "appended",
        Act::Updated => "updated",
        Act::Ok => "ok",
        Act::Kept => "kept",
    }
}

fn step(act: Act, root: &Path, path: &Path, detail: Option<String>) -> Step {
    Step {
        act,
        path: path
            .strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string(),
        detail,
    }
}

/// Write a file that `init` established is absent, parents included.
fn write_new(path: &Path, text: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
    }
    fs::write(path, text).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// Whether any `.arch` module lives under `dir`.
fn holds_arch(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|e| {
        let path = e.path();
        if path.is_dir() {
            holds_arch(&path)
        } else {
            path.extension().is_some_and(|x| x == "arch")
        }
    })
}

/// The manifest's project name: the directory's own, or `project` at a
/// filesystem root.
fn project_name(target: &Path) -> String {
    target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".to_string())
}

fn toml_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// The fenced archi region of a CLAUDE.md, markers included.
fn fenced(text: &str) -> Option<&str> {
    let start = text.find(FENCE_OPEN)?;
    let end = text[start..].find(FENCE_CLOSE)? + start + FENCE_CLOSE.len();
    Some(&text[start..end])
}

/// Swap the fenced archi region of `text` for `block`, leaving every byte
/// outside the fence — prose above and below — exactly where it was. The
/// caller has already confirmed the fence is present via `fenced`.
fn replace_fence(text: &str, block: &str) -> String {
    let start = text.find(FENCE_OPEN).expect("fence present");
    let end = text[start..].find(FENCE_CLOSE).expect("fence present") + start + FENCE_CLOSE.len();
    format!("{}{}{}", &text[..start], block, &text[end..])
}

/// The starter module: comment-only, so the model starts empty and the
/// syntax starts present.
fn starter(name: &str) -> String {
    format!(
        "// {name} — the model. One `.arch` file is one module; more files beside\n\
         // this one are more modules, composed by `import`.\n\
         //\n\
         // The shape of a module — nodes with ports, classified by a preset type:\n\
         //\n\
         // def node AuthService: // sheds the login burst before hashing\n\
         //   port handle_login\n\
         //\n\
         // Service type_of AuthService\n\
         //\n\
         // `archi check` compiles this tree and cross-checks the docs under\n\
         // `archi/`; the workflow lives in `.claude/skills/archi/SKILL.md`.\n"
    )
}

/// The CLAUDE.md block, fence included; `src` names the manifest's layout.
fn claude_block(src: &str) -> String {
    format!(
        "{FENCE_OPEN}\n\
         ## Archiplan\n\
         \n\
         For any architecture-related work — modeling systems, components,\n\
         relationships, requirements, specs, plans — use the `archi` CLI and its\n\
         skills; never design ad hoc in chat. `archi --help` lists the commands.\n\
         \n\
         This repository is modeled with archiplan: the spec is text under `archi/`,\n\
         the model is `.arch` source under `{src}/`, and lifecycle state moves only\n\
         through `archi` commands — never hand-edit `archi/versions/`, the link journal,\n\
         or `closed:` stamps.\n\
         \n\
         - After any model or doc edit run `archi check`: errors block, findings are\n\
         \x20 the worklist.\n\
         - Find anything by phrase: `archi search <phrase>` — ranked hits across\n\
         \x20 elements, intents, requirements, stressors, sessions and decisions,\n\
         \x20 each with its address.\n\
         - Spec work delegated to subagents or workflows returns as FILES written\n\
         \x20 under `archi/` (paths, not payloads): a finding that is not a file on\n\
         \x20 disk does not exist, and every fan-out is gated by `archi check` plus\n\
         \x20 a count of the files it claims to have written.\n\
         - The spec workflow (model, stress, version) is the `archi` skill in\n\
         \x20 `.claude/skills/archi/`; authoring a plan is `archi-plan`; executing\n\
         \x20 it in waves is `archi-implement`; closing a worktree is\n\
         \x20 `archi-finish-worktree`; merging parallel spec work is `archi-merge`,\n\
         \x20 and importing a project from the old fractal client is\n\
         \x20 `archi-migrate-fractal`.\n\
         \n\
         No silent assumptions: state what you assume, surface the tradeoffs.\n\
         Minimal design that solves the problem — no speculative features.\n\
         For any prose text you're writing about architecture, use ASD-STE100 Simplified Technical English.\n\
         {FENCE_CLOSE}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "archi-scaffold-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Every file under `dir` with its bytes, path-sorted.
    fn snapshot(dir: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        fn walk(dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
            for e in fs::read_dir(dir).unwrap().flatten() {
                let path = e.path();
                if path.is_dir() {
                    walk(&path, out);
                } else {
                    let bytes = fs::read(&path).unwrap();
                    out.push((path, bytes));
                }
            }
        }
        let mut out = Vec::new();
        walk(dir, &mut out);
        out.sort();
        out
    }

    fn acts(o: &Outcome) -> Vec<(&'static str, String)> {
        o.steps
            .iter()
            .map(|s| (act_verb(s.act), s.path.clone()))
            .collect()
    }

    #[test]
    fn a_fresh_emission_creates_everything_manifest_last() {
        let dir = temp_dir();
        let o = init(&dir).unwrap();
        let acts = acts(&o);
        assert!(acts.iter().all(|(v, _)| *v == "created"), "{acts:?}");
        assert_eq!(acts.last().unwrap().1, "archi.toml");
        assert_eq!(acts[0].1, "archi/src/model.arch");
        // The starter is comment-only: the model starts empty.
        let starter = fs::read_to_string(dir.join("archi/src/model.arch")).unwrap();
        assert!(
            starter
                .lines()
                .filter(|l| !l.trim().is_empty())
                .all(|l| l.starts_with("//")),
            "{starter}"
        );
        // The briefing is verbatim.
        for (skill, text) in SKILLS {
            let installed =
                fs::read_to_string(dir.join(".claude/skills").join(skill).join("SKILL.md"))
                    .unwrap();
            assert_eq!(installed, text, "{skill} drifted on install");
        }
        assert!(o.fresh());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_second_run_changes_no_bytes() {
        let dir = temp_dir();
        init(&dir).unwrap();
        let before = snapshot(&dir);
        let o = init(&dir).unwrap();
        assert!(!o.fresh(), "{}", render(&o));
        assert_eq!(before, snapshot(&dir));
        assert!(render(&o).contains("already initialized"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn hand_edits_survive_and_report_kept() {
        let dir = temp_dir();
        init(&dir).unwrap();
        let skill = dir.join(".claude/skills/archi/SKILL.md");
        fs::write(&skill, "tuned\n").unwrap();
        let claude = dir.join("CLAUDE.md");
        fs::write(&claude, format!("{FENCE_OPEN}\ntuned\n{FENCE_CLOSE}\n")).unwrap();
        let before = snapshot(&dir);
        let o = init(&dir).unwrap();
        assert_eq!(before, snapshot(&dir));
        let kept: Vec<_> = o
            .steps
            .iter()
            .filter(|s| s.act == Act::Kept)
            .map(|s| s.path.as_str())
            .collect();
        assert_eq!(kept, [".claude/skills/archi/SKILL.md", "CLAUDE.md"]);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn an_existing_claude_md_gains_the_fence_once_prose_intact() {
        let dir = temp_dir();
        let claude = dir.join("CLAUDE.md");
        fs::write(&claude, "# House rules\n\nTabs are love.\n").unwrap();
        let o = init(&dir).unwrap();
        assert!(o.steps.iter().any(|s| s.act == Act::Appended));
        let text = fs::read_to_string(&claude).unwrap();
        assert!(text.starts_with("# House rules\n\nTabs are love.\n"));
        assert_eq!(text.matches(FENCE_OPEN).count(), 1);
        // The second run finds the fence and leaves it.
        let o = init(&dir).unwrap();
        assert!(!o.fresh());
        assert_eq!(
            fs::read_to_string(&claude).unwrap().matches(FENCE_OPEN).count(),
            1
        );
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_manifest_routes_the_starter() {
        let dir = temp_dir();
        fs::write(
            dir.join("archi.toml"),
            "[project]\nname = \"t\"\nsrc = \"spec\"\npreset = \"default\"\n",
        )
        .unwrap();
        let o = init(&dir).unwrap();
        assert!(dir.join("spec/model.arch").is_file());
        assert!(!dir.join("archi/src").exists());
        // The block names the manifest's layout, and the manifest reports ok.
        assert!(
            fs::read_to_string(dir.join("CLAUDE.md"))
                .unwrap()
                .contains("`spec/`")
        );
        let manifest = o.steps.last().unwrap();
        assert!(manifest.act == Act::Ok && manifest.path == "archi.toml");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_broken_manifest_aborts_an_untouched_tree() {
        let dir = temp_dir();
        fs::write(dir.join("archi.toml"), "not toml at all [").unwrap();
        let err = init(&dir).unwrap_err();
        assert!(err.contains("archi.toml"), "{err}");
        let names: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect();
        assert_eq!(names, ["archi.toml"], "init wrote into a tree it should abort");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_tree_missing_only_the_manifest_completes() {
        let dir = temp_dir();
        init(&dir).unwrap();
        fs::remove_file(dir.join("archi.toml")).unwrap();
        let o = init(&dir).unwrap();
        let created: Vec<_> = o
            .steps
            .iter()
            .filter(|s| s.act == Act::Created)
            .map(|s| s.path.as_str())
            .collect();
        assert_eq!(created, ["archi.toml"]);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_nested_init_names_the_enclosing_root() {
        let dir = temp_dir();
        init(&dir).unwrap();
        let sub = dir.join("services/billing");
        let o = init(&sub).unwrap();
        assert_eq!(o.enclosing, Some(dir.canonicalize().unwrap()));
        assert!(render(&o).contains("enclosing project"));
        // Outside any project there is no note.
        let lone = temp_dir();
        let o = init(&lone).unwrap();
        assert_eq!(o.enclosing, None);
        fs::remove_dir_all(&dir).unwrap();
        fs::remove_dir_all(&lone).unwrap();
    }

    #[test]
    fn sync_leaves_a_fresh_tree_byte_identical() {
        let dir = temp_dir();
        init(&dir).unwrap();
        let before = snapshot(&dir);
        let o = sync_skills(&dir).unwrap();
        assert_eq!(before, snapshot(&dir));
        assert!(o.steps.iter().all(|s| s.act == Act::Ok), "{}", render_sync(&o));
        assert!(render_sync(&o).contains("already in sync"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn sync_creates_a_missing_skill() {
        let dir = temp_dir();
        init(&dir).unwrap();
        let skill = dir.join(".claude/skills/archi-merge/SKILL.md");
        fs::remove_file(&skill).unwrap();
        let o = sync_skills(&dir).unwrap();
        let created: Vec<_> = o
            .steps
            .iter()
            .filter(|s| s.act == Act::Created)
            .map(|s| s.path.as_str())
            .collect();
        assert_eq!(created, [".claude/skills/archi-merge/SKILL.md"]);
        let (_, embedded) = SKILLS.iter().find(|(n, _)| *n == "archi-merge").unwrap();
        assert_eq!(&fs::read_to_string(&skill).unwrap(), embedded);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn sync_overwrites_a_divergent_skill_unconditionally() {
        let dir = temp_dir();
        init(&dir).unwrap();
        let skill = dir.join(".claude/skills/archi/SKILL.md");
        fs::write(&skill, "locally tuned\n").unwrap();

        // No flag, no keeping: a divergent copy is overwritten with the embedded text.
        let o = sync_skills(&dir).unwrap();
        assert!(
            o.steps
                .iter()
                .any(|s| s.act == Act::Updated && s.path == ".claude/skills/archi/SKILL.md")
        );
        assert!(!o.steps.iter().any(|s| s.act == Act::Kept));
        let (_, embedded) = SKILLS.iter().find(|(n, _)| *n == "archi").unwrap();
        assert_eq!(&fs::read_to_string(&skill).unwrap(), embedded);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn sync_refreshes_the_claude_block_prose_intact() {
        let dir = temp_dir();
        init(&dir).unwrap();
        let claude = dir.join("CLAUDE.md");
        fs::write(
            &claude,
            format!("# House rules\n\nTabs are love.\n\n{FENCE_OPEN}\nstale\n{FENCE_CLOSE}\n"),
        )
        .unwrap();

        // Only the fenced region moves; the prose above it does not.
        let o = sync_skills(&dir).unwrap();
        assert!(o.steps.iter().any(|s| s.act == Act::Updated && s.path == "CLAUDE.md"));
        let text = fs::read_to_string(&claude).unwrap();
        assert!(text.starts_with("# House rules\n\nTabs are love.\n"), "{text}");
        assert!(text.contains("## Archiplan"));
        assert!(!text.contains("\nstale\n"));
        assert_eq!(text.matches(FENCE_OPEN).count(), 1);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn sync_needs_a_project() {
        let dir = temp_dir();
        let err = sync_skills(&dir).unwrap_err();
        assert!(err.contains("archi.toml"), "{err}");
        fs::remove_dir_all(&dir).unwrap();
    }
}
