# Docs

A **doc** is a pointer to an external document — a page in a knowledge base, a
file in a repository — attached to the parts of the spec it describes.

Docs live on their **own plane** and reference the spec. The spec stays pure
architecture (the [modeling language](../archi/requirements/modeling-lang/modeling-lang.md));
a doc hangs **on** spec elements through links.

## Shape

| Field | Notes |
| --- | --- |
| `id` | unique identifier |
| `storage_type` | where the document lives — `confluence`, `notion`, `github`, … The tool stamps it on sync; descriptive and observable (the agent / UI reads it to know the source and how to read `external_id`). |
| `external_id` | the document's id **within that `storage_type`**. The pair (`storage_type`, `external_id`) is a self-contained external address. |
| `hash` | pinned version of the external document. Freshness comes from comparing it with the document's current hash. |
| `links[]` | attachments to the spec — each `{ target, kind }`. |

### Links

Each link attaches the doc to one spec element:

- **`target`** — a reference to a spec **node or edge** (a doc may describe a
  connection, the same as a component). The target's type lives in the ref.
- **`kind`** — the **role** of the attachment: `documents`, `specifies`, … with
  `documents` as the default. This `kind` is the attachment role; it is its own
  thing, separate from an edge's archi kind (relation / connection / application),
  which lives inside the spec.

A doc carries **several** links — it describes as many spec elements as it covers.

## Freshness

The CM holds the pinned `hash`. A tool re-reads the document, computes its current
hash, and reports: a hash that matches is **fresh**, a hash that differs is
**stale**. The CM stores the marker; the tool re-reads (on a hook or on a schedule)
and reports the result ([tooling-layer.md](../../context-manager/tooling-layer.md)).

## Store data vs wiring

`storage_type` is **data about the document** — a property of the artifact, set by
the tool. It stays on the record.

The binding **which tool serves a doc** is **wiring**, held in the tooling config
and the generated edge (`type → tool`), so it stays out of the store row:

- **push** — the tool sends an update, identifies itself by its secret, and the CM
  matches the doc by `external_id`.
- **pull / resolve** — the CM picks the tool from the `type → tool` binding.

One doc type maps to one tool. A single doc type fed by several tools at once is the
case that would make per-record tool attribution data; that case is out of scope
here.
