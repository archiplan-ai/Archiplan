---
affects: [Links, Links.Journal, DocMint]
outcome: breaking
---

# the requirement the writer names has no address to become a link

Name a requirement in the declaration file and try to make a link out of it.

A link's spec side parses as exactly two shapes: `<fact-slug>#<scenario name>` and
`<element path>[@version]`. A requirement slug is neither. It carries no `#`, so it falls to
the element branch, and it resolves against no element of the model. The proposal asks the
writer to declare requirements alongside ports, and half of what they declare has nowhere
to go.

Both ways out cost something. A third address shape makes requirements first-class in the
journal — and the journal is append-only truth, so the shape is permanent from the first
entry, before anybody has used it enough to know whether it was right. Keeping the pair as
a plan record instead costs nothing at the journal and loses everything the journal gives:
the pair does not survive the plan, does not grade, does not drift, and is not there when
the reverse view goes looking.

## Attractor

The cheap path is taken, requirement pairs live in plan folders, and a plan folder is a
record of one unit of work. Six months on, the question "which code answers this
requirement" has to be answered by reading every closed plan that ever owned the slug, which
is the archaeology the whole tool exists to abolish.

## Resolution

The journal gains the third shape: derived `a-requirement-is-addressable-in-the-journal`,
priced by `the-journal-carries-what-it-is-asked-about`. The cheap path — the pair as a plan
record — was refused for the reason this stressor's own attractor names: a plan folder is
the record of one unit of work, and the pair would be gone from every reading that outlives
it.

The permanence is real and unmitigated: append-only truth means `req:<slug>` is fixed from
its first entry, chosen before anybody has used it enough to know it was right. The one
hedge is the prefix, which leaves room for a fourth kind of address beside it. A bare slug
would have left none.
