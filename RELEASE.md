# Release Manual

A release is four tarballs — `archi-<version>-<platform>.tar.gz` for
`macos-arm64`, `linux-x64`, `linux-arm64`, `windows-x64` — each carrying the
`archi` binary, the platform installer, and `README.txt`, with a `.sha256`
checksum file next to each. They are published as GitHub Release assets on
`oskin1/Archiplan` under tag `v<version>`.

`VERSION` defaults to the version in `crates/archi/Cargo.toml`, so the tarball
name always matches what `archi --version` prints. Never rebuild an
already-published version string — bump the version instead; old tarballs stay
downloadable and that is the migration model.

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
curl -fsSL https://raw.githubusercontent.com/oskin1/Archiplan/main/release/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/oskin1/Archiplan/main/release/install.ps1 | iex
```

The installer resolves the latest release, verifies the checksum, and installs
to `~/.local/bin`. Pin a version with `ARCHI_VERSION=x.y.z`; point at a
different asset host with `ARCHI_BASE_URL`.

While the repository is private, anonymous downloads 404. Authenticated users
install with:

```sh
gh release download v$V -R oskin1/Archiplan -p "archi-$V-macos-arm64.tar.gz"
tar -xzf archi-$V-macos-arm64.tar.gz
install -m 755 archi-$V-macos-arm64/archi "$HOME/.local/bin/archi"
```

## Troubleshooting

| Symptom | Fix |
|---|---|
| `cargo zigbuild` not found | `cargo install cargo-zigbuild`; confirm `zig version` works. |
| `error[E0463]: can't find crate for core` | Missing cross target: `rustup target add <target>`. |
| `make publish` fails | `gh` not installed/authenticated, or tag `v<version>` already exists — bump the version; never rebuild a published one. |
| Binary is tens of MB | `[profile.release] strip = "symbols"` missing from the workspace `Cargo.toml`. |
| Installer says checksum mismatch | Partial or cached download, or tarball re-uploaded under the same name. Bump the version and republish. |
