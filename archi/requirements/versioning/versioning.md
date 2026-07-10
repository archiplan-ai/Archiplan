# Versioning

An architecture that evolves needs the unit of "we agreed it looked like this": a
semantic snapshot to stress against, plan from and diff between — durable in a shallow
clone, immune to git rewrites, and minted only when meaning moves. The live tree holds
exactly one model, so past agreements are archived forms of the canonical render,
reconstructable bit for bit and cheap enough to keep forever; requirements and stress
rounds are living documents that reference version ids rather than being copied into
them, and the archive's economics — what a version costs in bytes, review and history —
decide whether the habit of saving survives contact with a real repository.
