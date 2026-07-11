---
affects: [Links.Grader, Archive, Members]
outcome: breaking
---

# The baseline commit is gone

The recorded baseline names a commit the member repository no longer holds. Three ordinary roads
there: the baseline sat on a branch deleted after merge, and `git gc` — automatic, on by default —
pruned it; the member rebased or amended and the old line was collected; or, the common one, CI
checks the member out with `actions/checkout` at its default `fetch-depth: 1`, a shallow clone that
never fetched the baseline commit at all. The SHA is intact in the version entry; the object it
names is absent from the tree.

## Attractor

`git diff <sha>` exits 128, `fatal: bad object`. `delta_hunks` turns that non-zero status into an
error, and the audit's per-member loop propagates it — so one member's missing object aborts the
*whole* `link audit`. Observed live: with a dangling baseline the command exits nonzero, prints
nothing at all to stdout — not the healthy members' rows, not even home's note — and emits the raw
`git diff … fatal: bad object <sha>` on stderr. This is exactly what `scans-see-every-mapped-member`
forbids — "an absent checkout narrows the scan and is reported, never silently shrinking coverage" —
resurfacing one layer down: the checkout is present, the *baseline* is unreachable, and the design
has no state for that. It lands first and hardest in CI, where the shallow clone is the default and
the audit is the job.

## Resolution

Decided: breaking. The floor an audit stands on can vanish while the checkout sits right there, and
when it does the failure is global, not local. The answer follows the shape the round already cut
for absence: a baseline whose commit does not resolve is a state of its own — probe it before the
diff, and when it is gone narrow *that member's* scan and say so, an unresolvable-baseline note
beside the unreachable-checkout and anchor-born ones, never letting it abort the others; `repo ls`
should read the baseline as unresolvable rather than print a SHA that no longer names anything. This
round stops at the diagnosis: the answering requirement (`origin:
stressor(the-baseline-commit-is-gone)`) and the `Grader.audit` fix are left for the operator to
green-light — `check` names the gap as `breaking_unanswered` until they land.
