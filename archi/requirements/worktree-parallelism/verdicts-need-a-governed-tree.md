---
kind: functional
origin: intent
satisfied-by: [Seats.Guard, Cli]
deferred:
---

# Verdicts need a governed tree

`check` and `build` answer anywhere — but never bless ungoverned work: an
unbound checkout whose spec carries uncommitted edits fails both verbs with
the seat recipe instead of a passing report. A bound seat never trips the
gate; a clean tree passes everywhere; a tree mid-merge is exempt — the join
triage needs `check` exactly while the spec is conflicted; a gitless tree
stays free for the post-init smoke.

## System Context

The mutation guard closes the verbs, but requirements and models are written
by editing files — no verb to refuse. The verdict verbs are where ungoverned
edits would get laundered into a green report, so the gate lives there,
wired at the same router as the mutation guard (mutation-needs-a-seat). CI
and the receiving checkout after a landing run on clean trees and pass
untouched (merge-retires-the-worktree).

## Satisfy

`Seats.Guard` (verdict: uncommitted paths under the governed surface — `archi/`, the
manifest, the model source dir — in an unbound checkout refuse with the seat recipe);
`Cli` wires the gate at the router for `check` and `build` alone.

- test — a dirty spec outside a seat fails check and build with the seat recipe; the same edit inside a seat answers (`a_dirty_spec_outside_a_seat_fails_check_and_build`)
- test — a mid-merge tree is exempt and a clean unbound tree passes (`the_verdict_gate_refuses_only_a_dirty_spec_outside_a_seat`)
