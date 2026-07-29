# update-feed

`check-update` and `update` move off the retiring VM lane onto the same
truth the installers ship with: the repository's GitHub releases. Latest
resolves like install.sh does, assets download from the release, and —
new bar — every tarball verifies against its published `.sha256` before
the swap. The old fractal's server is left entirely alone.

## Stack

- Rust — the repository's standing stack
- cargo test — the repository's standing test harness
- system curl and tar via Command — the git-plumbing doctrine; no new runtime dependency
- shasum -a 256 / sha256sum via Command — the checksum plumbing, first found wins
- ARCHI_REPO=owner/repo — fork override, mirroring install.sh
- ARCHI_BASE_URL — whole-feed override: file:// fixtures keep the e2e suite off the wire

## Architecture

- `Updater` — check resolves the feed's tag, fetch pulls and checksum-verifies the asset, swap renames it home
- `Updater` realizes crates/archi/src/update.rs
