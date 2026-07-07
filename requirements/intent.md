# Intent

Intent is a piece of problem statement, e.g. "create a crowdsourcing app that helps find good coffee in the
city".

In contrast to [requirements](requirements.md), intents are flat — they do not form hierarchies. They also do
not appear mid-stress-session: requirements added during a session answer pressure
([stressing.md](stressing.md)); a new problem statement is a conversation with the stakeholder, not a reaction
to a stressor.

## Stored as files

An intent anchors a requirements area: it is the folder-named file `archi/requirements/<intent>/<intent>.md`,
and the folder holds the requirements it gave rise to
([requirements.md#stored-as-files](requirements.md#stored-as-files)).

The schema is the minimal one: no frontmatter — an intent has no machine fields — just the H1 name and the
problem statement, summary paragraph first. The filename is the slug ([slug.md](slug.md)). Intent files compile
under the shared doc catalog ([requirements.md#compile](requirements.md#compile)): schema, slug uniqueness,
placement.
