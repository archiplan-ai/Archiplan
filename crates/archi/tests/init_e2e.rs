//! End to end through the real binary: one command stands a project up
//! (`archi/requirements/cold-start/one-verb-stands-a-project-up`), a second
//! run changes no bytes, the manifest routes the starter and aborts the
//! broken run, the briefing lands verbatim, and the commands around init keep
//! their contracts.
//!
//! The briefing carries the wing (`archi/requirements/world-facts/`): the
//! `world` verb and the no-model-nouns rule stand in both the workflow skill
//! and the CLAUDE.md block, the workflow captures the world before it derives
//! requirements, and `archi-migrate-world` installs beside the other skills so
//! a project that stands without a wing can gain one. The planning skill moved
//! with the behaviour too: it collects its closing block from the world and
//! asks for none of it.

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// This binary's embedded briefing sources, for byte-equality checks.
const SKILL_ARCHI: &str = include_str!("../../../skills/archi.md");
const SKILL_PLAN: &str = include_str!("../../../skills/archi-plan.md");
const SKILL_MERGE: &str = include_str!("../../../skills/archi-merge.md");
const SKILL_MIGRATE: &str = include_str!("../../../skills/archi-migrate-fractal.md");
const SKILL_MIGRATE_WORLD: &str = include_str!("../../../skills/archi-migrate-world.md");

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
    assert_eq!(created.len(), 12, "{out}");
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
fn the_briefing_carries_the_wing() {
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
    // stands in both carriers of the briefing. The verb and its subcommands
    // stand in the skill alone: the block stopped repeating what `archi --help`
    // prints (`the-briefing-says-what-help-does-not`), and a briefing whose two
    // carriers say the same thing twice is one carrier plus a copy that rots.
    let claude = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    for text in [&skill, &claude] {
        assert!(
            text.contains("without the nouns of the model"),
            "the no-model-nouns rule is missing:\n{text}"
        );
    }

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

    // Where the block comes from, and who authors it: nobody.
    for phrase in [
        "collected",
        "world fact",
        "holds a task for",
        "writes none of it",
        "archi plan scenarios list",
    ] {
        assert!(flat.contains(phrase), "the skill misses `{phrase}`");
    }

    // An empty block is spec work, not a blank to fill.
    for phrase in [
        "no world fact covers any node this plan builds",
        "spec work",
        "not a blank to fill",
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

/// The block says what help does not
/// (`archi/requirements/world-facts/the-briefing-says-what-help-does-not.md`).
/// `archi --help` prints every verb with its flags and the harness lists the
/// installed skills, so a copy of either in the block is a copy that goes
/// stale. What is left is the part an agent reads nowhere else — and the length
/// is counted, so the block cannot grow back one bullet at a time.
#[test]
fn the_briefing_says_what_help_does_not() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);
    let block = block_of(&root);

    // No flags: help owns them.
    let flags: Vec<&str> = block
        .split_whitespace()
        .filter(|t| {
            t.trim_matches(|c: char| c == '`' || c == '(' || c == ')' || c == ',')
                .strip_prefix("--")
                .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_alphanumeric()))
        })
        .collect();
    assert!(flags.is_empty(), "the block lists flags {flags:?}:\n{block}");

    // No command syntax beyond the one loop the block exists to state.
    let forms: Vec<&str> = block
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|s| s.starts_with("archi "))
        .collect();
    assert!(!forms.is_empty(), "the check loop lost its verb:\n{block}");
    assert!(
        forms.iter().all(|f| *f == "archi check"),
        "the block spells out {forms:?}:\n{block}"
    );

    // No skill inventory: the harness hands the agent that list already.
    assert!(!block.contains(".claude/skills"), "{block}");
    assert!(!block.contains("SKILL.md"), "{block}");
    for skill in [
        "archi-plan",
        "archi-implement",
        "archi-merge",
        "archi-finish-worktree",
        "archi-migrate-fractal",
        "archi-migrate-world",
    ] {
        assert!(
            !block.contains(skill),
            "the block still inventories `{skill}`:\n{block}"
        );
    }

    // What survives the cut: the lines that live only here.
    assert!(block.contains("never design"), "{block}");
    assert!(block.contains("archi/versions/"), "{block}");
    assert!(block.contains("FILES"), "{block}");
    assert!(block.contains("paths, not payloads"), "{block}");
    assert!(block.contains("without the nouns of the model"), "{block}");

    // The budget, over the installed text: a new bullet has to pay for itself.
    let lines = block.lines().count();
    assert!(lines < 20, "the block is {lines} lines:\n{block}");

    fs::remove_dir_all(&root).unwrap();
}

/// The migration skill asks the way the first real run taught it to
/// (`archi/requirements/world-facts/a-skill-migrates-a-standing-project-into-the-wing.md`):
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
fn a_pre_wing_project_syncs_into_the_wing() {
    let root = temp_dir();
    ok_in(&root, &["init", "."]);

    // A project scaffolded before the wing: its block and its workflow skill
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
    assert!(claude.contains("without the nouns of the model"), "{claude}");
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
    // stops a fact being written, the provenance a migrated fact records, the
    // brief it hands back, and the check it closes on
    // (archi/requirements/world-facts/a-skill-migrates-a-standing-project-into-the-wing.md).
    for phrase in [
        "archi world add",
        "workaround",
        "what people do today instead",
        "writes nothing",
        "only a wish",
        "did not map",
        "names the file the claim came from",
        "provenance, not observation",
        "deletes nothing",
        "archi check",
    ] {
        assert!(installed.contains(phrase), "the skill misses `{phrase}`");
    }

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_migrated_fact_rests_on_its_origin_file_and_reports_nothing() {
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

    // The record the skill leaves behind: the condition, its killer, its
    // scenarios, the node it conditions, and the intent the claim was lifted
    // from — the provenance that makes the fact rest on something recorded.
    util::Fact {
        covers: "AuthService",
        sources: "archi/requirements/riding/riding.md",
        uses: "",
        condition: "The carriage drops the network for minutes at a time, so a reader on the move \
                    works from what the device already holds.",
        killer: "Trackside coverage that never drops.",
        scenarios: "Feature: Offline open\n  \
                    Scenario: the app opens with no network\n    \
                    Given the device has no network\n    When the reader opens the app\n    \
                    Then the last synced view appears\n",
    }
    .write(&root, "trains-lose-the-signal", "Trains lose the signal");

    // The wing counts the fact as grounded and says nothing else about it: a
    // migration that swapped one finding for another would defeat its purpose.
    let check = ok_in(&root, &["check"]);
    assert!(check.contains("world — 1 facts · 0 ungrounded"), "{check}");
    assert!(!check.contains("world_"), "{check}");

    fs::remove_dir_all(&root).unwrap();
}

/// The machine-provable half of the migration, walked as one flow: a project
/// that stands from before the wing checks green with no wing at all, the
/// upgrade hands it the verb and the procedure without moving the check by a
/// byte, and the first fact written the way the skill prescribes lands clean
/// (`archi/requirements/world-facts/a-skill-migrates-a-standing-project-into-the-wing.md`,
/// `archi/requirements/world-facts/the-wing-arrives-without-noise.md`).
///
/// Running the procedure is not machine-provable: the skill is a text a
/// person or an agent reads, and no test can read an intent and decide which
/// of its claims is a condition of the world. What the test proves is the
/// ground the reader stands on before and after.
#[test]
fn a_pre_wing_project_upgrades_stays_green_and_takes_its_first_fact() {
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

    // The project as it stood before the wing: the old block, the old
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

    // It checks green, and the wing it never opted into says nothing.
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
    assert!(
        fs::read_to_string(root.join("CLAUDE.md"))
            .unwrap()
            .contains("without the nouns of the model")
    );
    assert_eq!(ok_in(&root, &["check"]), before);

    // The first fact, written as the skill prescribes: the condition, its
    // killer, its scenarios, the node it conditions, and the intent the
    // claim was lifted from as its source. It lands clean, and the wing is
    // born counted.
    util::Fact {
        covers: "AuthService",
        sources: "archi/requirements/riding/riding.md",
        uses: "",
        condition: "The carriage drops the network for minutes at a time, so a reader on the \
                    move works from what the device already holds.",
        killer: "Trackside coverage that never drops.",
        scenarios: "Feature: Offline open\n  \
                    Scenario: the app opens with no network\n    \
                    Given the device has no network\n    When the reader opens the app\n    \
                    Then the last synced view appears\n",
    }
    .write(&root, "trains-lose-the-signal", "Trains lose the signal");

    let after = ok_in(&root, &["check"]);
    assert!(after.contains("world — 1 facts · 0 ungrounded"), "{after}");
    assert!(!after.contains("world_"), "{after}");

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
