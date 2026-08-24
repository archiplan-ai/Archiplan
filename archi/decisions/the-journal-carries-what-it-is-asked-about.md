---
links: [the-requirement-the-writer-names-has-no-address-to-become-a-link, a-requirement-is-addressable-in-the-journal]
prefer: [evolvability]
over: [simplicity]
---

# The journal carries what it is asked about

A requirement gains an address in the journal rather than living as a plan record. The
journal is append-only, so the shape `req:<slug>` is permanent from its first entry, chosen
before anybody has used it enough to know it was right. That is the price, and it is paid
deliberately.

The alternative was cheaper by every immediate measure: keep the pair "requirement → code"
in the plan folder, add nothing to the journal, break nothing. It was refused because a plan
folder is the record of one unit of work. The pair would not survive the plan that made it,
would not grade, would not drift, and would be absent when the reverse view went looking.
"Which code answers this requirement" would then be answered by reading every closed plan
that ever owned the slug, which is the archaeology this tool exists to abolish.

So the rule is: the journal carries what the tool is expected to answer questions about. It
is asked about requirements. It therefore holds them, at the cost of a permanent shape and
a third branch in every reader of a spec ref.

What keeps the cost bounded is the prefix. `req:` says what the address is, so a fourth kind
can be added beside it without ambiguity — a bare slug could never have been extended that
way, and choosing a prefixed shape now is what buys room to be wrong later.
