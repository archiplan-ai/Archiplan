# Agent retrieval

The knowledge base grows past what an agent can read whole: dozens of requirements,
stress rounds, model elements with their identity prose. An agent that wants "where
does this repository talk about folding?" today greps blind — misses the model
(definitions live in the compiled graph, not on disk as prose), misses the doc schema
(slugs, origins, affects), and gets file dumps instead of addresses it can act on.
One retrieval verb should take a natural-language phrase and answer with a short
ranked list drawn from every archi object — model definitions, intents, requirements,
stressors, sessions — each hit addressed by its slug or path so the next verb
(`archi query`, `archi incidence`, the editor) starts exactly there. Archi stays a
CLI: no service, no index to keep alive, no second copy of the truth.
