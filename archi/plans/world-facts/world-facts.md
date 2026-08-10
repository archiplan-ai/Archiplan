# world-facts

Give archi a second kind of spec record: a fact about the world, the scenarios that fact
dictates, and three lists a checker can resolve. A verb mints the skeleton, a person writes
the prose, `archi check` holds the shape, the plan carries the facts covering the nodes it
builds, and a skill fills the record in on a project that has none.

## Stack

- Rust — existing codebase, `crates/archi` and `crates/modeling-lang`
- gherkin (cucumber-rs crate) — user choice; parses the whole language and hands back an AST with 1-indexed line and column, over which the six-keyword subset is enforced
- cargo test, plain `#[test]` integration files — existing convention, no dev-dependencies today
- `crates/archi/tests/world_e2e.rs` — user choice: one new file beside the per-verb e2e files
- no runtime infrastructure — user choice; archi is a CLI over files and git and never reaches the network

## Architecture

- `WorldDoc` — the world-fact record: frontmatter, headings and the three lists
- `WorldDoc` realizes `crates/archi/src/docs/world.rs` plus the shared schema types
- `Gherkin` — the scenario grammar: parse the block, then hold it to the named subset
- `Gherkin` realizes `crates/archi/src/docs/gherkin.rs` over the gherkin crate
- `DocsCompiler` — loads the wing, cross-checks it against the model, reports its states
- `DocsCompiler` realizes `crates/archi/src/docs/mod.rs` and `crates/archi/src/docs/schema.rs`
- `DocMint` — mints and retires a fact, and pre-flights everything that stands on it
- `DocMint` realizes `crates/archi/src/docs/mint.rs`
- `Cli` — the `world` verb: `add`, `rm`, `ls`, with the binding rule every mutating verb holds
- `Cli` realizes `crates/archi/src/main.rs`
- `Search` — one ranked card per fact, narrowed by `--kind world`
- `Search` realizes `crates/archi/src/search.rs`
- `Links` — a scenario as an addressable spec ref
- `Links` realizes `crates/archi/src/links/mod.rs`
- `Planner` — the plan carries covering facts, closes on their scenarios and reports their drift
- `Planner` realizes `crates/archi/src/plans/mod.rs` and `crates/archi/src/plans/records.rs`
- `Query` — the read envelope answers with the conditions bearing on a slice
- `Query` realizes `crates/modeling-lang` query surface and `crates/archi/src/main.rs`
- `Scaffold` — installs the briefing and the migration skill
- `Scaffold` realizes `crates/archi/src/scaffold.rs`
