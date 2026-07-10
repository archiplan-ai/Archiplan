---
version: v0012
closed: v0012
---

# Split-tree pressure

v0012 declares members: repositories with identity in the manifest, roots resolved locally,
baselines per member. This round presses that design exactly where a split tree differs from the
one tree it grew up in — a checkout that is not there, a baseline that arrives late, a git root
that never matched the project root, a name that someone renames, one path that exists twice, a
wave that outlives its mapping. Each stressor asks: does the split fail loud and local, or does it
quietly widen into wrong records?
