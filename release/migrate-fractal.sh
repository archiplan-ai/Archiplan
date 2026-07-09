#!/bin/sh
set -eu

# Migrates a machine from the old fractal-era client to the new Archiplan
# CLI. Both install as `archi`; this renames the old binary to `old-archi`
# (same directory, config untouched), then installs the new archi over the
# freed name. Old projects stay readable through `old-archi`, so an agent
# can migrate each one to the new Archiplan format using both binaries.

REPO='archiplan-ai/Archiplan'
INSTALLER="${ARCHI_INSTALLER_URL:-https://raw.githubusercontent.com/$REPO/main/release/install.sh}"

# Every fractal build carries its license-activation endpoint as a literal;
# the new archi has no activation flow. That string is the discriminator.
is_fractal() { grep -aq 'activate/start' "$1" 2>/dev/null; }

# Candidates: every archi on PATH, plus the old installer's fixed location.
candidates=$(
  { IFS=:
    for d in $PATH; do
      [ -n "$d" ] && [ -f "$d/archi" ] && [ -x "$d/archi" ] && echo "$d/archi"
    done
    [ -f "$HOME/.local/bin/archi" ] && echo "$HOME/.local/bin/archi"
  } | awk '!seen[$0]++'
)

renamed=''
recover_hint() {
  echo >&2
  echo "The new archi did not finish installing. Old client(s) preserved:" >&2
  echo "$renamed" >&2
  echo "Re-run this script to retry; to roll back a rename:" >&2
  echo "  mv <dir>/old-archi <dir>/archi" >&2
}
trap 'rc=$?; if [ "$rc" -ne 0 ] && [ -n "$renamed" ]; then recover_hint; fi' EXIT

old_ifs=$IFS; IFS='
'
for bin in $candidates; do
  is_fractal "$bin" || continue
  dir=$(dirname "$bin")
  if [ -e "$dir/old-archi" ] && ! cmp -s "$bin" "$dir/old-archi"; then
    echo "Cannot migrate $bin: $dir/old-archi already exists and differs." >&2
    echo "Move it out of the way, then re-run this script." >&2
    exit 1
  fi
  mv -f "$bin" "$dir/old-archi"
  echo "Preserved old fractal client: $dir/old-archi"
  renamed="${renamed:+$renamed
}  $dir/old-archi"
done
IFS=$old_ifs

if [ -z "$renamed" ]; then
  if command -v archi >/dev/null 2>&1; then
    echo "No old fractal client found; $(command -v archi) is already the new archi."
  else
    echo "No 'archi' found — nothing to migrate. Install the new CLI with:"
    echo "  curl -fsSL $INSTALLER | sh"
  fi
  exit 0
fi

# Install the new archi into the freed name. Prefer a sibling install.sh
# when this script runs from a checkout/tarball; otherwise fetch it.
echo
echo "Installing the new archi..."
script_dir=''
case "$0" in
  */*) script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd) ;;
esac
if [ -n "$script_dir" ] && [ -f "$script_dir/install.sh" ]; then
  sh "$script_dir/install.sh"
else
  curl -fsSL --proto '=https' --tlsv1.2 "$INSTALLER" | sh
fi

archi_bin="$HOME/.local/bin/archi"
command -v archi >/dev/null 2>&1 && archi_bin=$(command -v archi)

echo
echo "Migration complete."
echo "  new: $("$archi_bin" --version)  ($archi_bin)"
echo "  old: preserved as"
echo "$renamed"
echo
echo "Next, in each fractal project, have your agent migrate it to the new"
echo "Archiplan format: 'old-archi' still reads the old project, and 'archi'"
echo "rebuilds it in the new one. The agent's crossing guide:"
echo "  https://raw.githubusercontent.com/$REPO/main/skills/archi-migrate-fractal.md"
