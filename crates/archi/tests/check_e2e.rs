//! End to end through the real binary: a passing `check` closes on the
//! landscape read (`archi/requirements/cli/`, `archi/requirements/scoring/the-landscape-is-a-slice.md`) —
//! the NKP scoring line and the refactoring directions it implies — while
//! findings stay advisory and do not withhold it, an empty landscape earns
//! no read, and an error (archive, compile) withholds it entirely.

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// A landscape with one hotspot and one safe corridor: X gathers three
/// couplings (K̄ + σ_K puts it alone above the line), while the P → Q pair
/// couples only internally — a SAFE corridor whose action is ENCAPSULATE.
const COUPLED: &str = "def conn wire := * -> *\n\
                       def node A:\n  port out\n\
                       def node B:\n  port out\n\
                       def node C:\n  port out\n\
                       def node X:\n  port a\n  port b\n  port c\n\
                       def node P:\n  port out\n\
                       def node Q:\n  port inn\n\
                       A.out wire X.a\n\
                       B.out wire X.b\n\
                       C.out wire X.c\n\
                       P.out wire Q.inn\n";

/// A gate reaching an engine, an island nothing arrives at, and two payloads
/// the ontology classifies as `Data` — the smallest tree the coverage
/// question has both kinds of answer for.
const CARRIED: &str = "def conn wire := * -> *\n\
                       def node Gate:\n  port out\n\
                       def node Engine:\n  port inn\n\
                       def node Island\n\
                       def node Payload\n\
                       def node Receipt\n\
                       Gate.out wire Engine.inn\n\
                       Data type_of Payload\n\
                       Data type_of Receipt\n";

fn temp_project(model: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-check-e2e-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(dir.join("archi/src")).unwrap();
    fs::write(
        dir.join("archi.toml"),
        "[project]\nname = \"t\"\npreset = \"default\"\n",
    )
    .unwrap();
    fs::write(dir.join("archi/src/model.arch"), model).unwrap();
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

/// The note every fact here rests on, and the `sources` entry that names it:
/// a source names a file of the world and resolves against it
/// (`archi/requirements/world-facts/a-source-is-reachable-and-lives-in-the-world.md`).
const NOTE: &str = "archi/world/notes/the-guard-walked-the-platform.md";

fn put(root: &Path, rel_path: &str, text: &str) {
    let path = root.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// One strict record, in the layer the strict record lives in
/// (`archi/requirements/world-facts/the-world-holds-four-layers.md`), with the
/// note it is grounded in beside it. The shared skeleton still writes into the
/// wing's root, which is no layer at all, so this family writes its own.
fn fact(root: &Path, slug: &str, title: &str, covers: &str, condition: &str, scenarios: &str) {
    put(
        root,
        NOTE,
        "# The guard walked the platform\n\nHe timed the tunnel once at four minutes.\n",
    );
    put(
        root,
        &format!("archi/world/facts/{slug}.md"),
        &format!(
            "---\ncovers: [{covers}]\nsources: [{NOTE}]\nuses: []\n---\n\n\
             # {title}\n\n{condition}\n\n\
             ## What kills this\n\nTrackside coverage that never drops.\n\n\
             ## Scenarios\n\n{scenarios}"
        ),
    );
}

/// The one standing fact of the coverage tests: it conditions the gate, and
/// nothing else, so every other element on [`CARRIED`] answers the coverage
/// question for its own reason.
fn gate_fact(root: &Path) {
    fact(
        root,
        "trains-lose-the-signal",
        "Trains lose the signal",
        "Gate",
        "The carriage drops the network for minutes at a time.",
        "### the app opens with no network\n\n\
         Given the device has no network\nWhen the user opens the app\n\
         Then the last synced view appears\n",
    );
}

#[test]
fn a_passing_check_closes_on_the_landscape_read() {
    let root = temp_project(COUPLED);
    let out = ok(&root, &["check"]);

    // A blank line between the findings verdict and the read.
    assert!(out.contains("no findings\n\nnkp — N=6 · E=4"), "{out}");
    assert!(out.contains("K̄=0.67 (σ 1.11)"), "{out}");
    assert!(out.contains("P̄=0.78 · regime ORDERED"), "{out}");
    // The line stays bare — its symbol legend rides the agent briefing
    // (skills/archi.md), not the output.
    assert!(!out.contains("components in the landscape"), "{out}");
    assert!(
        out.contains("highest-risk refactoring targets: X (K=3)"),
        "{out}"
    );
    assert!(out.contains("refactoring directions"), "{out}");
    assert!(
        out.contains("C3 ENCAPSULATE — P, Q (SAFE_CORRIDOR, confidence 1.00)"),
        "{out}"
    );
    // The exposed singleton corridors carry no action and print no line.
    assert!(!out.contains("PARTIALLY_NEUTRAL"), "{out}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn the_json_envelope_carries_the_scoring_without_the_matrix() {
    let root = temp_project(COUPLED);
    let out = ok(&root, &["check", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();

    assert_eq!(v["status"], "ok", "{out}");
    assert_eq!(v["findings"], serde_json::json!([]), "{out}");
    let nkp = v["nkp"].as_object().expect("nkp is an object");
    assert_eq!(nkp["scope"]["node_count"], 6, "{out}");
    assert_eq!(nkp["metrics"]["regime"], "ORDERED", "{out}");
    assert_eq!(nkp["hotspots"][0]["node"], "X", "{out}");
    assert!(
        nkp["neutral_corridors"]
            .as_array()
            .is_some_and(|c| !c.is_empty()),
        "{out}"
    );
    // The matrix and the implementation notes stay with `archi nkp`.
    assert!(!nkp.contains_key("matrix"), "{out}");
    assert!(!nkp.contains_key("notes"), "{out}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn an_empty_landscape_earns_no_read() {
    let root = temp_project("");
    let out = ok(&root, &["check"]);
    assert!(out.contains("no findings"), "{out}");
    assert!(!out.contains("nkp"), "{out}");

    let json = ok(&root, &["check", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["status"], "ok", "{json}");
    assert!(v.get("nkp").is_none(), "{json}");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn findings_stay_advisory_and_do_not_withhold_the_read() {
    let root = temp_project(COUPLED);
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

    let out = ok(&root, &["check"]);
    assert!(out.contains("unsatisfied requirement"), "{out}");
    assert!(out.contains("nkp — N=6 · E=4"), "{out}");

    fs::remove_dir_all(&root).unwrap();
}

/// A fact written before the model reaches it: the wing says which fact
/// covers nothing yet and which elements no recorded behavior arrives at,
/// and none of it fails the tree — the wing's states are advisory like every
/// finding, and the read still closes the check
/// (`archi/requirements/world-facts/a-fact-may-stand-before-the-model-does.md`,
/// `archi/requirements/world-facts/coverage-reaches-down-the-graph.md`,
/// `archi/requirements/world-facts/the-check-counts-the-wing.md`).
#[test]
fn the_wing_names_what_it_never_reaches_and_the_tree_stands() {
    let root = temp_project(COUPLED);
    // One fact on the hotspot, and one recorded before the model reached it.
    fact(
        &root,
        "trains-lose-the-signal",
        "Trains lose the signal",
        "X",
        "The carriage drops the network for minutes at a time.",
        "### the app opens with no network\n\n\
         Given the device has no network\nWhen the user opens the app\n\
         Then the last synced view appears\n",
    );
    fact(
        &root,
        "the-guard-walks-the-line",
        "The guard walks the line",
        "",
        "The guard walks the length of the platform every hour.",
        "### the guard reaches the last door\n\n\
         Given the guard leaves the first door\nWhen the walk ends\n\
         Then every door was tried\n",
    );

    let out = ok(&root, &["check"]);

    // Every node the coverage never arrives at, named once; the covered one
    // is not among them.
    for element in ["A", "B", "C", "P", "Q"] {
        assert!(
            out.contains(&format!(
                "world_unreached: {element} — no world fact reaches it"
            )),
            "{out}"
        );
    }
    assert!(!out.contains("world_unreached: X"), "{out}");
    // The fact the model has not reached says so, and stands.
    assert!(
        out.contains("world fact `the-guard-walks-the-line`: world_uncovered"),
        "{out}"
    );
    assert!(!out.contains("world fact `trains-lose-the-signal`"), "{out}");
    // None of it withholds the read, and the wing closes on its own count.
    assert!(out.contains("nkp — N=6 · E=4"), "{out}");
    assert!(out.contains("world — 2 facts · 0 ungrounded"), "{out}");

    // The same picture in the envelope, and the status stays ok.
    let json = ok(&root, &["check", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["status"], "ok", "{json}");
    assert_eq!(v["world"]["facts"], 2, "{json}");
    let unreached: Vec<&str> = v["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["kind"] == "world_unreached")
        .map(|f| f["element"].as_str().unwrap())
        .collect();
    assert_eq!(unreached, ["A", "B", "C", "P", "Q"], "{json}");

    fs::remove_dir_all(&root).unwrap();
}

/// The coverage list names behavior only: a `Data`-classified element rides
/// inside a connection and is never its destination, so it never stands on
/// the list — by its type, whatever it is called
/// (`archi/requirements/world-facts/coverage-reaches-down-the-graph.md`).
#[test]
fn the_coverage_list_names_no_data_element() {
    let root = temp_project(CARRIED);
    gate_fact(&root);

    let out = ok(&root, &["check"]);
    assert!(
        out.contains("world_unreached: Island — no world fact reaches it"),
        "{out}"
    );
    for payload in ["Payload", "Receipt"] {
        assert!(!out.contains(&format!("world_unreached: {payload}")), "{out}");
    }

    let json = ok(&root, &["check", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let unreached: Vec<&str> = v["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["kind"] == "world_unreached")
        .map(|f| f["element"].as_str().unwrap())
        .collect();
    assert_eq!(unreached, ["Island"], "{json}");

    fs::remove_dir_all(&root).unwrap();
}

/// The floor under the coverage list: with every element either conditioned,
/// classified `Data`, or declared internal beside the wing, the list is empty
/// and the check still passes
/// (`archi/requirements/world-facts/an-internal-element-says-so.md`).
#[test]
fn a_declared_element_empties_the_coverage_list() {
    let root = temp_project(CARRIED);
    gate_fact(&root);
    fs::write(
        root.join("archi/world/.worldignore"),
        "# what no condition outside will ever reach\n\n\
         Island — a maintenance siding; no rider ever stands on it\n",
    )
    .unwrap();

    let out = ok(&root, &["check"]);
    assert!(!out.contains("world_unreached"), "{out}");
    assert!(out.contains("world — 1 facts · 0 ungrounded"), "{out}");

    let json = ok(&root, &["check", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["status"], "ok", "{json}");
    assert!(
        !v["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["kind"] == "world_unreached"),
        "{json}"
    );

    fs::remove_dir_all(&root).unwrap();
}

/// The wing in four layers: the strict record under `facts/`, a claim and an
/// observation the schema never touches, and raw material nothing opens. The
/// tree stands, and the closing line counts the facts alone
/// (`archi/requirements/world-facts/the-world-holds-four-layers.md`).
#[test]
fn the_four_layers_stand_and_the_count_reads_the_facts_alone() {
    let root = temp_project(CARRIED);
    gate_fact(&root);
    put(
        &root,
        "archi/world/hypotheses/the-tunnel-is-the-cause.md",
        "# The tunnel is the cause\n\nNobody has measured the dead zone against it.\n",
    );
    put(
        &root,
        "archi/world/notes/a-rider-said-the-app-froze.md",
        "# A rider said the app froze\n\nOn the northern line, twice in one week.\n",
    );
    // Raw material: no name, a half-written header, the markers a merge
    // leaves behind. Nothing opens it, so nothing has an opinion about it.
    put(
        &root,
        "archi/world/resources/the-support-thread.md",
        "---\nkind: ???\n\n<<<<<<< ours\nnot a document at all\n>>>>>>> theirs\n",
    );
    put(
        &root,
        "archi/world/resources/the-recording.txt",
        "00:14 the guard says the tunnel takes four minutes\n",
    );

    let out = ok(&root, &["check"]);
    assert!(out.contains("world — 1 facts · 0 ungrounded"), "{out}");

    let json = ok(&root, &["check", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["status"], "ok", "{json}");
    assert_eq!(v["world"]["facts"], 1, "{json}");
    assert_eq!(v["world"]["ungrounded"], 0, "{json}");

    fs::remove_dir_all(&root).unwrap();
}

/// This tree's own wing: four facts, and every one of them ungrounded. The
/// migration wrote the intent each was lifted from into `sources`, the world
/// may no longer reach the spec, and the empty field says the true thing —
/// nobody has grounded these yet
/// (`archi/requirements/world-facts/a-source-is-reachable-and-lives-in-the-world.md`).
#[test]
fn this_tree_s_facts_carry_no_source_and_each_reports_ungrounded() {
    let root = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let json = ok(&root, &["check", "--json"]);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(v["status"], "ok", "{json}");
    assert_eq!(v["world"]["facts"], 4, "{json}");
    assert_eq!(v["world"]["ungrounded"], 4, "{json}");

    let ungrounded: Vec<&str> = v["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["kind"] == "world_state")
        .filter(|f| {
            f["states"]
                .as_array()
                .is_some_and(|s| s.iter().any(|s| s == "world_ungrounded"))
        })
        .map(|f| f["fact"].as_str().unwrap())
        .collect();
    for slug in [
        "a-design-written-apart-from-the-code-falls-behind-it",
        "an-assistant-guesses-which-files-answer-a-written-obligation",
        "why-a-design-was-chosen-lives-in-one-person-s-memory",
        "work-runs-in-several-directions-at-once-and-more-than-one-person-joins-it",
    ] {
        assert!(ungrounded.contains(&slug), "`{slug}` is not ungrounded:\n{json}");
    }
}

#[test]
fn errors_withhold_the_read() {
    // A tampered archive is an error: the check fails and earns no read.
    let root = temp_project(COUPLED);
    ok(&root, &["version", "save", "-m", "seal"]);
    let versions = root.join("archi/versions");
    let keyframe = fs::read_dir(&versions)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("v0001"))
        })
        .expect("the save wrote v0001");
    let mut bytes = fs::read(&keyframe).unwrap();
    bytes.extend_from_slice(b"\ntampered\n");
    fs::write(&keyframe, bytes).unwrap();

    let (success, stdout, stderr) = run(&root, &["check"]);
    assert!(!success, "{stdout}");
    assert!(stderr.contains("E_ARCHIVE"), "{stderr}");
    assert!(!stdout.contains("nkp"), "{stdout}");

    let (success, json, _) = run(&root, &["check", "--json"]);
    assert!(!success, "{json}");
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["status"], "error", "{json}");
    assert!(v.get("nkp").is_none(), "{json}");

    // A model that does not compile never reaches the read either.
    let broken = temp_project("Ghost.out wire Phantom.inn\n");
    let (success, stdout, _) = run(&broken, &["check"]);
    assert!(!success, "{stdout}");
    assert!(!stdout.contains("nkp"), "{stdout}");

    fs::remove_dir_all(&root).unwrap();
    fs::remove_dir_all(&broken).unwrap();
}
