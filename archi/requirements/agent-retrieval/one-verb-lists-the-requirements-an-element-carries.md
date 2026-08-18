---
kind: functional
origin: intent
satisfied-by: [Cli, DocsCompiler]
deferred:
---

# One verb lists the requirements an element carries

`archi req ls [--satisfies <element>] [--intent <folder>] [--json]` prints one row per
standing requirement: slug, state, `satisfied-by`, and the first phrase of its summary.
`--satisfies` narrows to the requirements naming that element, and the element resolves
against the live model — a name no model holds refuses, naming it. `--intent` narrows to
one folder, and an unknown folder lists the folders, as `req add` already does. A
requirement whose `satisfied-by` is still empty lists with the empty field visible, because
born-before-the-model is a legal state. `--json` carries the same rows, as every `ls` here
does.

## System Context

Requirements had two verbs, `add` and `rm`, and no read. The consequences were measured on
this tree: the question "what stands on `Planner`" has thirty-one answers, lexical search
finds ten of them — two thirds of the claims never say the element's name in prose — and
the only complete path was a hand-rolled grep over the frontmatter. Every agent that needs
the answer invents that grep again, each one differently.

The read exists everywhere else: `world ls --covers` walks fact-to-element, `link ls
--spec` walks code-to-element, and the reverse view element-to-requirement is already
computed inside `plan task req suggest` — but reachable only through a plan. This verb is
that same view, bare.

The row is one line because the use is a sweep: read a cluster whole, then open the two
files that matter. Conflicting claims live on shared elements, and finding them is reading
one cluster, not one hundred sixty-five files.

## Satisfy

`Cli` (the verb, its flags and its refusals). `DocsCompiler` (serves the standing
requirement set the listing reads).

- test — `req ls` prints one row per standing requirement, and the count matches the files on disk
- test — `--satisfies <element>` prints exactly the requirements whose `satisfied-by` names it
- test — `--satisfies` with a name no model holds refuses, naming it
- test — `--intent` narrows to the folder, and an unknown folder lists the folders
- test — a requirement with an empty `satisfied-by` lists, its emptiness visible
- test — `--json` carries the same rows as the render
