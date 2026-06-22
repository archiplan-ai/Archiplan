# Episodes — grouping incidents into one failure event

Incidents are derived in isolation, but real failures are not independent: one
real-world event spawns many distinct incidents across services. An **episode**
is the set of incidents that belong to **one failure event**. It is the unit
**above** the incident, and it is where relational analytics live (see
[`fault-trees.md`](fault-trees.md)).

## What an episode is

A set of incidents the system believes share one underlying failure event. An
episode has its own **stable identity** (episodes recur — the same event
returns), occurrence stats, and a **blast radius** — the set of services it
touched.

## Correlation signals

An episode is formed from signals already present on the incidents — no new
ingestion:

| Signal | What it links |
| --- | --- |
| **trace correlation** | incidents appearing in the same request / trace |
| **time** | incidents clustered in a window |
| **build / deploy** | incidents sharing the build they appeared on |
| **service-dependency graph** | incidents on services that call each other (from the Context Manager's model) |

## Boundaries

- Correlation must **err toward separate episodes** when a link is ambiguous: a
  "mega-episode" that swallows everything is as wrong as missing the grouping.
- An over-grouped episode must be **splittable**, re-keying its incidents
  (mirrors the incident over-merge guarantee in [`incident-lifecycle.md`](incident-lifecycle.md)).

## Isolation carve-out

Grouping, and the analysis built on it, are the **one** place the system reasons
across incidents. Per-incident analysis stays isolated (see
[`context-isolation.md`](context-isolation.md)); episode work reads only the
finished per-incident **verdicts** — the leaves — never another incident's raw
evidence.

## Requirement

Incidents belonging to one real-world failure event must be grouped into an
**episode** — correlated by trace, time, build, and the service-dependency graph
— with a stable identity that recurs, a blast radius, and a boundary that errs
toward separate episodes and supports splitting.

## Questions

- Correlation thresholds: how strong a signal must be to form or break an episode.
- Whether one incident may belong to more than one episode.
- Episode recurrence identity: when is a returning event "the same episode"?
