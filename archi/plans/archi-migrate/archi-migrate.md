# archi-migrate

Three migrations wear three names, and the reader diagnoses themselves before they can
open the right page. The two in-project passes merge behind one triage head; the old
client's move keeps its page; sync-skills learns to name the folders this merge orphans.

## Stack

- Rust — existing codebase, `crates/archi`
- the skill sources under `skills/` — `scaffold.rs` embeds them with `include_str!`
- cargo test, plain `#[test]` files — existing convention
- `crates/archi/tests/init_e2e.rs` — existing home of the skill guards and the
  world-migration behaviour tests
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Scaffold` — embeds the briefing, installs it, and reports its drift
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the skill sources under `skills/`
