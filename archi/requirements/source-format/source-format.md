# Source format

The model needs a persistence that humans and agents can author, diff, review and
merge — and the statement layer is none of those things: absolute paths, one JSON
object per statement, no names in scope, no files, no modularity. It is the right
lowering target and the right read surface for agents, but authoring in it means
spelling every path in full, holding a whole project in one undiffable transcript,
and losing the source location the moment anything goes wrong. Yet giving authors a
friendlier front door must not mint a second semantics: a rule enforced only in the
surface — or only in the engine — is a rule that does not exist. So the surface
language has to be sugar: a project of `.arch` files, modular and reviewable, where
every construct lowers to ordinary statements executed by the ordinary engine, and
the compiler adds only what a text surface owes its authors — a manifest and module
discovery, imports and visibility, lexical name resolution, interface-first port
declarations, and diagnostics that read `file:line:col`. And because agents author
most edits and repair from what the tool tells them, the failure side is part of
the language: every statement's outcome is machine-readable, batches are atomic,
identical retries are safe, and error codes form a stable, append-only vocabulary.
