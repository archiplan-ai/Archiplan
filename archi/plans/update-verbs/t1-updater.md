---
node: Updater
owns: [check-update-names-the-drift, update-replaces-this-binary-in-place, the-network-rides-system-plumbing, the-swap-names-its-target, the-server-is-the-truth-of-latest]
---

# t1 — Updater

The update module and its two verbs. `check-update`: GET `/version`
through system curl, parse `latest`, compare semver triples, one line
out — up to date, newer named toward `archi update`, older named as the
server's rollback; a dead server is a named refusal, exit 1. `update`:
same check first, equal converges to "already up to date" exit 0;
otherwise download `archi-<latest>-<plat>.tar.gz` from `/download` into
a scratch dir, unpack with system tar, then one `rename` onto the
canonicalized `current_exe` — the report prints the replaced path, a
cargo `target/` path earns a warning line. Platform map compiled in
from the installer's contract (linux-x64, linux-arm64, macos-arm64);
Windows refuses toward `irm …/install.ps1 | iex`. Both verbs run
without a project, outside every guard; `ARCHI_BASE_URL` overrides the
base for tests.

## Spec

- `Updater`
- `Function type_of Updater`
- `Cli.drive consult(->Command, <-Report) Updater.check`
- `Cli.drive consult(->Command, <-Report) Updater.fetch`
- `Cli.drive consult(->Command, <-Report) Updater.swap`

## Inputs

## Outputs

- crates/archi/src/update.rs
- crates/archi/src/main.rs
- crates/archi/tests/update_e2e.rs

## Stack

- std::process::Command over `curl -fsSL` and `tar -xzf` — no reqwest, no tokio
- std::env::current_exe + fs::canonicalize + fs::rename — the atomic swap
- env var ARCHI_BASE_URL, default https://api.archiplan.ai
- file:// fixture layout in e2e: a dir with `version` (JSON) and `download/archi-<v>-<plat>.tar.gz`

## Verifications

### check-update-names-the-drift

- test — update_e2e: a file:// fixture answers all three ways: equal is "up to date" exit 0, newer names itself and `archi update`, a dead path is a named refusal exit 1

### update-replaces-this-binary-in-place

- test — update_e2e: a newer fixture tarball lands and the swapped binary answers the new number; equal exits 0 without touching the file; a poisoned tarball leaves the binary byte-identical

### the-network-rides-system-plumbing

- test — update_e2e runs entirely on ARCHI_BASE_URL=file:// fixtures; no http dependency enters Cargo.toml

### the-swap-names-its-target

- test — update_e2e: the running binary sits behind a symlink; the report names the resolved path, the symlink survives, the file behind it answers the new number

### the-server-is-the-truth-of-latest

- test — update_e2e: an older fixture number: check-update words it as the server's rollback; update installs it and the binary answers the older number
