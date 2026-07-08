---
kind: functional
origin: intent
satisfied-by: [Search, Cli]
deferred:
---

# One verb searches everything

`archi search <phrase>` returns the ranked hits for a natural-language phrase across
every object the knowledge base holds: model elements (nodes, rels, conns, views —
their paths and identity prose), intents, requirements, stressors and sessions. One
call, one ranked list; `--kind` narrows to a subset of object kinds, `--limit` bounds
the list (ten by default), `--json` emits the structured envelope for agents, and the
human default prints one hit per block with its address, score and snippet.

## System Context

The read surface today is addressed: `archi query` wants a path, `archi incidence`
wants a session, the docs want their file names known. Retrieval is the missing
entry point — phrase in, addresses out — and it composes with everything already
there: a hit's slug pipes into the next verb. The corpus is exactly what `archi
check` already walks (the compiled model, `archi/requirements/`, `archi/stress/`),
so search needs no new stores and no new schema.

## Satisfy

`Search` (builds one card per object from the compiled model dump and the doc tree,
scores them against the phrase, ranks and bounds the list). `Cli` (the `search` verb:
parses the phrase and flags, compiles the project, hands the model to the scan,
renders hits as human blocks or the JSON envelope).

- test — one phrase returns hits spanning element, requirement and stressor kinds from one call
- test — `--kind requirement` narrows to requirements only; `--limit 3` returns at most three hits
- test — the JSON envelope carries status, query and hits with kind, slug, score and snippet
- test — e2e: on this repository's own KB, `archi search "fold"` surfaces the fold-pressure round and the Sessions element
