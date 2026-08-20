//! End to end through the real binary: one command stands a project up
//! (`archi/requirements/cold-start/one-verb-stands-a-project-up`), a second
//! run changes no bytes, the manifest routes the starter and aborts the
//! broken run, the briefing lands verbatim, and the commands around init keep
//! their contracts.
//!
//! The skills carry the world (`archi/requirements/world-facts/`): the `world`
//! verb and the no-model-nouns rule stand in the workflow skill, the workflow
//! captures the world before it derives requirements, and
//! `archi-migrate` installs beside the other skills so a project that
//! stands without a world can gain one through its world pass — a
//! draft-first interview whose workaround is never answered from prose,
//! anchoring what a standing suite already proves as the fact is
//! written. The planning skill moved with the
//! behaviour too: it collects its closing block from the world and asks for
//! none of it. Both skills that ask for a scenario teach its shape — a heading
//! and its step lines. The same two skills carry the four folders of the world
//! and the rule that a `sources` entry points inside it, and a migrated fact
//! carries no source at all. They carry the workaround the same way: it is the
//! section a writer owes, nothing asks what would end the fact, and a fact
//! names no person and quotes nobody. The `CLAUDE.md` block says none of it: a
//! fact is an archi record like a requirement or a stressor, and none of those
//! carries a writing rule there.
//!
//! The implement skill carries the declaration loop
//! (`archi/requirements/code-link/`): the per-task contract sends the writer to
//! `archi plan task <id> link add` as its last act, several entries through
//! `archi batch -`; the rule that keeps every `plan` and `link` command with
//! the orchestrator holds and names that one verb as its exception; and no
//! embedded skill sends a reader to a candidate list capture no longer makes.
//!
//! The briefing sends its reader to the record before the tree
//! (`archi/requirements/agent-retrieval/`): the planning skill seeds
//! `## Outputs` with `archi link ls --spec`, the implement skill puts the
//! rows for the task's refs in the sub-agent prompt, the workflow skill
//! reads the standing claims with `archi req ls --satisfies` before it
//! derives one, and the candidate guard reads the sentences that name no
//! command too.
//!
//! The search doctrine lives in one skill
//! (`archi/requirements/agent-retrieval/the-search-doctrine-lives-in-one-skill.md`):
//! `archi-search` installs beside the other nine and alone carries the
//! order — the menu, the three structural reads, then search — and the
//! grep rule; every working skill points at it by bare name.
//!
//! The why reads back from the record
//! (`archi/requirements/agent-retrieval/the-why-reads-back-from-the-record.md`):
//! `archi-explain` resolves first — the identity sentence quoted, an
//! ambiguous address put to the user as options — then walks the chain
//! outside-in — the world condition, the standing claims, the recorded
//! trades, the pressure behind them (an intent-born claim answering from
//! its intent folder's own problem statement), the timeline, who realizes
//! it today — read-only, silence a real answer, invented rationale
//! forbidden.

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use util::{SKILL_PLAN, flat};

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// This binary's embedded briefing sources, for byte-equality checks. The
/// planning skill is read by a second family as well, so it stands in the
/// shared fixture module ([`util::SKILL_PLAN`]).
const SKILL_ARCHI: &str = include_str!("../../../skills/archi.md");
const SKILL_IMPLEMENT: &str = include_str!("../../../skills/archi-implement.md");
const SKILL_MERGE: &str = include_str!("../../../skills/archi-merge.md");
const SKILL_FINISH: &str = include_str!("../../../skills/archi-finish-worktree.md");
const SKILL_MIGRATE: &str = include_str!("../../../skills/archi-migrate.md");
const SKILL_MIGRATE_FRACTAL: &str = include_str!("../../../skills/archi-migrate-fractal.md");
const SKILL_STE: &str = include_str!("../../../skills/ste-writing.md");
const SKILL_SEARCH: &str = include_str!("../../../skills/archi-search.md");
const SKILL_EXPLAIN: &str = include_str!("../../../skills/archi-explain.md");

/// Every skill this binary embeds, name -> source. The guards that read all
/// installed skills iterate this list, so a new skill joins them by joining
/// it.
const EMBEDDED_SKILLS: [(&str, &str); 10] = [
    ("archi", SKILL_ARCHI),
    ("archi-search", SKILL_SEARCH),
    ("archi-explain", SKILL_EXPLAIN),
    ("archi-plan", SKILL_PLAN),
    ("archi-implement", SKILL_IMPLEMENT),
    ("archi-merge", SKILL_MERGE),
    ("archi-finish-worktree", SKILL_FINISH),
    ("archi-migrate", SKILL_MIGRATE),
    ("archi-migrate-fractal", SKILL_MIGRATE_FRACTAL),
    ("ste-writing", SKILL_STE),
];

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-init-e2e-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Run the real binary with `dir` as the working directory; exit code,
/// stdout, stderr.
fn run_in(dir: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_archi"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("archi runs");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn ok_in(dir: &Path, args: &[&str]) -> String {
    let (code, stdout, stderr) = run_in(dir, args);
    assert_eq!(code, Some(0), "archi {args:?} failed:\n{stdout}\n{stderr}");
    stdout
}

/// The fenced archi block of an initialized tree's CLAUDE.md, markers included.
fn block_of(root: &Path) -> String {
    let text = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    let start = text.find("<!-- archi:begin -->").expect("the fence opens");
    let end = text[start..].find("<!-- archi:end -->").expect("the fence closes")
        + start
        + "<!-- archi:end -->".len();
    text[start..end].to_string()
}

/// The two skills that ask a person to write into the world, read off an
/// initialized tree — and pinned byte-equal to the copies this binary embeds,
/// so what the suite reads is what a project gets.
fn world_skills(root: &Path) -> (String, String) {
    let workflow = fs::read_to_string(root.join(".claude/skills/archi/SKILL.md")).unwrap();
    let migration =
        fs::read_to_string(root.join(".claude/skills/archi-migrate/SKILL.md")).unwrap();
    assert_eq!(workflow, SKILL_ARCHI, "the workflow skill drifted on install");
    assert_eq!(migration, SKILL_MIGRATE, "the migration skill drifted on install");
    (workflow, migration)
}

/// The planning skill read off an initialized tree, pinned byte-equal to the
/// copy this binary embeds, the way [`implement_skill`] pins its own.
fn planning_skill(root: &Path) -> String {
    let installed = fs::read_to_string(root.join(".claude/skills/archi-plan/SKILL.md")).unwrap();
    assert_eq!(installed, SKILL_PLAN, "the planning skill drifted on install");
    installed
}

/// The implement skill read off an initialized tree, pinned byte-equal to the
/// copy this binary embeds — so what the suite reads is what a project gets.
fn implement_skill(root: &Path) -> String {
    let installed =
        fs::read_to_string(root.join(".claude/skills/archi-implement/SKILL.md")).unwrap();
    assert_eq!(installed, SKILL_IMPLEMENT, "the implement skill drifted on install");
    installed
}

/// One passage of a skill: from where `opens` first stands to the next `##`
/// heading. A rule is read where its reader meets it, so a passage that keeps
/// the words while the words move to another step is not the same skill.
fn passage<'a>(text: &'a str, opens: &str) -> &'a str {
    let start = text
        .find(opens)
        .unwrap_or_else(|| panic!("the skill has no `{opens}`"));
    let rest = &text[start..];
    match rest.find("\n## ") {
        Some(end) => &rest[..end],
        None => rest,
    }
}

/// One bullet of a skill, flattened: from where `opens` first stands to the
/// next top-level bullet or heading. A wrapped bullet is one bullet, and a
/// rule is read in the bullet it belongs to — a command named three steps
/// down the page is a command the author meets after the step it governs.
fn bullet(text: &str, opens: &str) -> String {
    let start = text
        .find(opens)
        .unwrap_or_else(|| panic!("the skill has no `{opens}` bullet"));
    let rest = &text[start + opens.len()..];
    let end = rest
        .match_indices('\n')
        .find(|(at, _)| {
            let line = &rest[at + 1..];
            line.starts_with("- ") || line.starts_with('#') || line.starts_with("```")
        })
        .map(|(at, _)| at)
        .unwrap_or(rest.len());
    flat(&text[start..start + opens.len() + end])
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

#[test]
fn a_fresh_init_stands_up_a_building_project() {
    let root = temp_dir();
    let out = ok_in(&root, &["init", "proj"]);

    // The report: every artifact created, the manifest on the last created
    // line, the verdict naming the project.
    let created: Vec<&str> = out.lines().filter(|l| l.starts_with("created")).collect();
    assert_eq!(created.len(), 14, "{out}");
    assert!(created.last().unwrap().contains("archi.toml"), "{out}");
    assert!(out.contains("initialized `proj`"), "{out}");

    // One door migrates the standing project: the merged page lands byte-equal,
    // and neither of the two pages it absorbed installs
    // (archi/requirements/cold-start/one-door-migrates-the-standing-project.md).
    let migrate =
        fs::read_to_string(root.join("proj/.claude/skills/archi-migrate/SKILL.md")).unwrap();
    assert_eq!(migrate, SKILL_MIGRATE, "archi-migrate drifted on install");
    assert!(!root.join("proj/.claude/skills/archi-migrate-world").exists());
    assert!(!root.join("proj/.claude/skills/archi-migrate-links").exists());

    // Worktree artifacts are ignored from birth — machine-local, never merged.
    let ignore = fs::read_to_string(root.join("proj/.gitignore")).unwrap();
    assert!(ignore.contains("archi/*.local.toml"), "{ignore}");
    assert!(ignore.contains("archi/plans/.current"), "{ignore}");

    // The protected branch is declared from birth. Deleting the line opts out.
    let manifest = fs::read_to_string(root.join("proj/archi.toml")).unwrap();
    assert!(manifest.contains("protected = [\"main\"]"), "{manifest}");

    // The scaffolded tree is a passing, empty project.
    let build = ok_in(&root, &["build", "--project", "proj"]);
    assert!(build.contains("ok: 0 statements"), "{build}");
    let check = ok_in(&root, &["check", "--project", "proj"]);
    assert!(check.contains("no findings"), "{check}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_second_init_changes_no_bytes_and_extra_args_are_usage_errors() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let before = snapshot(&root);

    let out = ok_in(&root, &["init", "."]);
    assert!(out.contains("already initialized"), "{out}");
    assert!(!out.contains("created"), "{out}");
    assert_eq!(before, snapshot(&root));

    // A second directory and a --project are both malformed invocations.
    let (code, _, _) = run_in(&root, &["init", "a", "b"]);
    assert_eq!(code, Some(2));
    let (code, _, _) = run_in(&root, &["init", "--project", "a"]);
    assert_eq!(code, Some(2));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn the_manifest_routes_the_starter_and_a_broken_one_aborts() {
    let root = temp_dir();
    fs::write(
        root.join("archi.toml"),
        "[project]\nname = \"t\"\nsrc = \"spec\"\npreset = \"default\"\n",
    )
    .unwrap();
    ok_in(&root, &["init", "."]);
    assert!(root.join("spec/model.arch").is_file());
    assert!(!root.join("archi/src").exists());
    let build = ok_in(&root, &["build", "--project", "."]);
    assert!(build.contains("ok: 0 statements"), "{build}");

    // A manifest that fails to parse stops the run before a byte lands.
    let broken = temp_dir();
    fs::write(broken.join("archi.toml"), "not toml at all [").unwrap();
    let (code, _, stderr) = run_in(&broken, &["init", "."]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("archi.toml"), "{stderr}");
    assert!(!broken.join(".claude").exists());
    assert!(!broken.join("CLAUDE.md").exists());

    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(&broken).unwrap();
}

#[test]
fn the_briefing_lands_verbatim_and_the_fence_appends_once() {
    let root = temp_dir();
    fs::write(root.join("CLAUDE.md"), "# House rules\n\nTabs are love.\n").unwrap();
    fs::write(root.join(".gitignore"), "target/\narchi/*.local.toml\n").unwrap();
    ok_in(&root, &["init", "."]);

    // A partial .gitignore gains only its missing line, exactly once.
    let ignore = fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(ignore.starts_with("target/\n"), "{ignore}");
    assert_eq!(ignore.matches("archi/*.local.toml").count(), 1, "{ignore}");
    assert_eq!(ignore.matches("archi/plans/.current").count(), 1, "{ignore}");

    for (skill, text) in [
        ("archi", SKILL_ARCHI),
        ("archi-merge", SKILL_MERGE),
        ("archi-migrate-fractal", SKILL_MIGRATE_FRACTAL),
        ("archi-migrate", SKILL_MIGRATE),
    ] {
        let installed =
            fs::read_to_string(root.join(".claude/skills").join(skill).join("SKILL.md")).unwrap();
        assert_eq!(installed, text, "{skill} drifted on install");
    }

    let claude = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert!(claude.starts_with("# House rules\n\nTabs are love.\n"), "{claude}");
    assert_eq!(claude.matches("<!-- archi:begin -->").count(), 1, "{claude}");

    ok_in(&root, &["init", "."]);
    let claude = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert_eq!(claude.matches("<!-- archi:begin -->").count(), 1, "{claude}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn init_owes_nothing_to_the_compiler_and_advisory_state_never_blocks() {
    // A model that does not compile: init still equips the tree, exit zero.
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    fs::write(root.join("archi/src/model.arch"), "Ghost.out wire Phantom.inn\n").unwrap();
    fs::remove_dir_all(root.join(".claude")).unwrap();
    let (code, _, _) = run_in(&root, &["check"]);
    assert_eq!(code, Some(1));
    let out = ok_in(&root, &["init", "."]);
    assert!(out.contains("created  .claude/skills/archi/SKILL.md"), "{out}");

    // A KB carrying advisory findings inits exactly as a clean one.
    fs::write(root.join("archi/src/model.arch"), "def node Lone\n").unwrap();
    fs::create_dir_all(root.join("archi/requirements/an-intent")).unwrap();
    fs::write(
        root.join("archi/requirements/an-intent/an-intent.md"),
        "# An intent\n\nA problem worth modeling.\n",
    )
    .unwrap();
    fs::write(
        root.join("archi/requirements/an-intent/still-open.md"),
        "---\nkind: functional\norigin: intent\nsatisfied-by: []\ndeferred:\n---\n\n\
         # Still open\n\nAn unsatisfied claim.\n\n## System Context\n\n## Satisfy\n",
    )
    .unwrap();
    let check = ok_in(&root, &["check"]);
    assert!(check.contains("unsatisfied requirement"), "{check}");
    let out = ok_in(&root, &["init", "."]);
    assert!(out.contains("already initialized"), "{out}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn the_verbs_around_init_keep_their_contracts() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);

    // Search runs on the fresh project — the dispatch gained a command and
    // lost none.
    let (code, _, _) = run_in(&root, &["search", "anything", "at", "all"]);
    assert_eq!(code, Some(0));

    // Gitless mutation is a full stop naming the repair, never a silent
    // bare run — and the discipline is unconditional: deleting the
    // `protected` line opts nothing out.
    let (code, _, stderr) = run_in(&root, &["version", "save", "-m", "first"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("git init"), "{stderr}");
    let manifest = fs::read_to_string(root.join("archi.toml")).unwrap();
    fs::write(root.join("archi.toml"), manifest.replace("protected = [\"main\"]\n", "")).unwrap();
    let (code, _, stderr) = run_in(&root, &["version", "save", "-m", "first"]);
    assert_eq!(code, Some(1), "no opt-out: {stderr}");
    assert!(stderr.contains("git init"), "{stderr}");

    // Bound, the flow runs: a save, an init, a save — init minted nothing
    // the second save could notice.
    let wt = util::worktree(&root);
    ok_in(&wt, &["version", "save", "-m", "first"]);
    let out = ok_in(&wt, &["init", "."]);
    assert!(out.contains("already initialized"), "{out}");
    let (_, stdout, stderr) = run_in(&wt, &["version", "save", "-m", "again"]);
    assert!(
        (stdout.clone() + &stderr).contains("unchanged since v0001"),
        "{stdout}\n{stderr}"
    );

    let _ = fs::remove_dir_all(root.parent().unwrap().join(format!(
        "{}-worktrees",
        root.file_name().unwrap().to_str().unwrap()
    )));
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn the_briefing_carries_the_world() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);

    // The workflow skill names the verb with every subcommand, and it captures
    // the world before it derives requirements: a condition from outside
    // decides which claims are requirements at all
    // (archi/requirements/world-facts/the-briefing-puts-the-world-in-the-loop.md).
    let skill = fs::read_to_string(root.join(".claude/skills/archi/SKILL.md")).unwrap();
    for form in ["archi world add", "archi world rm", "archi world ls"] {
        assert!(skill.contains(form), "the briefing misses `{form}`");
    }
    let capture = skill.find("**Capture the world.**").expect("the capture step");
    let derive = skill
        .find("**Derive requirements.**")
        .expect("the derivation step");
    assert!(capture < derive, "the world is captured after the derivation");

    // The rule that keeps a fact from arriving as a requirement in costume
    // stands in the skill, where a person writing a fact already is. The
    // `CLAUDE.md` block says nothing about the world: a fact is an archi record
    // like a requirement or a stressor, and none of those has a writing rule
    // there either.
    assert!(
        skill.contains("without the nouns of the model"),
        "the no-model-nouns rule is missing:\n{skill}"
    );

    fs::remove_dir_all(&root).unwrap();
}

/// The planning procedure moved with the behaviour
/// (`archi/requirements/world-facts/the-plan-collects-its-scenarios-and-authors-none.md`):
/// the closing block is collected from the world facts that cover the plan's
/// task nodes, the author writes none of it, and an empty block is spec work.
/// A rule that lives only in a requirement is a rule the planner meets after
/// they have already done the wrong thing.
#[test]
fn the_planning_skill_collects_its_scenarios_and_authors_none() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    // Installed byte-equal to the embedded copy, as every other skill is.
    let installed = planning_skill(&root);

    // The prose is hard-wrapped, so a sentence is read over its line breaks:
    // what the skill says must not depend on where a line ends.
    let flat = flat(&installed);

    // No instruction to author the file. The skill still names `scenarios.md`,
    // because it has to say the author writes nothing into it — so every line
    // that names the file carries the denial on it. A mention without one is
    // the old instruction coming back.
    for line in installed.lines().filter(|l| l.contains("scenarios.md")) {
        let padded = format!("{line} ");
        assert!(
            ["no ", "not ", "never", "nobody", "nothing", "none"]
                .iter()
                .any(|deny| padded.contains(deny)),
            "the skill still asks for `scenarios.md`: {line}"
        );
    }
    for gone in [
        "Walk the architecture as a user",
        "one bullet per flow",
        "delete its bullet",
        "the plan's own user stories",
    ] {
        assert!(!flat.contains(gone), "the skill still says `{gone}`");
    }

    // Where the block comes from. The skill says it positively — it names
    // the source and the command, and does not tell the reader what not to
    // do with a mechanism it has already described.
    for phrase in [
        "collected",
        "world fact",
        "holds a task for",
        "archi plan scenarios list",
    ] {
        assert!(flat.contains(phrase), "the skill misses `{phrase}`");
    }

    // An empty block is spec work: the skill sends the reader to capture the
    // condition, and the block fills itself from it.
    for phrase in [
        "no world fact covers any node this plan builds",
        "spec work",
        "fills itself",
        "/archi",
    ] {
        assert!(flat.contains(phrase), "the skill misses `{phrase}`");
    }

    // The close gates on the block: every collected scenario carries a link
    // to code (`the-close-gates-on-anchored-scenarios`).
    assert!(flat.contains("link to code"), "the skill misses the anchor gate");

    fs::remove_dir_all(&root).unwrap();
}

/// A project installed before the rewrite takes the new procedure the way it
/// takes any drifted skill: `sync-skills` reports it updated and the old
/// instruction is gone from the tree.
#[test]
fn a_standing_project_syncs_the_new_planning_skill() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    fs::write(
        root.join(".claude/skills/archi-plan/SKILL.md"),
        "# Generate an implementation plan\n\nWalk the architecture as a user, and write \
         one bullet per flow into `scenarios.md`.\n",
    )
    .unwrap();

    let out = ok_in(&root, &["sync-skills"]);
    assert!(out.contains("updated  .claude/skills/archi-plan/SKILL.md"), "{out}");
    let installed = fs::read_to_string(root.join(".claude/skills/archi-plan/SKILL.md")).unwrap();
    assert_eq!(installed, SKILL_PLAN);
    assert!(!installed.contains("one bullet per flow"), "{installed}");

    fs::remove_dir_all(&root).unwrap();
}


/// A scenario is a heading and its steps
/// (`archi/requirements/world-facts/the-grammar-is-a-named-subset.md`), so the
/// two skills that ask a person to write one teach that shape: `### <name>`
/// opens the scenario, four keywords open its step lines, and `Feature:` and
/// `Scenario:` appear only as the lines the check refuses. The block teaches
/// none of it. It stands at nineteen lines against a budget of twenty
/// (`the-briefing-says-what-help-does-not`), and a grammar does not fit in the
/// one line that is left — the skills carry it, where the writer reads it.
#[test]
fn the_skills_teach_the_scenario_shape_and_the_block_stays_short() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let (workflow, migration) = world_skills(&root);

    // The workflow skill says what a scenario is.
    let workflow_flat = flat(&workflow);
    for phrase in ["`### <name>`", "`Given`, `When`, `Then` and `And`", "the whole vocabulary"] {
        assert!(workflow_flat.contains(phrase), "the workflow skill misses `{phrase}`");
    }

    let installed = [("archi", workflow.as_str()), ("archi-migrate", migration.as_str())];
    for (name, text) in installed {
        // Not a subset of that grammar any more, and the tag went with it.
        assert!(!text.contains("Gherkin"), "{name} still calls the grammar Gherkin");
        assert!(!text.contains("@runs"), "{name} still carries the retired runs tag");

        // No line writes either replaced keyword. Prose quotes a keyword in
        // backticks; bare at the head of a line it is an example, which is the
        // old shape offered as the thing to copy.
        for line in text.lines() {
            let head = line.trim_start_matches(|c: char| matches!(c, '-' | '*' | '>' | ' '));
            assert!(
                !head.starts_with("Feature:") && !head.starts_with("Scenario:"),
                "{name} writes the old shape: {line}"
            );
        }

        // Named in prose, each is named as refused.
        let sentences = flat(text);
        for sentence in sentences.split(". ") {
            if !sentence.contains("Feature:") && !sentence.contains("Scenario:") {
                continue;
            }
            assert!(
                ["refus", "never", "not ", "no longer", "replaced"]
                    .iter()
                    .any(|deny| sentence.contains(deny)),
                "{name} asks for the old shape: {sentence}"
            );
        }
    }

    // The migration skill shows one fact written out, and its scenario carries
    // the shape the skill just described.
    let example = migration
        .split("```")
        .skip(1)
        .step_by(2)
        .find(|b| b.contains("## Scenarios"))
        .expect("the migration skill shows an example fact");
    let lines: Vec<&str> = example.lines().map(str::trim).collect();
    assert!(
        lines
            .iter()
            .any(|l| l.strip_prefix("### ").is_some_and(|n| !n.is_empty())),
        "the example names no scenario:\n{example}"
    );
    for keyword in ["Given ", "When ", "Then "] {
        assert!(
            lines.iter().any(|l| l.starts_with(keyword)),
            "the example misses `{keyword}`:\n{example}"
        );
    }

    // The block took none of the grammar, and its budget is why.
    let block = block_of(&root);
    let count = block.lines().count();
    for spelled in ["### ", "Given", "Scenarios"] {
        assert!(!block.contains(spelled), "the block spells `{spelled}` out:\n{block}");
    }

    fs::remove_dir_all(&root).unwrap();
}

/// The world is four folders, and a source lives inside it
/// (`archi/requirements/world-facts/the-world-holds-four-layers.md`,
/// `archi/requirements/world-facts/a-source-is-reachable-and-lives-in-the-world.md`).
/// `archi --help` prints neither rule and the block has no room for either
/// (`the-briefing-says-what-help-does-not`), so the two skills that ask a person
/// to write into the world carry them — the folder a file goes in, and the only
/// place a `sources` entry may point.
#[test]
fn the_skills_describe_the_four_layers_and_the_source_rule() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let (workflow, migration) = world_skills(&root);

    // The workflow skill names each folder and says what that folder holds.
    let workflow_flat = flat(&workflow);
    // `hypotheses/` and `notes/` are folders the checker knows and the skill
    // does not teach: nothing writes into them yet, so a reader told about
    // them would be told about a shape with no verb behind it.
    for (folder, holds) in [
        ("`archi/world/facts/`", "the strict record"),
        ("`archi/world/resources/`", "raw material"),
    ] {
        let at = workflow_flat
            .find(folder)
            .unwrap_or_else(|| panic!("the workflow skill names no {folder}"));
        let window: String = workflow_flat[at..].chars().take(220).collect();
        assert!(
            window.contains(holds),
            "{folder} is named without what it holds (`{holds}`): {window}"
        );
    }
    // And what a file directly under the world is: nothing the reader may write.
    assert!(
        workflow_flat.contains("in no layer"),
        "the workflow skill never says a file outside the four folders is refused"
    );

    // The source rule, in both texts that ask for one: every path a paragraph
    // about `sources` offers the reader is a path inside the world.
    let installed = [("archi", workflow.as_str()), ("archi-migrate", migration.as_str())];
    for (name, text) in installed {
        let mut explained = 0;
        for paragraph in text.split("\n\n") {
            let flattened = flat(paragraph);
            if !flattened.contains("`sources`") {
                continue;
            }
            explained += 1;
            for token in flattened.split('`').skip(1).step_by(2) {
                if !token.contains('/') {
                    continue;
                }
                assert!(
                    token.starts_with("archi/world/"),
                    "{name} sends a `sources` entry to `{token}`:\n{flattened}"
                );
            }
        }
        assert!(explained > 0, "{name} never explains `sources`");
    }

    // What a migration writes into the field, now that the origin file cannot
    // go there: nothing, and the skill says so in those words.
    let migration_flat = flat(&migration);
    assert!(
        migration_flat.contains("a claim lifted from prose carries no source"),
        "the migration skill does not say what a migrated fact rests on"
    );
    for retired in ["names the file the claim came from", "provenance, not observation"] {
        assert!(
            !migration_flat.contains(retired),
            "the migration skill still teaches `{retired}`"
        );
    }

    // None of it reached the block. It stands at nineteen lines against a
    // budget of twenty, and four folders plus a source rule do not fit in one.
    let block = block_of(&root);
    let count = block.lines().count();
    for spelled in ["facts/", "hypotheses/", "notes/", "resources/", "sources"] {
        assert!(!block.contains(spelled), "the block spells `{spelled}` out:\n{block}");
    }

    fs::remove_dir_all(&root).unwrap();
}

/// The workaround is the record, and the killer is gone
/// (`archi/requirements/world-facts/a-world-fact-carries-its-scenarios.md`,
/// `archi/requirements/world-facts/the-fact-speaks-the-world-and-check-says-when-it-does-not.md`).
/// The two skills that ask a person to write a fact are the texts that spell
/// the sections out, so the change lands there: `## What people do instead` is
/// a section the writer owes, nothing asks what would end the fact, and both
/// texts carry the rule that a fact names no person and quotes nobody — the
/// half of the content rule no check can hold. The block takes none of it: it
/// stands at nineteen lines against a budget of twenty
/// (`the-briefing-says-what-help-does-not`), and a reader meets both rules with
/// the skill already open.
#[test]
fn the_skills_ask_for_the_workaround_and_not_for_the_killer() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    // Byte-equal to the copies this binary embeds — asserted inside.
    let (workflow, migration) = world_skills(&root);
    let installed = [("archi", workflow.as_str()), ("archi-migrate", migration.as_str())];

    for (name, text) in installed {
        let text_flat = flat(text);

        // The section a writer owes, named as one of them.
        assert!(
            text_flat.contains("`## What people do instead`"),
            "{name} never names the workaround section"
        );

        // Nothing asks for a prediction of the end — not as a heading, not as
        // an interview question, not as a section of the file.
        for retired in [
            "What kills this",
            "The killer",
            "call the condition over",
            "would you have to see",
            "would end the fact",
        ] {
            assert!(!text_flat.contains(retired), "{name} still carries `{retired}`");
        }

        // The workaround took the killer's other job, and both texts say how it
        // does it: it is watched, not predicted.
        assert!(
            text_flat.contains("the day they stop, the fact is dead"),
            "{name} never says the workaround is what falsifies the fact"
        );

        // And its first job, in the same words in both texts.
        assert!(
            text_flat.contains("nobody can name a workaround for is a wish"),
            "{name} does not make the workaround the gate"
        );

        // A fact is about the world, not about whoever reported it — and the
        // text says where the person and their words go instead.
        for rule in ["names no person", "quotes nobody"] {
            assert!(text_flat.contains(rule), "{name} misses `{rule}`");
        }
        assert!(
            text_flat.contains("`archi/world/resources/`"),
            "{name} says a fact quotes nobody without saying where the words go"
        );
    }

    let migration_flat = flat(&migration);

    // The interview asks four things, in this order, and the workaround is the
    // second. The list is read off the skill, so a fifth question cannot bring
    // the killer back under another name.
    let labels: Vec<&str> = migration
        .lines()
        .filter(|l| l.starts_with(|c: char| c.is_ascii_digit()) && l.contains(". **"))
        .filter_map(|l| l.split("**").nth(1))
        .collect();
    assert_eq!(
        labels,
        ["The condition.", "The workaround.", "The behavior.", "The reach."],
        "the interview changed shape"
    );

    // The gate stops a fact being written and says what happens to the
    // candidate instead.
    for phrase in [
        "The gate is question 2",
        "this skill writes nothing for that candidate",
        "It goes in the brief",
    ] {
        assert!(migration_flat.contains(phrase), "the migration skill misses `{phrase}`");
    }

    // The second ask outlives the question that used to carry it: an apparent
    // axiom is asked again, and what it is asked again for is the workaround.
    let axiom = migration
        .split("\n\n")
        .map(flat)
        .find(|p| p.contains("Ask again"))
        .expect("the migration skill keeps the second ask");
    assert!(
        axiom.contains("instead"),
        "the second ask no longer asks for the workaround: {axiom}"
    );

    // The example fact carries the section the skill just asked for.
    let example = migration
        .split("```")
        .skip(1)
        .step_by(2)
        .find(|b| b.contains("## Scenarios"))
        .expect("the migration skill shows an example fact");
    assert!(
        example.contains("## What people do instead"),
        "the example fact skips the workaround:\n{example}"
    );

    // None of it reached the block. It stands at nineteen lines against a
    // budget of twenty, and the sections of a file the reader is not writing
    // yet are not what the one free line is for.
    let block = block_of(&root);
    let count = block.lines().count();
    for spelled in ["What people do instead", "workaround", "no person", "quotes"] {
        assert!(!block.contains(spelled), "the block spells `{spelled}` out:\n{block}");
    }

    fs::remove_dir_all(&root).unwrap();
}

/// The migration skill asks the way the first real run taught it to
/// (`archi/requirements/world-facts/a-skill-migrates-a-standing-project-into-the-world.md`):
/// by offering shapes instead of open questions, by asking a second time when
/// the answer sounds like an axiom, and by reading the suites the project
/// already runs for candidates.
#[test]
fn the_migration_skill_learns_to_ask() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let installed =
        fs::read_to_string(root.join(".claude/skills/archi-migrate/SKILL.md")).unwrap();

    // Options, not open questions — and the options are scaffolding, not a menu.
    for phrase in [
        "two or three",
        "open question",
        "not a menu",
        "poll tool",
    ] {
        assert!(installed.contains(phrase), "the skill misses `{phrase}`");
    }

    // A first "nothing would make this false" is not a verdict.
    for phrase in ["Ask again", "nothing would make this false"] {
        assert!(installed.contains(phrase), "the skill misses `{phrase}`");
    }

    // The third place candidates come from.
    for phrase in ["test suites", "worth pinning"] {
        assert!(installed.contains(phrase), "the skill misses `{phrase}`");
    }
    assert!(
        installed.contains("Three places hold candidates"),
        "the suites are not counted among the sources"
    );

    fs::remove_dir_all(&root).unwrap();
}

/// The interview is draft-first
/// (`archi/requirements/world-facts/a-skill-migrates-a-standing-project-into-the-world.md`):
/// the candidate is drafted whole from the prose it came from, the operator
/// confirms or corrects, and only the gaps are asked — the survey opener is
/// gone. The one answer never taken from prose is the workaround, in as many
/// words and again on the question itself, so the gate is never met from
/// paper.
#[test]
fn the_world_pass_drafts_from_the_prose_and_asks_only_the_gaps() {
    let flat = flat(SKILL_MIGRATE);

    for phrase in [
        "Draft the candidate whole",
        "confirm or correct",
        "ask only what the prose does not answer",
        "The one answer never taken from prose is the workaround",
        "Always asked, never drafted",
    ] {
        assert!(flat.contains(phrase), "the world pass misses `{phrase}`");
    }

    // The survey opener is gone: a reader told to ask every question asks
    // the operator to read the prose back to them.
    assert!(
        !flat.contains("you never fill an answer in yourself"),
        "the interview still opens as a survey"
    );
}

/// A scenario a standing suite already proves is bound when the fact is
/// written
/// (`archi/requirements/world-facts/a-skill-migrates-a-standing-project-into-the-world.md`):
/// the write step carries the `link add` line — the fact's scenario on the
/// left, the standing test on the right — so the fact arrives holding proof
/// the tree already runs instead of waiting on a plan close that may never
/// come.
#[test]
fn the_world_pass_anchors_a_scenario_a_standing_suite_already_proves() {
    let flat = flat(SKILL_MIGRATE);

    let write = flat.find("### 3. Write the fact").expect("the world pass has no write step");
    let close = flat.find("### 4. Check").expect("the world pass has no check step");
    let anchor = flat
        .find("archi link add \"<fact>#<scenario>\" <test file>#<test fn> --kind indirect")
        .expect("the write step never binds a proved scenario");
    assert!(write < anchor && anchor < close, "the anchoring move stands outside the write step");

    for phrase in [
        "often the very test the candidate came from",
        "at the moment the fact is written",
        "already standing",
    ] {
        assert!(flat.contains(phrase), "the anchoring move misses `{phrase}`");
    }
}

#[test]
fn a_pre_world_project_syncs_into_the_world() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);

    // A project scaffolded before the world: its block and its workflow skill
    // predate the verb, and the migration skill was never installed.
    fs::write(
        root.join("CLAUDE.md"),
        "<!-- archi:begin -->\n## Archiplan\n\nthe briefing as it stood\n<!-- archi:end -->\n",
    )
    .unwrap();
    fs::write(
        root.join(".claude/skills/archi/SKILL.md"),
        "# Archi workflow\n\nthe loop as it stood\n",
    )
    .unwrap();
    fs::remove_dir_all(root.join(".claude/skills/archi-migrate")).unwrap();

    let out = ok_in(&root, &["sync-skills"]);
    assert!(out.contains("updated  .claude/skills/archi/SKILL.md"), "{out}");
    assert!(
        out.contains("created  .claude/skills/archi-migrate/SKILL.md"),
        "{out}"
    );
    assert!(out.contains("updated  CLAUDE.md"), "{out}");

    // What the upgrade delivered: the rule in the block, the verb and the whole
    // procedure in the skills.
    let claude = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert_eq!(
        fs::read_to_string(root.join(".claude/skills/archi/SKILL.md")).unwrap(),
        SKILL_ARCHI
    );
    assert_eq!(
        fs::read_to_string(root.join(".claude/skills/archi-migrate/SKILL.md")).unwrap(),
        SKILL_MIGRATE
    );

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn the_migration_skill_installs_and_names_its_gate() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let path = root.join(".claude/skills/archi-migrate/SKILL.md");
    assert_eq!(fs::read_to_string(&path).unwrap(), SKILL_MIGRATE);

    // A drifted copy is reclaimed by sync, as every other skill's is.
    fs::write(&path, "locally tuned\n").unwrap();
    let out = ok_in(&root, &["sync-skills"]);
    assert!(
        out.contains("updated  .claude/skills/archi-migrate/SKILL.md"),
        "{out}"
    );
    let installed = fs::read_to_string(&path).unwrap();
    assert_eq!(installed, SKILL_MIGRATE);

    // The procedure the text must carry: the material it reads, the gate that
    // stops a fact being written, the state a migrated fact is left in, the one
    // place material may be carried into, the brief it hands back, and the
    // check it closes on
    // (archi/requirements/world-facts/a-skill-migrates-a-standing-project-into-the-world.md).
    for phrase in [
        "archi world add",
        "workaround",
        "what people do today instead",
        "writes nothing",
        "only a wish",
        "did not map",
        "world_ungrounded",
        "`archi/world/resources/`",
        "deletes nothing",
        "archi check",
    ] {
        assert!(installed.contains(phrase), "the skill misses `{phrase}`");
    }

    fs::remove_dir_all(&root).unwrap();
}

/// One door migrates the standing project
/// (`archi/requirements/cold-start/one-door-migrates-the-standing-project.md`):
/// the embedded page opens with the three measurements that name the gap,
/// carries the world, journal and scrap passes inline, and names
/// `archi-migrate-fractal` as the old client's own page. The reader does not
/// need to know the name of their staleness to cure it. The fn keeps its
/// name — the journal anchors it — while the page it guards grew a third
/// pass.
#[test]
fn the_migrate_page_opens_with_the_measurements_and_carries_both_passes() {
    let text = flat(SKILL_MIGRATE);

    // The triage head: all three measurements stand before any pass begins.
    let world_pass = text.find("## The world pass").expect("the page has no world pass");
    let journal_pass = text
        .find("## The journal pass")
        .expect("the page has no journal pass");
    let scrap_pass = text.find("## The scrap pass").expect("the page has no scrap pass");
    let passes = world_pass.min(journal_pass).min(scrap_pass);
    let world_measure = text.find("`archi world ls`").expect("no world measurement");
    let journal_measure = text.find("`archi link ls").expect("no journal measurement");
    let scrap_measure = text
        .find("`ls -d archi/plans/*/waves`")
        .expect("no scrap measurement");
    assert!(
        world_measure < world_pass && world_measure < journal_pass,
        "the world measurement does not open the page"
    );
    assert!(
        journal_measure < world_pass && journal_measure < journal_pass,
        "the journal measurement does not open the page"
    );
    assert!(scrap_measure < passes, "the scrap measurement does not open the page");
    let head = &text[..passes];
    assert!(head.contains("`inferred`"), "the head never names the fifth column's word");

    // The standing passes, moved whole: the interview's gate, the triage's
    // rule word — and the scrap pass ordered after the journal pass.
    assert!(journal_pass < scrap_pass, "the scrap pass does not follow the journal pass");
    for (pass, carries) in [
        ("the world pass", "The gate is question 2"),
        ("the world pass", "the day they stop, the fact is dead"),
        ("the journal pass", "The rule word is the whole triage"),
        ("the journal pass", "Measure the cost before paying it"),
    ] {
        assert!(text.contains(carries), "{pass} lost `{carries}`");
    }

    // The scrap pass: a completed plan's `waves/` goes — state confirmed in
    // `archi plan list`, old `plan.json` plans read the same way, one
    // `git rm -r`, one commit — a live plan's stays, and the journal is the
    // record deleting cannot lose.
    let scrap = &text[scrap_pass..];
    for carries in [
        "`archi plan list`",
        "`plan.json`",
        "`git rm -r",
        "A draft or started plan keeps its folder",
        "`archi check`",
        "commit naming the count",
        "The record of what those waves accounted for is the journal",
    ] {
        assert!(scrap.contains(carries), "the scrap pass lost `{carries}`");
    }

    // A tree holding `.fractal/` is the old client's, and its page keeps its
    // name — one pointer, no fourth in-project pass.
    let pointer = text.find(".fractal/").expect("the head never names `.fractal/`");
    assert!(pointer < passes, "the fractal pointer left the head");
    assert!(
        text.contains("`archi-migrate-fractal`"),
        "the page never names the old client's own page"
    );
}

/// A merge that retires a page leaves its installed copy standing as if
/// current unless the sync names it
/// (`archi/requirements/cold-start/one-door-migrates-the-standing-project.md`):
/// `sync-skills` reports an installed `archi-*` skill this binary does not
/// embed as orphaned, by name, removes nothing, and closes on its usual
/// verdict. A skill the user authored in the same folder is not archi's to
/// judge, so a name outside the `archi-` namespace is passed over.
#[test]
fn sync_skills_reports_an_orphaned_skill_and_removes_nothing() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);

    // The tree this very merge leaves behind on every deployed project: a
    // retired page still installed — and a skill of the user's own beside it.
    let orphan = root.join(".claude/skills/archi-migrate-world/SKILL.md");
    fs::create_dir_all(orphan.parent().unwrap()).unwrap();
    fs::write(&orphan, "the world interview, as it stood\n").unwrap();
    let own = root.join(".claude/skills/deploy/SKILL.md");
    fs::create_dir_all(own.parent().unwrap()).unwrap();
    fs::write(&own, "the user's own deploy notes\n").unwrap();

    let out = ok_in(&root, &["sync-skills"]);
    assert!(
        out.contains(
            "orphaned .claude/skills/archi-migrate-world/SKILL.md \
             (this binary embeds no such skill)"
        ),
        "{out}"
    );
    assert!(!out.contains("deploy"), "the user's own skill is judged:\n{out}");

    // Named, never removed — and the verdict is the usual one.
    assert_eq!(
        fs::read_to_string(&orphan).unwrap(),
        "the world interview, as it stood\n"
    );
    assert!(own.is_file());
    assert!(out.contains("already in sync"), "{out}");

    fs::remove_dir_all(&root).unwrap();
}

/// A migrated fact rests on nothing anybody observed, and the world says so
/// (`archi/requirements/world-facts/a-source-is-reachable-and-lives-in-the-world.md`).
///
/// This test asserted the opposite and is rewritten, not relaxed. It pinned the
/// rule the migration skill used to teach: write the intent a claim was lifted
/// from into `sources`, and the world counts the fact grounded and reports
/// nothing. The quiet was the defect. The spec is what the world conditions, so
/// a fact grounded in a requirement grounds itself in what it explains, and
/// `sources` now reaches only into `archi/world/`. What the requirement behind
/// this test asked for — a migrated project whose check a reader can trust — is
/// what it asks for still, and it is met now by the count telling the truth
/// instead of by the field being filled.
#[test]
fn a_migrated_fact_carries_no_source_and_the_world_says_so() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    fs::write(
        root.join("archi/src/model.arch"),
        "def node AuthService:\n  port handle_login\n",
    )
    .unwrap();
    let intent = root.join("archi/requirements/riding");
    fs::create_dir_all(&intent).unwrap();
    fs::write(
        intent.join("riding.md"),
        "# Riding\n\nPeople read on the move, and the line drops.\n",
    )
    .unwrap();

    let fact = |sources| util::Fact {
        covers: "AuthService",
        sources,
        uses: "",
        condition: "The carriage drops the network for minutes at a time, so a reader on the move \
                    works from what the device already holds.",
        workaround: "Readers load the page at the platform and redo the trip's work when they \
                     forget.",
        scenarios: "### The app opens with no network\n\n\
                    Given the device has no network\n\
                    When the reader opens the app\n\
                    Then the last synced view appears\n",
    };

    // The retired shape: the intent the sentence was lifted from, written in as
    // the fact's source. It is a located error now, and the message says why.
    let origin = "archi/requirements/riding/riding.md";
    fact(origin).write(&root, "trains-lose-the-signal", "Trains lose the signal");
    let (code, out, err) = run_in(&root, &["check"]);
    assert_eq!(code, Some(1), "the origin file still passes as a source:\n{out}{err}");
    let said = format!("{out}{err}");
    assert!(said.contains("lies outside"), "{said}");

    // The honest shape: the same record with an empty field. It lands clean,
    // and the one thing the world says about it is the one thing that is true —
    // nobody has been to look yet.
    fact("").write(&root, "trains-lose-the-signal", "Trains lose the signal");
    let check = ok_in(&root, &["check"]);
    assert!(check.contains("world — 1 facts · 1 ungrounded"), "{check}");
    assert!(
        check.contains("world fact `trains-lose-the-signal`: world_ungrounded"),
        "{check}"
    );

    fs::remove_dir_all(&root).unwrap();
}

/// The machine-provable half of the migration, walked as one flow: a project
/// that stands from before the world checks green with no world at all, the
/// upgrade hands it the verb and the procedure without moving the check by a
/// byte, and the first fact written the way the skill prescribes lands clean
/// (`archi/requirements/world-facts/a-skill-migrates-a-standing-project-into-the-world.md`,
/// `archi/requirements/world-facts/the-world-arrives-without-noise.md`).
///
/// Running the procedure is not machine-provable: the skill is a text a
/// person or an agent reads, and no test can read an intent and decide which
/// of its claims is a condition of the world. What the test proves is the
/// ground the reader stands on before and after.
#[test]
fn a_pre_world_project_upgrades_stays_green_and_takes_its_first_fact() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    fs::write(
        root.join("archi/src/model.arch"),
        "def node AuthService:\n  port handle_login\n",
    )
    .unwrap();
    // The material the skill reads: the intent the standing project captured.
    let intent = root.join("archi/requirements/riding");
    fs::create_dir_all(&intent).unwrap();
    fs::write(
        intent.join("riding.md"),
        "# Riding\n\nPeople read on the move, and the line drops.\n",
    )
    .unwrap();

    // The project as it stood before the world: the old block, the old
    // workflow skill, no migration skill — and no `archi/world/` at all.
    fs::write(
        root.join("CLAUDE.md"),
        "<!-- archi:begin -->\n## Archiplan\n\nthe briefing as it stood\n<!-- archi:end -->\n",
    )
    .unwrap();
    fs::write(
        root.join(".claude/skills/archi/SKILL.md"),
        "# Archi workflow\n\nthe loop as it stood\n",
    )
    .unwrap();
    fs::remove_dir_all(root.join(".claude/skills/archi-migrate")).unwrap();
    assert!(!root.join("archi/world").exists());

    // It checks green, and the world it never opted into says nothing.
    let before = ok_in(&root, &["check"]);
    assert!(!before.contains("world"), "{before}");

    // The upgrade: the rule in the block, the procedure in a skill beside
    // the others — and the check does not move by a byte.
    let out = ok_in(&root, &["sync-skills"]);
    assert!(
        out.contains("created  .claude/skills/archi-migrate/SKILL.md"),
        "{out}"
    );
    assert_eq!(
        fs::read_to_string(root.join(".claude/skills/archi-migrate/SKILL.md")).unwrap(),
        SKILL_MIGRATE
    );
    assert_eq!(ok_in(&root, &["check"]), before);

    // The first fact, written as the skill prescribes: the condition, what
    // people do instead, its scenarios, the node it conditions — and no source,
    // because the claim was lifted from prose and nobody has been to look. It
    // lands clean, and the world is born counted and honest about what it rests
    // on.
    util::Fact {
        covers: "AuthService",
        sources: "",
        uses: "",
        condition: "The carriage drops the network for minutes at a time, so a reader on the \
                    move works from what the device already holds.",
        workaround: "Readers load the page at the platform and redo the trip's work when they \
                     forget.",
        scenarios: "### The app opens with no network\n\n\
                    Given the device has no network\n\
                    When the reader opens the app\n\
                    Then the last synced view appears\n",
    }
    .write(&root, "trains-lose-the-signal", "Trains lose the signal");

    let after = ok_in(&root, &["check"]);
    assert!(after.contains("world — 1 facts · 1 ungrounded"), "{after}");
    assert!(
        after.contains("world fact `trains-lose-the-signal`: world_ungrounded"),
        "{after}"
    );

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_nested_init_names_the_enclosing_root() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let out = ok_in(&root, &["init", "services/billing"]);
    assert!(out.contains("enclosing project"), "{out}");
    assert!(root.join("services/billing/archi.toml").is_file());
    fs::remove_dir_all(&root).unwrap();
}

/// The writer declares what its code answers
/// (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`,
/// `archi/requirements/code-link/the-sub-agent-posts-its-declarations-before-it-returns.md`),
/// so the skill that briefs the writer names the verb whole. All three names
/// are required and all three resolve before a byte is written, so a skill
/// that names the verb with two of them briefs its reader into a refusal, and
/// a skill that hangs a fourth flag off it briefs them into a usage error.
#[test]
fn the_implement_skill_names_the_declaration_verb_with_its_three_flags() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let flat = flat(&implement_skill(&root));

    // Some mention of `link add` carries the whole invocation. Other mentions
    // are prose about the verb, and prose does not repeat a usage line.
    let full = flat
        .match_indices("link add")
        .map(|(at, _)| at)
        .find(|&at| {
            let window: String = flat[at..].chars().take(240).collect();
            ["--symbol", "--answers", "--proved-by"]
                .iter()
                .all(|flag| window.contains(flag))
        })
        .unwrap_or_else(|| panic!("no mention of `link add` names all three flags:\n{flat}"));

    // The plan's verb, not the journal's `archi link add`: the declaration is
    // filed against the task, which is what tells capture whose claim it is.
    let head = flat[..full]
        .rsplit_once("archi ")
        .map(|(_, tail)| tail.to_string())
        .unwrap_or_default();
    assert!(
        head.starts_with("plan task"),
        "the skill names `archi {head}link add`, not the plan's verb"
    );

    // The invocation is written as one code span, and every flag in it is one
    // the verb takes. An invented flag reaches the writer as a usage error
    // hours after the task, which is the whole cost this rewrite removes.
    let open = flat[..full].rfind('`').expect("the verb is written as code");
    let close = full + flat[full..].find('`').expect("the code span closes");
    let verb = &flat[open + 1..close];
    for token in verb.split_whitespace().filter(|t| t.starts_with("--")) {
        let flag = token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-');
        assert!(
            ["--symbol", "--answers", "--proved-by", "--project"].contains(&flag),
            "the skill names `{flag}`, which the verb does not take: `{verb}`"
        );
    }

    fs::remove_dir_all(&root).unwrap();
}

/// The declaration is the sub-agent's own last act, so it stands in the
/// contract the sub-agent is handed and not in the orchestrator's steps
/// (`archi/requirements/code-link/the-sub-agent-posts-its-declarations-before-it-returns.md`).
/// Several entries go through `archi batch -`: a task that defends four
/// symbols spends one process on them, and `--proved-by` names a test that
/// passes, which is why the declaration comes after the green run and not
/// before it.
#[test]
fn the_implement_skill_puts_the_declaration_in_the_sub_agent_contract() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let skill = implement_skill(&root);
    let contract = flat(passage(&skill, "The per-task contract"));

    for phrase in ["link add", "archi batch -"] {
        assert!(
            contract.contains(phrase),
            "the per-task contract never names `{phrase}`:\n{contract}"
        );
    }
    assert!(
        ["last act", "before it returns", "last thing"]
            .iter()
            .any(|when| contract.contains(when)),
        "the contract never says when the declaration is written:\n{contract}"
    );

    // The orchestrator builds the prompt from this contract, so the sentence
    // that lists what a prompt carries lists the declaration too. A step the
    // prompt omits is a step no sub-agent ever runs.
    let dispatch = flat(passage(&skill, "## Sub-agents"));
    assert!(
        dispatch.contains("declar"),
        "the dispatch never puts the declaration in the prompt:\n{dispatch}"
    );

    fs::remove_dir_all(&root).unwrap();
}

/// One verb is carved out, and the rule around it keeps its force
/// (`archi/requirements/code-link/the-sub-agent-posts-its-declarations-before-it-returns.md`).
/// A sub-agent that may run `link confirm` or `plan next` is a sub-agent that
/// closes its own wave, so the carve-out is named in the same sentence as the
/// rule — a reader who meets the rule meets its one exception, and cannot read
/// the exception as licence for the rest.
#[test]
fn the_implement_skill_keeps_plan_and_link_with_the_orchestrator_but_for_one_verb() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let skill = implement_skill(&root);
    let dispatch = flat(passage(&skill, "## Sub-agents"));

    assert!(
        dispatch.contains("Every `plan` and `link` command stays with you"),
        "the orchestrator-only rule is gone:\n{dispatch}"
    );
    let rule = dispatch
        .split(". ")
        .find(|s| s.contains("stays with you"))
        .expect("the rule is a sentence");
    assert!(
        ["except", "exception"].iter().any(|carve| rule.contains(carve)),
        "the rule names no exception: {rule}"
    );
    assert!(
        rule.contains("link add"),
        "the exception is not named as the declaration verb: {rule}"
    );

    fs::remove_dir_all(&root).unwrap();
}

/// The planner asks the record which files answer a ref
/// (`archi/requirements/agent-retrieval/the-briefing-sends-the-reader-to-the-record-before-the-tree.md`).
/// `## Outputs` is the slot a planner fills by guessing from file names, and
/// `archi link ls --spec <ref>` answers the same question from what is
/// recorded. The assertion stands on the bullet that authors the slot and not
/// on the skill as a whole: a read named in some other step is a read the
/// author meets after they have already guessed.
#[test]
fn the_planning_skill_seeds_its_outputs_from_the_record() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let installed = planning_skill(&root);

    let outputs = bullet(&installed, "- `## Outputs`");
    assert!(
        outputs.contains("archi link ls --spec"),
        "the `## Outputs` bullet names no read of the record: {outputs}"
    );
    // What the rows are. A command with no answer beside it is a command a
    // reader skips, and the guess it replaces survives.
    assert!(
        outputs.contains("record"),
        "the bullet never says the rows are what is already recorded: {outputs}"
    );

    fs::remove_dir_all(&root).unwrap();
}

/// The prompt carries the recorded files
/// (`archi/requirements/agent-retrieval/the-briefing-sends-the-reader-to-the-record-before-the-tree.md`).
/// The read belongs to the orchestrator, like every other `link` verb, so the
/// sub-agent runs no ritual of its own: the rows for the task's refs stand in
/// the prompt and the writer starts from them. An empty answer is an answer
/// too — nothing is recorded, the tree is the only source left, and what the
/// sub-agent finds there comes back as a report instead of an assumption
/// (`archi/world/facts/an-assistant-guesses-which-files-answer-a-written-obligation.md`).
#[test]
fn the_implement_skill_puts_the_recorded_files_in_the_sub_agent_prompt() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let skill = implement_skill(&root);
    let dispatch = flat(passage(&skill, "## Sub-agents"));

    assert!(
        dispatch.contains("archi link ls --spec"),
        "the dispatch never reads the record for the task's refs:\n{dispatch}"
    );
    assert!(
        ["nothing recorded", "nothing is recorded", "no rows"]
            .iter()
            .any(|empty| dispatch.contains(empty)),
        "the dispatch never says what an empty answer means:\n{dispatch}"
    );
    assert!(
        dispatch.contains("report"),
        "the dispatch never says the unrecorded finding is reported:\n{dispatch}"
    );

    fs::remove_dir_all(&root).unwrap();
}

/// Capture proposes no candidates any more — it mints what the declarations
/// name and nothing else
/// (`archi/requirements/code-link/the-writer-declares-what-the-code-answers.md`,
/// `archi/requirements/code-link/the-sub-agent-posts-its-declarations-before-it-returns.md`).
/// A skill that still sends its reader to review a candidate list sends them
/// to an empty one. Every embedded skill is read, not only the one this round
/// rewrote: the instruction is stale wherever it stands, and this is the test
/// that says so the next time a loop changes and a text does not.
#[test]
fn no_embedded_skill_sends_the_reader_to_confirm_candidates() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);

    for (name, embedded) in EMBEDDED_SKILLS {
        let installed =
            fs::read_to_string(root.join(".claude/skills").join(name).join("SKILL.md")).unwrap();
        assert_eq!(installed, embedded, "{name} drifted on install");
        // Read over the line breaks: a command split across two lines is the
        // same command.
        let flat = flat(&installed);
        // The verbs, and the vocabulary they were the verbs of. The verbs
        // alone were not enough: `skills/archi.md` promised the audit
        // reports "decayed evidence" and this guard read straight past it,
        // because the sentence names no command
        // (`archi/requirements/code-link/a-link-stands-asserted-or-it-does-not-stand.md`).
        for gone in [
            "link ls --evidence",
            "link confirm",
            "audit --prune",
            "decayed evidence",
            "evidence link",
            "confidence",
            // The verbs of the retired loop with the noun they acted on.
            // `candidate` alone is ordinary prose in these skills — a
            // candidate branch, a requirement candidate, a decomposition
            // candidate, the world conditions the migration interviews — so
            // what is forbidden is a candidate a reader is told to act on.
            // The `archi.md` failure mode said "confirm or retire the
            // candidates it just created" and the list above read past it,
            // because that sentence names no command
            // (`archi/requirements/agent-retrieval/the-briefing-sends-the-reader-to-the-record-before-the-tree.md`).
            "confirm the candidate",
            "confirm a candidate",
            "review the candidate",
            "review a candidate",
            "retire the candidate",
            "retire a candidate",
            "confirm or retire",
            "candidate link",
        ] {
            assert!(!flat.contains(gone), "{name} still sends its reader to `{gone}`");
        }
    }

    fs::remove_dir_all(&root).unwrap();
}

/// The writer of a claim reads the standing claims first
/// (`archi/requirements/agent-retrieval/the-briefing-sends-the-reader-to-the-record-before-the-tree.md`).
/// `archi req ls --satisfies <element>` puts the neighbouring claims on
/// screen before the file exists. The slice holds the derivation step
/// alone, opener to opener, and the read must stand before the mint: a
/// command met after `archi req add` is a command the writer meets once
/// the claim is already written.
#[test]
fn the_workflow_skill_reads_the_record_before_it_derives_a_requirement() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let skill = fs::read_to_string(root.join(".claude/skills/archi/SKILL.md")).unwrap();
    assert_eq!(skill, SKILL_ARCHI, "the workflow skill drifted on install");

    let start = skill.find("**Derive requirements.**").expect("the derivation step");
    let end = skill.find("**Draft the model.**").expect("the drafting step");
    let derive = flat(&skill[start..end]);

    let read = derive
        .find("archi req ls --satisfies")
        .expect("the derivation step names no read of the standing claims");
    let write = derive.find("archi req add").expect("the derivation step names the mint");
    assert!(read < write, "the read stands after the write: {derive}");
    // Why the rows are read. A command with no answer beside it is a
    // command a reader skips, and the guess it replaces survives.
    assert!(
        derive.contains("before the file exists"),
        "the step never says the claims are on screen before the file exists: {derive}"
    );

    fs::remove_dir_all(&root).unwrap();
}

/// A fresh init installs the search doctrine like every other skill —
/// byte-equal to the binary's embedded copy
/// (`archi/requirements/agent-retrieval/the-search-doctrine-lives-in-one-skill.md`).
#[test]
fn a_fresh_init_installs_the_search_skill_verbatim() {
    let root = temp_dir();
    let out = ok_in(&root, &["init", "."]);
    assert!(out.contains(".claude/skills/archi-search/SKILL.md"), "{out}");
    let installed =
        fs::read_to_string(root.join(".claude/skills/archi-search/SKILL.md")).unwrap();
    assert_eq!(installed, SKILL_SEARCH, "the search skill drifted on install");
    fs::remove_dir_all(&root).unwrap();
}

/// The embedded doctrine names its order — the semantic menu, then the
/// three structural reads from an element, then search — and says why
/// grep never answers
/// (`archi/requirements/agent-retrieval/the-search-doctrine-lives-in-one-skill.md`).
#[test]
fn the_search_skill_names_the_order_and_why_grep_misses() {
    let flat = flat(SKILL_SEARCH);
    let mut prev: Option<usize> = None;
    for step in [
        "`archi query --top`",
        "`archi req ls --satisfies <element>`",
        "`archi world ls --covers <element>`",
        "`archi link ls --spec <ref>`",
        "`archi search <phrase> [--kind",
    ] {
        let at = flat
            .find(step)
            .unwrap_or_else(|| panic!("the doctrine never names {step}"));
        if let Some(prev) = prev {
            assert!(prev < at, "{step} stands out of order: {flat}");
        }
        prev = Some(at);
    }
    assert!(
        flat.contains(
            "Grep misses the model, because definitions live in the compiled \
             graph and not on disk as prose."
        ),
        "the doctrine never says why grep misses the model: {flat}"
    );
}

/// Every working skill points at the doctrine by bare name, so the one
/// page is one pointer away wherever a reader stands
/// (`archi/requirements/agent-retrieval/the-search-doctrine-lives-in-one-skill.md`).
#[test]
fn every_working_skill_names_the_search_skill() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);

    for (name, embedded) in EMBEDDED_SKILLS {
        // Two exemptions. `ste-writing` is a prose-style reference, not an
        // archiplan workflow: it retrieves nothing from the spec, so a
        // pointer would brief nobody — the plan names it exempt.
        // `archi-finish-worktree` retrieves nothing either — its landing
        // follows the reports of the commands it runs — and it stands
        // outside the archi-search unit's outputs.
        if name == "ste-writing" || name == "archi-finish-worktree" {
            continue;
        }
        let installed =
            fs::read_to_string(root.join(".claude/skills").join(name).join("SKILL.md")).unwrap();
        assert_eq!(installed, embedded, "{name} drifted on install");
        assert!(installed.contains("archi-search"), "{name} never names `archi-search`");
    }

    fs::remove_dir_all(&root).unwrap();
}

/// The doctrine's distinctive sentences live in exactly one file, so there
/// is no second copy to drift
/// (`archi/requirements/agent-retrieval/the-search-doctrine-lives-in-one-skill.md`).
/// A bare command mention stays legitimate elsewhere — step 4 of the
/// workflow skill runs `req ls --satisfies`, the planning skill seeds
/// `## Outputs` with `link ls --spec` — so what is guarded is the phrase
/// and the glossed chain of the three reads, never a lone command.
#[test]
fn the_search_doctrine_lives_in_one_skill() {
    const PHRASE: &str = "Search, do not grep";
    const READS: &str = "`archi req ls --satisfies <element>` — the requirements that name it \
                         (similar and contradicting claims live in one cluster); \
                         `archi world ls --covers <element>` — the outside conditions on it; \
                         `archi link ls --spec <ref>` — the files recorded against an element";

    for (name, embedded) in EMBEDDED_SKILLS {
        let flat = flat(embedded);
        if name == "archi-search" {
            assert!(flat.contains(PHRASE), "the doctrine lost its phrase");
            assert!(flat.contains(READS), "the doctrine lost its ordered reads: {flat}");
        } else {
            assert!(!flat.contains(PHRASE), "{name} carries a second copy of `{PHRASE}`");
            assert!(!flat.contains(READS), "{name} carries a second copy of the ordered reads");
        }
    }
}

/// A fresh init installs the explain page like every other skill —
/// byte-equal to the binary's embedded copy — and the page's freshness
/// header names its own installed path, so the staleness loop can close
/// (`archi/requirements/agent-retrieval/the-why-reads-back-from-the-record.md`).
#[test]
fn a_fresh_init_installs_the_explain_skill_verbatim() {
    let root = temp_dir();
    let out = ok_in(&root, &["init", "."]);
    assert!(out.contains(".claude/skills/archi-explain/SKILL.md"), "{out}");
    let installed =
        fs::read_to_string(root.join(".claude/skills/archi-explain/SKILL.md")).unwrap();
    assert_eq!(installed, SKILL_EXPLAIN, "the explain skill drifted on install");
    assert!(
        installed.contains(".claude/skills/archi-explain/SKILL.md"),
        "the freshness header never names the installed path"
    );
    fs::remove_dir_all(&root).unwrap();
}

/// The embedded page orders the chain outside-in: the world condition the
/// behavior serves, then what must hold, then the recorded trades, then
/// the pressure behind them, then the timeline, then who realizes it
/// today. The stressor step is addressed by the requirement's own
/// `origin:` field, and the timeline reads as list and diff — the tree
/// never moves
/// (`archi/requirements/agent-retrieval/the-why-reads-back-from-the-record.md`).
#[test]
fn the_explain_page_orders_the_chain_world_first_links_last() {
    let flat = flat(SKILL_EXPLAIN);
    let mut prev: Option<usize> = None;
    for step in [
        "`archi world ls --covers <element>`",
        "`archi req ls --satisfies <element>`",
        "`archi decision ls --links <name>`",
        "origin: stressor(<slug>)",
        "`archi version list`",
        "`archi link ls --spec <ref>`",
    ] {
        let at = flat
            .find(step)
            .unwrap_or_else(|| panic!("the explain page never names {step}"));
        if let Some(prev) = prev {
            assert!(prev < at, "{step} stands out of order: {flat}");
        }
        prev = Some(at);
    }
    assert!(flat.contains("`archi version diff <a> <b>`"), "{flat}");
    assert!(flat.contains("the tree never moves"), "{flat}");
    assert!(
        !flat.contains("version checkout"),
        "the page still teaches the checkout dance: {flat}"
    );
}

/// Silence is a real answer and invention is forbidden, both in as many
/// words: a question with no recorded trade-off is answered "the record
/// holds no rationale here" plus an offer to record one
/// (`archi/requirements/agent-retrieval/the-why-reads-back-from-the-record.md`).
#[test]
fn the_explain_page_calls_silence_a_real_answer_and_never_invents() {
    let flat = flat(SKILL_EXPLAIN);
    assert!(flat.contains("Silence is a real answer"), "{flat}");
    assert!(flat.contains("the record holds no rationale here"), "{flat}");
    assert!(flat.contains("offer to record one"), "{flat}");
    assert!(flat.contains("Never invent rationale"), "{flat}");
}

/// The page is read-only in as many words and mutates nothing
/// (`archi/requirements/agent-retrieval/the-why-reads-back-from-the-record.md`).
#[test]
fn the_explain_page_is_read_only_in_as_many_words() {
    let flat = flat(SKILL_EXPLAIN);
    assert!(flat.contains("Read-only"), "{flat}");
    assert!(flat.contains("Mutate nothing"), "{flat}");
}

/// The question resolves before the chain
/// (`archi/requirements/agent-retrieval/the-why-reads-back-from-the-record.md`):
/// the element's definition is read and its identity sentence quoted — the
/// subject before the why — and a question that fits several addresses goes
/// to the user as options, never guessed. Both stand before the chain pulls.
#[test]
fn the_explain_page_resolves_the_subject_before_the_why() {
    let flat = flat(SKILL_EXPLAIN);
    let chain = flat.find("## Pull the explanation").expect("the page has no chain");
    for phrase in [
        "the element's definition",
        "identity sentence",
        "the subject before the why",
        "fits several addresses",
        "as options, never guessed",
    ] {
        let at = flat
            .find(phrase)
            .unwrap_or_else(|| panic!("the explain page misses `{phrase}`"));
        assert!(at < chain, "`{phrase}` stands after the chain opens: {flat}");
    }
}

/// The origin hop names both births
/// (`archi/requirements/agent-retrieval/the-why-reads-back-from-the-record.md`):
/// a claim's `origin:` is read in the requirements step, and beside the
/// stressor hop the page names the intent one — `origin: intent` answers
/// from the intent folder's own problem statement — without moving the
/// chain's numbered order.
#[test]
fn the_explain_page_names_the_intent_origin_beside_the_stressor() {
    let flat = flat(SKILL_EXPLAIN);
    let reqs = flat
        .find("`archi req ls --satisfies <element>`")
        .expect("the page has no requirements step");
    let trades = flat
        .find("`archi decision ls --links <name>`")
        .expect("the page has no decisions step");
    let intent = flat
        .find("`origin: intent`")
        .expect("the page never names the intent origin");
    assert!(
        reqs < intent && intent < trades,
        "the intent hop does not stand beside the stressor one in step 2"
    );
    assert!(
        flat.contains("the intent folder's own problem statement"),
        "the page never says where an intent-born claim answers from: {flat}"
    );
}
