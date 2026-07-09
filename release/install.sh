#!/bin/sh
set -eu

# Archiplan installer. Downloads the archi tarball for this platform from
# GitHub Releases and installs it to ~/.local/bin. Pin a version with
# ARCHI_VERSION=x.y.z; point at another asset host with ARCHI_BASE_URL.

REPO='oskin1/Archiplan'
BASE="${ARCHI_BASE_URL:-https://github.com/$REPO/releases/download}"

os=$(uname -s); arch=$(uname -m)
case "$os-$arch" in
  Linux-x86_64)        plat=linux-x64 ;;
  Linux-aarch64)       plat=linux-arm64 ;;
  Linux-arm64)         plat=linux-arm64 ;;
  Darwin-arm64)        plat=macos-arm64 ;;
  *) echo "Unsupported platform: $os-$arch" >&2
     echo "Supported: Linux x86_64/aarch64, macOS aarch64 (Apple Silicon)." >&2
     echo "On Windows, run install.ps1 instead." >&2
     exit 1 ;;
esac

VERSION="${ARCHI_VERSION:-}"
if [ -z "$VERSION" ]; then
  VERSION=$(curl -fsSL --proto '=https' --tlsv1.2 \
      "https://api.github.com/repos/$REPO/releases/latest" \
    | sed -n 's/.*"tag_name": *"v\{0,1\}\([^"]*\)".*/\1/p' | head -1)
  if [ -z "$VERSION" ]; then
    echo "Could not resolve the latest release." >&2
    echo "Set a version explicitly and retry: ARCHI_VERSION=x.y.z" >&2
    exit 1
  fi
fi

tarball="archi-$VERSION-$plat.tar.gz"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT

echo "Downloading $tarball..."
curl -fSL --proto '=https' --tlsv1.2 \
  -o "$tmp/$tarball" "$BASE/v$VERSION/$tarball"
curl -fsSL --proto '=https' --tlsv1.2 \
  -o "$tmp/$tarball.sha256" "$BASE/v$VERSION/$tarball.sha256"

if command -v shasum >/dev/null 2>&1; then
  (cd "$tmp" && shasum -a 256 -c "$tarball.sha256" >/dev/null)
else
  (cd "$tmp" && sha256sum -c "$tarball.sha256" >/dev/null)
fi || { echo "Checksum mismatch for $tarball — aborting." >&2; exit 1; }

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
echo "Get started: run 'archi init' in a project directory."
