---
kind: functional
origin: stressor(under-stressed-wall)
satisfied-by: [Incidence]
deferred:
---

# Under-stressed names behavior

The under-stressed sweep names behavioral terms by default: a zero column whose term sits in the
`type_of` closure of `Data` emits no finding, so the report's tail is the actionable list of
unpressed *components*, not the vocabulary. `--all-terms` widens the sweep back to every zero
column. The filter lives at the emission site alone — column construction, the matrix, hotspots,
coupling and compound findings always see every term, pressed data terms included — and a term
no ontology classifies is never muted: no `Data` in the preset means no filter at all.

## System Context

The report auto-fires at the close of every stress round — the reader's moment of maximum
attention — and NKP's slice already draws this exact boundary (`Data type_of _` dropped by
default, widened by flag).

## Satisfy

`Incidence` (its scan consults the model's `type_of` closure of `Data` when emitting
`under_stressed`, and nowhere else; the CLI's `--all-terms` flag sets the config that disables
the filter).

- test — incidence: a matrix with a pressed and an unpressed data term keeps the pressed column in matrix and findings, mutes only the unpressed term's info line
- test — incidence: `all_terms` restores the muted findings verbatim
- test — incidence: a core-preset model (no `Data` anywhere) emits under-stressed findings for every zero column with the filter on
- test — cli: `archi incidence --all-terms` widens the sweep; the default report over this repository's own rounds names no pure-data terms
