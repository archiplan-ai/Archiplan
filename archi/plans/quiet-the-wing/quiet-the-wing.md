# quiet-the-wing

Answer the debt the wing's first hour of real use produced. Coverage stops asking a question
of the wrong kind, an element that no condition will ever reach can say so, a scenario link
becomes a pair that notices when either side moves, a repeated mint converges like its
siblings, the closing refusal needs a wing to refuse against, and the briefing stops
reprinting what one command already says.

## Stack

- Rust — existing codebase, no change
- cargo test, plain `#[test]` integration files — existing convention, no new dependency
- one scenario digest for the whole tool — user choice: the plan already computes a fingerprint over the parsed block, and the link reuses it rather than growing a second one
- no runtime infrastructure — unchanged, archi is a CLI over files and git

## Architecture

- `DocsCompiler` — holds the wing: what coverage asks, what `.worldignore` answers, and the one scenario digest both the plan and the link read
- `DocsCompiler` realizes `crates/archi/src/docs/world_check.rs` and `crates/archi/src/docs/mod.rs`
- `DocMint` — mints and retires a fact, and converges on a skeleton nobody has written into
- `DocMint` realizes `crates/archi/src/docs/mint.rs`
- `Links` — a scenario link carries a digest per side and fails when either moves
- `Links` realizes `crates/archi/src/links/mod.rs`
- `Planner` — the closing refusal fires only where a wing stands
- `Planner` realizes `crates/archi/src/plans/mod.rs`
- `Scaffold` — the briefing it installs, and the migration skill's procedure
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the embedded skill texts under `skills/`
