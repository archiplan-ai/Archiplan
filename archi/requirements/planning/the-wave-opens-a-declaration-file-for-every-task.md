---
kind: functional
origin: intent
satisfied-by: [Planner, PlanFile]
deferred:
---

# The wave opens a declaration file for every task

Opening a wave writes an empty declaration file for every task it puts in flight, beside the
index it already writes. The file carries the shape as comments and no entry. So the close
never meets an absent file: it meets one that declares nothing, which is one refusal instead
of two. `plan reset` clears them with the rest of the wave's state.

## System Context

The absent file and the empty file were two refusals for one situation — a task that has
accounted for nothing — and the reader had to learn which was which. Writing the file at the
open collapses them: from the task agent's side the file is always there, and its job is to
fill it.

The shape lives in the file as comments rather than in a skill or a prompt, because a prompt
is retyped every wave and drifts from the parser while the file cannot: the same code writes
the template and reads it back.

## Satisfy

`Planner` (writes one empty declaration file per in-flight task when the wave opens, and
clears them on reset). `PlanFile` (the file's empty shape: the comment header and no entry).

- test — opening a wave writes one file per task in flight, beside the wave index
- test — the file carries the shape as comments and declares nothing
- test — the close refuses it as a file that declares nothing, not as an absent one
- test — `plan reset` removes the files with the rest of the wave's state
- test — a wave opened before this change, whose files are absent, still refuses and says so
