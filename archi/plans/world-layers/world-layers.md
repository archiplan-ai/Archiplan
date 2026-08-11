# world-layers

The world splits into four folders and a source stops reaching outside it. `facts/` keeps
the strict record; `hypotheses/`, `notes/` and `resources/` hold what arrives loose. A
`sources` entry names a file inside the world and resolves, so the four standing facts —
which named the intents they were lifted from — lose their borrowed ground and read as
what they are.

## Stack

- Rust — existing codebase, unchanged; this unit introduces no technology
- cargo test, plain `#[test]` integration files — existing convention
- no runtime infrastructure — unchanged

## Architecture

- `DocsCompiler` — walks the four folders, applies the strict schema to `facts/` alone, and resolves every source inside the world
- `DocsCompiler` realizes `crates/archi/src/docs/world_check.rs` and `crates/archi/src/docs/mod.rs`
- `WorldDoc` — the record, and which folder a file's kind comes from
- `WorldDoc` realizes `crates/archi/src/docs/world.rs` and the files under `archi/world/`
- `DocMint` — `world add` mints into `facts/` and creates the folder on the way
- `DocMint` realizes `crates/archi/src/docs/mint.rs`
- `Scaffold` — the installed texts describe the four layers and where a source may live
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the embedded texts under `skills/`
