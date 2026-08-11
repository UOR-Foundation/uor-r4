# `just vv` is the normative acceptance gate. Everything else is a slice of it.

default: vv

# The whole gate.
vv: fmt-check model lint test features bdd
    @echo "vv: the acceptance gate passed"

# R1, R4, R5 --- the repository gates, each falsifiable.
model:
    cargo run -q -p xtask -- validate

# Regenerate everything the model owns: CONFORMANCE.md.
model-write:
    cargo run -q -p xtask -- check-model --write

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

# A feature only its author has built is a feature that does not work: nothing
# else in the gate compiles a crate at anything but its default features, so a
# rename upstream of an optional dependency fails nowhere until someone turns
# the flag on. `--all-targets` because the tests behind a flag are code too.
#
# Every optional feature compiles, with its tests.
features:
    cargo check --workspace --all-features --all-targets

# R3: every capability begins as a Gherkin scenario, and every scenario has a
# test whose name ends in its ID.
bdd:
    cargo test -p repo-conformance

# R6: nothing shipped depends on a dev-only crate, no wildcard version
# requirement, no advisory against anything in the tree. Needs
# `cargo install cargo-deny`, which is why it is not in `just vv`.
#
# Advisories, bans, licences and sources, over the dependency graph.
deny:
    cargo deny --all-features check
