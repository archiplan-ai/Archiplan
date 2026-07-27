# Worktree parallelism

One checkout cannot hold two working agents: the active-plan pointer is a single slot, wave
snapshots assume one writer, and parallel saves mint colliding ids the operator untangles by
hand. Work wants to run as parallel units — spec, plan, code — each in its own git worktree,
and land back on the main line through the existing merge ceremonies, not through confusion.
The word "session" stays with stress rounds; this intent speaks of checkouts.
