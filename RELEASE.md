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

### 0.1.13

A seventh skill joins the briefing: `ste-writing`. It writes prose in
ASD-STE100 Simplified Technical English — documentation, READMEs,
pull-request text, error messages, release notes, and comments. It does not
touch code, identifiers, or command syntax. The rules are mechanical and
lintable: one name for one thing, the short common word, active voice, one
instruction per sentence. No semicolons, no contractions, no marketing
adjectives. Two modes carry it. Strict mode applies every rule and both
length caps to procedures, runbooks, and error text. STE-flavored mode keeps
the sentence, paragraph, and active-voice discipline for general prose, and
relaxes the dictionary so the text still reads naturally. Six self-lint
checks run before the text returns.

The `CLAUDE.md` block gains a third standing directive: write architecture
prose in Simplified Technical English. This entry is the first one written
under that rule.

Skill-only, like 0.1.7 and 0.1.8. The binary's embedded briefing changes, so
`archi init` and `archi sync-skills` write the newer copy. No verb, format or
finding moves. A 0.1.12 tree brought current with `archi sync-skills` reports
`.claude/skills/ste-writing/SKILL.md` as `created` and `CLAUDE.md` as
`updated`. Every other file stays byte-identical.

### 0.1.12

Self-update. Two project-less verbs run outside every guard, against the
same truth the installers ship with — the repository's GitHub releases:
`archi check-update` resolves the newest tag exactly as `install.sh` does
(the `releases/latest` redirect, the API as fallback, `ARCHI_REPO` for
forks) and answers in one line — up to date, a newer number named toward
`archi update`, or an older one worded as the feed's rollback (the feed is
the truth in both directions, so one `update` always converges);
`archi update` downloads the platform tarball and its published `.sha256`
from the release assets, proves the checksum through system hashing
(`shasum`/`sha256sum`) before anything unpacks — torn, tampered or
unverifiable assets refuse with the standing binary byte-identical — and
replaces the running binary with a single atomic rename onto the resolved
`current_exe`. Symlinks keep: the target file changes, never the link; the
report names the replaced path, and a path inside a cargo `target/` dir is
called out as a replaced build artifact. All network rides system `curl`
and unpacking rides system `tar` — the plumbing doctrine of the git layer,
zero new dependencies; `ARCHI_BASE_URL` swaps the whole feed for mirrors
and keeps the e2e suite on `file://` fixtures. Windows refuses toward the
PowerShell installer.

The old fractal client is walled off for good: its server keeps `/version`
answering the old world's own final number, so fractal-era binaries report
"up to date" and are never nudged onto the new tool.

### 0.1.11

Plans become folders of records. `archi/plans/<name>/` holds the charter
(`<name>.md`), one `t<N>-<node-slug>.md` per task, `scenarios.md` and a
verbs-only `state.json`; creation, removal and lifecycle stay verbs
(`plan use`, `plan task add|rm`, `start`/`next`/`close`/`reset`), prose and
curation are edits to the files, and `plan verify` is the worklist holding
them together. Waves still derive from the Inputs graph — never stored. A
legacy `plan.json` reads and runs its lifecycle but refuses authoring;
`plan show <name>` renders any plan by name without activating it. The
0.1.10 authoring-verb surface (problem/tech/summary/mapping/scenarios/task
field verbs) retires with the format.

Doc skeletons come from verbs. `archi req add|rm` and
`archi stress open|add|rm` mint and retire requirement and stressor records
with every machine field explicit — no defaults, unknown intents list the
folders, empty text slots are held by `check` as the un-skippable worklist.
Re-mints converge: an untouched skeleton reports "already minted", an edited
file refuses rather than overwrite, and removals pre-flight their blast
radius (a plan-owned requirement, a requirement-holding stressor).

The landing gets a baseline gate. `archi worktree merge` pre-flights every
cascaded member — worktree tip against the pinned version's recorded
baseline — and refuses stale or missing marks in one batched message with
the repair spelled out; `archi version anchor --repo <member>` now records a
missing baseline and re-records a moved one on the latest version
(anchor-born, older versions never move), and the mint's auto-base notes how
far behind a reachable baseline sits. `check` and `build` gain the verdict
gate: a dirty governed spec in an unbound checkout refuses to bless itself.
A seat extension resolves members from the seat, not the primary. The
briefings follow: implement anchors the seat at DONE, finish-worktree names
the stale-baseline repair, the plan skill teaches the record flow, the
hardened save closes on a commit poll, and a stress round widening past the
repository the user named must ask first.

### 0.1.10

Parallel work gets seats. A machine-local registry in the git common dir binds
each worktree to one unit of work — spec, then plan, then code — and `archi
status` plus `archi worktree mint|ls|drop|merge` drive that lifecycle: mint
cascades to member repositories on baseline-proven branches and writes the seat
overlay, merge lands the unit whole (members by push and PR only, closed plans
only), retires the worktree and clears its binding. Version pins carry content
hashes, so a remint can never silently reinterpret one.

The guard is unconditional and lives at the router: the binding, not the
branch, licenses a mutation. An unbound checkout — the primary included —
refuses every mutating verb and names the seats it could continue instead; a
gitless tree refuses loudly, since isolation, branches and merge need a
repository. `check` and `build` pass a verdict gate of their own: uncommitted
edits under a governed spec in an unbound checkout refuse rather than bless
themselves, while a bound seat, a clean tree (CI, the receiving checkout) and a
tree mid-merge all pass — the join triage needs `check` exactly while `archi/`
is conflicted. `protected` keeps one meaning: branches that never receive a
local merge. This is the upgrade-visible change — a 0.1.9 habit of mutating in
the primary checkout must now seat the work first (`archi plan use <name>`, or
`archi worktree mint <slug>` for spec work without a plan).

Plan authoring returns to the CLI. Every authoring mutation is a verb again —
`plan problem`, `tech add`, `architecture-summary add`, `stack-mapping add`,
`scenarios add|list|remove`, task `desc`/`show`/`stack-detail`/`spec-ref`/
`input --from`/`output`, `task req suggest|add|remove|req-list`, `task
verification add|remove`, `plan list|status` — each a validated
read-modify-write of `plan.json`, so the file stays the truth and the CLI is
its only author. `archi batch` re-invokes the binary per stdin line: no second
dispatch surface, every future verb batchable by construction, fail-fast with
the offending line named.

Requirement ownership is curated, not inferred. The derived matched set stays
the ever-fresh candidate list and each task selects a strict subset into
`owns`; verification duty follows ownership, so one requirement no longer
demands proofs from every task that touched its element. `plan verify` is the
whole worklist again — unowned candidates, `owns` outside the match, missing or
orphaned verifications, empty descriptions, the summary/stack-mapping
cross-check — and the reverse lookup now folds a port to its owning node's
surface and canonical edge text to both endpoints, so port-pinned requirements
stop vanishing from the planner. Satisfied requirements no task of the plan
reaches surface as notes: scope by decision, never by silence.

The briefing splits three ways. `/archi-plan` and `/archi-implement` are skills
of their own, `/archi-finish-worktree` closes a seat, and every skill opens by
policing its own freshness — run `archi sync-skills` first, an `updated` report
means re-read the file before acting. The `CLAUDE.md` block regains its two
directives. `archi init` writes `protected = ["main"]` into new manifests and
scaffolds `.gitignore`, so the discipline is on from birth; a declared
discipline without git refuses, create-or-cancel.

Distribution moves to GitHub: the repository is public under MIT, and
`install.sh` / `install.ps1` resolve the latest release, verify the published
`.sha256` and install from the release assets.

### 0.1.9

`archi viz` learns to draw the flow of data. A payload riding a connection's
lanes (`carrier`/`rev_carrier`) that is itself in the slice is drawn *through*
— source → data → target, the reverse lane routing back — so shared data
becomes the junction its producers and consumers meet at instead of an
unconnected box in a footnote. The diagram's vocabulary is now three-way:
`[Component]` square, `(Data)` rounded, and every direct edge tagged with its
rel/conn type on the path — `A → ‹wire› → B` — so no arrow is anonymous; a
routed edge carries no tag, its payload names the interaction. Parallel edges
of different types stay distinct paths, feedback notes name their type, and
the new `data carried on edges:` note doubles as the legend. Ports, views,
prose and out-of-slice carriers remain `--details`. The embedded `archi` skill
also gains a standing instruction to keep spec prose short, so `archi
sync-skills` reports it `updated`.

### 0.1.8

The `archi` skill learns to run a session as a guided loop. A second standing
instruction joins the stage-focus rule: walk the user through the steps in
order, and before each one ask whether to complete it autonomously and
summarize or to collaborate — propose, discuss, execute only after alignment —
then offer next directions afterward. Every such question goes through the
editor's poll tool (`AskUserQuestion` in Claude Code, the equivalent elsewhere)
rather than a freeform prompt. Skill-only, like 0.1.7: the binary's embedded
briefing changes, so `archi init` and `archi sync-skills` write the newer copy,
but no verb, format or finding moves. A 0.1.7 tree brought current with `archi
sync-skills` reports the `archi` skill as `updated` and is otherwise
byte-identical.

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

The installer resolves the newest tag from `/releases/latest` (falling back to
the GitHub API when a network eats the redirect), downloads the platform
tarball and its `.sha256` from that release's assets, verifies the checksum,
and installs to `~/.local/bin`. Pin a version with `ARCHI_VERSION=x.y.z`;
install from a fork or mirror with `ARCHI_REPO=owner/repo`.

The repository is public, so release assets download anonymously. The manual
path, for anyone who would rather not pipe a script into a shell:

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
