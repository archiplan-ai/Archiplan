# Release Manual

A release is four tarballs — `archi-<version>-<platform>.tar.gz` for
`macos-arm64`, `linux-x64`, `linux-arm64`, `windows-x64` — each carrying the
`archi` binary, the platform installer, and `README.txt`, with a `.sha256`
checksum file next to each. They are published as GitHub Release assets on
`archiplan-ai/Archiplan` under tag `v<version>`.

`VERSION` defaults to the version in `crates/archi/Cargo.toml`, so the tarball
name always matches what `archi --version` prints. Never rebuild an
already-published version string — bump the version instead; old tarballs stay
downloadable and that is the migration model.

## Releases

### 0.1.7

A sharper `archi` skill. The embedded operating manual gains a standing
instruction: don't run the whole cycle in one pass — ask the user which stage to
focus on (initial architecture + stress, stress + update, plan, execute) and do
only that. Skill-only: the binary's embedded briefing changes, so `archi init`
and `archi sync-skills` now write the newer copy, but no verb, format or finding
moves. A 0.1.6 tree brought current with `archi sync-skills` reports the `archi`
skill as `updated` and is otherwise byte-identical.

### 0.1.6

`archi sync-skills` — the deliberate verb that reconciles an initialized tree's
briefing with a newer binary. Where `init` is create-only, `sync-skills`
locates an existing project (never creates one) and overwrites any skill or
`CLAUDE.md` block that has drifted from the binary's embedded copy,
unconditionally: a briefing file that already matches is `ok`, an absent one is
`created`, and a divergent one is the new `updated` outcome — refreshed in
place. The `CLAUDE.md` fence marks the one region sync may reclaim, so its inner
block is rewritten while the surrounding prose is left untouched. It touches
only the briefing, never the model, so a reflexive re-run cannot lose source and
`init` stays create-only. `archi sync-skills [--project <dir>]` locates the
project the same way `check` and `build` do; running it in a tree without
`archi.toml` errors and points at `archi init`.
Fully additive: a tree checks, scores and searches byte-identically to 0.1.5,
and on 0.1.5 binaries `sync-skills` is simply an unknown verb.

### 0.1.5

Trade-off axes on the markdown base. Decisions are a new doc primitive — one
file per trade under `archi/decisions/`, frontmatter `links` / `prefer` /
`over` — and the sole carrier of the fixed nine axes (`archi axes` lists them
with definitions; an off-list label is legal, kept verbatim, surfaced as the
`off_list_axis` finding). Stressors gain a fourth outcome, `accepted`: the
break is kept, nothing derives (an origin naming an accepted stressor is
`E_DOC_REF`), and a decision must link it — otherwise the
`accepted_unjustified` finding, mirroring `breaking_unanswered`. An accepted
break is still a break for incidence: the row stands in the matrix under its
own outcome and compounds with survivors, and a compound pair carrying an
accepted member names it — flagged louder. `archi search --kind decision`
retrieves the records (element cards carry `decided-by`); `archi tradeoffs
show` tallies the revealed priority profile beside the declared stance.
Fully additive: a tree using none of the new constructs checks, scores and
searches byte-identically to 0.1.4; on 0.1.4 binaries an `accepted` outcome
is one located `E_DOC` and `archi/decisions/` is invisible.

## Prerequisites (one-time setup)

```sh
rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-pc-windows-gnu
cargo install cargo-zigbuild
brew install zig        # linker for the cross targets
brew install gh && gh auth login   # publishing uses GitHub Releases
```

macOS arm64 builds natively — no extra target or zig involved.

zig version note: `archi`'s dependency tree is pure Rust, so any current zig
works. If a future dependency pulls in C sources with pregenerated-object
archival (e.g. `ring`), zig 0.16's `ar cq` breaks that step — pin zig 0.13 in
the Makefile the way free-fractal does.

## Cutting a release

1. Bump `version` in `crates/archi/Cargo.toml`.
2. Build all four platforms:

```sh
make release
```

Output lands in `dist/`. A single platform can be rebuilt with
`make release-<platform>` (e.g. `make release-macos-arm64`).

## Verify before publishing

```sh
V=$(sed -n 's/^version = "\([^"]*\)".*/\1/p' crates/archi/Cargo.toml)

# correct tarball layout: binary + installer + README.txt
tar -tzf dist/archi-$V-linux-x64.tar.gz

# each binary is the right architecture and stripped
file dist/archi-$V-*/archi dist/archi-$V-windows-x64/archi.exe

# checksums are coherent
cd dist && shasum -a 256 -c archi-$V-*.tar.gz.sha256 && cd ..

# the native binary runs and prints the matching version
dist/archi-$V-macos-arm64/archi --version
```

## Publish

```sh
make publish
```

This creates GitHub release `v<version>` with all tarballs and checksums as
assets. Verify:

```sh
gh release view v$V
```

## Installing (what users run)

```sh
curl -fsSL https://raw.githubusercontent.com/archiplan-ai/Archiplan/main/release/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/archiplan-ai/Archiplan/main/release/install.ps1 | iex
```

The installer resolves the latest release, verifies the checksum, and installs
to `~/.local/bin`. Pin a version with `ARCHI_VERSION=x.y.z`; point at a
different asset host with `ARCHI_BASE_URL`.

While the repository is private, anonymous downloads 404. Authenticated users
install with:

```sh
gh release download v$V -R archiplan-ai/Archiplan -p "archi-$V-macos-arm64.tar.gz"
tar -xzf archi-$V-macos-arm64.tar.gz
install -m 755 archi-$V-macos-arm64/archi "$HOME/.local/bin/archi"
```

## Migrating from the old fractal client

The previous (fractal-era) client also installed as `archi`. On such machines
users run:

```sh
curl -fsSL https://raw.githubusercontent.com/archiplan-ai/Archiplan/main/release/migrate-fractal.sh | sh
```

It renames every fractal-flavored `archi` on PATH to `old-archi` (same
directory, config and license untouched), then installs the new `archi` over
the freed name. Detection keys on the license-activation endpoint literal
(`activate/start`) baked into every shipped fractal build and absent from the
new binary — no execution needed. Old projects stay readable through
`old-archi`, so an agent can migrate each one to the new Archiplan format
using both binaries. If the install step fails, the old client is already
preserved; re-run to retry, or roll back with `mv <dir>/old-archi <dir>/archi`.

The agent-facing crossing guide — which script to run, then how to read the
old project through `old-archi` and rebuild it as a checkable archiplan spec
with an import brief — is `skills/archi-migrate-fractal.md`.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `cargo zigbuild` not found | `cargo install cargo-zigbuild`; confirm `zig version` works. |
| `error[E0463]: can't find crate for core` | Missing cross target: `rustup target add <target>`. |
| `make publish` fails | `gh` not installed/authenticated, or tag `v<version>` already exists — bump the version; never rebuild a published one. |
| Binary is tens of MB | `[profile.release] strip = "symbols"` missing from the workspace `Cargo.toml`. |
| Installer says checksum mismatch | Partial or cached download, or tarball re-uploaded under the same name. Bump the version and republish. |
