# scenario-shape

A scenario becomes a heading and its steps. `Feature:` and `Scenario:` go, because the
fact's title is already the feature and the heading is already the scenario. The four step
keywords are the whole vocabulary, every other line under a heading is a located error, and
the closing step stops saying "clean" — it names the file a scenario is anchored to and asks
for the two to be read against each other.

## Stack

- Rust — existing codebase, unchanged
- cargo test, plain `#[test]` integration files — existing convention
- the `gherkin` crate is dropped — four keywords and a markdown heading need no parser, and the crate was bought to read a language this shape no longer speaks
- no runtime infrastructure — unchanged

## Architecture

- `Gherkin` — the reader: a heading opens a scenario, four keywords open its steps, everything else is an error that says where it belongs
- `Gherkin` realizes `crates/archi/src/docs/gherkin.rs`
- `WorldDoc` — the four standing facts, rewritten into the shape
- `WorldDoc` realizes the files under `archi/world/`
- `Links` — the digest reads the new parse, and the six standing scenario links stay clean across the change
- `Links` realizes `crates/archi/src/links/mod.rs` and the digest in `crates/archi/src/docs/world_check.rs`
- `Planner` — an anchored scenario names its file and symbol and asks for the re-read
- `Planner` realizes `crates/archi/src/plans/mod.rs`
- `Scaffold` — the briefing and the skill texts describe the shape that ships
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the embedded texts under `skills/`
