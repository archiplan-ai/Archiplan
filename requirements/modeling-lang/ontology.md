# Spec Ontology

Pre-configured ontology lets agents follow certain structure when describing systems, otherwise agents would have to invent ontology on the fly which may have unpredictable results;

## Preset = stdlib

An ontology ships as a **preset**: a named JSON array of creation statements (`define`, `rel-edge`, `conn-edge`,
`app`) defining relations, nodes and how they are connected. The preset is loaded into a fresh model **as its
[standard library](./modeling-lang.md#standard-library)** before any user statements:

- Every preset must define the classifier `rel trans type_of := * -> *` — by that exact shape, not just the name.
  [Layers](./layers.md) and the `types` query filter key off it. A preset that omits it or bends its shape is
  rejected (`E_PRESET_INVALID`).
- Preset elements are **substrate**: dumps omit them, `check` does not report them, and mutating them —
  delete, rename, divergent redefine, tagging a preset edge into views — is `E_STDLIB_PROTECTED`. Users may
  reference them, attach edges to them, and augment their scopes.
- A model **pins its preset at creation**: the model file records the preset (name + statements) and every
  restore loads it before replaying the dump. The CLI resolves a new model's preset from `--preset <file>`,
  else an `ontology.json` next to the model file, else the built-in default below.
- Analyses read the ontology through the preset: NKP's default class filter names preset members
  (see [scoring/nkp](../scoring/nkp.md)).

## Default preset

Nodes:
- Data
- Service
- Function
- Storage

As a preset:

```json
[
  { "stmt": "define", "rel": "type_of", "trans": true, "directed": true,
    "source": "*", "target": "*" },
  { "stmt": "define", "node": "Data" },
  { "stmt": "define", "node": "Service" },
  { "stmt": "define", "node": "Function" },
  { "stmt": "define", "node": "Storage" }
]
```
