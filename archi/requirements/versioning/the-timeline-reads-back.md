---
kind: functional
origin: intent
satisfied-by: [Archive, Cli]
deferred:
---

# The timeline reads back

The archive answers questions without git. `version list` prints every version with its
note and metadata from the manifest; `version show <id>` materializes a version's
canonical source — compilable, so it can seed a tree or ground a read; `version diff`
renders the semantic delta between any two versions — for adjacent ones it is the stored
patch, verbatim — and takes `live` on either side to compare the working tree
(`merge-deltas-are-reviewable` owns that review posture); `version current` reports which
version the live render matches, or that the tree is dirty since the latest — derived by
hashing on demand, never stored, so it cannot lie.

## System Context

Reconstruction is the load-bearing operation under every read — `the-archive-is-sealed`
guarantees it — and "current" doubles as the pin gate for plans and the anchor's match
test, so the derived answer is consulted by verbs that must not guess.

## Satisfy

`Archive` (reconstructs, hashes and compares renders on demand). `Cli` (the version read
verbs and their reports).

- test — versions::first_save_keyframes_then_patches_and_reconstructs
- test — multiplayer_e2e::diff_live_shows_the_unsaved_delta
- test — read_e2e::query_composes_filters_and_reads_sealed_versions
