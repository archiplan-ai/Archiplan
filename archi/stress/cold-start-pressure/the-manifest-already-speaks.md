---
affects: [Scaffold, Compiler]
outcome: breaking
---

# The manifest already speaks

A brownfield repo adopted archiplan early: `archi.toml` says `src = "spec/model"`.
A later `archi init` — run for the briefing it now installs — scaffolds the
default `archi/src/` beside it and drops the starter there. The compiler never
looks: the starter is a dark file wearing the model's clothes, and the next reader
finds two trees claiming to be the source.

## Attractor

Init's defaults encode one layout, but the manifest exists precisely to override
layout — and a scaffolder with its own reading of `src` is a second parser of the
same file, free to disagree with the compiler's. The disagreement is invisible on
greenfield runs, which is where all the tests live.

## Resolution

Init reads the manifest it finds through the compiler's own reader — the starter
follows `[project] src` wherever it points, the CLAUDE.md block names that dir,
and a manifest that fails to parse stops the run before a byte is written: init
guesses no layout.
Answered by `init-honors-the-manifest`.
