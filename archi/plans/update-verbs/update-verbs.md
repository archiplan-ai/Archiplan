# update-verbs

Two client verbs against the standing server: `archi check-update` says
whether the server's latest differs and in which direction; `archi
update` downloads the platform tarball and atomically swaps the running
binary. The old fractal client is untouched; the server already serves
`/version` and `/download`.

## Stack

- Rust — the repository's standing stack
- cargo test — the repository's standing test harness
- system curl and tar via Command — the git-plumbing doctrine; no new runtime dependency
- ARCHI_BASE_URL override — file:// fixtures keep the e2e suite off the wire

## Architecture

- `Updater` — check names the drift, fetch pulls the tarball, swap renames it home
- `Updater` realizes crates/archi/src/update.rs
