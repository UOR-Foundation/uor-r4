# Track B: Metal harmonic floor failure and reduction diagnosis

Status: measured feature-contraction limitation; no checkpoint or language result.

On September 30, source `dcb8b107f499395c65b86404d67e9e83916054fd` compiled with
Metal enabled in an internal cache. The unchanged
`distributed_antipodal_scores_preserve_measured_floor_metal` test failed at
dimension 32, degree 3, perturbation 0.01: score
`1.5497062122449279e-6`, expected `1.0000000117348499e-6`. The fixed criterion is
absolute error at most `5e-7 + 2e-5 * (expected - 1e-6)`. Neither the kernel,
delta, tolerance nor original test was changed. CPU had passed this fixture.

The succeeding ignored diagnostic adds no production behavior and does not
replace the failed gate. Its source is the parent above plus patch SHA256
`4e1b1f497966b4ceabf41a598fe8021ae2b678ceb0f09a3644b94c016ee111a6`.
Both feature constructors receive the same Metal-normalized F32 input bytes;
there is no second normalization. Each produces 6,512 features per row.

| Feature construction | Host F64 dot of exported F32 features | CPU matrix product | Metal matrix product | Metal elementwise product then sum |
|---|---:|---:|---:|---:|
| CPU | 9.537330709852786e-7 | 1.009088009595871e-6 | 1.550938691252668e-6 | 9.611248970031738e-7 |
| Metal | 9.538933082305312e-7 | 1.0086223483085632e-6 | 1.5497062122449279e-6 | 9.611248970031738e-7 |

For Metal-produced features, the final Metal matrix product differs from the
host F64 dot by approximately `5.95812904e-7`. The host-F64 dot changes by only
`1.60237245e-10` between CPU- and Metal-produced features. This localizes the
dominant discrepancy in this fixture to the final Metal contraction, rather
than feature construction. Elementwise product plus reduction is a different
F32 computation and falls inside the existing bound here; this is not a
qualification of a replacement operator or all possible input rows.

The existing `largest_harmonic_arm_backpropagates_on_metal` test also passed,
including its finite-difference comparison. This verifies that separate tiny
gradient fixture, not the failed floor criterion.

## Consequence

Retain the original failure. Do not tune delta, weaken the tolerance, silently
switch the acceptance test to another contraction, or claim universal numerical
positivity. The unregistered B2 transfer draft uses matrix contractions in its
quadratic and recurrent paths; the actual integrated contraction must satisfy
its declared numerical checks when B2 becomes eligible. A small stable helper
alone would not qualify the recurrence or its accumulated state.

Continue the independent dense checkpoint CPU/Metal all-logit `1e-4` parity
check: it does not use these harmonic features. B2 fitting remains unqualified.
This result does not reject geometry, establish language quality, or qualify
integer serving.

## Execution and review

- Metal build plus original floor test: exit 101; 77.09 seconds total, 1,294,893,056 bytes maximum process RSS reported by `time -l`.
- Cross-backend diagnostic: completed; 36.03 seconds total, 1,153,859,584 bytes maximum process RSS. Its passing test-harness exit means the diagnostic ran, not that the original gate passed.
- Largest-arm Metal gradient check: pass; 0.87 seconds total, 84,443,136 bytes maximum process RSS.
- CPU evidence before this diagnostic: 17 registered Track B tests and 3 parity-gate tests passed at the parent source. No model checkpoint was loaded by these checks.
- An independent source/evidence reader verified the diagnostic's identical-input comparison and attribution, and recommended preserving the failed gate while continuing independent dense parity.

Raw logs and source identities are retained in the operational evidence branch
and linked from PR #1518. Builds used
`CARGO_TARGET_DIR=/Users/casey.allard/.cache/uor-r4-track-b-direct/target`;
no external build cache, quarantine change, or mount change was used.
