# ORDER for the #4059 adoption scope

Lab: codex. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md). References #3168. Base: `9465b92d6e11ca0ee913ebe34aaaddff0cdf97f5`.

## Cancel probe

The same-repository adoption of #3424 used the infra-outsider-repairs roadmap item for #3168. Of the eight source paths in #4059, that item's base scope covers d11_export.rs alone. The policy checks scope at merge base.

## Scope and done check

Register the seven other Rust paths under infra-outsider-repairs and record the current #3168 unit. Verify the union covers all eight paths with a base-versus-head comparison. Preserve all existing scope entries, item status, historical acceptance criteria and research bars. Stop if the source head or scope grows. The adoption proceeds in its own PR.
