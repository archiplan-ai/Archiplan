---
affects: [WorldDoc, DocMint, Links]
outcome: breaking
---

# The fact dies and the link survives

Retire a world fact whose scenario a code link anchors to a step definition. `world
rm` pre-flights one thing: another fact that names this slug in `uses`. It reads
nothing of the link journal. The file goes, its scenarios go with it, and the journal
keeps events that point at a spec ref which no longer resolves. Deletion is the whole
purpose of the wing, so this is the common path and not the rare one.

## Attractor

The journal fills with links to scenarios that no longer exist. `link verify` grades
them as drift, the code they name is untouched and correct, and the operator learns
that the grade means nothing. The one machine-decided edge from spec to code loses
its authority exactly where the wing was supposed to give it authority.

## Resolution

The removal reads the journal beside the `uses` graph and refuses, naming every
stranded link with the code item it holds: derived `removal-names-the-code-it-strands`.
It refuses rather than cascades, because a link is a recorded human assertion and no
verb retires one on the operator's behalf. The price is a new dependency from the
minting verb to the link set, along an axis the wing already keeps: name the blast
radius before the file goes.
