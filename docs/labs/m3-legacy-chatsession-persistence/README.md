# m3-legacy-chatsession-persistence (#2854, folds #1476)

## Design note

- **Decision:** Retire the ignored width-576 `ChatSession` continuation test (`restored_dialogue576_session_continues_identically` in [`crates/uor-r4-integer/tests/capability_api_tests.rs`](../../../crates/uor-r4-integer/tests/capability_api_tests.rs)) in favour of the `IntegerStackSession` reload gate (`uor-r4.stack-session/1` in [`crates/uor-r4-integer/src/session_codec.rs`](../../../crates/uor-r4-integer/src/session_codec.rs)). Posted on [`#2031`](https://github.com/UOR-Foundation/uor-r4/issues/2031#issuecomment-6335423222).
- **Why:** No shipped binary or serving path builds a width-576 `ChatSession` (`uor-chat` uses `DialogueConversation` for `--bundle` and `IntegerStackModel` / `StackChat` for `--stack`; `wasm_bindgen_exports.rs` uses `IntegerStackSession` via `wasm_serve`), and the local `dialogue-child-bundle-1` fixture is retired.
- **Exact reload gate (`#1476`):** Covered on every run by `session_save_and_restore_produces_bit_identical_logits` and `session_pause_save_fresh_restore_greedy_continuation` in [`crates/uor-r4-integer/src/stack/tests.rs`](../../../crates/uor-r4-integer/src/stack/tests.rs), alongside width-256 `uor-r4.integer-session/1` round-trip continuation in `restored_session_keeps_sampler_policy_and_read_mode_and_continues_identically` ([`crates/uor-r4-integer/tests/capability_api_tests.rs`](../../../crates/uor-r4-integer/tests/capability_api_tests.rs)).
- **API wording updated:** [`CapabilityTruthMatrix::default`](../../../crates/uor-r4-integer/src/capability_api.rs) (`durable_memory`) and [`IntegerCapabilityApi::restore_session`](../../../crates/uor-r4-integer/src/capability_api.rs) document width-256 `uor-r4.integer-session/1` save/restore and `uor-r4.stack-session/1` (`IntegerStackSession`) exact stack-session reload without the retired "omits the width-576 value stores" defect text.

## Notebook row (in-scope record for `m3-durable-memory`)

Because `docs/research/notebook/m3-durable-memory.md` is not listed in `m3-legacy-chatsession-persistence.md`'s frontmatter `notebook:` / `scope:` (queried with the PM in [`#2028`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6335479675)), the notebook row is recorded here in the in-scope lab README:

| Date | Item | Change / Finding | Verification |
|---|---|---|---|
| 2026-10-22 | `m3-legacy-chatsession-persistence` (`#2854`, folds `#1476`) | Retired ignored `restored_dialogue576_session_continues_identically` in `crates/uor-r4-integer/tests/capability_api_tests.rs` in favour of the `IntegerStackSession` reload gate (`uor-r4.stack-session/1` in `crates/uor-r4-integer/src/session_codec.rs`) and updated `CapabilityTruthMatrix::default().durable_memory` and `restore_session` doc in `crates/uor-r4-integer/src/capability_api.rs`. | `cargo test -p uor-r4-integer --features serving-wasm --test capability_api_tests` (3 passed, 1 lifecycle fixture test ignored) and `cargo test -p uor-r4-integer --features serving-wasm --lib stack::tests::session_` (7 passed, including `session_save_and_restore_produces_bit_identical_logits` and `session_pause_save_fresh_restore_greedy_continuation`). |
