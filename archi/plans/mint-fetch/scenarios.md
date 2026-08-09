# Scenarios

- An operator mints a seat while the remote has moved and the local base carries unpushed work: the mint refreshes, branches from the safe side, and names the ref, the commit and the divergence it chose by. Runs on cargo test (worktree_e2e — no infrastructure).
