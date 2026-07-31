# overlay-gate

A stale overlay row silently based a colleague's seat on a dead feature
branch. Two gates close the hole: `repo map` refuses linked-worktree
paths at write time; the mint refuses a member resolved into one at use
time, with the repair verbatim. An explicit `--base` lifts the mint
gate; seat-owned member worktrees stay legal; new member worktrees
anchor beside the main checkout.

## Stack

- Rust — the repository's standing stack
- cargo test — the repository's standing test harness
- git rev-parse --git-dir vs --git-common-dir — the linked-worktree detection
- git worktree list — finds the main checkout the refusals name

## Architecture

- `Members` — the map write refuses linked-worktree paths
- `Members` realizes crates/archi/src/members.rs
- `Seats` — the cascade gate and the main-checkout placement anchor
- `Seats` realizes crates/archi/src/worktrees.rs
