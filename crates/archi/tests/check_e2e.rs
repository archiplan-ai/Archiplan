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
