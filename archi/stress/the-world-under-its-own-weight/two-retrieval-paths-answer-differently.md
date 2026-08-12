---
affects: [Search, Cli, WorldDoc]
outcome: breaking
---

# Two retrieval paths answer differently

Ask the world the same question two ways. `world ls --covers Worktree` walks `covers` and
answers exactly. `archi search "worktree" --kind world` ranks phrases and answers by
vocabulary — and a fact written under the rule that forbids model nouns will not contain
the word. The two verbs answer the same intent with different sets, and neither says the
other exists.

## Attractor

An operator picks whichever verb they learned first. The one who learned `search` concludes
the world holds nothing about a node it in fact conditions, and stops looking; the decision
that traded phrase-findability for exactness gets paid twice and collected once. Two paths
that disagree are worse than one path that is narrow, because a disagreement teaches
mistrust of both.

## Resolution

Both verbs keep their answers and name the other when they come up empty: derived
`each-retrieval-path-names-the-other`. The paths answer different questions — `covers` is
exact and blind to wording, the phrase scan reads the fact's own vocabulary — and the
failure was never the narrowness but the silence about it. `refusals-name-the-continuation`
already holds this shape; an empty answer is the same situation at exit code zero.
