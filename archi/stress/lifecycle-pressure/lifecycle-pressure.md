---
version: v0004
closed: v0004
---

# Lifecycle pressure

Fifth round, fourth issue replay (`issues/rounds-without-model-change-cannot-close.md`, fused
with `issues/no-op-save-exits-nonzero.md`): the pressure is on the round lifecycle itself — a
round whose answers change no model must still close, and a save that finds nothing to mint must
say so as a success. Three survivors fence the fix: minting stays reserved for semantic change,
changed rounds keep minting and closing exactly as today, and genuine failures keep their
nonzero exits.
