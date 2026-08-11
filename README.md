# Template

A repository template with the gate machinery and none of the content.

`just vv` passes as it stands. It has no shipped crates, an empty claim
register, and an empty ledger --- and every check re-arms the moment the first
capability is added, because the anti-vacuity checks are keyed to the register
rather than asserted outright.

## Start here

1. **Name it.** `Cargo.toml` has `CHANGEME` in `repository` and `homepage`, and
   empty `keywords` and `categories`. They are inherited by every crate.
2. **Rename the tooling crates if you want to.** `repo-model` and
   `repo-conformance` are deliberately neutral; they are `publish = false` and
   nothing outside the workspace sees them.
3. **Add your first crate** under `crates/`. It is *shipped* unless its manifest
   says `publish = false`, and the gates read that rather than a list.
4. **Add your first capability** in the order `AGENTS.md` sets out: a row in
   `model/ids.toml`, a scenario in `features/suites/`, a failing test named for
   the ID, then the implementation.

## What is here

| Path | What it is |
| --- | --- |
| `model/` | the single source of every claim: the ID register, the ledger, the authorities |
| `features/suites/` | one Gherkin scenario per conformance ID |
| `crates/model` | parses `model/*.toml` and generates `CONFORMANCE.md` |
| `crates/conformance` | the BDD runner and the honesty meta-gate |
| `xtask/` | the gates: `check-model`, `audit-limits`, `audit-deferral` |

## The gate

| Recipe | What it does |
| --- | --- |
| `just vv` | the whole gate; everything below in order |
| `just fmt-check` | formatting |
| `just model` | the repository gates: R1, R4, R5 |
| `just lint` | clippy at `-D warnings` |
| `just test` | the workspace suite |
| `just features` | every optional feature compiles, with its tests |
| `just bdd` | R3 and the honesty meta-gate |
| `just deny` | advisories, bans, licences and sources (needs `cargo-deny`) |

`AGENTS.md` defines R1 through R6 and is the brief for changing anything here.
`VERIFICATION.md` maps each gate to what it discharges and records the defect
planted to prove it can fail.

## Claim discipline

Every claim carries one of three honesty levels, and the build fails if the two
registers are blurred:

| Level | Meaning |
| --- | --- |
| `some-true` | reproduced from an authority. **Not established here.** |
| `build` | constructed here and validated against its oracle. Evidence, not proof. |
| `open` | measured and reported, **never asserted**. |

`CONFORMANCE.md` is generated from `model/`, so a claim cannot exist in the
documentation without a register row, or in the register without appearing in
the documentation.

## Licence

Dual-licensed under either of

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT license ([`LICENSE-MIT`](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this work by you, as defined in the Apache-2.0 licence, shall
be dual-licensed as above, without any additional terms or conditions.
