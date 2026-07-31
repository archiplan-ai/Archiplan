//! End to end through the real binary: doc skeletons come from commands —
//! `req add|rm`, `stress open|add|rm` — every machine field explicit or
//! derived, text slots held empty by the schema's own diagnostics, removals
//! pre-flighted (`archi/requirements/spec-docs/skeletons-come-from-a-verb.md`).

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
    assert!(e.contains("unbound") || e.contains("worktree"), "{e}");
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
