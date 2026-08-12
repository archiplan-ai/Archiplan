# ports-speak

Search scores a node's own definition as summary and its ports' definitions as body, so a
node with no definition forfeits a weight class and loses to any node that happens to carry
a sentence. The split is not random: of 54 top-level nodes here, all 30 that carry a
definition are portless and none of the 24 that carry ports carries one, because a portless
node is one line with room for a comment and a ported node opens a block with no end to
land on. The nodes with ports are the ones that do something, so asking the index who does
a thing answers with what is lying around. A definitionless node is indexed by its ports'
prose in the slot its missing prose would have filled.

## Stack

- Rust — existing codebase, `crates/archi`
- cargo test, plain `#[test]` files — existing convention, no new dependency
- `crates/archi/src/search.rs` unit tests — existing home, user choice: no new suite
- no runtime infrastructure — archi is a CLI over files and git

## Architecture

- `Search` — ranked retrieval over every archi object, and the field weights that order it
- `Search` realizes `crates/archi/src/search.rs`
