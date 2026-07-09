---
name: archi-migrate-fractal
description: Migrate a machine and its projects off the old fractal client — swap the binary (old archi becomes old-archi via migrate-fractal.sh), then cross each .fractal/ project into a standing, checkable archiplan spec with a brief of what didn't map. Use when a user of the old fractal-era archi wants their projects in the new Archiplan format.
---

# Migrate from fractal

The old fractal client and the new Archiplan CLI both install as `archi`.
Migration is two moves: swap the binaries so both are callable, then cross
each project. The old `.fractal/` tree is never mutated — it stays on disk
as the frozen reference; the new spec must land whole and checkable or not
at all.

## 1. Swap the binaries (the script)

Check what `archi` currently is: `archi --help`. If it mentions `activate`,
it is the old fractal client — run the migration script:

```sh
# from a checkout of archiplan-ai/Archiplan:
sh release/migrate-fractal.sh
# or standalone:
curl -fsSL https://raw.githubusercontent.com/archiplan-ai/Archiplan/main/release/migrate-fractal.sh | sh
```

It renames every fractal-flavored `archi` on PATH to `old-archi` (same
directory, config and license untouched) and installs the new `archi` over
the freed name. Safe to re-run; if the install half fails, the old client
is already preserved — re-run to retry, or roll back with
`mv <dir>/old-archi <dir>/archi`.

Verify before proceeding: `old-archi --help` answers (old client) and
`archi --version` answers (new CLI). If `old-archi` does not exist and
`archi --help` shows no `activate`, the swap already happened — go to §2.

Old-client etiquette from here on: local reads only. Never run
`old-archi activate`, `deactivate`, or bare `old-archi version` (it phones
the old update server — use `old-archi version list`). The old service is
being sunset; cloud verbs may fail and must not gate the migration.

## 2. Read the old project (old-archi, read-only)

Run inside the project (the directory holding `.fractal/`). Pull the whole
picture through the CLI — prefer it over raw `.fractal/` reads; fall back
to reading `.fractal/` files only if a query verb refuses, and never write:

```sh
old-archi problem show            # the framing
old-archi scope map               # nesting structure
old-archi query subgraph          # full typed graph: nodes + edges
old-archi query stats             # counts — the completeness yardstick for §4
old-archi query reqs              # requirements with satisfaction state
old-archi query unsatisfied       # the open subset
old-archi query decisions --json  # decisions with alternatives
old-archi stress list             # stress sessions; drill: old-archi stress show <id>
old-archi version list            # evolution trajectory + current version id
old-archi link ls                 # code links, if any
```

## 3. Rebuild in the new format (archi)

`archi init` in the same project — create-only, reports rather than rewrites
existing files. Then translate, and loop `archi check` to zero errors:

- **Graph crosses whole.** Every old node and typed edge lands in `.arch`
  sources under `archi/src/`. Old nested scopes become real nested nodes.
  Write each element's identity prose from the old descriptions — where the
  old spec had none, write the thinnest honest line and flag it in the brief;
  never invent semantics the old spec didn't state.
- **Names bend, never break.** Where an old id violates new naming, rename
  minimally and add the pair to a rename ledger in the brief. No element is
  dropped over a name.
- **Claims keep their standing.** Each old requirement becomes
  `archi/requirements/<area>/<claim>.md` — old origin mapped to `origin:`
  (intent vs stressor), satisfied ones get `satisfied-by` pointing at the
  crossed elements, open ones stay open. Do not close anything the old spec
  had open, or reopen what it had satisfied.
- **Pressure history stays on record.** Old stress rounds are not replayed
  or faked into new sessions — transcribe each round (session, stressors,
  outcomes, requirements it spawned) into the brief. New pressure starts
  fresh against the imported version.
- **What cannot cross becomes the brief.** Decisions, dangling links, dead
  code-links, unmappable machinery — recorded, not silently dropped. The
  new format's richer machinery (ports, carried payloads, conn types) that
  has no old counterpart goes in the brief as augment-next work, not as
  invented content.

Code links: for each old link whose file and symbol still exist, re-assert
with `archi link add <spec> <file#symbol> --kind <literal|indirect>`;
stale ones go to the brief.

## 4. Land it — or don't

A broken import never lands. Gate in order:

1. `archi check` — zero errors (open requirements are worklist, not errors).
2. Completeness: node/edge/requirement counts against §2's `query stats`;
   every old element is either in the model or named in the brief with a
   reason. No third bucket.
3. Seal: `archi version save -m "imported from fractal (<old current version id>)"`.

If check cannot reach clean, fix or move the offender to the brief — never
save a half-crossed model, and never touch `.fractal/` to make it fit.

## 5. Hand off the brief

Write `fractal-import-brief.md` at the project root: what crossed (counts),
the rename ledger, what stayed behind and why, augment-next list (thin
prose to deepen, ports/payloads to model, stressors worth re-running), and
a pointer to `.fractal/` as the frozen reference. Tell the operator the
project now stands at the imported version — hardening continues with the
standard `archi` skill (stress → answer → save). Leave `old-archi` and
`.fractal/` in place until the operator confirms every project has crossed.
