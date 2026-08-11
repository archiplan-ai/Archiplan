//! End to end through the real binary: `version save` mints on semantic
//! change only, and the round ceremony — stamping the open stress session
//! closed, firing the incidence report — finishes whether or not a
//! version minted; the bare no-op is a success and genuine failures stay
//! loud (`archi/requirements/self-hosting/unchanged-saves-close-rounds.md`).
//!
//! The save is also the gate on the world wing: a tree that holds a fact and
//! still carries an element no fact reaches saves nothing, and the refusal
//! names every element and both exits
//! (`archi/requirements/world-facts/the-save-refuses-an-unconditioned-element.md`).
//! A tree with no fact saves exactly as it always did.

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const MODEL: &str = "def conn wire := * -> *\n\
                     def node Gate:\n  port out\n\
                     def node Auth:\n  port inn\n\
                     Gate.out wire Auth.inn\n";

fn temp_project() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-version-e2e-{}-{}",
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
    util::worktree(&dir)
}

/// An open stress session pinned to `version`, with one stressor so the
/// round is schema-shaped like a real one.
fn open_session(root: &Path, slug: &str, title: &str, version: &str) {
    let dir = root.join("archi/stress").join(slug);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("{slug}.md")),
        format!("---\nversion: {version}\nclosed:\n---\n\n# {title}\n\nThe round.\n"),
    )
    .unwrap();
    fs::write(
        dir.join("push.md"),
        "---\naffects: [Gate]\noutcome: surviving\n---\n\n# Push\n\nPressure on the gate.\n\n\
         ## Attractor\n\nDrift.\n\n## Resolution\n\nHolds.\n",
    )
    .unwrap();
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

fn ok(root: &Path, args: &[&str]) -> String {
    let (success, stdout, stderr) = run(root, args);
    assert!(success, "archi {args:?} failed:\n{stdout}\n{stderr}");
    stdout
}

fn version_count(root: &Path) -> usize {
    fs::read_to_string(root.join("archi/versions/index.toml"))
        .unwrap()
        .matches("[[version]]")
        .count()
}

#[test]
fn unchanged_save_closes_the_open_session() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    open_session(&root, "round-one", "Round one", "v0001");
    let out = ok(&root, &["version", "save", "-m", "behavior-only round"]);
    assert!(
        out.contains("nothing to mint: the model is unchanged since v0001"),
        "{out}"
    );
    assert!(
        out.contains("closed stress session `round-one` at v0001"),
        "{out}"
    );
    assert!(out.contains("incidence — session `round-one`"), "{out}");
    let session = fs::read_to_string(root.join("archi/stress/round-one/round-one.md")).unwrap();
    assert!(session.contains("closed: v0001"), "{session}");
    assert_eq!(version_count(&root), 1, "the close mints nothing");
}

#[test]
fn unchanged_save_with_no_open_session_is_a_benign_noop() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    let out = ok(&root, &["version", "save", "-m", "again"]);
    assert!(
        out.contains("nothing to save: the model is unchanged since v0001 and no session is open"),
        "{out}"
    );
    assert_eq!(version_count(&root), 1);
}

#[test]
fn changed_save_still_mints_and_closes_at_the_minted_id() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    open_session(&root, "round-one", "Round one", "v0001");
    fs::write(
        root.join("archi/src/model.arch"),
        format!("{MODEL}def node Store:\n  port inn\n"),
    )
    .unwrap();
    let out = ok(&root, &["version", "save", "-m", "grew"]);
    assert!(out.contains("saved v0002"), "{out}");
    assert!(out.contains("closed stress session `round-one`"), "{out}");
    let session = fs::read_to_string(root.join("archi/stress/round-one/round-one.md")).unwrap();
    assert!(session.contains("closed: v0002"), "{session}");
    assert_eq!(version_count(&root), 2);
}

/// The model with one node nothing reaches — the element the wing has
/// something to say about.
const WITH_ISLAND: &str = "def node Island\n";

/// One world fact under `archi/world/facts/`, covering what the caller
/// names — the shared skeleton ([`util::Fact`]) with this family's words in
/// it. Its schema is the wing's; the gate reads its `covers` alone.
fn world_fact(root: &Path, covers: &str) {
    util::Fact {
        covers,
        sources: "",
        uses: "",
        condition: "The carriage drops the network for minutes at a time.",
        workaround: "Riders screenshot the timetable before they go down, and the shot goes \
                     stale.",
        scenarios: "### the app opens with no network\n\n\
                    Given the device has no network\n\
                    When the user opens the app\n\
                    Then the last synced view appears\n",
    }
    .write(root, "riders-lose-the-signal", "Riders lose the signal");
}

/// A project holding the island, one fact covering `Gate`, and one saved
/// version behind it — the state the gate refuses from.
fn island_and_one_fact() -> PathBuf {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    world_fact(&root, "Gate");
    fs::write(
        root.join("archi/src/model.arch"),
        format!("{MODEL}{WITH_ISLAND}"),
    )
    .unwrap();
    root
}

/// The gate: a tree that holds a fact and still carries an element no fact
/// reaches saves nothing, and the refusal names the element
/// (`the-save-refuses-an-unconditioned-element`).
#[test]
fn an_unreached_element_refuses_the_save() {
    let root = island_and_one_fact();
    let (success, _, stderr) = run(&root, &["version", "save", "-m", "grew"]);
    assert!(!success, "an unreached element must fail the save");
    assert!(stderr.contains("`Island`"), "{stderr}");
    assert_eq!(version_count(&root), 1, "the refusal mints nothing");
    fs::remove_dir_all(&root).unwrap();
}

/// The refusal names both exits — cover it with a fact, or declare it
/// internal — because a gate with one exit is a wall
/// (`the-save-refuses-an-unconditioned-element`).
#[test]
fn the_refusal_names_both_exits() {
    let root = island_and_one_fact();
    let (_, _, stderr) = run(&root, &["version", "save", "-m", "grew"]);
    assert!(stderr.contains("archi world add"), "{stderr}");
    assert!(stderr.contains("covers"), "{stderr}");
    assert!(stderr.contains("archi/world/.worldignore"), "{stderr}");
    fs::remove_dir_all(&root).unwrap();
}

/// The first exit: a fact reaches the element, and the save proceeds
/// (`the-save-refuses-an-unconditioned-element`).
#[test]
fn covering_the_element_clears_the_refusal() {
    let root = island_and_one_fact();
    world_fact(&root, "Gate, Island");
    let out = ok(&root, &["version", "save", "-m", "grew"]);
    assert!(out.contains("saved v0002"), "{out}");
    assert_eq!(version_count(&root), 2);
    fs::remove_dir_all(&root).unwrap();
}

/// The second exit: the element is declared internal, with the reason a
/// reader can argue with (`the-save-refuses-an-unconditioned-element`).
#[test]
fn declaring_the_element_internal_clears_the_refusal() {
    let root = island_and_one_fact();
    fs::write(
        root.join("archi/world/.worldignore"),
        "Island — a scratch fixture; no behavior from outside arrives at it\n",
    )
    .unwrap();
    let out = ok(&root, &["version", "save", "-m", "grew"]);
    assert!(out.contains("saved v0002"), "{out}");
    assert_eq!(version_count(&root), 2);
    fs::remove_dir_all(&root).unwrap();
}

/// The threshold: a project that has written no fact is not behind on the
/// wing, and its archive is byte for byte what it was before the gate
/// existed (`the-wing-arrives-without-noise`).
#[test]
fn a_tree_with_no_world_facts_saves_byte_identically() {
    let bare = temp_project();
    let winged = temp_project();
    for root in [&bare, &winged] {
        fs::write(
            root.join("archi/src/model.arch"),
            format!("{MODEL}{WITH_ISLAND}"),
        )
        .unwrap();
    }
    // A wing folder with no fact in it is still no fact.
    let notes = winged.join("archi/world/notes");
    fs::create_dir_all(&notes).unwrap();
    fs::write(
        notes.join("the-guard-walked-the-platform.md"),
        "# The guard walked the platform\n\nHe timed the tunnel once at four minutes.\n",
    )
    .unwrap();

    for root in [&bare, &winged] {
        let out = ok(root, &["version", "save", "-m", "first"]);
        assert!(out.contains("saved v0001"), "{out}");
    }
    assert_eq!(
        fs::read(bare.join("archi/versions/v0001.arch")).unwrap(),
        fs::read(winged.join("archi/versions/v0001.arch")).unwrap(),
        "the wing changes no archived byte"
    );
    fs::remove_dir_all(&bare).unwrap();
    fs::remove_dir_all(&winged).unwrap();
}

#[test]
fn two_open_sessions_keep_failing_the_save() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "first"]);
    open_session(&root, "round-one", "Round one", "v0001");
    open_session(&root, "round-two", "Round two", "v0001");
    let (success, _, stderr) = run(&root, &["version", "save", "-m", "jam"]);
    assert!(!success, "two open sessions must fail the save");
    assert!(stderr.contains("are all open"), "{stderr}");
    assert_eq!(version_count(&root), 1);
}
