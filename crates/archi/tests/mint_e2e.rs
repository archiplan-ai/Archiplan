//! End to end through the real binary: doc skeletons come from commands —
//! `req add|rm`, `stress open|add|rm` — every machine field explicit or
//! derived, text slots held empty by the schema's own diagnostics, removals
//! pre-flighted (`archi/requirements/spec-docs/skeletons-come-from-a-verb.md`).
//! `req ls` is the read over the standing set
//! (`archi/requirements/agent-retrieval/one-verb-lists-the-requirements-an-element-carries.md`);
//! `decision ls` is the same read over the standing decisions
//! (`archi/requirements/agent-retrieval/one-verb-lists-the-decisions-on-an-element.md`).

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const MODEL: &str = "def conn wire := * -> *\n\
                     def node Gate:\n  port serve\n\
                     def node Ledger:\n  port keep\n\
                     Gate.serve wire Ledger.keep\n\
                     Service type_of Gate\n";

fn temp_project() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-mint-e2e-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(dir.join("archi/src")).unwrap();
    fs::write(
        dir.join("archi.toml"),
        "[project]\nname = \"t\"\npreset = \"default\"\n",
    )
    .unwrap();
    fs::write(dir.join("archi/src/model.arch"), MODEL).unwrap();
    let intent = dir.join("archi/requirements/hardening");
    fs::create_dir_all(&intent).unwrap();
    fs::write(
        intent.join("hardening.md"),
        "# Hardening\n\nThe area under pressure.\n",
    )
    .unwrap();
    util::worktree(&dir)
}

fn run(root: &Path, args: &[&str]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_archi"))
        .args(args)
        .args(["--project", root.to_str().unwrap()])
        .output()
        .expect("archi runs");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn ok(root: &Path, args: &[&str]) -> String {
    let (success, stdout, stderr) = run(root, args);
    assert!(success, "archi {args:?} failed:\n{stdout}\n{stderr}");
    stdout
}

fn refuse(root: &Path, args: &[&str]) -> String {
    let (success, stdout, stderr) = run(root, args);
    assert!(!success, "archi {args:?} passed but had to refuse:\n{stdout}");
    stderr
}

#[test]
fn the_round_lifecycle_is_verbs_with_derived_placement() {
    let root = temp_project();

    // no saved version: nothing to press
    let e = refuse(&root, &["stress", "open", "First pressure"]);
    assert!(e.contains("version save"), "{e}");
    ok(&root, &["version", "save", "-m", "seed"]);

    // open derives the pinned version and the folder from the slug
    let out = ok(&root, &["stress", "open", "First pressure"]);
    assert!(out.contains("archi/stress/first-pressure/first-pressure.md"), "{out}");
    let charter = fs::read_to_string(root.join("archi/stress/first-pressure/first-pressure.md")).unwrap();
    assert_eq!(charter, "---\nversion: v0001\nclosed:\n---\n\n# First pressure\n");

    // one round at a time
    let e = refuse(&root, &["stress", "open", "Second pressure"]);
    assert!(e.contains("first-pressure"), "{e}");
    assert!(e.contains("already open"), "{e}");

    // a moved model refuses the next open toward save (after this round closes)
    // -- checked below, after the close.

    // stressors land in the open round; affects validate against the pin
    let e = refuse(&root, &["stress", "add", "Burst load"]);
    assert!(e.contains("--affects"), "{e}");
    let e = refuse(&root, &["stress", "add", "Burst load", "--affects", "Ghost,Gate.serve wire Ledger.keep"]);
    assert!(e.contains("Ghost"), "{e}");
    assert!(e.contains("never edges"), "{e}");
    let out = ok(&root, &["stress", "add", "Burst load", "--affects", "Gate,Service"]);
    assert!(out.contains("archi/stress/first-pressure/burst-load.md"), "{out}");
    let st = fs::read_to_string(root.join("archi/stress/first-pressure/burst-load.md")).unwrap();
    assert_eq!(
        st,
        "---\naffects: [Gate, Service]\noutcome: pending\n---\n\n# Burst load\n\n## Attractor\n\n## Resolution\n"
    );
    // a re-mint with different affects is a different record — loud
    let e = refuse(&root, &["stress", "add", "Burst load", "--affects", "Gate"]);
    assert!(e.contains("moved past its skeleton"), "{e}");
    // the identical re-mint converges
    let out = ok(&root, &["stress", "add", "Burst load", "--affects", "Gate,Service"]);
    assert!(out.contains("already minted"), "{out}");

    // the empty text slots are the un-skippable worklist: check errors
    let (success, _out, err) = run(&root, &["check"]);
    assert!(!success, "empty slots must hold the check");
    assert!(err.contains("summary paragraph"), "{err}");

    // fill the prose; check comes back
    fs::write(
        root.join("archi/stress/first-pressure/first-pressure.md"),
        "---\nversion: v0001\nclosed:\n---\n\n# First pressure\n\nPress the seed model.\n",
    )
    .unwrap();
    fs::write(
        root.join("archi/stress/first-pressure/burst-load.md"),
        "---\naffects: [Gate, Service]\noutcome: pending\n---\n\n# Burst load\n\n\
         Organic peaks arrive 100x.\n\n## Attractor\n\nThe gate saturates.\n\n## Resolution\n",
    )
    .unwrap();
    ok(&root, &["check"]);

    // a pending stressor in the open round removes; the closed round is sealed
    ok(&root, &["stress", "add", "Cold cache", "--affects", "Ledger"]);
    ok(&root, &["stress", "rm", "cold-cache"]);
    assert!(!root.join("archi/stress/first-pressure/cold-cache.md").exists());
    ok(&root, &["version", "save", "-m", "close the round"]);
    let e = refuse(&root, &["stress", "rm", "burst-load"]);
    assert!(e.contains("sealed"), "{e}");
}

#[test]
fn requirements_mint_explicitly_and_removals_preflight() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);

    // nothing is defaulted: missing parameters are a usage refusal
    let (_, _, e) = run(&root, &["req", "add", "Gate throttles"]);
    assert!(e.contains("--intent"), "{e}");
    // unknown intent lists the existing ones
    let e = refuse(&root, &[
        "req", "add", "Gate throttles", "--intent", "nope", "--kind", "functional", "--origin", "intent",
    ]);
    assert!(e.contains("hardening"), "{e}");
    // kind and origin validate
    let e = refuse(&root, &[
        "req", "add", "Gate throttles", "--intent", "hardening", "--kind", "sorta", "--origin", "intent",
    ]);
    assert!(e.contains("functional | non-functional"), "{e}");
    let e = refuse(&root, &[
        "req", "add", "Gate throttles", "--intent", "hardening", "--kind", "functional", "--origin", "parent",
    ]);
    assert!(e.contains("not mintable"), "{e}");
    let e = refuse(&root, &[
        "req", "add", "Gate throttles", "--intent", "hardening", "--kind", "functional",
        "--origin", "stressor(ghost)",
    ]);
    assert!(e.contains("no stressor `ghost`"), "{e}");

    // the mint: exact schema shape, empty slots
    ok(&root, &[
        "req", "add", "Gate throttles", "--intent", "hardening", "--kind", "functional", "--origin", "intent",
    ]);
    let req = fs::read_to_string(root.join("archi/requirements/hardening/gate-throttles.md")).unwrap();
    assert_eq!(
        req,
        "---\nkind: functional\norigin: intent\nsatisfied-by: []\ndeferred:\n---\n\n\
         # Gate throttles\n\n## System Context\n\n## Satisfy\n"
    );
    let out = ok(&root, &[
        "req", "add", "Gate throttles", "--intent", "hardening", "--kind", "functional", "--origin", "intent",
    ]);
    assert!(out.contains("already minted"), "{out}");

    // a stressor-derived requirement validates its origin against a real slug
    ok(&root, &["stress", "open", "Round one"]);
    ok(&root, &["stress", "add", "Replay burst", "--affects", "Gate"]);
    ok(&root, &[
        "req", "add", "Replays are refused", "--intent", "hardening", "--kind", "functional",
        "--origin", "stressor(replay-burst)", "--deferred", "until the gateway lands",
    ]);
    let req = fs::read_to_string(root.join("archi/requirements/hardening/replays-are-refused.md")).unwrap();
    assert!(req.contains("origin: stressor(replay-burst)"), "{req}");
    assert!(req.contains("deferred: until the gateway lands"), "{req}");

    // the stressor a requirement derives from is held in place
    let e = refuse(&root, &["stress", "rm", "replay-burst"]);
    assert!(e.contains("replays-are-refused"), "{e}");

    // an unheld requirement removes; a plan-owned one refuses with the plan
    ok(&root, &["req", "rm", "replays-are-refused"]);
    assert!(!root.join("archi/requirements/hardening/replays-are-refused.md").exists());
    // fill prose so the plan sees a satisfied requirement; wire it to Gate
    fs::write(
        root.join("archi/requirements/hardening/gate-throttles.md"),
        "---\nkind: functional\norigin: intent\nsatisfied-by: [Gate]\ndeferred:\n---\n\n\
         # Gate throttles\n\nThe gate sheds load.\n\n## System Context\n\n## Satisfy\n\n\
         The gate throttles.\n\n- test — burst returns 429\n",
    )
    .unwrap();
    // close the open round so the tree is check-clean for the plan
    fs::write(
        root.join("archi/stress/round-one/round-one.md"),
        "---\nversion: v0001\nclosed:\n---\n\n# Round one\n\nPress.\n",
    )
    .unwrap();
    fs::write(
        root.join("archi/stress/round-one/replay-burst.md"),
        "---\naffects: [Gate]\noutcome: pending\n---\n\n# Replay burst\n\nReplays.\n\n\
         ## Attractor\n\n## Resolution\n",
    )
    .unwrap();
    ok(&root, &["version", "save", "-m", "wire"]);
    ok(&root, &["plan", "use", "guard"]);
    ok(&root, &["plan", "task", "add", "Gate"]);
    // ownership is authored in the task file now — edit its `owns`
    let task = root.join("archi/plans/guard/t1-gate.md");
    let owned = fs::read_to_string(&task)
        .unwrap()
        .replace("owns: []", "owns: [gate-throttles]");
    fs::write(&task, owned).unwrap();
    let e = refuse(&root, &["req", "rm", "gate-throttles"]);
    assert!(e.contains("plan `guard`"), "{e}");
    assert!(e.contains("t1"), "{e}");
}

#[test]
fn a_whole_round_materializes_from_one_batch_and_the_guard_covers_every_line() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);

    let batch = "stress open \"Round one\"\n\
                 stress add \"Burst load\" --affects Gate\n\
                 stress add \"Cold cache\" --affects Ledger\n\
                 req add \"Gate throttles\" --intent hardening --kind functional --origin intent\n";
    let out = Command::new(env!("CARGO_BIN_EXE_archi"))
        .args(["batch", "-", "--project", root.to_str().unwrap()])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut c| {
            use std::io::Write;
            c.stdin.take().unwrap().write_all(batch.as_bytes())?;
            c.wait_with_output()
        })
        .expect("batch runs");
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    for f in [
        "archi/stress/round-one/round-one.md",
        "archi/stress/round-one/burst-load.md",
        "archi/stress/round-one/cold-cache.md",
        "archi/requirements/hardening/gate-throttles.md",
    ] {
        assert!(root.join(f).is_file(), "{f} did not materialize");
    }

    // the mutation guard covers the new commands — an unbound checkout refuses
    let primary = {
        // the fixture root's primary checkout is the worktree's origin
        let top = root.parent().unwrap().parent().unwrap();
        top.join(root.parent().unwrap().file_name().unwrap().to_str().unwrap().trim_end_matches("-worktrees"))
    };
    let (success, _o, e) = run(&primary, &["stress", "add", "Rogue", "--affects", "Gate"]);
    assert!(!success);
    assert!(e.contains("unbound") || e.contains("archi worktree mint"), "{e}");
}

#[test]
fn a_replayed_batch_converges() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);

    fn batch(root: &Path, lines: &str) -> (bool, String, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_archi"))
            .args(["batch", "-", "--project", root.to_str().unwrap()])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut c| {
                use std::io::Write;
                c.stdin.take().unwrap().write_all(lines.as_bytes())?;
                c.wait_with_output()
            })
            .expect("batch runs");
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    // The round's batch dies on line 3 — a bad affects entry.
    let (success, _o, e) = batch(
        &root,
        "stress open \"Round one\"\n\
         stress add \"Burst load\" --affects Gate\n\
         stress add \"Cold cache\" --affects Ghost\n\
         stress add \"Slow drain\" --affects Ledger\n",
    );
    assert!(!success);
    assert!(e.contains("Ghost"), "{e}");
    assert!(root.join("archi/stress/round-one/burst-load.md").is_file());
    assert!(!root.join("archi/stress/round-one/slow-drain.md").exists(), "fail-fast held");

    // Replaying the whole batch, fixed, converges: applied lines answer
    // with continuations and exit zero, the tail lands.
    let (success, out, e) = batch(
        &root,
        "stress open \"Round one\"\n\
         stress add \"Burst load\" --affects Gate\n\
         stress add \"Cold cache\" --affects Ledger\n\
         stress add \"Slow drain\" --affects Ledger\n",
    );
    assert!(success, "the replay must converge:\n{e}");
    assert!(out.contains("already open — this is it"), "{out}");
    assert!(out.contains("already minted"), "{out}");
    assert!(root.join("archi/stress/round-one/cold-cache.md").is_file());
    assert!(root.join("archi/stress/round-one/slow-drain.md").is_file());

    // A skeleton that moved past itself is not re-mintable.
    let filled = root.join("archi/stress/round-one/burst-load.md");
    let text = fs::read_to_string(&filled).unwrap().replace(
        "# Burst load\n",
        "# Burst load\n\nOrganic peaks arrive 100x.\n",
    );
    fs::write(&filled, text).unwrap();
    let e = refuse(&root, &["stress", "add", "Burst load", "--affects", "Gate"]);
    assert!(e.contains("moved past its skeleton"), "{e}");

    // req add converges the same way.
    ok(&root, &[
        "req", "add", "Gate throttles", "--intent", "hardening", "--kind", "functional", "--origin", "intent",
    ]);
    let out = ok(&root, &[
        "req", "add", "Gate throttles", "--intent", "hardening", "--kind", "functional", "--origin", "intent",
    ]);
    assert!(out.contains("already minted"), "{out}");
    // ...but different parameters are a different record: loud.
    let e = refuse(&root, &[
        "req", "add", "Gate throttles", "--intent", "hardening", "--kind", "non-functional", "--origin", "intent",
    ]);
    assert!(e.contains("moved past its skeleton") || e.contains("not re-mintable"), "{e}");
}

// ---- `req ls` — the read verb over the standing requirements ---------------
// (`archi/requirements/agent-retrieval/one-verb-lists-the-requirements-an-element-carries.md`)

/// One standing requirement on disk, in the shape `req add` mints and a
/// person fills. `satisfied_by` is the frontmatter list body (`Gate, Ledger`),
/// `deferred` the reason or empty, `summary` the prose paragraph — empty
/// leaves the minted hole. `satisfied-by` and the `Satisfy` prose hold
/// together, so a non-empty claim brings its section prose.
fn req_file(root: &Path, intent: &str, slug: &str, title: &str, satisfied_by: &str, deferred: &str, summary: &str) {
    let dir = root.join("archi/requirements").join(intent);
    fs::create_dir_all(&dir).unwrap();
    let deferred = if deferred.is_empty() {
        String::new()
    } else {
        format!(" {deferred}")
    };
    let summary = if summary.is_empty() {
        String::new()
    } else {
        format!("\n{summary}\n")
    };
    let satisfy = if satisfied_by.is_empty() {
        String::new()
    } else {
        "\nHeld by the named elements.\n\n- test — the named suite passes\n".to_string()
    };
    fs::write(
        dir.join(format!("{slug}.md")),
        format!(
            "---\nkind: functional\norigin: intent\nsatisfied-by: [{satisfied_by}]\ndeferred:{deferred}\n---\n\n\
             # {title}\n{summary}\n## System Context\n\n## Satisfy\n{satisfy}"
        ),
    )
    .unwrap();
}

/// One standing decision on disk, in the shape a person writes
/// (`archi/requirements/agent-retrieval/one-verb-lists-the-decisions-on-an-element.md`):
/// the checked `links` — doc slugs and model elements mixed — the trade's
/// two sides, both legally empty, and the rationale prose behind the name.
fn decision_file(root: &Path, slug: &str, title: &str, links: &str, prefer: &str, over: &str, rationale: &str) {
    let dir = root.join("archi/decisions");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("{slug}.md")),
        format!(
            "---\nlinks: [{links}]\nprefer: [{prefer}]\nover: [{over}]\n---\n\n\
             # {title}\n\n{rationale}\n"
        ),
    )
    .unwrap();
}

/// The standing set the listing tests read: two intent areas, six file-scale
/// requirements — satisfied, deferred and open, one satisfied by a port, one
/// still the minted skeleton — and two standing decisions: one linking a
/// model element and a requirement slug, one an empty-sided, link-less record
/// (one-verb-lists-the-decisions-on-an-element).
fn listing_project() -> PathBuf {
    let root = temp_project();
    let retrieval = root.join("archi/requirements/retrieval");
    fs::create_dir_all(&retrieval).unwrap();
    fs::write(retrieval.join("retrieval.md"), "# Retrieval\n\nThe reading area.\n").unwrap();
    req_file(&root, "hardening", "gate-throttles", "Gate throttles", "Gate", "",
        "The gate sheds load before the ledger sees it. A second sentence rides behind.");
    req_file(&root, "hardening", "replays-are-refused", "Replays are refused", "",
        "until the gateway lands", "Replays die at the door.");
    req_file(&root, "hardening", "still-a-hole", "Still a hole", "", "", "");
    req_file(&root, "retrieval", "born-before-the-model", "Born before the model", "", "",
        "A claim minted before any element stands.");
    req_file(&root, "retrieval", "keep-answers", "Keep answers", "Ledger.keep", "",
        "The keep port answers every read.");
    req_file(&root, "retrieval", "ledger-is-append-only", "Ledger is append only", "Ledger, Gate", "",
        "The ledger takes writes at the end only.");
    decision_file(&root, "accept-shed-load", "Accept shed load", "Gate, gate-throttles",
        "simplicity", "scalability",
        "The gate sheds before the ledger sees the spike. A second sentence stays behind.");
    decision_file(&root, "keep-the-monolith", "Keep the monolith", "", "", "",
        "One deployable until the seams are proven.");
    root
}

/// The standing requirement files on disk: every `.md` under an intent
/// folder except the intent's own anchor. The fixture is flat — no promoted
/// requirement folders — so the walk is one level.
fn files_on_disk(root: &Path) -> usize {
    let base = root.join("archi/requirements");
    let mut n = 0;
    for intent in fs::read_dir(&base).unwrap().flatten() {
        if !intent.path().is_dir() {
            continue;
        }
        let islug = intent.file_name().to_string_lossy().into_owned();
        for f in fs::read_dir(intent.path()).unwrap().flatten() {
            let p = f.path();
            if p.extension().is_some_and(|e| e == "md")
                && p.file_stem().is_some_and(|s| s != islug.as_str())
            {
                n += 1;
            }
        }
    }
    n
}

/// `req ls` prints one row per standing requirement — slug, state,
/// `satisfied-by`, the first phrase of the summary — and the count matches
/// the files on disk (one-verb-lists-the-requirements-an-element-carries).
#[test]
fn the_listing_prints_one_row_per_standing_requirement() {
    let root = listing_project();
    let out = ok(&root, &["req", "ls"]);
    assert_eq!(out.lines().count(), files_on_disk(&root), "{out}");
    for slug in [
        "gate-throttles",
        "replays-are-refused",
        "still-a-hole",
        "born-before-the-model",
        "keep-answers",
        "ledger-is-append-only",
    ] {
        assert_eq!(
            out.lines().filter(|l| l.starts_with(slug)).count(),
            1,
            "{slug}\n{out}"
        );
    }
    // The intent anchors are areas, not rows.
    assert!(!out.lines().any(|l| l.starts_with("hardening") || l.starts_with("retrieval")), "{out}");
    // The row: slug, state, `satisfied-by`, first phrase — one line, and the
    // second sentence stays behind.
    assert!(
        out.contains("gate-throttles  satisfied  [Gate]  The gate sheds load before the ledger sees it."),
        "{out}"
    );
    assert!(!out.contains("A second sentence"), "{out}");
    // The state is the compiler's own grading: a deferral reason defers.
    assert!(
        out.contains("replays-are-refused  deferred  []  Replays die at the door."),
        "{out}"
    );
    // Tree order: intent folder, then path within it.
    let pos = |s: &str| out.find(s).unwrap();
    assert!(pos("gate-throttles") < pos("replays-are-refused"), "{out}");
    assert!(pos("still-a-hole") < pos("born-before-the-model"), "{out}");
    assert!(pos("keep-answers") < pos("ledger-is-append-only"), "{out}");
}

/// `--satisfies <element>` prints exactly the requirements whose
/// `satisfied-by` names it — the entry, not the hub around it
/// (one-verb-lists-the-requirements-an-element-carries).
#[test]
fn satisfies_narrows_to_the_requirements_naming_the_element() {
    let root = listing_project();
    let out = ok(&root, &["req", "ls", "--satisfies", "Gate"]);
    assert_eq!(out.lines().count(), 2, "{out}");
    assert!(out.contains("gate-throttles"), "{out}");
    assert!(out.contains("ledger-is-append-only"), "{out}");
    // The port's claim is the port's: `Ledger` does not gather `Ledger.keep`.
    let out = ok(&root, &["req", "ls", "--satisfies", "Ledger"]);
    assert_eq!(out.lines().count(), 1, "{out}");
    assert!(out.contains("ledger-is-append-only"), "{out}");
    let out = ok(&root, &["req", "ls", "--satisfies", "Ledger.keep"]);
    assert_eq!(out.lines().count(), 1, "{out}");
    assert!(out.contains("keep-answers"), "{out}");
}

/// A `--satisfies` name no model holds refuses, naming it — a refusal is
/// never an empty list (one-verb-lists-the-requirements-an-element-carries).
#[test]
fn satisfies_refuses_a_name_no_model_holds() {
    let root = listing_project();
    let e = refuse(&root, &["req", "ls", "--satisfies", "Ghost"]);
    assert!(e.contains("Ghost"), "{e}");
    assert!(e.contains("--satisfies"), "{e}");
    assert!(e.contains("names no element"), "{e}");
}

/// `--intent` narrows to the folder; an unknown folder lists the folders,
/// as `req add` already does
/// (one-verb-lists-the-requirements-an-element-carries).
#[test]
fn intent_narrows_and_an_unknown_folder_lists_the_folders() {
    let root = listing_project();
    let out = ok(&root, &["req", "ls", "--intent", "retrieval"]);
    assert_eq!(out.lines().count(), 3, "{out}");
    for slug in ["born-before-the-model", "keep-answers", "ledger-is-append-only"] {
        assert!(out.contains(slug), "{out}");
    }
    assert!(!out.contains("gate-throttles"), "{out}");
    let e = refuse(&root, &["req", "ls", "--intent", "nope"]);
    assert!(e.contains("no intent `nope`"), "{e}");
    assert!(e.contains("hardening"), "{e}");
    assert!(e.contains("retrieval"), "{e}");
}

/// A requirement with an empty `satisfied-by` lists, its emptiness visible —
/// born-before-the-model is a legal state
/// (one-verb-lists-the-requirements-an-element-carries).
#[test]
fn an_empty_satisfied_by_lists_visibly() {
    let root = listing_project();
    let out = ok(&root, &["req", "ls"]);
    assert!(
        out.contains("born-before-the-model  open  []  A claim minted before any element stands."),
        "{out}"
    );
    // The minted skeleton — no summary yet — is still a whole row.
    assert!(out.lines().any(|l| l == "still-a-hole  open  []"), "{out}");
}

/// `--json` carries the same rows as the render, in the same order
/// (one-verb-lists-the-requirements-an-element-carries).
#[test]
fn json_carries_the_same_rows_as_the_render() {
    let root = listing_project();
    let rendered = ok(&root, &["req", "ls"]);
    let v: serde_json::Value =
        serde_json::from_str(&ok(&root, &["req", "ls", "--json"])).unwrap();
    assert_eq!(v["status"], "ok");
    let rows = v["requirements"].as_array().unwrap();
    assert_eq!(rows.len(), rendered.lines().count());
    for (line, row) in rendered.lines().zip(rows) {
        let slug = row["slug"].as_str().unwrap();
        let state = row["state"].as_str().unwrap();
        let named: Vec<&str> = row["satisfied_by"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap())
            .collect();
        let summary = row["summary"].as_str().unwrap();
        let mut want = format!("{slug}  {state}  [{}]", named.join(", "));
        if !summary.is_empty() {
            want.push_str("  ");
            want.push_str(summary);
        }
        assert_eq!(line, want);
    }
    // The row is addressable: the envelope carries the file.
    assert_eq!(rows[0]["path"], "archi/requirements/hardening/gate-throttles.md");
    // The filter narrows the envelope the same way.
    let v: serde_json::Value = serde_json::from_str(&ok(
        &root,
        &["req", "ls", "--satisfies", "Gate", "--json"],
    ))
    .unwrap();
    assert_eq!(v["requirements"].as_array().unwrap().len(), 2);
    assert_eq!(v["satisfies"], "Gate");
}

// ---- `decision ls` — the read verb over the standing decisions -------------
// (`archi/requirements/agent-retrieval/one-verb-lists-the-decisions-on-an-element.md`)

/// The standing decision files on disk: every `.md` under the flat
/// `archi/decisions/`.
fn decision_files_on_disk(root: &Path) -> usize {
    fs::read_dir(root.join("archi/decisions"))
        .unwrap()
        .flatten()
        .filter(|f| f.path().extension().is_some_and(|e| e == "md"))
        .count()
}

/// `decision ls` prints one row per decision file — slug, `prefer -> over`,
/// the first phrase of the rationale — and the count matches the files on
/// disk (one-verb-lists-the-decisions-on-an-element).
#[test]
fn the_decision_listing_prints_one_row_per_decision_file() {
    let root = listing_project();
    let out = ok(&root, &["decision", "ls"]);
    assert_eq!(out.lines().count(), decision_files_on_disk(&root), "{out}");
    // The row: slug, the trade, first phrase — one line, and the second
    // sentence stays behind.
    assert!(
        out.contains(
            "accept-shed-load  [simplicity] -> [scalability]  \
             The gate sheds before the ledger sees the spike."
        ),
        "{out}"
    );
    assert!(!out.contains("A second sentence"), "{out}");
    // An empty trade is a legal non-comparative record: both sides shown
    // empty, the row whole.
    assert!(
        out.contains("keep-the-monolith  [] -> []  One deployable until the seams are proven."),
        "{out}"
    );
}

/// `--links <element>` narrows to the decisions naming it, and
/// `--links <slug>` does the same for a doc slug; a link-less decision never
/// matches a filter (one-verb-lists-the-decisions-on-an-element).
#[test]
fn links_narrows_by_element_and_by_doc_slug() {
    let root = listing_project();
    // The model currency: `Gate` is a live element.
    let out = ok(&root, &["decision", "ls", "--links", "Gate"]);
    assert_eq!(out.lines().count(), 1, "{out}");
    assert!(out.contains("accept-shed-load"), "{out}");
    // The doc currency: `gate-throttles` is a requirement slug.
    let out = ok(&root, &["decision", "ls", "--links", "gate-throttles"]);
    assert_eq!(out.lines().count(), 1, "{out}");
    assert!(out.contains("accept-shed-load"), "{out}");
    // A name that resolves but no decision links is an empty list, not a
    // refusal — and the link-less record matches nothing.
    let out = ok(&root, &["decision", "ls", "--links", "Ledger"]);
    assert_eq!(out.lines().count(), 0, "{out}");
    // The decisions are records themselves: their slugs resolve too.
    let out = ok(&root, &["decision", "ls", "--links", "keep-the-monolith"]);
    assert_eq!(out.lines().count(), 0, "{out}");
}

/// `--links` with a name that resolves as neither element nor record
/// refuses, naming it — a refusal is never an empty list
/// (one-verb-lists-the-decisions-on-an-element).
#[test]
fn links_refuses_a_name_that_is_neither_element_nor_record() {
    let root = listing_project();
    let e = refuse(&root, &["decision", "ls", "--links", "Ghost"]);
    assert!(e.contains("Ghost"), "{e}");
    assert!(e.contains("--links"), "{e}");
    assert!(e.contains("names no"), "{e}");
}

/// `--json` carries the same rows as the render, in the same order
/// (one-verb-lists-the-decisions-on-an-element).
#[test]
fn decision_json_carries_the_same_rows_as_the_render() {
    let root = listing_project();
    let rendered = ok(&root, &["decision", "ls"]);
    let v: serde_json::Value =
        serde_json::from_str(&ok(&root, &["decision", "ls", "--json"])).unwrap();
    assert_eq!(v["status"], "ok");
    let rows = v["decisions"].as_array().unwrap();
    assert_eq!(rows.len(), rendered.lines().count());
    for (line, row) in rendered.lines().zip(rows) {
        let side = |key: &str| -> Vec<&str> {
            row[key]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap())
                .collect()
        };
        let mut want = format!(
            "{}  [{}] -> [{}]",
            row["slug"].as_str().unwrap(),
            side("prefer").join(", "),
            side("over").join(", ")
        );
        let summary = row["summary"].as_str().unwrap();
        if !summary.is_empty() {
            want.push_str("  ");
            want.push_str(summary);
        }
        assert_eq!(line, want);
    }
    // The row is addressable: the envelope carries the file.
    assert_eq!(rows[0]["path"], "archi/decisions/accept-shed-load.md");
    // The filter narrows the envelope the same way.
    let v: serde_json::Value = serde_json::from_str(&ok(
        &root,
        &["decision", "ls", "--links", "Gate", "--json"],
    ))
    .unwrap();
    assert_eq!(v["decisions"].as_array().unwrap().len(), 1);
    assert_eq!(v["links"], "Gate");
}
