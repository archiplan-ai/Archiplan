# Multi-repo

Archi assumes the code lives beside the spec in one repository: capture diffs one
tree, the audit's delta source is one commit, every anchor is a bare path under one
root. Real projects are not shaped like that — code is spread across several
repositories, and the spec then lives in a repository of its own, where the tool
today sees no code at all: capture finds nothing to attribute, the audit anchors to
a tree with no code in it, and links cannot even name a file outside the root. The
project's repositories need to be first-class: declared, addressed, scanned and
baselined each in their own right — while a project that keeps everything in one
repository, today's shape, keeps working untouched.
