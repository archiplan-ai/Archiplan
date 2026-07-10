# Planning

A hardened spec says what must be true; it says nothing about how the work is cut. Execution
left to itself grows a shadow copy — task lists that re-type requirements and spec references by
hand, correct the day they are written and quietly wrong after the next save. A plan closes the
gap by projection instead of transcription: it targets an intent or a set of requirements not
yet implemented — ones without asserted code-links — pins the hardened version it projects, and
derives the executable task graph from it: each task pinned to one node in one scope, its spec
references, requirements, inputs and outputs pulled from the spec rather than invented, the work
then carried through waves whose close is gated on the traceability it just captured. Keeping
plan and spec apart lets the spec stay about architecture and the plan stay about execution;
deriving instead of retyping removes a whole class of drift — when the spec changes,
`plan verify` flags every task whose obligations no longer hold.
