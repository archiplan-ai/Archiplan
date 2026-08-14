//! End to end through the real binary: one command stands a project up
//! (`archi/requirements/cold-start/one-verb-stands-a-project-up`), a second
//! run changes no bytes, the manifest routes the starter and aborts the
//! broken run, the briefing lands verbatim, and the commands around init keep
//! their contracts.
//!
//! The skills carry the world (`archi/requirements/world-facts/`): the `world`
//! verb and the no-model-nouns rule stand in the workflow skill, the workflow
//! captures the world before it derives requirements, and
//! `archi-migrate-world` installs beside the other skills so a project that
//! stands without a world can gain one. The planning skill moved with the
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
//! rows for the task's refs in the sub-agent prompt, and the candidate guard
//! reads the sentences that name no command too.

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
const SKILL_MIGRATE: &str = include_str!("../../../skills/archi-migrate-fractal.md");
const SKILL_MIGRATE_WORLD: &str = include_str!("../../../skills/archi-migrate-world.md");
const SKILL_MIGRATE_LINKS: &str = include_str!("../../../skills/archi-migrate-links.md");
const SKILL_STE: &str = include_str!("../../../skills/ste-writing.md");

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
        fs::read_to_string(root.join(".claude/skills/archi-migrate-world/SKILL.md")).unwrap();
    assert_eq!(workflow, SKILL_ARCHI, "the workflow skill drifted on install");
    assert_eq!(migration, SKILL_MIGRATE_WORLD, "the migration skill drifted on install");
    (workflow, migration)
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
    assert_eq!(created.len(), 13, "{out}");
    assert!(created.last().unwrap().contains("archi.toml"), "{out}");
    assert!(out.contains("initialized `proj`"), "{out}");

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
        ("archi-migrate-fractal", SKILL_MIGRATE),
        ("archi-migrate-world", SKILL_MIGRATE_WORLD),
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
    let installed = fs::read_to_string(root.join(".claude/skills/archi-plan/SKILL.md")).unwrap();

    // Installed byte-equal to the embedded copy, as every other skill is.
    assert_eq!(installed, SKILL_PLAN, "the planning skill drifted on install");

    // The prose is hard-wrapped, so a sentence is read over its line breaks:
    // what the skill says must not depend on where a line ends.
    let flat = installed.split_whitespace().collect::<Vec<_>>().join(" ");

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

    let installed = [("archi", workflow.as_str()), ("archi-migrate-world", migration.as_str())];
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
    let installed = [("archi", workflow.as_str()), ("archi-migrate-world", migration.as_str())];
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
    let installed = [("archi", workflow.as_str()), ("archi-migrate-world", migration.as_str())];

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
        fs::read_to_string(root.join(".claude/skills/archi-migrate-world/SKILL.md")).unwrap();

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
    fs::remove_dir_all(root.join(".claude/skills/archi-migrate-world")).unwrap();

    let out = ok_in(&root, &["sync-skills"]);
    assert!(out.contains("updated  .claude/skills/archi/SKILL.md"), "{out}");
    assert!(
        out.contains("created  .claude/skills/archi-migrate-world/SKILL.md"),
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
        fs::read_to_string(root.join(".claude/skills/archi-migrate-world/SKILL.md")).unwrap(),
        SKILL_MIGRATE_WORLD
    );

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn the_migration_skill_installs_and_names_its_gate() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let path = root.join(".claude/skills/archi-migrate-world/SKILL.md");
    assert_eq!(fs::read_to_string(&path).unwrap(), SKILL_MIGRATE_WORLD);

    // A drifted copy is reclaimed by sync, as every other skill's is.
    fs::write(&path, "locally tuned\n").unwrap();
    let out = ok_in(&root, &["sync-skills"]);
    assert!(
        out.contains("updated  .claude/skills/archi-migrate-world/SKILL.md"),
        "{out}"
    );
    let installed = fs::read_to_string(&path).unwrap();
    assert_eq!(installed, SKILL_MIGRATE_WORLD);

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
    fs::remove_dir_all(root.join(".claude/skills/archi-migrate-world")).unwrap();
    assert!(!root.join("archi/world").exists());

    // It checks green, and the world it never opted into says nothing.
    let before = ok_in(&root, &["check"]);
    assert!(!before.contains("world"), "{before}");

    // The upgrade: the rule in the block, the procedure in a skill beside
    // the others — and the check does not move by a byte.
    let out = ok_in(&root, &["sync-skills"]);
    assert!(
        out.contains("created  .claude/skills/archi-migrate-world/SKILL.md"),
        "{out}"
    );
    assert_eq!(
        fs::read_to_string(root.join(".claude/skills/archi-migrate-world/SKILL.md")).unwrap(),
        SKILL_MIGRATE_WORLD
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
    let installed = fs::read_to_string(root.join(".claude/skills/archi-plan/SKILL.md")).unwrap();
    assert_eq!(installed, SKILL_PLAN, "the planning skill drifted on install");

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

    for (name, embedded) in [
        ("archi", SKILL_ARCHI),
        ("archi-plan", SKILL_PLAN),
        ("archi-implement", SKILL_IMPLEMENT),
        ("archi-merge", SKILL_MERGE),
        ("archi-finish-worktree", SKILL_FINISH),
        ("archi-migrate-fractal", SKILL_MIGRATE),
        ("archi-migrate-world", SKILL_MIGRATE_WORLD),
        ("archi-migrate-links", SKILL_MIGRATE_LINKS),
        ("ste-writing", SKILL_STE),
    ] {
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
