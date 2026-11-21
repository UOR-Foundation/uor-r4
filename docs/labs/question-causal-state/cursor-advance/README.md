# Cursor-read causal diagnostic

This is a diagnostic patch against source revision `6b6e28cb88829a7944467ebc7bfeddc57e8f4f26`, not a model change. Apply only to a separate full source snapshot; do not patch a shared checkout or sealed source. The snapshot used on Wick started from `/mnt/plexpool/uor-r4-corpus/runs/20261121-orderfree-trunk/integrated-client-v1/source`. Before patching, `crates/uor-r4-discrete/src/conjunction.rs` had SHA-256 `d0bfd18f2ca06cb24a5f105b258a3d3a4d1d978077eedeb2036a11b9bf965b84`.

Copy this directory into the snapshot at `docs/labs/question-causal-state/cursor-advance`, then from the snapshot root:

```sh
patch -p1 < docs/labs/question-causal-state/cursor-advance/conjunction.patch
patch -p1 < docs/labs/question-causal-state/cursor-advance/free.patch
CARGO_TARGET_DIR="$OWNED_RUN/target" cargo test --manifest-path crates/uor-r4-discrete/reader-runtime/Cargo.toml cursor_diagnostic_tests
CARGO_TARGET_DIR="$OWNED_RUN/target" cargo build --release --locked --manifest-path docs/labs/question-causal-state/cursor-advance/Cargo.toml
```

Use an isolated `OWNED_RUN` and a separate target directory. The Wick release built the same Rust sources with a tool manifest outside the snapshot whose dependencies point into the snapshot. Its executable SHA-256 is `ea6b766ccbb0723a120f7390d095c192d70cd5a69bc1db55b851756f6db15380`; the differing manifest location means a rebuild need not have the same binary digest. The frozen artifact and reader hashes are checked by the executable. No training or weight changes are made.

`supplied-form-trace ARTIFACT predict INPUT` consumes `{job_form:1223,home_form:1553,rows:[{id,role,prompt,ordinary_reply,job_value_start}]}`. `job_value_start` is the supplied original first-user job-value token position, needed only for job rows. It is bound before the first reply; it is not inferred from answer IDs. The ordinary reply is checked, and promoted form precedence retains the other lexical forms in order. Output contains `lexical`, `promoted`, `current`, and `original` arms, each with full 23-word steps, replies, and retry flags. Home arms copy the promoted baseline. For jobs, at the first selected concrete source equal to the entering cursor, the current arm substitutes the source successor, while the original arm substitutes supplied original start plus one, even if copy progress has advanced. Each substitution occurs before native state and stop updates. A retry restarts the one-shot and logs the discarded pass separately. Invalid successors do not apply and still consume that pass's trigger. `control` uses the same input without job addresses and performs only baseline replays.

The historical misbound fixture `dual-p1-a1-o1-job-familiar` used original start 24, encountered reread source 52, and substituted positions 53 and 25 respectively. The latter continuation stopped without EOS. These interventions use supplied role and address information and are causal diagnostics, not an autonomous selector or product capability.

For the supplemental finite equality control, take a private copy of the patched source, apply `same-token-control.patch` there with `patch -p1`, and repeat the `cursor_diagnostic_tests` command in a separate target. This adds a test only. It forces two different positions holding the same replacement ID, checks that both addresses survive selection, and verifies their successors differ. The ordinary predictions above were made with the original release build.
