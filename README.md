# Archiplan

An environment where your software's context live

![](archiplan.svg)

Models are stored as source code: a project of `.arch` files — diffable, modular, compiled fresh on every run
(`requirements/modeling-lang/source-format.md`). The JSON statement API (`requirements/modeling-lang/modeling-lang.md`)
remains the machine interface. `archi check` compiles and lints a project; `archi nkp` analyzes it; `archi exec`
runs statement batches against JSON models.