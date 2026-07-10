//! End to end through the real binary: the joins that merge-pressure broke
//! (`archi/requirements/self-hosting/parallel-editing-discipline.md`). The journal folds concurrent branch
//! histories in either union order; `version remint` re-mints a discarded
//! save onto the merged lineage and re-stamps its round; `version diff`
//! takes `live`; archive diagnostics name their recipes instead of
//! cascading.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const MODEL: &str = "def conn wire := * -> *\n\
                     def node Gate:\n  port out\n\
                     def node Auth:\n  port inn\n\
                     Gate.out wire Auth.inn\n";

const GATE_RS: &str = "pub fn gate() -> bool {\n    true\n}\n";

fn temp_project() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-multiplayer-e2e-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(dir.join("archi/src")).unwrap();
    fs::create_dir_all(dir.join("code")).unwrap();
    fs::write(
        dir.join("archi.toml"),
        "[project]\nname = \"t\"\npreset = \"default\"\n",
    )
    .unwrap();
    fs::write(dir.join("archi/src/model.arch"), MODEL).unwrap();
    fs::write(dir.join("code/gate.rs"), GATE_RS).unwrap();
    dir
}

/// Run the binary; return (success, stdout, stderr).
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

fn journal_path(root: &Path) -> PathBuf {
    root.join("archi/links/journal.jsonl")
}

fn read_journal(root: &Path) -> String {
    fs::read_to_string(journal_path(root)).unwrap_or_default()
}

fn write_journal(root: &Path, text: &str) {
    fs::create_dir_all(root.join("archi/links")).unwrap();
    fs::write(journal_path(root), text).unwrap();
}

// ---- the fold survives a merge ---------------------------------------------

#[test]
fn parallel_adds_union_fold_in_either_order() {
    let root = temp_project();

    // Writer one mints on their branch.
    let (ok, out, err) = run(&root, &["link", "add", "Gate", "code/gate.rs#gate", "--kind", "indirect"]);
    assert!(ok, "{err}");
    let line_a = read_journal(&root).trim().to_string();

    // Writer two mints from the same base: same sequence number, different
    // suffix — the ids cannot collide.
    write_journal(&root, "");
    let (ok, _, err) = run(&root, &["link", "add", "Auth", "code/gate.rs#gate", "--kind", "indirect"]);
    assert!(ok, "{err}");
    let line_b = read_journal(&root).trim().to_string();

    let id = |line: &str| {
        let at = line.find("\"id\":\"").unwrap() + 6;
        line[at..].split('"').next().unwrap().to_string()
    };
    let (id_a, id_b) = (id(&line_a), id(&line_b));
    assert!(id_a.starts_with("l0001-"), "{id_a}");
    assert!(id_b.starts_with("l0001-"), "{id_b}");
    assert_ne!(id_a, id_b, "parallel mints must not collide");
    assert!(out.contains(&id_a));

    // The union, in either order, folds to the same two live links.
    for union in [
        format!("{line_a}\n{line_b}\n"),
        format!("{line_b}\n{line_a}\n"),
    ] {
        write_journal(&root, &union);
        let (ok, out, err) = run(&root, &["link", "ls"]);
        assert!(ok, "{err}");
        assert!(out.contains(&id_a) && out.contains(&id_b), "{out}");
    }
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn retire_repin_union_folds_both_orders_and_surfaces_the_absorption() {
    let root = temp_project();
    let (ok, _, err) = run(&root, &["link", "add", "Gate", "code/gate.rs#gate", "--kind", "indirect"]);
    assert!(ok, "{err}");
    let base = read_journal(&root);
    let id = {
        let at = base.find("\"id\":\"").unwrap() + 6;
        base[at..].split('"').next().unwrap().to_string()
    };

    // Writer one retires; writer two repins, each from the same base.
    let (ok, _, err) = run(&root, &["link", "rm", &id]);
    assert!(ok, "{err}");
    let retire_line = read_journal(&root).lines().last().unwrap().to_string();

    write_journal(&root, &base);
    let (ok, _, err) = run(&root, &["link", "repin", &id]);
    assert!(ok, "{err}");
    let repin_line = read_journal(&root).lines().last().unwrap().to_string();

    // Both union orders fold, to the same live set: the link is retired —
    // the subtraction sticks — and the tombstone-landing event is absorbed
    // and surfaced, not corruption.
    for (union, absorbed_expected) in [
        (format!("{base}{repin_line}\n{retire_line}\n"), false),
        (format!("{base}{retire_line}\n{repin_line}\n"), true),
    ] {
        write_journal(&root, &union);
        let (ok, out, err) = run(&root, &["link", "ls"]);
        assert!(ok, "{err}");
        assert!(!out.contains(&id), "retired in either order: {out}");
        let (ok, out, err) = run(&root, &["link", "verify"]);
        assert!(ok, "{err}");
        assert_eq!(
            out.contains("absorbed"),
            absorbed_expected,
            "verify surfaces exactly the tombstone-landing interleave: {out}"
        );
    }
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn an_id_never_minted_still_refuses_the_fold() {
    let root = temp_project();
    write_journal(
        &root,
        "{\"event\":\"confirm\",\"id\":\"l9999-abcdef\",\"at\":\"2026-01-01T00:00:00Z\"}\n",
    );
    let (ok, _, err) = run(&root, &["link", "ls"]);
    assert!(!ok);
    assert!(err.contains("journal corrupt") && err.contains("never minted"), "{err}");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn the_union_merge_attribute_ships_beside_the_journal() {
    let root = temp_project();
    let (ok, _, err) = run(&root, &["link", "add", "Gate", "code/gate.rs#gate", "--kind", "indirect"]);
    assert!(ok, "{err}");
    let ga = fs::read_to_string(root.join("archi/links/.gitattributes")).unwrap();
    assert!(ga.contains("journal.jsonl merge=union"), "{ga}");
    fs::remove_dir_all(&root).unwrap();
}

// ---- remint rejoins the lineage ---------------------------------------------

fn git(root: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["-c", "user.name=t", "-c", "user.email=t@t.t"])
        .args(args)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn remint_rejoins_the_lineage_and_restamps_the_round() {
    let root = temp_project();
    if !git(&root, &["init", "-q"]) {
        return; // no git in this environment
    }
    let (ok, _, err) = run(&root, &["version", "save", "-m", "base"]);
    assert!(ok, "{err}");
    assert!(git(&root, &["add", "-A"]) && git(&root, &["commit", "-qm", "base"]));

    // The winner lands first: a port on Gate, saved as v0002 on main.
    assert!(git(&root, &["checkout", "-qb", "winner"]));
    fs::write(
        root.join("archi/src/model.arch"),
        MODEL.replace("def node Gate:\n  port out", "def node Gate:\n  port out\n  port late"),
    )
    .unwrap();
    let (ok, _, err) = run(&root, &["version", "save", "-m", "winner: Gate.late"]);
    assert!(ok, "{err}");
    assert!(git(&root, &["add", "-A"]) && git(&root, &["commit", "-qm", "winner v0002"]));

    // The loser runs a whole round from the same base: session, answer
    // (node Loser), save — also v0002, closed: stamped v0002.
    assert!(git(&root, &["checkout", "-q", "main"]) || git(&root, &["checkout", "-q", "master"]));
    assert!(git(&root, &["checkout", "-qb", "loser"]));
    let sdir = root.join("archi/stress/loser-round");
    fs::create_dir_all(&sdir).unwrap();
    fs::write(
        sdir.join("loser-round.md"),
        "---\nversion: v0001\nclosed:\n---\n\n# Loser round\n\nThe round the merge discards.\n",
    )
    .unwrap();
    fs::write(
        sdir.join("squeeze.md"),
        "---\naffects: [Gate]\noutcome: breaking\n---\n\n# Squeeze\n\nPressure.\n\n\
         ## Attractor\n\nBends.\n\n## Resolution\n\nAnswered by node Loser.\n",
    )
    .unwrap();
    fs::write(root.join("archi/src/model.arch"), format!("{MODEL}def node Loser\n")).unwrap();
    let (ok, out, err) = run(&root, &["version", "save", "-m", "loser: node Loser"]);
    assert!(ok, "{err}");
    assert!(out.contains("closed stress session `loser-round`"), "{out}");
    assert!(
        fs::read_to_string(sdir.join("loser-round.md")).unwrap().contains("closed: v0002")
    );
    assert!(git(&root, &["add", "-A"]) && git(&root, &["commit", "-qm", "loser v0002"]));

    // The merge: both minted v0002. Keep the winner's archive, keep the
    // merged model, then remint the loser's round onto the lineage.
    assert!(!git(&root, &["merge", "winner"]), "the collision conflicts");
    // Keep the winner's archive wholesale: the manifest and whichever
    // encoding its v0002 took (this tiny model keyframes).
    assert!(git(&root, &["checkout", "--theirs", "archi/versions/"]));
    assert!(git(&root, &["add", "-A"]) && git(&root, &["commit", "-qm", "merge: keep winner v0002"]));

    let (ok, out, err) = run(
        &root,
        &["version", "remint", "-m", "loser: node Loser (reminted)", "--session", "loser-round"],
    );
    assert!(ok, "{err}");
    assert!(out.contains("reminted v0003"), "{out}");
    assert!(out.contains("re-stamped session `loser-round` — closed: v0003"), "{out}");
    assert!(out.contains("commit as one:"), "{out}");
    assert!(
        fs::read_to_string(sdir.join("loser-round.md")).unwrap().contains("closed: v0003"),
        "the round record follows its answers onto the merged lineage"
    );

    // The lineage is whole and the reminted patch is exactly the loser's delta.
    let (ok, out, _) = run(&root, &["version", "list"]);
    assert!(ok);
    for id in ["v0001", "v0002", "v0003"] {
        assert!(out.contains(id), "{out}");
    }
    let (ok, out, _) = run(&root, &["version", "diff", "v0002", "v0003"]);
    assert!(ok);
    assert!(out.contains("+def node Loser"), "{out}");
    assert!(!out.contains("+  port late"), "the winner's delta is not re-minted: {out}");
    let (ok, _, _) = run(&root, &["check"]);
    assert!(ok, "the merged, reminted tree checks clean");
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn remint_refuses_unchanged_unknown_and_open() {
    let root = temp_project();
    let (ok, _, err) = run(&root, &["version", "save", "-m", "base"]);
    assert!(ok, "{err}");

    // Unchanged model: nothing to carry.
    let (ok, _, err) = run(&root, &["version", "remint", "-m", "x"]);
    assert!(!ok);
    assert!(err.contains("nothing to remint"), "{err}");

    // Unknown session.
    fs::write(root.join("archi/src/model.arch"), format!("{MODEL}def node Extra\n")).unwrap();
    let (ok, _, err) = run(&root, &["version", "remint", "-m", "x", "--session", "ghost"]);
    assert!(!ok);
    assert!(err.contains("no session `ghost`"), "{err}");

    // Open session: an open round closes through `version save`.
    let sdir = root.join("archi/stress/open-round");
    fs::create_dir_all(&sdir).unwrap();
    fs::write(
        sdir.join("open-round.md"),
        "---\nversion: v0001\nclosed:\n---\n\n# Open round\n\nStill pressing.\n",
    )
    .unwrap();
    let (ok, _, err) = run(&root, &["version", "remint", "-m", "x", "--session", "open-round"]);
    assert!(!ok);
    assert!(err.contains("is open"), "{err}");
    fs::remove_dir_all(&root).unwrap();
}

// ---- merge deltas are reviewable --------------------------------------------

#[test]
fn diff_live_shows_the_unsaved_delta() {
    let root = temp_project();
    let (ok, _, err) = run(&root, &["version", "save", "-m", "base"]);
    assert!(ok, "{err}");
    fs::write(root.join("archi/src/model.arch"), format!("{MODEL}def node Extra\n")).unwrap();

    let (ok, out, err) = run(&root, &["version", "diff", "v0001", "live"]);
    assert!(ok, "{err}");
    assert!(out.contains("+def node Extra"), "{out}");
    let (ok, out, _) = run(&root, &["version", "diff", "live", "v0001"]);
    assert!(ok);
    assert!(out.contains("-def node Extra"), "{out}");

    let (ok, _, err) = run(&root, &["version", "diff", "v0001", "nope"]);
    assert!(!ok, "unknown ids still refuse: {err}");
    fs::remove_dir_all(&root).unwrap();
}

// ---- diagnostics name their recipes ------------------------------------------

#[test]
fn conflict_markers_name_the_recipe_and_sessions_stay_quiet() {
    let root = temp_project();
    let (ok, _, err) = run(&root, &["version", "save", "-m", "base"]);
    assert!(ok, "{err}");
    let sdir = root.join("archi/stress/quiet-round");
    fs::create_dir_all(&sdir).unwrap();
    fs::write(
        sdir.join("quiet-round.md"),
        "---\nversion: v0001\nclosed: v0001\n---\n\n# Quiet round\n\nClosed and valid.\n",
    )
    .unwrap();

    let index = root.join("archi/versions/index.toml");
    let text = fs::read_to_string(&index).unwrap();
    fs::write(&index, format!("<<<<<<< HEAD\n{text}=======\n{text}>>>>>>> theirs\n")).unwrap();

    let (ok, _, err) = run(&root, &["check"]);
    assert!(!ok, "conflict markers are loud");
    assert!(
        err.contains("merge conflict markers") && err.contains("version remint"),
        "the diagnostic names the recipe: {err}"
    );
    assert!(
        !err.contains("names no archived version"),
        "one archive error, no session cascade: {err}"
    );
    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_half_shipped_save_names_the_travel_rule() {
    let root = temp_project();
    let (ok, out, err) = run(&root, &["version", "save", "-m", "base"]);
    assert!(ok, "{err}");
    assert!(
        out.contains("commit as one: archi/versions/index.toml, archi/versions/v0001.arch"),
        "the save prints its commit unit: {out}"
    );
    fs::remove_file(root.join("archi/versions/v0001.arch")).unwrap();
    let (ok, _, err) = run(&root, &["check"]);
    assert!(!ok);
    assert!(err.contains("travel as one commit"), "{err}");
    fs::remove_dir_all(&root).unwrap();
}
