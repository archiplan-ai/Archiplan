---
name: archi-migrate-fractal
description: Migrate a machine and its projects off the old fractal client — swap the binary (old archi becomes old-archi through migrate-fractal.sh), then import each .fractal/ project into a standing, checkable archiplan spec with a brief of what did not map. Use when a user of the old fractal-era archi wants their projects in the new Archiplan format.
---

> **Skill freshness — the first step.** In an initialized project, run
> `archi sync-skills` before anything else. The report names
> `.claude/skills/archi-migrate-fractal/SKILL.md`. When the act is
> `updated` or `created`, the text you follow is stale. Read that file
> again, follow it, and only then continue. `ok` means continue.

# Migrate from fractal

The old fractal client and the new Archiplan CLI both install as `archi`.
The migration is two moves. First swap the binaries, so that both are
callable. Then import each project. The old `.fractal/` tree is never
mutated. It stays on disk as the frozen reference. The new spec must land
whole and checkable, or not at all.

## 1. Swap the binaries (the script)

Check what `archi` currently is with `archi --help`. When it mentions
`activate`, it is the old fractal client. Run the migration script:

```sh
curl -fsSL https://raw.githubusercontent.com/archiplan-ai/Archiplan/main/release/migrate-fractal.sh | sh
```

It renames every fractal-flavored `archi` on PATH to `old-archi`, in the
same directory, and it does not touch the config or the license. Then it
installs the new `archi` over the freed name. The script is safe to run
again. When the install half fails, the old client is already preserved:
run the script again to retry, or roll back with `mv <dir>/old-archi
<dir>/archi`.

Verify this before you continue: `old-archi --help` answers, which is the
old client, and `archi --version` answers, which is the new CLI. When
`old-archi` does not exist and `archi --help` shows no `activate`, the
swap already happened. Go to §2.

Rules for the old client from here on: local reads only. Never run
`old-archi activate`, `old-archi deactivate`, or bare `old-archi
version`, because that command calls the old update server. Use `old-archi
version list` instead. The old service is being shut down. Its cloud
commands may fail, and they must not gate the migration.

## 2. Read the old project (old-archi, read-only)

Run these inside the project, in the directory that holds `.fractal/`.
Pull the whole picture through the CLI, and prefer it over raw
`.fractal/` reads. Read the `.fractal/` files only when a query command
refuses, and never write to them:

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

Run `archi init` in the same project. It is create-only: it reports
existing files, and it does not rewrite them. Then translate, and loop
`archi check` to zero errors:

- **The whole graph imports.** Every old node and typed edge lands in
  `.arch` source under `archi/src/`. Old nested scopes become real nested
  nodes. Write the identity prose of each element from the old
  descriptions. Where the old spec had none, write the thinnest honest
  line and flag it in the brief. Never invent semantics that the old spec
  did not state.
- **Rename. Never drop.** Where an old id violates the new naming, rename
  it minimally. Add the pair to a rename ledger in the brief. No element
  is dropped over a name.
- **Claims keep their standing.** Each old requirement becomes
  `archi/requirements/<area>/<claim>.md`. Map the old origin to
  `origin:`, as intent or as stressor. A satisfied requirement gets a
  `satisfied-by` that points at the imported elements. An open one stays
  open. Do not close anything that the old spec had open. Do not reopen
  what it had satisfied.
- **The pressure history stays on record.** Do not replay the old stress
  rounds, and do not fake them into new sessions. Transcribe each round
  into the brief: the session, the stressors, the outcomes, and the
  requirements it spawned. New pressure starts fresh against the imported
  version.
- **What cannot import goes in the brief.** Record decisions, dangling
  links, dead code-links and unmappable machinery. Do not drop them
  silently. The new format has richer machinery — ports, carried
  payloads, conn types — with no old counterpart. That goes in the brief
  as later work, not as invented content.

Code links: for each old link whose file and symbol still exist, assert
it again with `archi link add <spec> <file#symbol> --kind
<literal|indirect>`. Stale links go to the brief.

## 4. Land it — or do not

A broken import never lands. Gate it in this order:

1. `archi check` reports zero errors. Open requirements are work to do,
   not errors.
2. Completeness. Compare the node, edge and requirement counts against
   the `query stats` of §2. Every old element is either in the model or
   named in the brief with a reason. There is no third bucket.
3. Seal it: `archi version save -m "imported from fractal (<old current
   version id>)"`.

When check cannot reach clean, fix the offender or move it to the brief.
Never save a half-imported model. Never touch `.fractal/` to make it fit.

## 5. Hand off the brief

Write `fractal-import-brief.md` at the project root. It holds what
imported, with counts, the rename ledger, what stayed behind and why, and
the list of later work: thin prose to deepen, ports and payloads to
model, stressors worth a re-run. Add a pointer to `.fractal/` as the
frozen reference. Tell the operator that the project now stands at the
imported version, and that hardening continues with the standard `archi`
skill: stress, answer, save. Leave `old-archi` and `.fractal/` in place
until the operator confirms that every project imported.
