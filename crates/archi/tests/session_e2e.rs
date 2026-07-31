//! End to end through the real binary: a merge-fused round record is one
//! recipe-naming `E_SESSION` instead of silent charter prose, and
//! `archi session fold` is the only path that merges round records — both
//! charters kept, folds across pins refused, and on a fused sealed pair the
//! folded stamp waits for `version remint --session` to make it true
//! (fold-pressure; rounds-fold-deliberately, parallel-editing-discipline).

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
        "archi-session-e2e-{}-{}",
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

fn session_file(root: &Path, slug: &str) -> PathBuf {
    root.join("archi/stress").join(slug).join(format!("{slug}.md"))
}

fn write_session(root: &Path, slug: &str, title: &str, version: &str, closed: &str) {
    let dir = root.join("archi/stress").join(slug);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        session_file(root, slug),
        format!("---\nversion: {version}\nclosed: {closed}\n---\n\n# {title}\n\nThe {slug} charter.\n"),
    )
    .unwrap();
}

fn write_stressor(root: &Path, session: &str, slug: &str, title: &str) {
    fs::write(
        root.join("archi/stress").join(session).join(format!("{slug}.md")),
        format!(
            "---\naffects: [Gate]\noutcome: surviving\n---\n\n# {title}\n\nPressure.\n\n\
             ## Attractor\n\nDrift.\n\n## Resolution\n\nHolds.\n"
        ),
    )
    .unwrap();
}

/// A session file as a same-slug add/add merge leaves it: common frontmatter
/// and H1 merged clean, the two charters in markers (the lab's topology).
fn write_fused_session(root: &Path, slug: &str, title: &str, fm: &str) {
    let dir = root.join("archi/stress").join(slug);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        session_file(root, slug),
        format!(
            "---\n{fm}\n---\n\n# {title}\n\n<<<<<<< HEAD\nOurs presses the storage floor.\n\
             =======\nTheirs presses the auth boundary.\n>>>>>>> abc1234\n"
        ),
    )
    .unwrap();
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

fn fails(root: &Path, args: &[&str]) -> String {
    let (success, stdout, stderr) = run(root, args);
    assert!(!success, "archi {args:?} unexpectedly passed:\n{stdout}");
    format!("{stdout}{stderr}")
}

// ---- detection ---------------------------------------------------------------

#[test]
fn markers_in_a_charter_are_one_recipe_naming_session_error() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);
    write_fused_session(&root, "hardening", "Hardening", "version: v0001\nclosed:");
    write_stressor(&root, "hardening", "push", "Push");
    let out = fails(&root, &["check"]);
    assert!(
        out.contains("E_SESSION") && out.contains("claimed by two charters"),
        "{out}"
    );
    assert!(out.contains("archi session fold hardening"), "{out}");
    assert!(!out.contains("E_DOC"), "no parse noise beside the recipe:\n{out}");
}

#[test]
fn markers_in_frontmatter_name_the_fusion_not_a_syntax_accident() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);
    let dir = root.join("archi/stress/pinned");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("pinned.md"),
        "---\n<<<<<<< HEAD\nversion: v0002\n=======\nversion: v0001\n>>>>>>> abc1234\nclosed:\n---\n\n\
         # Pinned\n\nCharter.\n",
    )
    .unwrap();
    let out = fails(&root, &["check"]);
    assert!(out.contains("claimed by two charters"), "{out}");
    assert!(!out.contains("frontmatter lines"), "{out}");
}

#[test]
fn markers_in_a_stressor_name_the_split_recipe() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);
    write_session(&root, "hardening", "Hardening", "v0001", "");
    write_stressor(&root, "hardening", "push", "Push");
    let stressor = root.join("archi/stress/hardening/push.md");
    let fused = format!(
        "{}<<<<<<< HEAD\nOurs line.\n=======\nTheirs line.\n>>>>>>> abc1234\n",
        fs::read_to_string(&stressor).unwrap()
    );
    fs::write(&stressor, fused).unwrap();
    let out = fails(&root, &["check"]);
    assert!(
        out.contains("two branches wrote this stressor") && out.contains("session `hardening`"),
        "{out}"
    );
}

#[test]
fn two_open_sessions_name_the_fold_verb() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);
    write_session(&root, "alice-round", "Alice round", "v0001", "");
    write_stressor(&root, "alice-round", "push", "Push");
    write_session(&root, "bob-round", "Bob round", "v0001", "");
    write_stressor(&root, "bob-round", "shove", "Shove");
    let out = fails(&root, &["check"]);
    assert!(
        out.contains("session fold bob-round --into alice-round"),
        "{out}"
    );
}

// ---- the in-place fold -------------------------------------------------------

#[test]
fn a_fused_open_pair_folds_in_place_and_validates_forever() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);
    write_fused_session(&root, "hardening", "Hardening", "version: v0001\nclosed:");
    write_stressor(&root, "hardening", "push", "Push");
    let out = ok(&root, &["session", "fold", "hardening", "-m", "one round, two writers"]);
    assert!(out.contains("folded the `abc1234` side"), "{out}");
    let text = fs::read_to_string(session_file(&root, "hardening")).unwrap();
    assert!(text.contains("Ours presses the storage floor."), "{text}");
    assert!(text.contains("## Folded: abc1234"), "{text}");
    assert!(text.contains("Theirs presses the auth boundary."), "{text}");
    assert!(text.contains("pin: v0001"), "{text}");
    assert!(text.contains("note: one round, two writers"), "{text}");
    assert!(!text.contains("<<<<<<<"), "{text}");
    ok(&root, &["check"]);
}

#[test]
fn keep_theirs_picks_the_surviving_charter() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);
    write_fused_session(&root, "hardening", "Hardening", "version: v0001\nclosed:");
    write_stressor(&root, "hardening", "push", "Push");
    ok(&root, &[
        "session", "fold", "hardening", "-m", "theirs charter reads better", "--keep", "theirs",
    ]);
    let text = fs::read_to_string(session_file(&root, "hardening")).unwrap();
    assert!(text.contains("Theirs presses the auth boundary."), "{text}");
    assert!(text.contains("## Folded: HEAD"), "{text}");
    ok(&root, &["check"]);
}

#[test]
fn a_fold_across_pins_refuses_and_names_the_rule() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);
    let dir = root.join("archi/stress/pinned");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("pinned.md"),
        "---\n<<<<<<< HEAD\nversion: v0002\n=======\nversion: v0001\n>>>>>>> abc1234\nclosed:\n---\n\n\
         # Pinned\n\nCharter.\n",
    )
    .unwrap();
    let out = fails(&root, &["session", "fold", "pinned", "-m", "n"]);
    assert!(out.contains("same ground"), "{out}");
    assert!(out.contains("v0002") && out.contains("v0001"), "{out}");
}

#[test]
fn a_clean_session_has_nothing_to_fold_in_place() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);
    write_session(&root, "hardening", "Hardening", "v0001", "");
    write_stressor(&root, "hardening", "push", "Push");
    let out = fails(&root, &["session", "fold", "hardening", "-m", "n"]);
    assert!(out.contains("no complete conflict"), "{out}");
}

// ---- the two-folder fold -----------------------------------------------------

#[test]
fn fold_into_moves_stressors_keeps_both_charters_and_the_loser_folder_dies() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);
    write_session(&root, "alice-round", "Alice round", "v0001", "");
    write_stressor(&root, "alice-round", "push", "Push");
    write_session(&root, "bob-round", "Bob round", "v0001", "");
    write_stressor(&root, "bob-round", "shove", "Shove");
    let out = ok(&root, &[
        "session", "fold", "bob-round", "--into", "alice-round", "-m", "one pressure campaign",
    ]);
    assert!(out.contains("folded `bob-round` into `alice-round`"), "{out}");
    assert!(out.contains("1 stressor moved"), "{out}");
    let text = fs::read_to_string(session_file(&root, "alice-round")).unwrap();
    assert!(text.contains("## Folded: bob-round"), "{text}");
    assert!(text.contains("The bob-round charter."), "{text}");
    assert!(root.join("archi/stress/alice-round/shove.md").is_file());
    assert!(!root.join("archi/stress/bob-round").exists());
    ok(&root, &["check"]);
    // The folded round closes as one: save stamps the survivor.
    fs::write(
        root.join("archi/src/model.arch"),
        format!("{MODEL}def node Extra\n"),
    )
    .unwrap();
    let out = ok(&root, &["version", "save", "-m", "answers"]);
    assert!(out.contains("closed stress session `alice-round`"), "{out}");
}

#[test]
fn fold_into_refuses_stressor_name_collisions() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);
    write_session(&root, "alice-round", "Alice round", "v0001", "");
    write_stressor(&root, "alice-round", "push", "Push");
    write_session(&root, "bob-round", "Bob round", "v0001", "");
    write_stressor(&root, "bob-round", "push", "Push");
    let out = fails(&root, &[
        "session", "fold", "bob-round", "--into", "alice-round", "-m", "n",
    ]);
    assert!(out.contains("`push.md`") && out.contains("rename"), "{out}");
    assert!(root.join("archi/stress/bob-round/push.md").is_file(), "nothing moved");
}

#[test]
fn fold_into_is_for_rounds_still_in_flight() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);
    write_session(&root, "alice-round", "Alice round", "v0001", "v0001");
    write_stressor(&root, "alice-round", "push", "Push");
    write_session(&root, "bob-round", "Bob round", "v0001", "");
    write_stressor(&root, "bob-round", "shove", "Shove");
    let out = fails(&root, &[
        "session", "fold", "bob-round", "--into", "alice-round", "-m", "n",
    ]);
    assert!(out.contains("still in flight"), "{out}");
}

// ---- the sealed fuse and its remint ------------------------------------------

#[test]
fn a_fused_sealed_pair_folds_pending_and_remint_makes_it_true() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "seed"]);
    // Both writers sealed their round `closed: v0001` — the archive
    // collision's ambiguous shared stamp, fused by the merge.
    write_fused_session(
        &root,
        "hardening",
        "Hardening",
        "version: v0001\nclosed: v0001",
    );
    write_stressor(&root, "hardening", "push", "Push");

    // Remint refuses to touch the fused record: fold first.
    fs::write(
        root.join("archi/src/model.arch"),
        format!("{MODEL}def node Extra\n"),
    )
    .unwrap();
    let out = fails(&root, &["version", "remint", "-m", "rejoin", "--session", "hardening"]);
    assert!(out.contains("fold them first"), "{out}");

    let out = ok(&root, &["session", "fold", "hardening", "-m", "two sealed rounds, one merge"]);
    assert!(out.contains("awaits its re-mint"), "{out}");
    let text = fs::read_to_string(session_file(&root, "hardening")).unwrap();
    assert!(text.contains("closed: pending remint"), "{text}");
    // The half-done sequence is a finding until the re-mint lands.
    let (_, check_out, _) = run(&root, &["check"]);
    assert!(check_out.contains("folded round awaits remint"), "{check_out}");

    let out = ok(&root, &["version", "remint", "-m", "rejoin", "--session", "hardening"]);
    assert!(out.contains("reminted v0002"), "{out}");
    let text = fs::read_to_string(session_file(&root, "hardening")).unwrap();
    assert!(text.contains("closed: v0002"), "folded stamp re-stamped:\n{text}");
    assert!(
        text.contains("---\nversion: v0001\nclosed: v0001\n---"),
        "the surviving stamp stays true:\n{text}"
    );
    ok(&root, &["check"]);
}
