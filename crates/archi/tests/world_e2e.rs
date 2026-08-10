//! End to end through the real binary: the `world` verb
//! (`archi/requirements/world-facts/`). `add` and `rm` mutate, so they meet
//! the same seat rule and the same exit codes as `req add|rm`; `ls` reads
//! and answers anywhere. `--covers <element>` is the traversal the wing is
//! reached by (`archi/decisions/the-wing-is-reached-by-traversal.md`), and
//! an empty answer on either retrieval path names the other one.

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};

const MODEL: &str = "\
def node AuthService:
  port handle_login
def node RateLimiter:
  port take
def node Island
def conn calls := * -> *
AuthService.handle_login calls RateLimiter.take
";

/// The scaffolded project and the worktree minted from it. The primary
/// checkout stays unbound — the case every refusal answers — and the
/// worktree is where the mutations run.
fn temp_project() -> (PathBuf, PathBuf) {
    let dir = util::scratch("archi-world-e2e", "p");
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
    let wt = util::worktree(&dir);
    (dir, wt)
}

fn put(root: &Path, rel_path: &str, text: &str) {
    let path = root.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// One whole world fact: the three lists as given, the conditioning
/// paragraph, the killer and one scenario.
fn fact(root: &Path, slug: &str, title: &str, covers: &str, sources: &str, uses: &str) {
    util::Fact {
        covers,
        sources,
        uses,
        condition: "The carriage drops the network for minutes at a time.",
        killer: "Trackside coverage that never drops.",
        scenarios: "Feature: Offline open\n  \
                    Scenario: the app opens with no network\n    \
                    Given the device has no network\n    When the user opens the app\n    \
                    Then the last synced view appears\n",
    }
    .write(root, slug, title);
}

/// Three standing facts: one on the gate, one on both, one on the limiter —
/// and one of them ungrounded.
fn three_facts(root: &Path) {
    put(root, "notes/line.md", "the ride, written down\n");
    fact(
        root,
        "trains-lose-the-signal",
        "Trains lose the signal",
        "AuthService",
        "https://example.org/thread/42",
        "",
    );
    fact(
        root,
        "tunnels-run-long",
        "Tunnels run long",
        "AuthService, RateLimiter",
        "",
        "trains-lose-the-signal",
    );
    fact(
        root,
        "the-guard-walks-the-line",
        "The guard walks the line",
        "RateLimiter",
        "notes/line.md",
        "",
    );
}

/// The binary's exit code, stdout and stderr.
fn run(root: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_archi"))
        .args(args)
        .args(["--project", root.to_str().unwrap()])
        .output()
        .expect("archi runs");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn ok(root: &Path, args: &[&str]) -> String {
    let (code, stdout, stderr) = run(root, args);
    assert_eq!(code, Some(0), "archi {args:?} failed:\n{stdout}\n{stderr}");
    stdout
}

/// A refusal: its exit code and its message.
fn refuse(root: &Path, args: &[&str]) -> (Option<i32>, String) {
    let (code, stdout, stderr) = run(root, args);
    assert_ne!(code, Some(0), "archi {args:?} passed but had to refuse:\n{stdout}");
    (code, stderr)
}

fn parsed(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|e| panic!("json: {e}\n{text}"))
}

/// `add` and `rm` mutate, so the seat rule holds for them exactly as it
/// holds for `req`; `ls` reads and runs anywhere
/// (`the-world-verb-refuses-like-the-others`).
#[test]
fn the_world_verb_refuses_like_the_others() {
    let (primary, wt) = temp_project();

    // The unbound checkout refuses the mint, and names the standing seat.
    let (code, err) = refuse(&primary, &["world", "add", "Trains lose the signal"]);
    assert!(err.contains("unbound"), "{err}");
    assert!(err.contains(wt.to_str().unwrap()), "{err}");

    // The same class of refusal as `req add`, down to the exit code.
    let (req_code, _) = refuse(
        &primary,
        &[
            "req", "add", "Gate throttles", "--intent", "hardening", "--kind", "functional",
            "--origin", "intent",
        ],
    );
    assert_eq!(code, req_code);

    // `rm` is a mutation too.
    let (rm_code, err) = refuse(&primary, &["world", "rm", "trains-lose-the-signal"]);
    assert_eq!(rm_code, code);
    assert!(err.contains("unbound"), "{err}");

    // The refusal left no file and no folder behind.
    assert!(!primary.join("archi/world").exists());

    // `ls` reads: it answers in the same unbound checkout, exit zero, and a
    // tree with no wing prints nothing.
    assert_eq!(ok(&primary, &["world", "ls"]), "");
}

/// The mint and the removal ride the same dispatch as the other doc verbs:
/// the skeleton, the refusals, the exit codes.
#[test]
fn the_verb_mints_and_retires_from_the_bound_seat() {
    let (_primary, wt) = temp_project();

    let out = ok(&wt, &["world", "add", "Trains lose the signal"]);
    assert!(out.contains("archi/world/trains-lose-the-signal.md"), "{out}");
    let path = wt.join("archi/world/trains-lose-the-signal.md");
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "---\ncovers: []\nsources: []\nuses: []\n---\n\n\
         # Trains lose the signal\n\n## What kills this\n\n## Scenarios\n"
    );
    assert!(ok(&wt, &["world", "ls"]).contains("trains-lose-the-signal"));

    // A standing file is a wall, at the exit code every doc verb refuses with.
    let (code, err) = refuse(&wt, &["world", "add", "Trains lose the signal"]);
    assert_eq!(code, Some(1));
    assert!(err.contains("archi/world/trains-lose-the-signal.md"), "{err}");

    ok(&wt, &["world", "rm", "trains-lose-the-signal"]);
    assert!(!path.exists());
    let (code, err) = refuse(&wt, &["world", "rm", "trains-lose-the-signal"]);
    assert_eq!(code, Some(1));
    assert!(err.contains("no world fact"), "{err}");

    // A missing parameter is a usage refusal, before the tree is touched.
    let (code, err) = refuse(&wt, &["world", "add"]);
    assert_eq!(code, Some(2));
    assert!(err.contains("archi world add"), "{err}");
    assert!(!wt.join("archi/world").join("trains-lose-the-signal.md").exists());
}

/// `world ls` lists the wing, and `--covers` walks the bridge
/// (`one-verb-walks-the-bridge`).
#[test]
fn one_verb_walks_the_bridge() {
    let (_primary, wt) = temp_project();
    three_facts(&wt);

    // One block per fact: slug, path, covers — and whether it is grounded.
    let out = ok(&wt, &["world", "ls"]);
    for slug in [
        "the-guard-walks-the-line",
        "trains-lose-the-signal",
        "tunnels-run-long",
    ] {
        assert!(
            out.contains(&format!("{slug}  archi/world/{slug}.md")),
            "{out}"
        );
    }
    assert_eq!(out.matches("covers:").count(), 3, "{out}");
    assert!(out.contains("covers: AuthService, RateLimiter"), "{out}");
    assert!(out.contains("sources: none"), "{out}");
    assert!(out.contains("uses: trains-lose-the-signal"), "{out}");

    // `--covers` returns the facts naming that element, and only those.
    let out = ok(&wt, &["world", "ls", "--covers", "RateLimiter"]);
    assert!(out.contains("tunnels-run-long"), "{out}");
    assert!(out.contains("the-guard-walks-the-line"), "{out}");
    // The fact that covers the gate alone is not a block here — it rides
    // the listing only as the `uses` of one that is.
    assert!(
        !out.contains("trains-lose-the-signal  archi/world/"),
        "{out}"
    );

    // A name no model element matches refuses and says so.
    let (code, err) = refuse(&wt, &["world", "ls", "--covers", "Ghost"]);
    assert_eq!(code, Some(1));
    assert!(err.contains("Ghost"), "{err}");
    assert!(err.contains("names no element"), "{err}");

    // The envelope carries the record: slug, path, covers, sources, uses.
    let v = parsed(&ok(&wt, &["world", "ls", "--json"]));
    assert_eq!(v["status"], "ok");
    let facts = v["facts"].as_array().unwrap();
    assert_eq!(facts.len(), 3);
    let f = facts
        .iter()
        .find(|f| f["slug"] == "tunnels-run-long")
        .expect("the fact");
    assert_eq!(f["path"], "archi/world/tunnels-run-long.md");
    assert_eq!(f["covers"], json!(["AuthService", "RateLimiter"]));
    assert_eq!(f["sources"], json!([]));
    assert_eq!(f["uses"], json!(["trains-lose-the-signal"]));
    let f = facts
        .iter()
        .find(|f| f["slug"] == "the-guard-walks-the-line")
        .expect("the fact");
    assert_eq!(f["sources"], json!(["notes/line.md"]));

    // The filter narrows the envelope the same way.
    let v = parsed(&ok(&wt, &["world", "ls", "--covers", "AuthService", "--json"]));
    assert_eq!(v["facts"].as_array().unwrap().len(), 2);
}

/// An empty answer on either path names the other one
/// (`each-retrieval-path-names-the-other`).
#[test]
fn each_retrieval_path_names_the_other() {
    let (_primary, wt) = temp_project();
    three_facts(&wt);

    // No fact covers the island: the phrase path is named, exit stays zero.
    let out = ok(&wt, &["world", "ls", "--covers", "Island"]);
    assert!(out.contains("archi search"), "{out}");
    assert!(out.contains("--kind world"), "{out}");

    // A non-empty answer carries no such line.
    let out = ok(&wt, &["world", "ls", "--covers", "AuthService"]);
    assert!(!out.contains("archi search"), "{out}");

    // In JSON the note is a field of the envelope, never inside the facts.
    let v = parsed(&ok(&wt, &["world", "ls", "--covers", "Island", "--json"]));
    assert!(
        v["note"].as_str().expect("the note").contains("archi search"),
        "{v}"
    );
    assert!(v["facts"].as_array().unwrap().is_empty(), "{v}");
    let v = parsed(&ok(&wt, &["world", "ls", "--covers", "AuthService", "--json"]));
    assert!(v["note"].is_null(), "{v}");

    // The phrase path, empty, names the covers path.
    let out = ok(&wt, &["search", "dugong", "--kind", "world"]);
    assert!(out.contains("archi world ls --covers"), "{out}");
    let v = parsed(&ok(&wt, &["search", "dugong", "--kind", "world", "--json"]));
    assert!(
        v["note"]
            .as_str()
            .expect("the note")
            .contains("archi world ls --covers"),
        "{v}"
    );
    assert!(v["hits"].as_array().unwrap().is_empty(), "{v}");

    // A non-empty search carries no such line.
    let out = ok(&wt, &["search", "carriage", "--kind", "world"]);
    assert!(out.contains("trains-lose-the-signal"), "{out}");
    assert!(!out.contains("archi world ls --covers"), "{out}");
    let v = parsed(&ok(&wt, &["search", "carriage", "--kind", "world", "--json"]));
    assert!(v["note"].is_null(), "{v}");
}

/// The check prints what the wing's pass computed: its advisory lines and
/// its closing count (`the-check-counts-the-wing`).
#[test]
fn the_check_prints_the_wings_report() {
    let (_primary, wt) = temp_project();
    three_facts(&wt);

    let (_code, out, _err) = run(&wt, &["check"]);
    assert!(out.contains("world — 3 facts · 1 ungrounded"), "{out}");
    assert!(out.contains("world_unreached: Island"), "{out}");
    assert!(out.contains("world fact `tunnels-run-long`"), "{out}");

    let (_code, out, _err) = run(&wt, &["check", "--json"]);
    let v = parsed(&out);
    assert_eq!(v["world"]["facts"], 3);
    assert_eq!(v["world"]["ungrounded"], 1);
    let kinds: Vec<&str> = v["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["kind"].as_str())
        .collect();
    assert!(kinds.contains(&"world_unreached"), "{out}");
    assert!(kinds.contains(&"world_state"), "{out}");

    // A tree with no wing says nothing new — the count is not born.
    fs::remove_dir_all(wt.join("archi/world")).unwrap();
    let (_code, out, _err) = run(&wt, &["check"]);
    assert!(!out.contains("world —"), "{out}");
    let (_code, out, _err) = run(&wt, &["check", "--json"]);
    assert!(parsed(&out)["world"].is_null(), "{out}");
}
