# REPORT: authorize and register the CPU training repair program, deepseek-1

Lab: deepseek. Item: [cpu-training-program](../../../../roadmap/items/cpu-training-program.md) (#3507). Base: `381ae4d98`.

## What did not work

- **No CPU training method has been repaired in this unit.** It records authority, dispositions and the implementation sequence. No model compute ran, and the new notebook has no result rows.
- **The repository-wide `cargo test -p uor-r4-harness` is red on the base.** On clean `origin/main`, the representative tests for the milestone-registry mismatch, the malformed CUDA performance sheet and the registry PR gate fail unchanged. The full suite at this head reports 663 passed, 14 failed and 1 ignored. These findings predate this documentation branch and are outside its scope.
- **The exact-head GitHub `harness rules and tests` job is red on those same baseline failures:** https://github.com/UOR-Foundation/uor-r4/actions/runs/41356999172/job/124999322336. This is reported, not waived.
- **`uor-harness check` is red on the base at one error plus warnings.** The error is the existing malformed `docs/harness/prep/uor-cuda-inverted-file-performance-2026-11-01.md` sheet (six missing shape blocks). Most warnings are existing claims, presence, pre-registration and documentation-command debt. This PR does not claim a green global check.
- **The initial commit combined ORDER registration with the unit content.** That raises the D76 commit-order advisory; it is not a source or owner-path failure. The branch stays as pushed because rewriting a protected-topic branch is unnecessary for an advisory.

## What changed

- D84 amends CHARTER §3.5 from absolute GPU-only research compute to GPU-first compute. Only a named, separately scanned CPU program authorized by the owner may run CPU training, evaluation or parity as development research. CPU evidence does not replace a GPU qualification bar without a later owner decision. The program item is marked done because this PR delivers its registration/authority scope; the seven child items carry the actual repairs.
- AGENTS.md, the hard-rule summary, PM rules, all four operational compute/training skills, the milestone registry and the live onboarding/model/geometry/start pages now state the same boundary.
- The CPU training program preserves ten historical family dispositions, including the banned teacher/dense/softmax forms, the retained whole-path/exact-update reference and the old loss-versus-capability negatives.
- Seven child items register the order: exact discrete writer core; discrete reads/counterfactuals/BVG-K; shared-vector forward; lawful shared-vector training; quantization plus exact memory; whole-path exact-update/recovery parity; and the bounded pilot.
- Roadmap sync created #3501-#3507 and #3510. This registration PR closes #3507; the seven child issues remain open.

## Measurements

This unit runs no model compute. The registration counts are 10 historical families, 7 child implementation items, 8 new roadmap issues (#3501-#3507 and #3510), and 133 total roadmap items after sync. The local diagnostic readings retained in the program page are labelled as unsealed component ceilings and are not re-run here.

## Checks run at the head

- `cargo run -q -p xtask -- roadmap check`: PASS, 133 items.
- `cargo run -q -p xtask -- roadmap render --check`: PASS, current.
- `cargo run -q -p xtask -- links`: exit 0; 23,852 links checked, 1,216 pre-existing broken-link reports remain in the repository-wide count.
- `cargo test -q -p uor-r4-harness --test lab_registry_cli --test loop_shared_files --test loop_review --test loop_claim --test loop_next`: PASS, 5, 7, 34, 33 and 37 tests.
- `uor-harness loop gates --full`: PASS, 11/11 green.
- `uor-harness loop pr-check --local`: PASS. The expected scope advisory names 18 files outside the reused documentation parent, because the new program and child items do not exist at the merge base; the commit-order and FILEMAP advisories also remain. Owner paths, rule sources, ORDER and REPORT pass.
- `cargo test -q -p uor-r4-harness`: FAIL, 663 passed / 14 failed / 1 ignored. Representative failures were rerun at clean `origin/main` and fail there unchanged.
- `uor-harness check`: FAIL on the same pre-existing malformed pre-registration sheet, with warnings; no claim of a green global check.

## Skills used

No repository skill was invoked. Three fresh-context subagents were used earlier for independent architecture, FBS-path and historical-family audits; their findings were checked against the source and consolidated here by the top session.

## Still open

- Merge this registration unit, sync main and then claim `cpu-discrete-writer-core` (#3502).
- The first implementation unit must prove exact writer/admission parity before any CPU read, update or corpus pilot.
- No CPU result may enter a milestone claim until a later owner decision names that qualification path.

## Plain summary

The owner has authorized a scoped CPU research exception. GPU remains the authority, all historical training families keep their recorded dispositions, and seven isolated units now give the repair a concrete order. No trainer has been claimed as working yet.
