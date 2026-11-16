# ORDER: one-model-end-to-end, codex-25 (the slide on the product path)

Lab: codex. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md).
Parent: [codex-24](../codex-24/REPORT.md). Owner direction: make the run's artifact usable where the
claim is made.

## Why

The item's claim is made on the product `uor-chat --stack` path (the 2 Nov public baseline ran
there). A read-bearing artifact refuses past its context, and `uor-chat` had no way to enable the
slide codex-23 added, so the read that now serves the model could not be used on a prompt longer
than 192 tokens — which is every held-out conversation. The gap is not the mechanism; it is the
mechanism's reachability from the claiming surface.

## Design

- `StackChat::set_context_slide(on)`: forward to the session, so the text path carries the policy.
- `uor-chat --context-slide`: accepted only with `--stack`; applied to the chat session and to the
  raw-id session; the session's own refusals surface (a Hamming read, turn memory, a pointer head or
  a copy boost still refuses).
- No new arithmetic and no default change: without the flag, a full context refuses exactly as
  before.

## Done when

1. A 200-token prompt on the read-bearing artifact refuses without the flag and serves with it.
2. Two runs with the flag produce identical generated ids and session bytes, and report tok/s.
3. Below the bound the flag changes nothing: the same tokens and the same read picks.
4. The suites stay green.

## Cancel

Stop if the slide changes a token or a pick while the window is not yet full.
