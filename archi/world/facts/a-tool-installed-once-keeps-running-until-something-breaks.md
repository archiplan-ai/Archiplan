---
covers: [Updater]
sources: []
uses: []
---

# A tool installed once keeps running until something breaks

Somebody installs a tool once and then stops thinking about it. It works, so there is no
occasion to wonder whether a newer one exists, and no occasion arrives on its own: the tool
that is running is the only one visible from inside the work. Meanwhile the thing does move
— it gains what it was missing, and it loses the defects people already hit. The gap between
what somebody runs and what is available grows quietly and is discovered by accident: a
defect that was fixed months ago, a step somebody else describes that does not exist here.
The behavior follows from that silence: the tool has to say for itself that something newer
is out, and it has to be able to put that newer thing in place without the person going and
finding it.

## What people do instead

They reinstall by hand, now and then, on the chance that something has changed. It costs
little each time and it is done from a hunch rather than from knowing, so it happens too
often when nothing moved and not at all in the stretch when something did. Between two
reinstalls somebody works against defects already repaired and asks for help against a
behavior nobody else still has.

## Scenarios

### The tool says when a newer one is out

Given a tool that is not the newest available
When work begins with it
Then one line says a newer one exists and names it
And nothing is installed without being asked

### The tool puts the newer one in place when told to

Given a newer release exists
When somebody asks for it
Then the running tool is replaced by that release
And what is now in place is named back

## Open questions

How long somebody stays on an old build before noticing is not measured, and neither is how
much of the reinstalling by hand changed anything.
