# World facts

The spec says what the system must do and why the architecture chose it. It
never says which outside condition made the behavior necessary. That claim
hides inside intent prose, where no test reaches it and no verb ages it:
"work wants to run as parallel units" stands in an intent forever, because
nothing in the repository can make it false. Two consequences follow. The
spec only grows — a requirement retires when a person calls it surplus,
never because the situation that demanded it has gone. And one class of
error stays unsayable: the system is correct, the check is green, and the
whole thing points at a situation that no longer exists.

A second world records those conditions as their own objects under
`archi/world/`. One object is one fact about the world, the scenarios that
fact dictates, and the statement of what would make it false. The header
points three ways: down to the nodes the fact covers, out to the raw
material it rests on, and sideways to the facts it presumes. A verb mints
the skeleton and a person writes the prose, as with requirements and plans.
`archi check` holds the shape, resolves every reference, and reports what
stands in the air when a fact goes. The world checks form. It never checks
truth: no command can tell whether a fact about the world still holds.
