---
affects: [Archive, Links.Grader]
outcome: surviving
---

# A switched branch still resolves

A member's baseline is recorded while a feature branch is checked out. The team merges it, switches
back to `main`, and keeps working; HEAD is now many commits away from the one the baseline names.
Every audit after the switch diffs against that older commit.

## Attractor

Were the baseline a branch name, or resolved through HEAD, the audit's floor would drift with every
checkout — it would measure the delta since "wherever HEAD is now," and a plain branch switch would
silently redraw the window the audit reports as clean.

## Resolution

Holds by construction, and this is the true kernel of the claim. A baseline is the commit's own
content hash, not a ref: `git diff <sha>` addresses that exact object no matter where HEAD points,
so a switch, a merge, or a new branch never move the floor — as long as the object is still in the
database. The SHA's stability under a branch *switch* is real; it is its stability under
*collection* that the next stressor presses.
