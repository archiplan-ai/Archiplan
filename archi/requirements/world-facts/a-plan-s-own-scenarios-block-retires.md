---
kind: functional
origin: stressor(the-standing-plan-loses-its-stories)
satisfied-by: [Planner, PlanFile]
deferred:
---

# A plan's own scenarios block retires

No verb reads a plan's `scenarios.md` any more, and no verb deletes it. A plan with open
lifecycle that still holds one raises the `unmigrated_scenarios` finding, naming the file
and the stories inside it, so the operator moves them into the wing or drops them
deliberately. A completed plan keeps its file untouched and raises nothing: it is the
record of a ceremony that ran under the rule of its day.

## System Context

Twenty-six plan folders in this repository hold stories written under the old rule, and a
silent rule change would leave every one of them as prose that nobody reads and nobody
dares delete. The migration itself cannot be mechanical: turning a story into a world
fact means naming the condition it rests on, and no verb can infer that from the
sentence. So the finding is the whole mechanism — it puts the file in front of a person
exactly while that plan is still live, and stays quiet about history.

## Satisfy

`Planner` (raises `unmigrated_scenarios` for a plan with open lifecycle holding a
non-empty `scenarios.md`, and never reads or writes the file otherwise). `PlanFile` (the
old block stays on disk as the record of what that plan closed on).

- test — an open plan with a non-empty `scenarios.md` raises `unmigrated_scenarios`
- test — a completed plan with the same file raises nothing
- test — the finding names the file path and does not change the exit code
- test — no verb writes or deletes `scenarios.md`
- test — an open plan whose `scenarios.md` is absent or empty raises nothing
