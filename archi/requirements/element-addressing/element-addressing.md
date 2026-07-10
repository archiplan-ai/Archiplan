# Element addressing

A visual bug or a wrong diagnostic today has no addressable way in. Archi is drivable by agents
and CLI-literate humans, but when something reads wrong — a finding, a rendered report, a log
line — the reporter can only paraphrase the spec, and the agent that picks the report up fixes
the wrong thing. The system already speaks in element ids internally — node paths, requirement
slugs, edge surface text — yet drops them from the surfaces a human actually reads. Every
diagnostic, finding, and rendered report line should carry the id of the node or requirement it
concerns, so "this is broken" can name the element instead of describing it, and the id resolves
straight back to one element. This is the first, smallest slice of the larger human-facing surface
the bootstrap deferred — the one that unblocks the rest, because every later view needs its
elements addressable before it can point at them.
