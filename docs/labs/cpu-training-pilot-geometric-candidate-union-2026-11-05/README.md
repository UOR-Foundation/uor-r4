# Geometric writer/copy candidate union

Deepseek-177 tested whether adding the writer's top 128 non-BOS geometric
candidates to exact-copy history would provide enough supervised next-token
groups to train the integer candidate ranker.

The fresh seed-2768 panel was generated successfully. Artifact seed 1 passed
the registered floors with **2,442** supervised groups and **332** target
tokens. Artifact seed 2 stopped during eligibility with **1,847** groups,
below the registered **2,048** floor.

The registered stop rule therefore classifies the run as
**INSUFFICIENT_TRAINING_DATA**. The shell classifier emitted
`INSTRUMENT_FAULT` because its error-message regex expected a different word
order; the raw output and the post-run classification correction are both
preserved. Seed 2768 is closed for this construction. Because artifact seed 2
stopped before producing a report, the two-seed bar was not evaluated and no
fresh-process repeat ran.

Seed 1 is diagnostic only. Its integer ranker scored **265** correct body
positions versus **601** for longest-32, **61** for the writer and **46** for
fixed geometry-only. It beat the geometry-only and writer controls but failed
the registered longest-32 comparison. No complete-reply, language or serving
claim follows.

Published evidence is in
[`evidence/geometric-candidate-union-2768/`](evidence/geometric-candidate-union-2768/).
Raw corpus payloads remain in the local sealed root; the committed manifest
binds their identities.
