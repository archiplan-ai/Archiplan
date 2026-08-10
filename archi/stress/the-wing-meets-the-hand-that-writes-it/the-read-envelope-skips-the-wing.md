---
affects: [Agent, Cli, WorldDoc]
outcome: breaking
---

# The read envelope skips the wing

An agent asks the tool what it knows. `archi read` answers from a request envelope over
the model; `archi search` ranks phrases; `world ls` needs a node in hand. None of them
gives an agent the conditions bearing on the work it was handed. The wing was built so an
agent writing code knows why the behavior is what it is, and the agent's own read surface
does not carry it.

## Attractor

Agents keep working the way they did before the wing existed. They read requirements,
they read the model, they never open `archi/world/` because nothing in their briefing's
read path leads there. The facts get written by agents and read by nobody, and the
conditioning that was supposed to reach the hand at the keyboard reaches a folder.

## Resolution

The read surface carries the wing: derived `the-read-envelope-carries-the-conditions`, and
`Query` gains the port that resolves covering facts for the elements of a slice.
`world ls --covers` answers for a person holding a node; the envelope answers for the
reader who did not know to ask, which is the reader the wing was built for.
