# INVENTORY: native-space state and operators (claude-371)

Lab: claude. Code read at `675f5eb43b99e5ee634cd45b5fd517f69a638c52`. Every row names the file and line it was read from.

## Two served paths, and which one this card is about

| Path | Training | Served artifact | Meets "one state transition"? |
|---|---|---|---|
| **Discrete SpiralCore/FBS** (`lut-discrete`) | `crates/uor-r4-discrete/src/discrete.cu` (CUDA, nvcc AOT) and `crates/uor-r4-cpu-train/src/discrete_route.rs` (CPU, D84) | `UORLUT01` `uor-r4.lut-discrete/1`, four tables | **Yes for its learned state.** The served tables are the training words, copied unchanged (`d11_export.rs:286-330`). It has no generation loop or stop state (below). |
| **Geometric stack** (`lut-stack`, what `uor-chat` talks with) | `crates/uor-r4-float-train` (F32 master, BF16/F32 CUDA) | `uor-r4.lut-stack/1..9`, 4-bit groups with a scale byte (`stack/format.rs:2-11`) | **No.** The served words are a quantization of a different F32 master. This is the export gap the hand-off excludes. |

So the native path to build on is the discrete one. The stack path is listed at the end only where the hand-off names an area that exists there alone.

## Shared constants (both sides compile the same tables)

| Constant | CUDA training | Integer serving | Width | Check |
|---|---|---|---|---|
| 16 orbit roots, `Cl(0,6)` anchor orbit | `g[ROOT+r*8+d]`, `discrete.cu:7` | `ROOTS: [[i8;8];16]`, `discrete_d11.rs:227` | i8 coordinates, held as u32 on device and i32 in use | root coordinates are in {-2,0,2} (test fixtures `discrete_d11.rs:1830-1835`) |
| 64 signed-axis actions | `g[AXIS+a*8+d]`, `discrete.cu:8,20-22` | `AXES: [u8;512]`, `discrete_d11.rs:35` | u8: low 3 bits = source axis, bit 3 = sign | CUDA tests `axis&8`, serving tests `axis&8` (`discrete_d11.rs:715`). Same mask. |
| 64×64 composition | `g[COMPOSE+a*64+b]`, `discrete.cu:19` | `COMPOSE: [u8;4096]`, `discrete_d11.rs:57,721` | u8 action index < 64 | |
| BFS, BRANCH, GENERATOR (15), INVERSE, IDENTITY | `discrete.cu:11-14` | `BFS`, `BRANCH`, `IDENTITY=0`, `discrete_d11.rs:246,259,278` | u8 | |
| Geometry identity | `operator_kappa`, `composition_kappa` in the snapshot | `OPERATOR_KAPPA`, `COMPOSITION_KAPPA`, `discrete_d11.rs:279-281`, checked at load (`:940`) | string | a mismatch refuses the load |

## The card

Columns: **Field / operator** · **Domain, width, scale** · **Lifetime** · **CUDA training** · **Integer serving** · **CPU oracle** · **WASM** · **Parity evidence now** · **Gap**.

### 1. Token → address → vector

| Field / operator | Domain, width, scale | Lifetime | CUDA training | Integer serving | CPU oracle | WASM | Parity evidence now | Gap |
|---|---|---|---|---|---|---|---|---|
| Token id | u32 < vocab (2..=8192), refused at or above vocab | prompt | `U* ids`, `discrete.cu:29-34` | `evaluate_prefix` refuses `tok >= vocab`, `discrete_d11.rs:1633` | `CpuDiscreteRoute::read`, `discrete_route.rs:220` | `wasm_discrete_predict(ids)`, `wasm_serve.rs:305` | same ids into both reads (`export_parity`, `discrete_route.rs:1556`) | none |
| Codebook block (learned state) | i32 ×8 per (token, block); every block must be one of the 16 roots; blocks 2..=16 | model | `int* table`, index `(token*blocks+block)*8`, `discrete.cu:24-28` | `token_blocks`, loaded from `discrete.table` (I32 LE), `discrete_d11.rs:1614` | same table bytes | same loader | `export_parity` read-vector words; `discrete_validate` (`discrete.cu:524`) and export (`d11_export.rs:124-150`) refuse a non-root block | none for the value. The i32 width carries 4 bits of information (16 roots). Serving can produce packed roots (`packed_codebook_roots`, `discrete_d11.rs:1310`), but the artifact stores i32; a lossless 4-bit stored format is allowed and not built. |
| Token cell (memory address) | u32 = root(block 0)·16 + root(last block), < 256; ABSENT `0xffffffff` if a block is not a root | derived from the table | `token_cell`, `discrete.cu:340-344`, recomputed from the live table at every writer call, including virtual moves | `discrete.cells` table, written by export (`d11_export.rs:194-210`), read as stored (`discrete_d11.rs:1306`) | computed from the table | stored | `export_parity` admitted-cell words; the serving loader refuses a stored cell that disagrees with the table's endpoint roots (`discrete_d11.rs:1115-1118`) | none: the stored copy is redundant and checked at load |

### 2. SpiralCore/FBS carry and transport

| Field / operator | Domain, width, scale | Lifetime | CUDA training | Integer serving | CPU oracle | WASM | Parity evidence now | Gap |
|---|---|---|---|---|---|---|---|---|
| `action(a, x)` | x: i32×8; a < 64; `y[d] = ±x[axis&7]` | per step | `discrete.cu:20-22` | `discrete_d11.rs:710-719` | yes | yes | indirect, through read words | none: the same mask and sign rule |
| `compose(a, b)` | u8 table lookup | per step | `discrete.cu:19` | `discrete_d11.rs:721-723` | yes | yes | indirect | none |
| Route port (learned state) | u32 < 64 per (block, stage); stages from `expected_stages(context)` (3, 9, 27, … up to 243) | model | `U* routes`, `discrete_init` sets IDENTITY (`discrete.cu:429`) | `discrete.routes` (U32), `discrete_d11.rs:1317-1379` | yes | yes | read-vector words only | **The port chosen at each stage is not exposed by serving.** A route fault is seen only if it changes the read words (contract C-1). |
| Route read (`route_vector`) | staged port choice by `root_cost`; result i32×8 per block | per prefix | `discrete.cu:41-94` | `route_read`, `discrete_d11.rs:1317` | `discrete_route.rs:220` | yes | `export_parity` and the public DEV panel (`cpu-public-discrete-parity.rs`: 14,287 rows, 0 query mismatches required) | none on the words; no CUDA leg |
| Position transport `s[14]` | 0 BFS, 1 prefix holonomy, 2 matched, 3 identity | spec | `discrete.cu:133-152` | **absent** | absent | absent | export refuses `position.is_some()` (`d11_export.rs:235-238`) | training-only modes; the served path is mode 0 only |
| Route bypass `s[13]` | 0 or 1 | spec | `route_vector`, `discrete.cu:44-45` (bypass reads the last input block); spec comment `:83`; checked in `discrete_validate` `:534` | absent | absent | absent | export refuses `RouteBypassV1` and `RoutePermutedActionsV1` (`d11_export.rs:254-262`) | control arms only |
| State-transport knockout | bool | serving only | absent | `with_state_transport_knockout`, `discrete_d11.rs:1267` | absent | absent | none | a serving-side ablation with no training counterpart; never shipped |

### 3. Angular/Hopf routing; icosian or orbit-root snapping; E8/H4/root selection

| Field / operator | Domain, width, scale | Lifetime | CUDA training | Integer serving | CPU oracle | WASM | Parity evidence now | Gap |
|---|---|---|---|---|---|---|---|---|
| `root_cost(x, r)` | (L1 u32, Hamming u32); L1 then Hamming, then the lower port | per step | `discrete.cu:37-40`; `magnitude(x-y)` in **signed int** | `discrete_d11.rs:725-734`; `(x-y).unsigned_abs()` | yes | yes | indirect | **Wrap rule differs at the edge:** signed `int` overflow in CUDA's subtraction is undefined behaviour (PTX wraps in practice); Rust's panics in a debug build and wraps in release. The u32 distance sum has the same split. Unreachable while every coordinate is a sum of at most a few roots; no range check pins it. |
| `snap_n(x, n)` | argmin over 16 roots of (L1(x − n·r), Hamming); n·r built by repeated addition, never a multiply | per flock node | `discrete.cu:108-116`, n = 3 by default, 1 under `s[12]=1` | `discrete_d11.rs:736-759`. The four-table read uses n = 3 only (`:1429`). The 19-table trunk artifact (`lut-discrete-trunk/1`) also calls n = 4 in `compute_rec_code` (`:615`) and n = 1 in `apply_joint_pointer_gate` (`:658`). Both run only on that artifact (`:1645`), which has no CUDA counterpart; its count tables are trained on the CPU (`cpu-trunk-count-driver`). `orbit_snap` (`:855`) is a crate accessor. | yes | yes | indirect | **Start values differ, outputs do not:** CUDA starts `best = ABSENT`, serving `best = 0`. On CUDA, root 0 always beats the start, because its Hamming count of at most 8 is below `ABSENT`. So both return the same root for every input short of u32 overflow in the distance sum. Snap scales 1 and 2 on the four-table path are refused by export (`d11_export.rs:229-233`). **The trunk's n = 4 and n = 1 snaps have no CUDA training implementation.** |
| `root_id` | exact 8-word match, or ABSENT | per check | `discrete.cu:336-339` | `discrete_d11.rs:706-708` | yes | yes | indirect | none |
| Hopf, icosian, H4, E8-240 selection | — | — | **not on this path.** The 16 roots are a `Cl(0,6)` orbit (`spiralcore_operator.rs:1-16`, which also says it does not identify E8 with H4 or the Hopf S2). E8/H4 work lives on the float stack (claude-346's E8 address memory) | — | — | — | — | **The native path has no E8/H4/Hopf operator at all.** Bringing one in is new mechanism, not parity. |

### 4. Hamming, copy and exact-memory addresses and reads

| Field / operator | Domain, width, scale | Lifetime | CUDA training | Integer serving | CPU oracle | WASM | Parity evidence now | Gap |
|---|---|---|---|---|---|---|---|---|
| Gate trit (learned state) | i32 ∈ {-1, 0, +1}; count `blocks·stages` (+ `5·stages` in Relation) | model | `int* gates`, `gate_count`, `discrete.cu:95-107` | `discrete.gates` (I32) | yes | yes | export refuses a non-trit (`d11_export.rs:185-193`) | none |
| Flock read: kept K ≤ 8 by (rel desc, distance asc, Hamming asc, recency desc, node asc); consensus `snap_3` of the carried sum; gates add or subtract | i32 sums; K ≤ 8; sweeps ≤ 5 | per prefix | `flock_vector`, `discrete.cu:154-323`, writes `trace` | `flock_or_relation_read`, `discrete_d11.rs:1380-1607` | yes | yes | read-vector words; `serialized_flock_matches_integer_discrete_model` and the relation tests (`discrete_route.rs:4671-5138`) | **The kept list (the memory read choice) is not exposed by serving.** CUDA writes it to `trace`; serving returns only the read vector and `relation_scores`. |
| Writer admission (memory address read) | first `kept_cells` (1..=16) cells by (outer two-block L1, cell id) | per prefix | `writer`, `discrete.cu:375-418`, metric `s[15]` 0/1/2 | `evaluate_prefix`, `discrete_d11.rs:1650-1670`, outer metric only | yes | yes | `export_parity` admitted-cell words | metrics 1 and 2 are refused by export (`d11_export.rs:240-243`) |
| Token choice inside admitted cells | argmin token L1, ties to the lower id | per prefix | `discrete.cu:392-397` (`best_token`) | top-8 insertion with the same tie rule, `discrete_d11.rs:1672-1700` | yes | yes | `export_parity` top-8 and token words | none on the words |
| Hamming copy store, turn memory, point key | — | — | not on this path | `stack/hamming.rs`, `stack/memory.rs`, `stack/point_key.rs` (lut-stack only) | — | — | — | **The native path has no exact copy or turn memory.** The talking model's memory is on the F32-master path. |

### 5. Writer, readout and emission

| Field / operator | Domain, width, scale | Lifetime | CUDA training | Integer serving | CPU oracle | WASM | Parity evidence now | Gap |
|---|---|---|---|---|---|---|---|---|
| Token hinge `tm = max(0, width + td − rival)` | u64; width = 8·blocks | per row | `discrete.cu:401` | `observe_target`, `discrete_d11.rs:1771-1775` | `writer_observation`, `discrete_route.rs:3791` | absent | `proposal_writer_baseline_matches_host_for_both_objectives` (`discrete_route.rs:5414`) | none |
| Address hinge `am = max(0, 8 + ad − rival)`, rank hinge `rm = max(0, rank − kept)` | u64 | per row | `discrete.cu:402-409` | `discrete_d11.rs:1776-1796` | yes | absent | same | none |
| Loss: TA = `8·tm + width·am`, TR = `8·tm + width·rm` by shift-add | u64; overflow is status 12 on CUDA and an `Err` on serving | per row | `checked_scaled_add`, `discrete.cu:346-354`; overflow → `~0` and status 12 | `checked_scaled_add`, `discrete_d11.rs:761-777`; overflow → `None` → `Err` | yes | absent | same | none: both refuse, with different carriers |
| Emitted id | u32 = top-1 of top-8 | per prefix | `discrete_evaluate` global top-8, `discrete.cu:543` | `prediction`, `discrete_d11.rs:1707` | yes | `wasm_discrete_predict` | `export_parity` token words; public panel 0 mismatches | none for one step |
| **Generation loop and stop state** | — | — | **absent** | **absent**: `uor-chat --discrete` is one-shot `--prompt-ids` only; text chat and REPL are refused (`uor-chat.rs:960-970`) | absent | absent | none | **The native path cannot answer a turn.** No stop rule, no cycle check, no maximum on new tokens. The stack path has `Stop::{ShortCycle, MaximumNewTokens}` (`uor-chat.rs:1483-1511`). |

### 6. Artifact serialization

| Field / operator | Domain, width, scale | Lifetime | CUDA training | Integer serving | CPU oracle | WASM | Parity evidence now | Gap |
|---|---|---|---|---|---|---|---|---|
| Container | `UORLUT01` magic, u64 LE header length, JSON header, zero padding to 64, tables LE | file | `Snapshot` JSON (`snapshot.rs:15-41`) → `export_d11` (`d11_export.rs:227-345`) | `from_bytes`, `discrete_d11.rs:911-1257`; refuses bad magic, header length, padding and table spans | `CpuDiscreteProposalApplier::artifact_bytes`, `discrete_route.rs:2671` (writes) | same loader | `cpu_discrete_writeback_round_trips_serving_and_advances_epoch` (`discrete_route.rs:4600`); `loader_and_evaluator_refuse_corrupted_containers…` (`discrete_d11.rs:2083`) | **Serving has no save.** Load-then-save parity is checked on the CPU applier only. |
| Shape and provenance | vocab, blocks, context, kept cells, stages, read mode, objective, init, seed, epoch u64; seven provenance strings | file | snapshot fields | `DiscreteLutShape`, `DiscreteLutSource`, `discrete_d11.rs:301-327` | same | same | the loader checks consistency (`:951-959`) | spec words `s[12]`–`s[15]` have no header field; export refuses their non-default values instead |

### 7. Training update and optimizer state

| Field / operator | Domain, width, scale | Lifetime | CUDA training | Integer serving | CPU oracle | WASM | Parity evidence now | Gap |
|---|---|---|---|---|---|---|---|---|
| Init | xorshift `seed ^ i/8 ^ 0x9e3779b9` (the global block index, token·blocks + block) picks a root per block; routes IDENTITY; gates 0 | once | `discrete_init`, `discrete.cu:421-432` | n/a | starts from an artifact | n/a | none | **No CPU oracle of `discrete_init`.** A CUDA-run start cannot yet be checked against a CPU start from the same seed. |
| Counterfactual | 16 candidate moves (3 for a gate), each a virtual move applied inside the read; summed u64 loss over rows | per proposal | `discrete_counterfactual`, `discrete.cu:456-479` | n/a | `CpuDiscreteProposalScorer::score`, `discrete_route.rs:1299` | n/a | `proposal_virtual_moves_match_serialized_host_states` (`:5510`), CPU against CPU | **No CUDA-to-CPU loss-word comparison on the same rows.** The CPU tests copy CUDA's rules by hand. |
| Choose | strict improvement; a tie with the current candidate keeps it; a tie between two improving candidates goes to the lower resulting port; a gate tie keeps the lower ordinal; `sensitive` flag | per proposal | `discrete_choose`, `discrete.cu:485-502` | n/a | `CpuDiscreteProposalChooser::choose`, `discrete_route.rs:987` | n/a | `route_and_table_ties_choose_the_lower_resulting_port` (`:5266`), `gate_chooser_uses_strict_improvement…` (`:5229`) | same: CPU against hand-copied rules |
| Apply | composes a generator into a route, acts on a table block, or changes a trit; refuses a stale epoch (3), a bad index (4), a non-improvement or full epoch (5), or a mode mismatch (9) | per proposal | `discrete_apply`, `discrete.cu:503-523` | n/a | `CpuDiscreteProposalApplier::apply`, `discrete_route.rs:2452` | n/a | `apply_status_order_and_noop_match_cuda` (`:5886`), `exact_gate_route_and_table_applies_mutate_only_the_coordinate` (`:5753`) | same |
| Optimizer state | epoch u64, incremented once per applied move, refused at `~0`; proposal words (best, current loss, selected loss, epoch, sensitive) as u64 | run | `W* epoch`, `W* proposal`, `discrete.cu:500,503-523` | `shape.epoch` (read only) | `CpuDiscreteCheckpoint`, `discrete_route.rs:3231`, with schedule cursor | n/a | `cpu_discrete_checkpoint_restart_continues_identically` (`:6286`) | none: there is no float optimizer state on this path at all |
| Gradients | none: every update is an exact discrete move | — | — | — | — | — | — | none (by design) |
| Evaluation-only `carrier_select` | splitmix-style `carrier_mix` with 64-bit multiplies | evaluation | `discrete.cu:661-718` | n/a | n/a | n/a | — | a sampling helper only, never on the served path (integer multiply is allowed since D80) |

## Backend identities

- **CUDA:** `nvcc -ptx -std=c++17 -arch=$UOR_NVCC_ARCH` (it must be `compute_<digits>`; `build.rs:375-391`). The PTX is checked against an entry inventory and a list of banned instructions (`build.rs:43-130`; `tf32` and `ex2.approx` are banned). Exact cubins are loaded with no JIT fallback (`device.rs:4-5,31-35`). Identity hashes: PTX SHA-256, and a source hash over `device.rs`, `snapshot.rs`, `discrete.cu`, `build.rs` and others (`device.rs:479-500`). For `wick`'s RTX 4090 the architecture is `compute_89`.
- **Integer serving:** `crates/uor-r4-integer/src/discrete_d11.rs`, with schema `uor-r4.lut-discrete/1` and snapshot schema `uor-r4.discrete-spiralcore-fbs/2`.
- **CPU oracle:** `crates/uor-r4-cpu-train/src/discrete_route.rs` (D84 development evidence, not GPU qualification).
- **WASM:** `crates/uor-r4-integer/src/wasm_serve.rs:261-340`, the same `IntegerDiscreteModel` (load, predict, info).

## What the card says, in one place

1. **The discrete path already is one representation from training to serving.** The table, routes, gates and epoch are the same i32/u32/u64 words in CUDA, on the CPU and in the artifact. Export copies them; it does not quantize.
2. **It cannot answer.** It has no generation loop, no stop state, no turn memory and no copy store. Everything that makes the talking model talk is on the F32-master stack path.
3. **The parity that exists is CPU against serving**, on read words, cells, top-8 and the token. Nothing compares a CUDA run with the CPU or with serving on the same bytes. The CPU tests copy CUDA's rules by hand.
4. **Three internal choices are not exposed by serving:** the route port chosen at each stage, the flock's kept list, and the touched words of an update. A fault in them is caught only when it changes an output word.
5. **Edge rules.** `root_cost` overflow is undefined on CUDA and a debug panic in Rust; it is unreachable in range, and no check pins the range. The `snap` start value differs but never changes the output.
6. **The trunk artifact's snaps (n = 4, n = 1) are served but have no CUDA training implementation.**
