---
kind: functional
origin: stressor(the-requirement-the-writer-names-has-no-address-to-become-a-link)
satisfied-by: [Links, Links.Journal, Links.Grader]
deferred:
---

# A requirement is addressable in the journal

A link's spec side takes a third shape: `req:<slug>`, resolving against the requirement set
of the pinned version. It grades as the other shapes do — the code side moves, the link
moves — and it carries no version pin, because a requirement slug is stable across versions
by the rule that already governs plan ownership. The two standing shapes are untouched.

## System Context

`SpecRef::parse` takes exactly two shapes today, and a requirement slug is neither: it holds
no `#`, so it falls to the element branch and resolves against no element. Half of what a
writer is asked to declare has nowhere to go.

The alternative was to keep requirement pairs as plan records, and it was refused. A plan
folder is the record of one unit of work; the pair would not survive the plan, would not
grade, would not drift, and would be absent when the reverse view went looking. The question
"which code answers this requirement" would then be answered by reading every closed plan
that ever owned the slug — the archaeology this tool exists to abolish.

The cost is that the journal is append-only, so the shape is permanent from its first entry,
chosen before anyone has used it enough to know it was right. `the-journal-carries-what-it-is-asked-about`
prices that. The mitigation is the prefix: an addressing scheme that says what it is can be
joined later by another that says what it is, where a bare slug could not.

## Satisfy

`Links` (accepts and renders the third shape). `Links.Journal` (stores it beside the other
two). `Links.Grader` (grades it, and reports it unresolvable when the slug retires).

- test — `req:<slug>` resolves against the pinned version's requirement set
- test — `req:` naming no requirement is refused, and the refusal names the slug
- test — a bare slug with no prefix still falls to the element branch and its message is
  unchanged
- test — the code side moving fails a `req:` link exactly as it fails an element link
- test — retiring the requirement makes the link unresolvable, distinct from missing
