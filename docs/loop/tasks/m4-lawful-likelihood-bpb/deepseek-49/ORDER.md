# ORDER: m4-lawful-likelihood-bpb, deepseek-49 (lawful likelihood design, instrument and finite controls)

Lab: deepseek. Item: [m4-lawful-likelihood-bpb](../../../../roadmap/items/m4-lawful-likelihood-bpb.md) (#2861). Base: `775d6aad2`. `References #2861` (the PM closes it only when `Closes` is earned; this unit lands the design, the likelihood and the finite controls, and drafts the pre-registration).

## Goal

Build the lawful likelihood the M4 0.90 bits-per-byte bar needs: a geometric construction that turns the integer model's scores into an **exact rational** distribution over the next byte, with no softmax, cross-entropy, normalized exponential, sigmoid or table form of them (D23/D25), no dense matmul, and bits computed by integer arithmetic. Ship it in the crate that fits (expected `crates/uor-r4-integer`, beside the serving path) with finite host controls, a design note on #2861, a drafted one-page pre-registration (PENDING, no GPU), a notebook row, and the PM/owner decision card for the pieces the charter reserves (the 0.90 bar stays the owner's; naming/hash of the chat held-out stream is step 4 of the item).

## Design direction (to be argued fully in the design note before code)

- **Combinatorial probability, not a squashing surrogate:** the integer read predicts a state in the clamped `E8`/frame geometry; the weight for a candidate byte is an **integer geodesic count** in that geometry (for example the number of shortest compositions carrying the predicted state to the byte's code), and the probability is the exact ratio `count_b / sum(counts)`. Non-negative, sums to exactly 1 in exact rational arithmetic over the full alphabet; no exp-like normalisation anywhere.
- **Bits without banned functions:** score with integer `log2` on `u128` (bit length plus a documented fixed-point refinement), so bits-per-byte is exact to a stated rounding bound; no float `log`/`exp` in the instrument. The only float anywhere is an offline export-gap check, if one is needed, and it is never served.
- **Lawful as a testable property:** `p_b >= 0`, `sum(p_b) == 1` exactly, the closed-form enumeration on a tiny alphabet matches a brute-force count, and a count-table baseline is scored by the same code.
- **Two candidate frames must be compared on paper first** (geodesic counts vs exact histogram-of-reads) and one chosen in the design note with the reason; if the choice needs a charter-reserved call, it becomes a decision card, not a lab decision.

## Cancel probe (run first)

Class **instrument** (host-only design; no GPU). At the base: `grep -rn 'bits_per_byte\|bp b\|lawful likelihood' crates/ docs/mechanisms docs/model | grep -iv test` finds no likelihood implementation or likelihood module, and the item's issue #2861 has no claimed implementation. The M1 top-1 bar change (CHARTER §4.8) confirms BPB has no current scorer. Task stands.

## Plan and files in scope

1. Post a one-line intent note on #2861; write the design note (construction, lawfulness property, banned-function audit, the two candidate frames and the choice, the crate choice and why) and post it **before** code.
2. Implement the likelihood + BPB scorer in Rust with integer arithmetic; finite controls in the same crate's tests: tiny alphabet with exact enumeration, full-alphabet sum-to-one, count-table baseline scored by the same code, an impossible/alphabet-edge case, and a cancel probe that forces a non-normalized weight vector and must be rejected.
3. Draft the one-page pre-registration sheet naming the chat held-out stream file, byte count and sha256, the frozen scoring formula, the 0.90 threshold as the owner's, and `TEST FITNESS: PENDING`; then request the PM/owner registration card.
4. Notebook row in `docs/research/notebook/m4-serving.md`; `uor-harness check`, `loop report --check`, `loop pr-check --local`, `loop gates --full` pass.

- `crates/uor-r4-integer/**` (likelihood module + tests; chosen crate recorded in the design note), `crates/uor-r4-discrete/**` (read side if the design requires it), `docs/mechanisms/lut-serving.md`, `docs/model/serving-and-export.md`, `docs/research/findings.md`, `docs/research/open-questions.md`, `docs/labs/m4-lawful-likelihood-bpb/**`, `docs/harness/prep/m4-lawful-likelihood-bpb*`.

## Done when

- The design note is on #2861 with the construction, the lawfulness property, the banned-function audit and the chosen crate; the likelihood and BPB scorer are implemented in Rust with each finite control passing on the host (`cargo test -p <crate>`, no CUDA).
- The drafted pre-registration sheet names and hashes the chat held-out stream, fixes the scoring formula and carries `TEST FITNESS: PENDING`; the owner-bar/registration request is posted as a card, not decided here.
- `References #2861` PR merged with a counted non-author record; no GPU run, no bar change, no spend.

## Stop and ask if

- A numeric BPB bar would have to be set or changed (it would not; the owner's 0.90 bar stays, this unit only makes it scoreable).
- The lawful construction needs a mechanism-design choice the charter reserves, or a file outside the item scope (for example a new crate) is required.