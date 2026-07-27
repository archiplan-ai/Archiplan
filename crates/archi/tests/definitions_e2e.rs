//! End to end through the real binary: definitions ride the archive
//! (`archi/requirements/element-definitions/definitions-are-semantic`) — a
//! save after definitions land mints, the archived render reconstructs
//! against its seal with the definitions inside, pre-definition versions
//! reconstruct unchanged, and changing a definition is a meaning change.

mod util;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const BARE: &str = "def conn wire := * -> *\n\
                    def node Gate:\n  port out\n\
                    def node Auth:\n  port inn\n\
                    Gate.out wire Auth.inn\n";

const DEFINED: &str = "def conn wire := * -> * // typed traffic\n\
                       def node Gate: // the entry point\n  port out // requests leave here\n\
                       def node Auth:\n  port inn\n\
                       Gate.out wire Auth.inn\n";

fn temp_project() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-definitions-e2e-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(dir.join("archi/src")).unwrap();
    fs::write(
        dir.join("archi.toml"),
        "[project]\nname = \"t\"\npreset = \"default\"\n",
    )
    .unwrap();
    fs::write(dir.join("archi/src/model.arch"), BARE).unwrap();
    util::seat(&dir)
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

#[test]
fn definitions_ride_the_archive_and_seals_hold() {
    let root = temp_project();
    ok(&root, &["version", "save", "-m", "bare"]);

    // Definitions land: meaning changed, the save mints.
    fs::write(root.join("archi/src/model.arch"), DEFINED).unwrap();
    let out = ok(&root, &["version", "save", "-m", "defined"]);
    assert!(out.contains("v0002"), "{out}");

    // The archived render carries them; reconstruction verifies the seal.
    let shown = ok(&root, &["version", "show", "v0002"]);
    assert!(shown.contains("def node Gate: // the entry point"), "{shown}");
    assert!(shown.contains("  port out // requests leave here"), "{shown}");
    assert!(shown.contains("def conn wire := * -> * // typed traffic"), "{shown}");

    // The pre-definition version reconstructs unchanged.
    let bare = ok(&root, &["version", "show", "v0001"]);
    assert!(!bare.contains("//"), "{bare}");

    // check compiles model + docs + archive: everything holds together.
    ok(&root, &["check"]);

    // An unchanged save is not a new meaning.
    let noop = ok(&root, &["version", "save", "-m", "again"]);
    assert!(noop.contains("nothing to save"), "{noop}");

    // Editing only a definition is a meaning change: the save mints.
    fs::write(
        root.join("archi/src/model.arch"),
        DEFINED.replace("// the entry point", "// the boundary gate"),
    )
    .unwrap();
    let out = ok(&root, &["version", "save", "-m", "reworded"]);
    assert!(out.contains("v0003"), "{out}");
    let diff = ok(&root, &["version", "diff", "v0002", "v0003"]);
    assert!(
        diff.contains("the entry point") && diff.contains("the boundary gate"),
        "{diff}"
    );
}

#[test]
fn obligation_prose_never_reaches_the_archive() {
    let root = temp_project();
    fs::write(
        root.join("archi/src/model.arch"),
        BARE.replace(
            "def node Gate:",
            "def node Gate: // must hold the boundary",
        ),
    )
    .unwrap();
    let out = fails(&root, &["version", "save", "-m", "smuggle"]);
    assert!(
        out.contains("E_DEFINITION") && out.contains("obligation"),
        "{out}"
    );
    assert!(!root.join("archi/versions/index.toml").exists());
}
