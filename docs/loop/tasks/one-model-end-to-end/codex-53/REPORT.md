# REPORT: one-model-end-to-end, codex-53 (the first non-default-shape artifact exports and serves)

Lab: codex. A 30 M geometric model — **8,192 vocabulary, 512 width, 12 layers, 512 context** —
trained on the donor corpus, exported through the integer path, and served. No format, container or
serving change was needed.

## The numbers

| step | result |
|---|---|
| `fbs-snapshot` on the step-8,192 checkpoint | snapshot `d79a0287…` |
| `fbs-export --legacy-artifact` | artifact `fca74215…`, **8,558,848 B (8.2 MiB)** |
| `uor-chat --stack … --prompt-ids …` | serves; 24 tokens, **25.5 tok/s in a debug build** |

The size matches the export investigation's estimate (token table 8 MiB + routes ~21 KiB + gains
~135 KiB). The 25.5 tok/s is a **debug** binary and not a serving claim; the release figure is the
one to report.

## What did not work, first

**`fbs-snapshot` refused with "shared native vector head artifact identity missing or stale."**
The cause was not the model: `NativeVectorWriter`'s **own source hash** is part of the model's saved
binding, and the `fbs-snapshot` binary predated the edit that added the centroid address to that
file, so it recomputed an old hash and called the model stale. Rebuilt with the current source, the
same checkpoint snapshots and exports.

**Lesson:** the writer's source hash is model identity. Any change to
`native_vector_writer.rs` requires rebuilding every tool that loads a checkpoint before those tools
can read a model trained after the change — and a model trained before it can no longer be exported
by the new tools.

## What this unlocks

The instruction panel can now be scored on a real artifact of this shape. It should not be scored
yet: at 0.22 epochs the writer picks the target in **0.3 %** of rows, so the panel would measure
admission, not the model. It is scored after the writer's admission unit (`codex-52`) or on the
long run's final checkpoint if admission moves with it.

## Checks

Artifact parsed and served by `uor-chat`; export reproducible from the snapshot. The panel file
(`instr-panel-2026-11-17.json`, 196 items) and its grader are ready.
