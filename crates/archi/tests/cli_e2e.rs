//! End to end through the real binary: the standalone meta flags
//! (`archi/requirements/cli/`) — `--help`/`-h` and `--version`/`-V` answer on
//! stdout with exit zero and need no project anywhere, while the malformed
//! invocation keeps its stderr-and-exit-2 contract.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// A directory that is not a project and has no project above it.
fn bare_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-cli-e2e-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

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

#[test]
fn help_and_version_answer_on_stdout_without_a_project() {
    let dir = bare_dir();

    for flag in ["--help", "-h"] {
        let (code, stdout, stderr) = run_in(&dir, &[flag]);
        assert_eq!(code, Some(0), "{flag}: {stderr}");
        assert!(stdout.starts_with("usage:"), "{flag}: {stdout}");
        assert!(stdout.contains("archi init"), "{flag}: {stdout}");
        assert!(stdout.contains("archi --help | --version"), "{flag}: {stdout}");
        assert!(stderr.is_empty(), "{flag}: {stderr}");
    }

    for flag in ["--version", "-V"] {
        let (code, stdout, stderr) = run_in(&dir, &[flag]);
        assert_eq!(code, Some(0), "{flag}: {stderr}");
        assert_eq!(
            stdout,
            format!("archi {}\n", env!("CARGO_PKG_VERSION")),
            "{flag}"
        );
        assert!(stderr.is_empty(), "{flag}: {stderr}");
    }

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn the_malformed_invocation_keeps_its_contract() {
    let dir = bare_dir();

    // No command and an unknown command both report usage on stderr, exit 2.
    let (code, stdout, stderr) = run_in(&dir, &[]);
    assert_eq!(code, Some(2), "{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("usage:"), "{stderr}");

    let (code, _, stderr) = run_in(&dir, &["frobnicate"]);
    assert_eq!(code, Some(2), "{stderr}");
    assert!(stderr.contains("unknown command"), "{stderr}");

    fs::remove_dir_all(&dir).unwrap();
}
