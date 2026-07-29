//! End to end through a COPY of the real binary: the self-updater —
//! `check-update` reads the feed's word on latest, `update` verifies the
//! checksum and swaps the binary atomically in either direction, and
//! every refusal leaves the standing binary untouched.
//!
//! The release feed is a `file://` fixture in the `ARCHI_BASE_URL` shape:
//! a dir holding `latest` (a plain text file naming `v<V>`) and
//! `download/v<V>/archi-<V>-<plat>.tar.gz` beside its `.sha256` — curl
//! serves file URLs like any other. CRITICAL: every test copies the built
//! binary into its own scratch and runs the copy — the real build
//! artifact is never the swap target.

#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// This build's own version — the fixture number for the "equal" case.
const CURRENT: &str = env!("CARGO_PKG_VERSION");

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "archi-update-e2e-{tag}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(&dir).unwrap();
    fs::canonicalize(&dir).unwrap()
}

/// The release platform tag — the same cfg! map the binary compiles in.
fn plat() -> &'static str {
    if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        "linux-x64"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        "linux-arm64"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        "macos-arm64"
    } else {
        panic!("this host has no release platform tag — the update e2e cannot run here")
    }
}

/// The SHA-256 of a file, first-found-wins like the updater itself:
/// `shasum -a 256`, then `sha256sum`.
fn sha256_hex(path: &Path) -> String {
    let tools: [(&str, &[&str]); 2] = [("shasum", &["-a", "256"]), ("sha256sum", &[])];
    for (tool, args) in tools {
        if let Ok(out) = Command::new(tool).args(args).arg(path).output()
            && out.status.success()
        {
            let stdout = String::from_utf8_lossy(&out.stdout);
            return stdout.split_whitespace().next().expect("a hash field").to_string();
        }
    }
    panic!("neither shasum nor sha256sum on PATH — the update e2e cannot hash its fixture")
}

/// Where the fixture feed keeps the assets of version `latest`.
fn asset_dir(server: &Path, latest: &str) -> PathBuf {
    server.join("download").join(format!("v{latest}"))
}

/// Stand up a fixture release feed in `dir` and return its base URL:
/// `latest` is a plain text file naming `v<latest>`; `download/v<latest>/`
/// holds a tarball unpacking to `archi-<latest>-<plat>/archi` — a script
/// that prints `archi <latest>`, so a swapped binary names its new number
/// when run — beside its `.sha256`.
fn fixture(dir: &Path, latest: &str) -> String {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("latest"), format!("v{latest}\n")).unwrap();
    let name = format!("archi-{latest}-{}", plat());
    let stage = dir.join("stage");
    fs::create_dir_all(stage.join(&name)).unwrap();
    let payload = stage.join(&name).join("archi");
    fs::write(&payload, format!("#!/bin/sh\necho \"archi {latest}\"\n")).unwrap();
    fs::set_permissions(&payload, fs::Permissions::from_mode(0o755)).unwrap();
    let assets = asset_dir(dir, latest);
    fs::create_dir_all(&assets).unwrap();
    let tarball = assets.join(format!("{name}.tar.gz"));
    let out = Command::new("tar")
        .arg("-czf")
        .arg(&tarball)
        .arg("-C")
        .arg(&stage)
        .arg(&name)
        .output()
        .unwrap();
    assert!(out.status.success(), "tar -czf: {}", String::from_utf8_lossy(&out.stderr));
    fs::write(
        assets.join(format!("{name}.tar.gz.sha256")),
        format!("{}  {name}.tar.gz\n", sha256_hex(&tarball)),
    )
    .unwrap();
    format!("file://{}", dir.display())
}

/// The swap target every test runs: a copy of the built binary, never the
/// build artifact itself.
fn copy_bin(dir: &Path) -> PathBuf {
    let bin = dir.join("archi");
    fs::copy(env!("CARGO_BIN_EXE_archi"), &bin).unwrap();
    bin
}

fn run(bin: &Path, base: &str, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin)
        .env("ARCHI_BASE_URL", base)
        .args(args)
        .output()
        .expect("the archi copy runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// What a binary answers to `--version` — after a swap, the fixture
/// payload prints its own number.
fn version_of(bin: &Path) -> String {
    let out = Command::new(bin).arg("--version").output().expect("the binary runs");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn equal_version_is_up_to_date_and_update_touches_nothing() {
    let ws = scratch("equal");
    let base = fixture(&ws.join("server"), CURRENT);
    let bin = copy_bin(&ws);
    let before = fs::read(&bin).unwrap();

    let (code, out, err) = run(&bin, &base, &["check-update"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(out.trim(), format!("archi {CURRENT} — up to date"));

    let (code, out, err) = run(&bin, &base, &["update"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(out.trim(), format!("already up to date ({CURRENT})"));
    assert_eq!(fs::read(&bin).unwrap(), before, "the binary must stay untouched");
}

#[test]
fn newer_version_names_itself_and_update_swaps() {
    let ws = scratch("newer");
    let base = fixture(&ws.join("server"), "9.9.9");
    let bin = copy_bin(&ws);

    let (code, out, err) = run(&bin, &base, &["check-update"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(out.trim(), format!("archi {CURRENT} — newer 9.9.9 available: `archi update`"));

    let (code, out, err) = run(&bin, &base, &["update"]);
    assert_eq!(code, 0, "{out}{err}");
    // The report words the move and names the resolved target.
    assert!(out.contains("updated"), "{out}");
    assert!(
        out.contains(&format!("updated {CURRENT} → 9.9.9 at {}", bin.display())),
        "{out}"
    );
    assert_eq!(version_of(&bin), "archi 9.9.9", "the copy must answer the new number");
}

#[test]
fn older_version_words_a_rollback_and_update_follows_it() {
    let ws = scratch("older");
    let base = fixture(&ws.join("server"), "0.0.1");
    let bin = copy_bin(&ws);

    let (code, out, err) = run(&bin, &base, &["check-update"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(
        out.trim(),
        format!("archi {CURRENT} — the feed offers 0.0.1 (a rollback): `archi update` follows it")
    );

    let (code, out, err) = run(&bin, &base, &["update"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.contains(&format!("updated {CURRENT} → 0.0.1 at ")), "{out}");
    assert_eq!(version_of(&bin), "archi 0.0.1", "the rollback must land");
}

#[test]
fn a_dead_base_refuses_naming_the_continuation() {
    let ws = scratch("dead");
    let base = format!("file://{}", ws.join("no-server-here").display());
    let bin = copy_bin(&ws);
    let before = fs::read(&bin).unwrap();

    let (code, out, err) = run(&bin, &base, &["check-update"]);
    assert_eq!(code, 1, "a dead base must refuse the check\n{out}{err}");
    assert!(err.contains("ARCHI_BASE_URL"), "the refusal names the continuation: {err}");

    let (code, out, err) = run(&bin, &base, &["update"]);
    assert_eq!(code, 1, "a dead base must refuse the update\n{out}{err}");
    assert!(err.contains("ARCHI_BASE_URL"), "the refusal names the continuation: {err}");
    assert_eq!(fs::read(&bin).unwrap(), before, "the binary must stay untouched");
}

#[test]
fn a_poisoned_tarball_leaves_the_binary_byte_identical() {
    let ws = scratch("poison");
    let server = ws.join("server");
    let base = fixture(&server, "9.9.9");
    // Poison the bytes after the honest checksum is minted — the hash
    // gate, not tar, must catch it.
    let tarball = asset_dir(&server, "9.9.9").join(format!("archi-9.9.9-{}.tar.gz", plat()));
    fs::write(&tarball, b"these bytes are no tarball").unwrap();
    let bin = copy_bin(&ws);
    let before = fs::read(&bin).unwrap();

    let (code, out, err) = run(&bin, &base, &["update"]);
    assert_eq!(code, 1, "a poisoned tarball must refuse\n{out}{err}");
    assert!(err.contains("checksum mismatch"), "the refusal names the checksum: {err}");
    assert!(!err.contains("unpack"), "the hash gate fires before tar: {err}");
    assert_eq!(fs::read(&bin).unwrap(), before, "the binary must stay byte-identical");
}

#[test]
fn a_lying_checksum_refuses_naming_both_hashes() {
    let ws = scratch("lying");
    let server = ws.join("server");
    let base = fixture(&server, "9.9.9");
    let name = format!("archi-9.9.9-{}", plat());
    let assets = asset_dir(&server, "9.9.9");
    let honest = sha256_hex(&assets.join(format!("{name}.tar.gz")));
    let lie = "f".repeat(64);
    fs::write(assets.join(format!("{name}.tar.gz.sha256")), format!("{lie}  {name}.tar.gz\n"))
        .unwrap();
    let bin = copy_bin(&ws);
    let before = fs::read(&bin).unwrap();

    let (code, out, err) = run(&bin, &base, &["update"]);
    assert_eq!(code, 1, "a lying checksum must refuse\n{out}{err}");
    assert!(err.contains("checksum mismatch"), "the refusal names the mismatch: {err}");
    assert!(err.contains(&lie), "the refusal names the expected hash: {err}");
    assert!(err.contains(&honest), "the refusal names the actual hash: {err}");
    assert_eq!(fs::read(&bin).unwrap(), before, "the binary must stay byte-identical");
}

#[test]
fn a_missing_checksum_refuses_as_unverifiable() {
    let ws = scratch("nosum");
    let server = ws.join("server");
    let base = fixture(&server, "9.9.9");
    let name = format!("archi-9.9.9-{}", plat());
    fs::remove_file(asset_dir(&server, "9.9.9").join(format!("{name}.tar.gz.sha256"))).unwrap();
    let bin = copy_bin(&ws);
    let before = fs::read(&bin).unwrap();

    let (code, out, err) = run(&bin, &base, &["update"]);
    assert_eq!(code, 1, "a missing checksum must refuse\n{out}{err}");
    assert!(err.contains("unverifiable is not installable"), "the refusal words the spirit: {err}");
    assert_eq!(fs::read(&bin).unwrap(), before, "the binary must stay untouched");
}

#[test]
fn a_symlink_survives_and_the_file_behind_it_updates() {
    let ws = scratch("symlink");
    let base = fixture(&ws.join("server"), "9.9.9");
    // The copy lives in stash/; bin/archi is only a link to it.
    let stash = ws.join("stash");
    fs::create_dir_all(&stash).unwrap();
    let real = stash.join("archi-real");
    fs::copy(env!("CARGO_BIN_EXE_archi"), &real).unwrap();
    let bin_dir = ws.join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let link = bin_dir.join("archi");
    std::os::unix::fs::symlink(&real, &link).unwrap();

    let (code, out, err) = run(&link, &base, &["update"]);
    assert_eq!(code, 0, "{out}{err}");
    // The report names the resolved file, never the link.
    assert!(out.contains(&format!("at {}", real.display())), "{out}");
    assert!(!out.contains(&link.display().to_string()), "{out}");

    let meta = fs::symlink_metadata(&link).unwrap();
    assert!(meta.file_type().is_symlink(), "the link must survive as a link");
    assert_eq!(fs::read_link(&link).unwrap(), real, "the link must still point home");
    assert_eq!(version_of(&link), "archi 9.9.9", "the file behind the link answers anew");
}
