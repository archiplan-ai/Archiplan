#!/bin/sh
set -eu

# Build the release tarballs for every supported platform into dist/.
# Usage: release/build.sh <version>   (must match the crate version)
# Cross-compiles linux and windows via cargo-zigbuild; macos is native.

V="${1:?usage: release/build.sh <version>}"
crate_v=$(grep '^version' crates/archi/Cargo.toml | head -1 | cut -d'"' -f2)
[ "$V" = "$crate_v" ] || { echo "version $V != crate $crate_v" >&2; exit 1; }

rm -rf dist && mkdir -p dist

pack() { # pack <plat> <bin> [bin_name] [installer]
  plat="$1"; bin="$2"; bin_name="${3:-archi}"; installer="${4:-release/install.sh}"
  d="dist/archi-$V-$plat"
  mkdir -p "$d"
  install -m 755 "$bin" "$d/$bin_name"
  cp "$installer" release/README.txt "$d/"
  tar -czf "dist/archi-$V-$plat.tar.gz" -C dist "archi-$V-$plat"
  (cd dist && shasum -a 256 "archi-$V-$plat.tar.gz" > "archi-$V-$plat.tar.gz.sha256")
  echo "packed archi-$V-$plat"
}

cargo build --release -p archi
pack macos-arm64 target/release/archi

cargo zigbuild --release -p archi --target x86_64-unknown-linux-gnu
pack linux-x64 target/x86_64-unknown-linux-gnu/release/archi

cargo zigbuild --release -p archi --target aarch64-unknown-linux-gnu
pack linux-arm64 target/aarch64-unknown-linux-gnu/release/archi

cargo zigbuild --release -p archi --target x86_64-pc-windows-gnu
pack windows-x64 target/x86_64-pc-windows-gnu/release/archi.exe archi.exe release/install.ps1

ls -la dist/*.tar.gz
