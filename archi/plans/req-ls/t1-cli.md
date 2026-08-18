---
node: Cli
owns: [one-verb-lists-the-requirements-an-element-carries]
---

# t1 — Cli

req ls: the read verb over the standing requirements

## Spec

- `Cli`
- `Service type_of Cli`
- `Agent.drive invoke(->Command, <-Report) Cli.req`
- `DocsCompiler`

## Inputs

## Outputs

- crates/archi/src/main.rs
- crates/archi/src/docs/mod.rs
- crates/archi/tests/mint_e2e.rs

## Stack

- the record answers where this lands (`link ls --spec` per ref): the `req` arm is
  `crates/archi/src/main.rs#run_req`, the requirement set is served from
  `crates/archi/src/docs/mod.rs` (`discover_tree` at line 370 walks the doc tree)
- the shape to mirror is `run_world` at `crates/archi/src/main.rs:2693` and `world ls`:
  flat rows, `--json` on the same data, refusal on an unresolvable name
- `--satisfies` resolves the element against the live model the way `world ls --covers`
  does; `--intent` reuses `req add`'s unknown-folder refusal, which lists the folders
- the row: slug, state, `satisfied-by`, first phrase of the summary — one line, because
  the use is a sweep over a cluster
- usage lines in `main.rs` (the module doc and the USAGE block) gain the verb beside
  `req add | rm`

## Verifications

### one-verb-lists-the-requirements-an-element-carries

- test — `req ls` prints one row per standing requirement, and the count matches the
  files on disk
- test — `--satisfies <element>` prints exactly the requirements whose `satisfied-by`
  names it
- test — `--satisfies` with a name no model holds refuses, naming it
- test — `--intent` narrows to the folder, and an unknown folder lists the folders
- test — a requirement with an empty `satisfied-by` lists, its emptiness visible
- test — `--json` carries the same rows as the render
