---
kind: functional
origin: intent
satisfied-by: [Seats.Landing, Seats.Mint, Archive]
deferred:
---

# A landing carries fresh baselines

The closing verb refuses while any cascaded member's worktree tip differs from
the baseline the latest version records for it — the refusal names each stale
member with the repair, `archi version anchor --repo <member>`, and applies to
the sideways landing (`--to`) alike. A mint whose auto-base sits behind the
member checkout's branch proceeds — continuing an older pinned version is
legitimate — but says how far behind, so fresh work anchors first instead of
inheriting a foreign delta window.

## System Context

A member baseline is the "accounted up to here" mark capture and the audit
diff from and the next mint grows from. Members move under other people's
commits, so a stale mark silently attributes foreign work to the next plan's
waves — the failure is invisible until a capture drowns in strangers'
candidates. The landing is the one moment the mark must be true (the landed
archive is what every future unit inherits) and the one moment it is cheap to
fix: the seat is still bound, so `anchor` passes the mutation guard, and the
trees are committed. A skill step can be skipped; a refusal cannot
(mutation-needs-a-seat). Home provenance stays advisory — the seat is
single-writer, foreign deltas cannot enter its window.

## Satisfy

`Seats.Landing` (the gate: per-member tip-vs-baseline pre-flight, refusals
batched in one message, `--to` included); `Seats.Mint` (the behind-by-N note on
a stale-but-reachable auto-base); `Archive` (serves the recorded baselines the
gate compares against).

- test — a merge with un-anchored member work refuses naming the member and the anchor recipe; after `anchor --repo` in the seat it lands
- test — the `--to` landing runs the same gate
- test — a mint from a baseline behind the member's branch proceeds with the behind-by-N note
