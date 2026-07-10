---
kind: functional
origin: intent
satisfied-by: [Members]
deferred:
---

# Members are declared

A project names the repositories its code lives in — members — in the manifest:
the name is the stable identity refs and journal events carry, the url is
provenance for humans and CI, and the committed path is a layout convention
relative to the project root. Where a checkout actually sits on one machine is a
machine fact: a local, uncommitted override maps a member to its directory, and
resolution reports each member as mapped or unreachable instead of guessing.

## System Context

The manifest is the layout's one authority and is read by the compiler's single
reader. Checkout paths differ per machine and per CI job; committing them would
make the manifest lie everywhere but one desk. Identity must survive forks and
remote renames, so the name keys everything and the url keys nothing.

## Satisfy

`Members` (declarations read through the manifest's one reader; local mapping rows override the
committed path; resolution names every member mapped or unreachable, home implicit).

- test — a declared member with no checkout resolves unreachable and `repo ls` reports it as such
- test — a local mapping row overrides the manifest's committed path for the same member
- test — `repo ls` reports name, resolved root, reachability, cleanliness and baseline per member
