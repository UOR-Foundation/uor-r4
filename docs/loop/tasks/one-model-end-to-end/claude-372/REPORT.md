# REPORT: one-model-end-to-end, claude-372 (CUDA leg of the native-space parity contract)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Order: [ORDER.md](ORDER.md). PR #3963. Follows claude-371 (PR #3962).

The order asked for the CUDA training kernels and the CPU training oracle to start from byte-identical bytes on `wick`'s RTX 4090, and for raw words to be compared at each step of one training transition. On two discrete fixtures, every learned-state word agrees: read vectors, counterfactual loss words, choices, refusals, and the data section of the artifact after one committed step. Two header words do not.

No training run, no corpus, no capability or speed claim.

## What did not work, first

1. **After a committed step, the CPU writes a stale snapshot hash.** CUDA exports the stepped model with a new `source.snapshot_sha256`. The CPU applier's `artifact_bytes` keeps the hash of the bytes it loaded. Every other header value and every data byte agree. A CPU-trained artifact therefore names the snapshot it started from as its source. Pinned in the test: one such pair per fixture.
2. **The two writers produce different bytes for the same model.** At load, CUDA's `export_d11` writes header keys in struct order and the CPU re-save writes them sorted. The values are the same and both loaders accept both files, but "byte-identical" holds only after parsing the header. Pinned: one such pair per fixture.
3. **CUDA's default Flock model has no CPU oracle.** `Device::new_with_read(…, Flock, …)` initializes `zero_gates_v1`. The CPU Flock reader refuses it ("Flock CPU read refuses non-Flock initialization"). The Flock fixture here uses `active_leaf_v1`, as claude-371's did. A CUDA run from the default Flock initialization is not covered by this contract.
4. **One step, two small fixtures.** Vocabulary 3, two blocks, context 3, 39 prefixes, one committed proposal per fixture. Relation mode, position modes and multi-step runs are not exercised. Agreement here does not show agreement at corpus scale.
5. **The wick sync kit misses test inputs.** `wick.sh sync` copies neither `docs/labs` nor `archive/labs`, and `uor-r4-discrete`'s lib tests `include_str!` files from both. The first two runs failed to compile (an instrument fault). The 36 named files were copied by hand into the container. The kit itself is not changed in this unit.

## What changed

- `crates/uor-r4-discrete/src/cpu_oracle_parity.rs` (new, 548 lines, `#[cfg(all(test, feature = "cuda"))]` only). Two tests: `cuda_and_cpu_oracle_agree_word_for_word_from_identical_bytes` and `cuda_parity_catches_one_planted_fault_per_step`.
- `crates/uor-r4-discrete/src/lib.rs`: one test-only module line.
- `crates/uor-r4-discrete/Cargo.toml`, `Cargo.lock`: `uor-r4-cpu-train` as a dev-dependency. It is test-only (not a normal dependency) and only bytes and integer words cross between the crates.
- `docs/research/notebook/cpu-training.md`: one row.

**Why the test lives in `uor-r4-discrete`.** `Device::apply` is `pub(crate)`: committing an update is part of the training hold, and widening it was out of scope. A unit test inside the crate can call it; the CPU oracle comes in as a dev-dependency.

**The start.** `Device::new_with_initialization` builds the model on the GPU (seed `0x5172`). `snapshot` and `export_d11` write `UORLUT01` bytes, and the CPU scorer, chooser and applier load those same bytes. The CPU chooser is handed the CUDA loss words, so a choice difference would be a choose-rule difference, not an inherited loss difference.

## Measurements

| Check | Route (`zero_gates_v1`) | Flock K = 2 (`active_leaf_v1`) |
|---|---|---|
| Read vectors, `discrete_forward` against `CpuDiscreteProposalScorer::query` | 624 of 624 words equal | 624 of 624 equal |
| Loss words, `discrete_counterfactual` against `score().losses` | 160 of 160 equal (10 parameters × 16) | 160 of 160 equal |
| Choice, `discrete_choose` (lane, before, after, epoch, sensitive) against `choose` | 10 of 10 equal | 10 of 10 equal |
| Parameters CUDA refuses, also refused by the CPU | 0 | 4 of 4 (Route parameters in a Flock model) |
| CPU load-then-save of the CUDA export | data equal; header keys reordered | same |
| After one committed step (Route(0); VectorBlock(2)) | committed on both sides; data section byte-equal; epoch 1 on both; `snapshot_sha256` differs | same |

| Planted fault (Route fixture) | Caught by |
|---|---|
| One byte of the CPU's copy changed (a loadable fault that changes a read) | the read-vector check |
| One target moved on the CPU side | the loss-word check |
| The chosen lane swapped into lane 0 of the loss words | the choice check |
| The CPU commits the no-op lane where CUDA committed | the after-step section check |
| One data byte changed in a copy | the section check, as a data difference; a key-order-only re-save is not reported |

**Backend identities.**

| | |
|---|---|
| GPU | NVIDIA GeForce RTX 4090 (24 GB), Casey's `wick`, container `uor-r4-claude` |
| Driver / toolkit | 595.99.02 / nvcc `cuda_12.8.r12.8/compiler.35583870_0`, `UOR_NVCC_ARCH=compute_89`, ahead-of-time PTX |
| `kernel_sha256` | `c08595e3ff0e3c1ef3f80624b728b36812f5dcb8a53c8b91eae3bdadb08a7b4c` |
| `source_sha256` | `7c854830a0442d4b1370b248350aba599abb9c43d808f645ed32548c01f487bb` |
| `geometry_sha256` | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| Test file run | `cpu_oracle_parity.rs` sha256 `beb2ed4e262d728d616442e79827d448eec7556ab2924139512aa9880fea00a5` |

The GPU was shared: another lab's `train_gllm_v10` held 6.8 GB and 100 % utilization throughout. That process was not touched. The test used one device, ran in 4.8 s, and needs no exclusive GPU.

## Checks run at the head

| Check | Result |
|---|---|
| `cargo test -p uor-r4-discrete --features cuda --lib cpu_oracle_parity` on `wick` | 2 passed |
| `cargo check` and `cargo clippy -p uor-r4-discrete --features cuda --lib --tests` on the laptop with a stub nvcc | clean; no clippy finding in the new file |
| `rustfmt --edition 2021 --check` on the new file | clean |
| `cargo test -p uor-r4-discrete --lib driver_source` (laptop, no cuda) | 1 passed; the driver-source pin is unmoved (`lib.rs` and `Cargo.toml` are not hashed sources) |

## Skills used

None.

## Still open

- **The smallest next training dependency:** make the CPU applier's saved header carry the stepped snapshot's hash, as CUDA's export does. Then the after-step bytes agree once the header is parsed, and one writer's key order (or a canonical header) closes item 2. Both are small changes to `uor-r4-cpu-train`'s writer, and the pins in this test say where to look.
- **A CPU oracle for CUDA's default Flock initialization** (`zero_gates_v1`), or a recorded decision that Flock training starts from `active_leaf_v1` only.
- **Wider coverage:** Relation mode, a position mode, and a multi-step sequence (apply, re-propose, apply), on the same contract.
- **The wick kit** should copy the `docs/labs` and `archive/labs` files the crates `include_str!`.
- From claude-371, still open: serving does not expose route choices or the kept list; serving accepts non-zero Route gates the training side refuses; the discrete path has no generation loop or stop state.

## Plain summary

The project trains its discrete model with GPU kernels and keeps a separate CPU program that is meant to compute exactly the same thing. This unit checked that on the real GPU. Both sides loaded the same file, read the same 39 short prompts, scored every possible one-word change, picked the same change, and wrote the same model afterwards. Every number in the model matched.

Two labels in the file's header did not match. The CPU side writes the header in a different order, and after a training step it forgets to update the label that says which model the file came from. Neither changes what the model computes, but the second one would mislabel a CPU-trained model. Fixing it is the next small step.
