# Exact legal-constructor basis diagnosis — 9 October 2026

## Question and fixed scope

The [direct legal-set constructor](../direct-legal-construction-2026-10-09/README.md)
returned `InternalError("Singular matrix")` for both absolute and equivalent
centered encodings. Which solver phase and specific selected basis fail, and is
the rejection an empty stored column or a nonzero pivot below the backend's
threshold? This is a saved-problem execution diagnosis before any numerical
repair. It is not a new training attempt or a model-quality comparison.

[Prospective M2 claim](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6092679883).
Use the unchanged centered constructor, all 1,920 original master coordinates,
20 fractional side encodings, 380 unit-normalized guard derivatives and original
objective. All 1,960 variables, 440 constraints, 15 legal destinations per
coordinate, zero-margin construction target, zero warm start and 4,096 branch-node
bound remain. No coefficient pruning, tolerance change, new gradient, native
scoring, candidate commitment, or held-out use.

The standalone Rust harness includes the production constructor by path. The
backend is pinned `microlp=0.6.0`; observation-only instrumentation applies to a
hash-verified local copy, never the shared Cargo registry. It records exact f64
bits for the selected basis and failed pivot context. The diagnostic patch is
retained research tooling, not a new production or serving dependency.

## Artifact recovery

`prepare.py` verifies every file in the saved #2101 attempt3 manifest and the
SHA256 of each required raw file, then packages their f32 bit patterns without
model arithmetic. Rust performs the same ordered normalization as the production
`protected_joint_vector::unit_rows` function. The original artifact is development
input245, 17 references and 380 original guards; it is not the separate 9/15
conditional artifact or a fresh evaluation.

- Archive: `icloud:UOR-R4/results/codex/codex-protected-joint-20261009.tar`,
  MD5 `467c102adc510a6352951596b969f1be`.
- Saved report SHA256: `b1d27f7b15e65d0aa8ea7e0a995b8bc76979610f97dbcb3055c8bbb434d0ac9b`.
- Saved manifest SHA256: `cdadd7c5cbbbdca2225a71006ab58f70129e8df9e015b92f35b19359028a7167`.
- 1,676 complete manifest files and 384 required raw files verified.
- Bit-container SHA256: `0fe91060f10c0d6107be3565f9ae0746c30e742a00d02935398f6764b84b974d`,
  3,925,770 bytes. The full receipt and original guard map remain with the evidence.

## Interpretation boundary

A small computed pivot localizes a numerical rejection; it is not an exact-rank
proof, condition-number estimate or global infeasibility result. A computed zero
can arise from floating-point elimination. Backend diagnostics do not qualify a
model or weaken the original actual-bit screen and native acceptance gates.
Accepted Source48/Generate64 remains 8/512 complete development replies; the
separate conditional 9/15 artifact remains unchanged. Actual-nine, full512, fresh
and multi-turn qualification are not part of this diagnostic.

## Resources

Admission projected 45 minutes for preparation, compile, replay, review and
protected delivery; two CPU threads, 4 GiB process RAM, at most 3 GiB new temporary
storage and 512 MiB retained evidence. Free space at admission was 40,019,226,624
bytes against the 30 GiB plus 128 MiB floor. The existing shared ledger was
1,459,465,346 ms used / 1,463,400,028 ms allowed; charges continue from
2026-10-10T02:18:26.318584Z. No GPU, pod or paid compute; no new model derivatives
or native grading.
