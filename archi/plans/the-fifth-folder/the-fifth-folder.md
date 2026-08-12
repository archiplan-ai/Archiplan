# the-fifth-folder

The world declares four layers and the rule reads exactly those four by name, so a folder
nobody declared takes files and reports nothing. The top of the world is closed — a loose
`.md` directly under `archi/world/` is already a located error — and one level down is
open. This closes it at any depth, with the message the loose file already raises.

## Stack

- Rust — the tree it edits
- `cargo test -p archi --all-targets` — the suite this repository already runs

## Architecture

- `DocsCompiler` — reads the world tree and holds which folder a file may sit in
- `DocsCompiler` realizes `crates/archi/src/docs/world_check.rs`
