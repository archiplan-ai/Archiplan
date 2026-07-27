---
kind: functional
origin: intent
satisfied-by: [Seats.Guard, Cli]
deferred:
---

# Mutation needs a seat

The seat discipline is unconditional: a mutating verb runs only in a
checkout the registry binds. An unbound checkout, the primary included,
refuses on any branch; read verbs answer everywhere. The binding, not the
branch, is the license to mutate. Without a git repository the refusal is
loud, naming `git init` and the user's consent — the discipline never
silently evaporates and no manifest switch opts out.

## System Context

Cli wires the gate once, at the router — verb bodies never guard themselves;
`protected` keeps a single, separate meaning: branches that refuse a local
landing (protected-branches-land-by-pr). The
refusal is what makes the discipline enforced rather than conventional — an
agent can skip a skill step, not a refusal. Litter lands on no branch of the
primary checkout: work happens in seats (the-registry-binds-the-worktree),
and only the closing verb brings it back.

## Satisfy

`Cli` wires the gate once, at the router — verb bodies never guard themselves; `Seats.Guard`
refuses every mutating verb outside a bound checkout, unconditionally.

- test — an unbound checkout refuses mutation on any branch, gitless refuses naming `git init` (`the_guard_is_unconditional`)
- test — read verbs answer on an unbound checkout while mutation refuses (`read_verbs_answer_on_an_unbound_checkout`)
