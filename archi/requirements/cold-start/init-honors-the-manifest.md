---
kind: functional
origin: stressor(the-manifest-already-speaks)
satisfied-by: [Scaffold]
deferred:
---

# Init honors the manifest

In a target that already holds `archi.toml`, init reads it before writing
anything, through the same reader the compiler uses: the starter module lands in
the manifest's `src`, the CLAUDE.md block names that dir, and a manifest that
fails to parse ends the run with the compiler's diagnostic and an untouched tree.
Init never writes a layout the manifest contradicts.

## System Context

The manifest is the layout's one authority (`[project] src`, default `archi/src`),
and two parsers of one file is how tools drift apart. The compiler's manifest
reader is the single reading, exposed to the scaffolder so init and `build` cannot
disagree about where sources live.

## Satisfy

`Scaffold` (manifest read precedes emission; `src` routes the starter; parse
failure aborts before the first write).

- test — a manifest with `src = "spec"` routes the starter to `spec/model.arch` and no `archi/src/` appears
- test — a manifest that fails to parse exits 1 and creates nothing
