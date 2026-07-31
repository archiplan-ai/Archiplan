---
kind: functional
origin: stressor(an-explicit-base-drifts-off-the-audited-line)
satisfied-by: [Seats.Mint]
deferred:
---

# an off-baseline base says so

When a mint takes an explicit `--base <member>=<branch>` and the pinned
version records a baseline for that member, the mint checks whether the
named branch contains the baseline commit. If it does not — or the
commit is not in the repository at all — the mint prints one note that
names the member, the branch and the baseline, and continues. It is a
note, never a refusal: the explicit base is the escape for histories
the baseline cannot prove (a squashed merge).

## System Context

The auto-base arm already refuses a base that lost its baseline. The
explicit arm is the operator's override and stays open — but silence
there hides typos and haste behind the same flag that covers the
legitimate squash case. One line at mint time moves the discovery from
a foreign diff days later to the moment of choice.

## Satisfy

`Seats.Mint`'s explicit-base arm reads the member baseline from the
pinned version and asks git whether the named base contains it
(`merge-base --is-ancestor`); the note prints on "no" and on a missing
object; no baseline recorded stays silent.

- test — worktree_e2e: `--base` onto a branch without the recorded
  baseline prints the note naming member, branch and sha; `--base` onto
  the branch that carries it stays silent
