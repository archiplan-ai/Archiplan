---
affects: [Updater]
outcome: breaking
---

# the server walks backward

Prod rolls back: `/version` now answers 0.1.9 while the client runs
0.1.11. Naive "latest != current" arithmetic calls that an update;
naive "latest > current" silently hides a deliberate rollback.

## Attractor

Either a downgrade loop the user never asked to understand — update,
"update available", update, forever — or a client that refuses to
follow an operational rollback and strands users on a version the
server no longer serves tarballs for.

## Resolution

The server is the truth of latest, in both directions — `/download`
only serves the current version's tarballs, so following it is the only
move that keeps working. The verbs follow but say the direction: drift
reports name newer as an update and older as the server's rollback;
converging is one more `update`, equal stays "up to date". Derived
`the-server-is-the-truth-of-latest`.
