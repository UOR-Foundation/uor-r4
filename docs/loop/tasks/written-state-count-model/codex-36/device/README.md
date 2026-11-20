# D36 CUDA reader for UORKNC02

`lookup.cuh` exposes the batched CUDA entrypoint for callers with a resident,
immutable compact artifact. `device.cu` is the finite AOT launcher. The
standalone Rust witness prepares only metadata and probes, then compares
result words after the device process completes.

All tape words are little-endian. One order descriptor has `u64 keys`,
`u64 rows`, `u32 contexts`, `u32 followers` and three `u64` Q48 discounts.
The eleven descriptors take 528 bytes. Each 24-byte probe contains a `u16`
length followed by eleven `u16` token slots. History is oldest first, the
target follows it, and unused slots are zero. Each probe returns eleven
32-byte order records, each with `u32 raw`, `u32 adjusted`, `u64 raw_total`,
`u64 adjusted_total` and `u64 q48`. The kernel API also returns one `u32`
status per probe. The launcher requires all statuses to be zero before
writing the records. The Q48 field floors zero to one for the exposed order.
The internal chained mass is carried without injecting that floor.

From the Wick source root, build the locked witness and the SASS-only
executable for Wick's GPU. Scan the compiled image and linked libraries as
in the D31 dispatch before sealing it.

```sh
unit=docs/loop/tasks/written-state-count-model/codex-36/device
CARGO_TARGET_DIR="$stage/target" cargo build --locked --offline --release \
  --manifest-path "$unit/Cargo.toml"
/usr/local/cuda/bin/nvcc -O3 -std=c++17 -I crates/uor-r4-discrete/src \
  -gencode arch=compute_89,code=sm_89 "$unit/device.cu" -o "$stage/d36-count-device"
"$stage/target/release/d36-device-witness" prepare "$witness" "$counts" "$packet"
```

For the reviewed all-retained, 8 MiB and exact-floor arms, stage packet
directories named `packet-all`, `packet-8m` and `packet-floor` with their
corresponding `prod-c128-*.json` witness inputs. The dispatch verifies exact
hashes and expects the pruner's final artifacts in `../pruner/final`.
The stage holds `build-and-input.sha256`, `source.sha256`, `FIT.md`,
`GO.json` and `GO.sha256`. The parent GO fields are
`build_and_input_sha256`, `source_sha256`, `fit_sha256` and `device_uuid`.
Run `dispatch.sh STAGE NEW-OUTPUT-DIR` after the [finite sheet](SHEET.md)'s
review and device admission.
