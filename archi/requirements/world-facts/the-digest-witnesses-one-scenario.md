---
kind: functional
origin: stressor(the-digest-is-wider-than-the-pair)
satisfied-by: [Links, Links.Canonizer, WorldDoc]
deferred:
---

# The digest witnesses one scenario

The digest a scenario link records covers that scenario alone — its name, its step
keywords and its step text. Reworking a sibling scenario in the same fact moves nothing.
One function still serves both readers: it takes the scenario to fingerprint, and the
plan's drift line asks for the whole block by passing none.

## System Context

`a-scenario-link-is-a-witnessed-pair` weighed one cost and accepted it: a step reworded
for clarity fails a link over untouched code, and the operator repins. What shipped
charged a wider one — every link in a fact decays when any scenario in it moves — so a
fact that elaborates its condition into four scenarios makes each of its links four times
noisier. That is not the trade that was signed.

The single shared function stays, because the plan's drift report and the link's grade
must never disagree about what "the scenario changed" means. The grain becomes an
argument instead of a second function: whole block for the plan, one scenario for the
link, one contract for both.

## Satisfy

`Links.Canonizer` (the digest over one parsed scenario, and over the whole block when
none is named). `Links` (the grade reads the per-scenario digest). `WorldDoc` (the parsed
scenario is what gets fingerprinted).

- test — rewording a step in a sibling scenario leaves the link clean
- test — rewording a step in the addressed scenario fails the link and names the scenario side
- test — renaming the addressed scenario still unresolves the address, not the digest
- test — the plan's drift line still moves when any scenario in the fact changes
- test — one function serves both grains; there is no second digest in the crate
