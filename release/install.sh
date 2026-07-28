#!/bin/sh
set -eu

# Archiplan installer. Resolves the latest GitHub release of
# archiplan-ai/Archiplan, downloads the archi tarball for this platform,
# verifies its checksum and installs the binary to ~/.local/bin:
#
#   curl -fsSL https://raw.githubusercontent.com/archiplan-ai/Archiplan/main/release/install.sh | sh
#
# Pin a version with ARCHI_VERSION=x.y.z; install from a fork or mirror with
# ARCHI_REPO=owner/repo.

REPO="${ARCHI_REPO:-archiplan-ai/Archiplan}"

os=$(uname -s); arch=$(uname -m)
case "$os-$arch" in
  Linux-x86_64)        plat=linux-x64 ;;
  Linux-aarch64)       plat=linux-arm64 ;;
  Linux-arm64)         plat=linux-arm64 ;;
  Darwin-arm64)        plat=macos-arm64 ;;
  *) echo "Unsupported platform: $os-$arch" >&2
     echo "Supported: Linux x86_64/aarch64, macOS aarch64 (Apple Silicon)." >&2
     echo "On Windows (PowerShell): irm https://raw.githubusercontent.com/$REPO/main/release/install.ps1 | iex" >&2
     exit 1 ;;
esac

# The /releases/latest redirect names the newest tag and is not rate-limited;
# the API is the fallback for networks that swallow the redirect.
latest_version() {
  redirect=$(curl -fsSLI --proto '=https' --tlsv1.2 -o /dev/null \
    -w '%{url_effective}' "https://github.com/$REPO/releases/latest" 2>/dev/null || true)
  case "$redirect" in
    */releases/tag/v*) echo "${redirect##*/releases/tag/v}"; return 0 ;;
  esac
  curl -fsSL --proto '=https' --tlsv1.2 \
    "https://api.github.com/repos/$REPO/releases/latest" 2>/dev/null \
    | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v\{0,1\}\([^"]*\)".*/\1/p' \
    | head -1
}

VERSION="${ARCHI_VERSION:-$(latest_version)}"
if [ -z "$VERSION" ]; then
  echo "Could not resolve the latest archi release from $REPO." >&2
  echo "Check https://github.com/$REPO/releases, then retry pinned:" >&2
  echo "  ARCHI_VERSION=x.y.z sh install.sh" >&2
  exit 1
fi

tarball="archi-$VERSION-$plat.tar.gz"
base="https://github.com/$REPO/releases/download/v$VERSION"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT

echo "Downloading $tarball..."
curl -fSL --proto '=https' --tlsv1.2 -o "$tmp/$tarball" "$base/$tarball"
curl -fsSL --proto '=https' --tlsv1.2 -o "$tmp/$tarball.sha256" "$base/$tarball.sha256"

expected=$(cut -d' ' -f1 < "$tmp/$tarball.sha256")
if command -v shasum >/dev/null 2>&1; then
  actual=$(shasum -a 256 "$tmp/$tarball" | cut -d' ' -f1)
elif command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$tmp/$tarball" | cut -d' ' -f1)
else
  actual="$expected"
  echo "Warning: no shasum or sha256sum on PATH — checksum not verified." >&2
fi
if [ "$actual" != "$expected" ]; then
  echo "Checksum mismatch for $tarball — refusing to install." >&2
  echo "  expected $expected" >&2
  echo "  actual   $actual" >&2
  exit 1
fi

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
