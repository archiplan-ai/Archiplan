# Spec Ontology

Pre-configured ontology lets agents follow certain structure when describing systems, otherwise agents would have to invent ontology on the fly which may have unpredictable results;

## Preset = stdlib

An ontology ships as a **preset**: a named JSON array of creation statements (`define`, `rel-edge`, `conn-edge`,
`app`) defining relations, nodes and how they are connected. The preset is loaded into a fresh model **as its
[standard library](./modeling-lang.md#standard-library)** before any user statements:

- Every preset must define the classifier `rel trans type_of := * -> *` — by that exact shape, not just the name.
  [Layers](./layers.md) and the `types` query filter key off it. A preset that omits it or bends its shape is
  rejected (`E_PRESET_INVALID`).
- Preset elements are **substrate**: dumps omit them, `check` does not report them, and tagging a preset edge
  into views is `E_STDLIB_PROTECTED` (a `define` colliding divergently with a preset name is ordinary
  `E_REDECLARED`). Users may reference them, attach edges to them, and augment their scopes.
- A model **pins its preset in the project manifest** — `preset` in `archi.toml`: `"core"`, `"default"`, or a
  relative path to a JSON preset file (see [source-format](./source-format.md#project-layout)). Every compile
  loads it into the fresh workspace before the lowered batch replays; dumps omit preset elements and replay on
  the same preset.
- Analyses read the ontology through the preset: NKP's default class filter names preset members
  (see [scoring/nkp](../scoring/nkp.md)).

## Default preset

Nodes, each defined by its `doc` — the identity prose a query returns, so an agent reads what the types mean
before classifying a single term:

- **Data** — information at rest or in flight: the payloads connections carry and stores hold
- **Service** — a long-lived active component: it waits, answers and coordinates
- **Function** — a computation: input in, output out, no state of its own
- **Storage** — a keeper of state: what it holds outlives the interactions that touch it

As a preset:

```json
[
  { "stmt": "define", "rel": "type_of", "trans": true, "directed": true,
    "source": "*", "target": "*" },
  { "stmt": "define", "node": "Data",
    "doc": "information at rest or in flight: the payloads connections carry and stores hold" },
  { "stmt": "define", "node": "Service",
    "doc": "a long-lived active component: it waits, answers and coordinates" },
  { "stmt": "define", "node": "Function",
    "doc": "a computation: input in, output out, no state of its own" },
  { "stmt": "define", "node": "Storage",
    "doc": "a keeper of state: what it holds outlives the interactions that touch it" }
]
```
