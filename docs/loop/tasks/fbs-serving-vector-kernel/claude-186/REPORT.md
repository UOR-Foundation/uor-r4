# REPORT: fbs-serving-vector-kernel, claude-186 (the integer square root starts at the value's top bit)

Lab: claude. Order: [ORDER.md](ORDER.md). The item's done-when is met: 1 by claude-185, 2 and 3 here. The item has no GitHub issue; its claims and merges are noted on #2028.

**Speed.** The unsnapped `raw_both` decode rose from 4,176 / 4,195 to 4,491 / 4,504 tok/s on this M1 (+7.5%), and context from 4,153 / 4,171 to 4,476 / 4,483. Snapped is unchanged, within noise: 4,789 / 4,795 → 4,831 / 4,821.

## What did not work, first

- **One base run's decode read 3,089 tok/s,** where its own `--parts` step median was 239.1 µs, the same as the other base runs. It is most likely a disturbance on the machine during that run's decode timing: the same binary's step median, from the same invocation, is normal. A third alternating pair was run, and the outlier is not used.
- **The norms did not speed up:** layer 0's two norms stay at 1.67 µs. Their values are `ms << 68`, near `2^100`, so the walk the change removes was already short there. The gain is in the rotation's parts.
- **Snapped serving gained nothing measurable.** It calls `stack_rotation` nowhere: its transition and gates use the snap (`session.rs:3419`, `:3446`). Its only `stack_isqrt` calls are the norms, the lift and the complement, whose values are already near the top.
- **The default prompt was not used on `raw_both`.** claude-185's review found that its reply is one token repeated. This unit used the prompt `id_i = (613 i) mod 4000 + 50`, whose reply has 29 distinct tokens of 65.

## What changed

- **`crates/uor-r4-integer/src/stack/kernels.rs`:** `stack_isqrt` returns 0 for 0. Otherwise it takes its first probe bit, the largest power of four at most the value, as `1 << ((127 − value.leading_zeros()) & !1)`, in place of the loop from `2^126`. The digit-by-digit loop after it is unchanged.
- **A new test** in that file's tests, `isqrt_from_the_top_bit_equals_the_walking_form`, holds it equal to the walking form.
- **`docs/roadmap/items/fbs-serving-vector-kernel.md`:** `status: done`, with a done note. The roadmap is re-rendered.
- **One notebook row.**

## Measurements

**Exactness.** The new `stack_isqrt` equals the walking form on 2,048,961 values:
- every value below `2^20`;
- every power of two `2^0`..`2^127`, with its two neighbours;
- `u128::MAX`;
- a million seeded values at every bit length.

**Negative gate:** starting one power of four low, `(… & !1).saturating_sub(2)`, fails the test at once (`kernels.rs:1865`). Restored, it passes, and both builds recompiled.

**Speed, alternating with claude-185's sealed base** (`fbs-toks --parts`, M1, AC power):

| Run | Artifact | Base decode / context tok/s, step | New decode / context tok/s, step | Replies |
|---|---|---|---|---|
| 1 | claude-126 snapped (default prompt) | 4,789 / 4,773, 207.7 µs | 4,831 / 4,804, 207.2 µs | identical |
| 2 | claude-126 snapped | 4,795 / 4,795, 207.5 µs | 4,821 / 4,807, 219.8 µs | identical |
| 1 | dual-c131 `raw_both` (varying prompt) | 4,176 / 4,153, 238.9 µs | 4,491 / 4,476, 222.6 µs | identical |
| 2 | dual-c131 `raw_both` | 3,089 / 2,743, 239.1 µs (outlier, see above) | 4,466 / 4,442, 222.5 µs | identical |
| 3 | dual-c131 `raw_both` | 4,195 / 4,171, 238.5 µs | 4,504 / 4,483, 221.9 µs | identical |

Snapped run 2's new step median (219.8 µs) is above its own decode rate's implied step. That is one `--parts` pass's noise, and its decode matches run 1.

**Layer 0's parts** (run 1, µs):

| | Snapped, base | Snapped, new | `raw_both`, base | `raw_both`, new |
|---|---|---|---|---|
| `recurrence.core` | 15.6 | 15.5 | 18.4 | 16.6 |
| `mlp.product` | 9.1 | 9.0 | 11.6 | 10.5 |

**So the unsnapped contract's extra cost over snapped falls** from about 31 µs a token to about 15. That is 238.9 − 207.7, against 222.6 − 207.2, in run 1's step medians.

**The serving audit** (`audit_zero_matmul_serving.py --stack`, `uor-r4-stack` `954de574…`): FULL PASS. The multiplies / divides are 47 / 1 in the symbol ranges and 65 / 9 in the call graph, unchanged.

**Binaries:** base `f78428981f9575c9…e2a3` (sealed by claude-185); new `fbs-toks` `7f1a1ab2…`.

**Sealed** to `caseyallard/uor-r4-store:results/claude/fbs-isqrt-start-2026-10-29/`: 17 files. The `SHA256SUMS` sha256 is `eca10c42ab7abb152e48b460102cfc00f22242f3ff18c37ef046a0e2a72378c8`. Read back, every file verifies.

## Checks run at the head

__CHECKS__

## Skills used

- **holding-the-goal.** The change was checked against the item's three bars before the item was set done: exact (with a negative gate), replies identical (on a reply that can differ), and faster against the sealed base. The outlier run is reported and set aside with its reason, not dropped silently.

## Still open

- **The item's other candidates** for the unsnapped rotation, for a later item:
  - the remaining digit-by-digit loop (about 48 `u128` iterations for a `2^97` value);
  - the four divides, in a form faster than claude-185's reciprocal;
  - fewer golden candidates for the snapped contract;
  - a vector kernel the audit allows.
- **The unsnapped contract is still about 15 µs a token behind snapped.**
- **The map's gap 5** is still stale (claude-185's list).

## Plain summary

The integer model computes an exact square root for each of its rotations. That calculation used to begin by scanning down from the largest possible number until it reached the value's size. It now jumps straight to the right starting point with one instruction that counts leading zero bits.

The answers are the same on two million test values, and the replies are the same, word for word. The newer exact-rotation model now serves 7.5% faster on this laptop, at about 4,500 tokens a second, which halves its gap to the older model. That finishes this task's goal.
