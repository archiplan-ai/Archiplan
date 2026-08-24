---
kind: functional
origin: stressor(the-verb-meets-an-unbound-checkout)
satisfied-by: [Cli, DocMint]
deferred:
---

# The world verb refuses like the others

`world add` and `world rm` mutate, so they refuse in a checkout no worktree binds, with
the exit code every mutating verb uses and a refusal that names the standing seats and
the command that mints one. `world ls` reads and runs anywhere. A missing parameter
refuses before the tree is touched.

## System Context

`exit-codes-are-the-contract` and `refusals-name-the-continuation` hold for every verb in
this tool, and the seat rule is unconditional: a mutation runs inside a bound worktree or
it does not run. A new verb that skipped either would be the one door that lets a fact be
written on the main line, outside any unit of work, to be landed by a hand commit no
worktree carries. The uniformity is the whole value — an operator who learned the rule
once should not have to learn where it does not apply.

## Satisfy

`Cli` (the binding check and the refusal text shared with the other mutating verbs; `ls`
exempt as a read). `DocMint` (no write reaches the tree before the checkout is settled).

- test — `world add` in an unbound checkout refuses and names the standing seats
- test — the refusal exit code equals the one `req add` returns for the same case
- test — `world rm` refuses the same way
- test — `world ls` runs in an unbound checkout and exits zero
- test — a refused `world add` leaves no file and no folder behind
