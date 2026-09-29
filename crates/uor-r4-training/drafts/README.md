# Track B drafts: not registered or executed

These sources preserve independent B2 engineering while checkpoint parity is
pending. Neither file is registered in the crate, compiled, or qualified by the
earlier loader tests. No token corpus, fitted operator or model result exists
from these drafts.

- `track_b_transfer.rs`: full-width frozen Q/K/V plus paired rank-4 LoRA;
  dense control and harmonic polynomial attention; separate detached feature
  recurrence with explicit state limits. Training currently materializes a
  quadratic Gram matrix. Optimizer, flock/hybrid integration and fit driver are
  unfinished.
- `track_b_source_data.rs`: bounded, two-pass source preparation using the
  original tokenizer, document-isolated splits and exact-text duplicate
  exclusion. Its output is UORT v1, with a 64-byte header. Consumers must use
  `MmapCorpusReader` and payload-token offsets from the windows manifest;
  the old headerless training reader is incompatible.

Both files were recovered into this internal worktree after the owner's
September 29 workspace-image restoration. The original data draft was recovered
byte-for-byte at SHA-256
`93458b4718be3919848567ca4ff1379fa9929ae64c838efe453617d8766c9866`, then its
report-metadata reserve and non-UTF8 argument handling were corrected after
independent review. The transfer draft was restored from recorded successful
patches at SHA-256
`ad3936b6fcaa9b9714ac1f45c2d4700638fe131cebc9c42c9673618408916cf5`;
no prior external checksum survives for an independent byte comparison.

Register and compile these only as part of the next declared B2 implementation
step. Fitting remains gated on the shared model's unchanged exact-reference
parity test. Authored tests and static review do not substitute for execution.
