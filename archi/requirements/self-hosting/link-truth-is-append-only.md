---
kind: non-functional
origin: stressor(one-blob-of-links)
satisfied-by: [Links.Journal]
deferred:
---

# Link truth is append-only

The link journal is the single truth of traceability, and it only grows: add, confirm, repin,
retire, touch and decay are events; the live link set is the fold of the event stream, and
confidence is derived from it — recomputed, never stored. Any question about how a link came to
its standing has an answer in the record.

## System Context

The journal is a JSONL file in the tree, edited by capture and the link verbs, merged by git
like any other append-mostly file.

## Satisfy

`Links.Journal` accepts events from the verbs and from capture, serves the folded live set to
the graders and the plan's coverage gate, and never rewrites a line once appended.

- test — retire a link and re-add its spec ref: the fold shows a fresh id and the tombstone stays in the record
- test — replay a journal into a fresh fold: standing and confidence recompute identically from the events alone
