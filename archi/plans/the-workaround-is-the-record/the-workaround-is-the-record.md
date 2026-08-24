# the-workaround-is-the-record

`## What kills this` leaves the fact and `## What people do instead` takes its slot. The
killer was a guess about the future written at the moment its author had just decided to
build, and every one of the four written turned out to be the negation of the workaround
beside it. The workaround does both jobs: no workaround means no condition, and the day
people stop doing it the fact is dead. A fact also stops naming who reported it and stops
quoting them — that is what `sources` and a resource are for.

## Stack

- Rust — existing codebase; this unit introduces no technology
- cargo test, plain `#[test]` integration files — existing convention
- no runtime infrastructure — unchanged

## Architecture

- `WorldDoc` — the record's shape, and the four facts written in it
- `WorldDoc` realizes `crates/archi/src/docs/world.rs` and the files under `archi/world/facts/`
- `Scaffold` — the installed texts that tell a person how to write one
- `Scaffold` realizes `crates/archi/src/scaffold.rs` and the embedded texts under `skills/`
