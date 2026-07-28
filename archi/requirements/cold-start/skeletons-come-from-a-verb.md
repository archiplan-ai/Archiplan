---
kind: functional
origin: intent
satisfied-by: [DocMint, Cli]
deferred:
---

# Skeletons come from a verb

Doc records mint through verbs — `req add`, `stress open`, `stress add` — never by
hand-typing frontmatter. Every machine field is an explicit parameter or derived
from an invariant, never defaulted: a missing parameter is a usage refusal, so
nothing is skippable; a value that exists only one way — the open round's folder,
the version a fresh round pins — is computed, never asked. The verb writes the
exact schema shape with the text slots empty; the schema's own diagnostics hold
them as the un-skippable worklist until the author writes the prose.

## System Context

The doc schema is strict on purpose: every field present (empty is a state, absence
is not), fixed section order, an H1 that slugifies to the filename. A generator
turns that gauntlet into a fill-in, and explicit-only parameters turn "the agent
forgot" into a refusal instead of a silently defaulted record. Placement follows
invariants the CLI already owns: at most one round is open (the stressor's folder),
a round presses the version just saved (the charter's pin), `--intent` names the
one content decision a requirement's placement carries. Prose stays the author's:
records through verbs, text by editing — the settled boundary. Affects resolve
against the pinned version at the write, the same check the round's validation
runs. Every verb is batchable line-per-line, so a whole round's skeletons
materialize in one call under the same seat guard.

## Satisfy

`DocMint` (the emitter: schema shape out, slots empty, placement derived from the
tree and the archive); `Cli` (the `req` and `stress` verbs, guarded at the router
like every mutation).

- test — a fresh round materializes from one batch: open, stressors, a derived requirement (`a_whole_round_materializes_from_one_batch_and_the_guard_covers_every_line`)
- test — missing parameters are usage refusals; unknown intents list the candidates (`requirements_mint_explicitly_and_removals_preflight`)
- test — a minted skeleton is byte-exact schema shape and the empty slots fail `check` until filled (`the_round_lifecycle_is_verbs_with_derived_placement`)
- test — affects resolve against the round's pinned version at the write, misses named in one message (`the_round_lifecycle_is_verbs_with_derived_placement`)
