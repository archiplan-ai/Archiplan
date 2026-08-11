# The words in the code are not the words in the design

A design and the code that realizes it are written by the same people, at different times,
for different readers. The design names a thing for what it is to the reader of the design.
The code names it for what it is to the person who will next open that file. The two names
are usually not the same word, and nobody notices, because a person reading both supplies
the connection without effort.

One kind of project hides this. A tool that models itself writes the design and the code
in one vocabulary, because there is only one subject and one set of names for it. What is
`Archive.serve` in the design is a function called `serve` in the code. The words line up
by construction, not by care.

That is the exception and not the rule. In a product built for somebody else, the design
speaks of the trade the product serves and the code speaks of the machinery that serves it:
one says *baseline*, the other says *fork point*; one says *serve a pinned render*, the
other says *load*. Both are correct for their reader. Neither is careless.

Anything that pairs a design with its code by shared words will therefore work well on the
project where it was built, and go quiet on the projects it was built for. Quiet is the
dangerous state, because it looks exactly like agreement.

This was seen from inside a self-modeling tool, which is the easy case, and it has not been
watched on a project where the two vocabularies actually diverge. That is the observation
and the whole of it.
