Archiplan — release tarball
---------------------------

To install: move the archi binary onto $PATH.

Example:

  install -m 755 ./archi "$HOME/.local/bin/archi"

Windows: copy archi.exe into a directory on %PATH%.

Then open your coding agent in a project and run /archi — or start
by hand with 'archi init'.

Most users should use the scripted installer instead (install.sh or
install.ps1, bundled in this archive); this manual copy is the fallback.
It pulls the newest tarball from the GitHub releases of
archiplan-ai/Archiplan and checks it against the .sha256 published
beside it — the same checksum file that sits next to this tarball on
https://github.com/archiplan-ai/Archiplan/releases.
