# mint-fetch

A seat used to grow from whatever the local checkout happened to hold.
The mint now refreshes that base first, chooses the branch point by the
divergence — never dropping unpushed work — names what it grew from, and
lands its folder beside the repository's main checkout.

## Stack

- Rust — the repository's standing stack
- cargo test — the repository's standing test harness
- git fetch of one branch; rev-list counts against the remote ref

## Architecture

- `Seats` — the refresh, the branch point and the folder anchor
- `Seats` realizes crates/archi/src/worktrees.rs
