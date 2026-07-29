---
node: Updater
owns: [check-update-names-the-drift, the-server-is-the-truth-of-latest, assets-verify-by-checksum, update-replaces-this-binary-in-place, the-network-rides-system-plumbing]
---

# t1 — Updater

Port the verbs from the retiring VM lane to the release feed. Latest
resolves as install.sh does: the `releases/latest` redirect names the
tag (curl -I, url_effective), the GitHub API's `tag_name` is the
fallback; `ARCHI_REPO` overrides the repo. Downloads move to the
release assets: `releases/download/v<V>/archi-<V>-<plat>.tar.gz` plus
its `.sha256`; the tarball hashes through system plumbing (`shasum -a
256`, else `sha256sum`) and a mismatch or a missing checksum refuses
with the standing binary untouched. `ARCHI_BASE_URL` stays as the
whole-feed override for tests and mirrors: `<base>/latest` is a text
file carrying the tag, `<base>/download/v<V>/…` carries the assets —
file:// fixtures ride it. The swap, the direction-worded drift line,
the symlink resolution, the cargo-target warning and the report
strings stay exactly as shipped.

## Spec

- `Updater`
- `Function type_of Updater`
- `Cli.drive consult(->Command, <-Report) Updater.check`
- `Cli.drive consult(->Command, <-Report) Updater.fetch`
- `Cli.drive consult(->Command, <-Report) Updater.swap`

## Inputs

## Outputs

- crates/archi/src/update.rs
- crates/archi/tests/update_e2e.rs

## Stack

- curl -fsSLI -o /dev/null -w %{url_effective} — the redirect walk; api.github.com releases/latest JSON as fallback
- releases/download/v<V>/ asset URLs; .sha256 fetched beside every tarball
- shasum -a 256 / sha256sum via Command — first found wins
- env ARCHI_REPO (fork), ARCHI_BASE_URL (whole-feed override; file:// in e2e)

## Verifications

### check-update-names-the-drift

- test — update_e2e: a file:// feed answers all three ways: equal is "up to date" exit 0, newer names itself and `archi update`, a dead feed is a named refusal exit 1

### the-server-is-the-truth-of-latest

- test — update_e2e: an older feed tag: check-update words it as the feed's rollback; update installs it and the binary answers the older number

### assets-verify-by-checksum

- test — update_e2e: a lying .sha256 refuses naming the mismatch and the binary stays byte-identical; the honest checksum swaps; a feed with no checksum file refuses the same way

### update-replaces-this-binary-in-place

- test — update_e2e: a newer feed release lands checksum-verified and the swapped binary answers the new number; equal exits 0 without touching the file; a poisoned tarball leaves the binary byte-identical

### the-network-rides-system-plumbing

- test — update_e2e runs entirely on ARCHI_BASE_URL=file:// fixtures; no http or hashing dependency enters Cargo.toml
