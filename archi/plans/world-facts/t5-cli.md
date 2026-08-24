---
node: Cli
owns: [the-world-verb-refuses-like-the-others, one-verb-walks-the-bridge, each-retrieval-path-names-the-other]
---

# t5 — Cli

the world verb: add, rm, ls

## Spec

- `Cli`
- `Service type_of Cli`
- `Agent.drive invoke(->Command, <-Report) Cli.build`
- `Agent.drive invoke(->Command, <-Report) Cli.check`
- `Agent.drive invoke(->Command, <-Report) Cli.incidence`
- `Agent.drive invoke(->Command, <-Report) Cli.init`
- `Agent.drive invoke(->Command, <-Report) Cli.link`
- `Agent.drive invoke(->Command, <-Report) Cli.nkp`
- `Agent.drive invoke(->Command, <-Report) Cli.plan`
- `Agent.drive invoke(->Command, <-Report) Cli.query`
- `Agent.drive invoke(->Command, <-Report) Cli.read`
- `Agent.drive invoke(->Command, <-Report) Cli.repo`
- `Agent.drive invoke(->Command, <-Report) Cli.req`
- `Agent.drive invoke(->Command, <-Report) Cli.search`
- `Agent.drive invoke(->Command, <-Report) Cli.session`
- `Agent.drive invoke(->Command, <-Report) Cli.status`
- `Agent.drive invoke(->Command, <-Report) Cli.stress`
- `Agent.drive invoke(->Command, <-Report) Cli.update`
- `Agent.drive invoke(->Command, <-Report) Cli.version`
- `Agent.drive invoke(->Command, <-Report) Cli.worktree`
- `Agent.drive invoke(->Command, <-Report) Cli.world`

## Inputs

- from t4 — the mint and remove entry points and their refusal text
- from t3 — the loaded facts and their resolved `covers`, for the listing

## Outputs

- crates/archi/src/main.rs
- crates/archi/tests/world_e2e.rs

## Stack

- the verb dispatch and flag parsing sit beside `req` and `stress` in `main.rs`
- the binding check and refusal text are the ones the other mutating verbs already call
- `--json` reuses the envelope shape `search` and `link ls` emit

## Verifications

### the-world-verb-refuses-like-the-others

- test — world_e2e: `world add` in an unbound checkout refuses and names the standing seats
- test — world_e2e: the refusal exit code equals the one `req add` returns for the same case
- test — world_e2e: `world ls` runs in an unbound checkout and exits zero
- test — world_e2e: a refused `world add` leaves no file and no folder behind

### one-verb-walks-the-bridge

- test — world_e2e: `world ls` on three facts prints three blocks with slug, path and covers
- test — world_e2e: `--covers <node>` returns only the facts naming that element
- test — world_e2e: `--covers` on a name no element matches refuses and says so
- test — world_e2e: the JSON envelope carries slug, path, covers, sources and uses

### each-retrieval-path-names-the-other

- test — world_e2e: `world ls --covers` with no hits names the phrase path
- test — search_e2e: `search --kind world` with no hits names the covers path
- test — world_e2e: a non-empty result carries no such line
