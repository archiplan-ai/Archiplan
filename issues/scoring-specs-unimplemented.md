# Scoring specs promise more than the analyses deliver

**Kind:** missing feature (spec'd analyses) · confirmed during the self-hosting bootstrap

Two gaps between `requirements/scoring/` and the code:

- **NKP stages.** `nkp.rs` self-reports the gap: every `NkpReport` carries notes that the
  adaptive-walk simulation is not implemented and the dependency matrix is returned unclustered
  (no spectral decomposition) — `crates/modeling-lang/src/nkp.rs:789-793`. The regime, K/P
  metrics, hotspots and corridors work (and read sensibly on the self-model: CRITICAL, hub
  `SourceTree`), but two of the spec's stages are honest IOUs printed on every run.
- **Functional-requirements load.** `requirements/scoring/functional-reqs-load.md` is a
  zero-byte file: a name with no claim, no schema, and no analysis behind it.

## Impact

An operator reading `scoring/` cannot tell shipped analysis from aspiration without running the
tool (NKP at least says so in its output; the empty file says nothing). Spec-vs-code drift is the
exact disease this tool exists to prevent.

## Fix shape

Implement the two NKP stages or move them to an explicit deferred section of
`scoring/nkp.md`; write `functional-reqs-load.md` for real or delete the file. Either direction
restores the invariant that a spec file makes a checkable promise.
