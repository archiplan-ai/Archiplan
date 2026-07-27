#!/bin/sh
set -eu

# Archiplan installer, served at https://archiplan.ai/install.sh; the
# tarballs ride api.archiplan.ai. Downloads the archi tarball for this
# platform and installs the binary to ~/.local/bin. Pin a version with ARCHI_VERSION=x.y.z; point
# at another host with ARCHI_BASE_URL.

BASE="${ARCHI_BASE_URL:-https://api.archiplan.ai}"
VERSION="${ARCHI_VERSION:-__INJECT_AT_DEPLOY__}"

os=$(uname -s); arch=$(uname -m)
case "$os-$arch" in
  Linux-x86_64)        plat=linux-x64 ;;
  Linux-aarch64)       plat=linux-arm64 ;;
  Linux-arm64)         plat=linux-arm64 ;;
  Darwin-arm64)        plat=macos-arm64 ;;
  *) echo "Unsupported platform: $os-$arch" >&2
     echo "Supported: Linux x86_64/aarch64, macOS aarch64 (Apple Silicon)." >&2
     echo "On Windows (PowerShell): irm $BASE/install.ps1 | iex" >&2
     exit 1 ;;
esac

tarball="archi-$VERSION-$plat.tar.gz"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT

echo "Downloading $tarball..."
curl -fSL --proto '=https' --tlsv1.2 \
  -o "$tmp/$tarball" "$BASE/download/$tarball"

tar -xzf "$tmp/$tarball" -C "$tmp"
src="$tmp/archi-$VERSION-$plat"

[ "$os" = "Darwin" ] && command -v xattr >/dev/null && \
  xattr -cr "$src" 2>/dev/null || true

bin_dir="$HOME/.local/bin"
mkdir -p "$bin_dir"
install -m 755 "$src/archi" "$bin_dir/archi"

case ":$PATH:" in
  *":$bin_dir:"*) ;;
  *) echo
     echo "Note: $bin_dir is not on your PATH."
     echo "Add this to your shell rc (.zshrc / .bashrc / .profile):"
     echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
     echo
     echo "For this session only:"
     echo "  export PATH=\"$bin_dir:\$PATH\""
     ;;
esac

echo
echo "Archiplan $VERSION is installed."
echo
echo "Next: open your coding agent in a project and run /archi —"
echo "the agent drives everything from there. 'archi --help' lists the verbs."
