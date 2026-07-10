# Tradeoff configuration

Before a design is pressed, the operator holds priorities the tool never asks for: what matters
most in the case at hand, and what can be sacrificed for it — scalability traded away for
simplicity and speed of development, say. Today nothing captures that, so every analysis weighs the
landscape the same way regardless of what the project is actually optimizing for, and the scoring
read is a general verdict where a situated one was wanted. A trade-off configuration, set up front,
should let the operator declare what to favor and what to spend — with an auto mode where the agent
polls the operator and derives a suitable configuration itself rather than demanding it cold. Its
natural consumers are the scoring layer, which would weight the landscape read by it, and
stress-round prioritization, which would press what the configuration says matters first.
