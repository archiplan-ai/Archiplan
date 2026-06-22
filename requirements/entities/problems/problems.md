# Problems

A **problem** is a **business task** — a pointer to a work item in the customer's
tracker — with the **archiplan tasks** that address it attached.

It bridges two sides:

- the **business** side — the work item as the customer keeps it in their tracker,
  held here as a pointer;
- the **machine** side — the [archi tasks](../archi/requirements/tasks.md):
  the actionable, spec-informed breakdown that carries the problem out.

A problem carries an **array** of archi tasks — the machine work that answers one
business item.

## Shape

| Field | Notes |
| --- | --- |
| `id` | unique identifier |
| `storage_type` | where the business task lives — `jira`, `linear`, … The tool stamps it on sync; descriptive and observable. |
| `external_id` | the work item's id **within that `storage_type`**. The pair (`storage_type`, `external_id`) is a self-contained external address. |
| `hash` | pinned version of the external work item. Freshness comes from comparing it with the item's current hash. |
| `archi_tasks[]` | the archi tasks attached — the machine breakdown that addresses this problem. |

## Freshness

The CM holds the pinned `hash`. A tool re-reads the work item, computes its current
hash, and reports: a hash that matches is **fresh**, a hash that differs is
**stale**. The CM stores the marker; the tool re-reads (on a hook or on a schedule)
and reports the result ([tooling-layer.md](../../context-manager/tooling-layer.md)).

## Full info lives at the source

The CM holds the pointer and the linkage to archi tasks. The work item's live
detail — its status and the rest — lives in the customer's system; the CM gathers
it **through the tool** when full info is needed.

## Store data vs wiring

`storage_type` is **data about the work item** and stays on the record. The binding
**which tool serves a problem** is **wiring** (`type → tool`), held in the tooling
config and the generated edge — the same split as [docs](../docs/docs.md).
